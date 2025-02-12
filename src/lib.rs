mod channel;
mod protocol;

pub use protocol::{
    BroadcastConfig, BroadcastPayload, PostgresConfig, PostgresEvent, PresenceConfig,
};

use channel::{Broadcast, Subscription};
use futures::{SinkExt, StreamExt};
use protocol::{
    AccessTokenMessage, AccessTokenPayload, BroadcastMessage, Config, JoinMessage, JoinPayload,
    PhoenixMessage, Status, Topic,
};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use tokio::{
    sync::{mpsc, Mutex},
    task,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct Client {
    sender: mpsc::Sender<PhoenixMessage>,
    receivers: Arc<Mutex<Vec<(String, mpsc::Sender<PhoenixMessage>)>>>,
    reference: Arc<AtomicU32>,
}

impl Client {
    /// Creates a new realtime client.
    pub async fn connect(
        endpoint: impl Into<String>,
        token: impl Into<String>,
    ) -> Result<Self, Error> {
        let url = format!("{}?apikey={}", endpoint.into(), token.into());
        let (stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = stream.split();

        let receivers: Arc<Mutex<Vec<(String, mpsc::Sender<PhoenixMessage>)>>> =
            Arc::new(Mutex::new(Vec::new()));
        let incomming_receivers = receivers.clone();

        // incomming event listener
        task::spawn(async move {
            while let Some(message) = read.next().await {
                let message = match message {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<PhoenixMessage>(&text) {
                            Ok(message) => {
                                tracing::debug!("recv: {:?}", message);
                                message
                            }
                            Err(e) => {
                                tracing::error!("failed to parse: {}", e);
                                continue;
                            }
                        }
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        tracing::error!("{}", e);
                        continue;
                    }
                };

                incomming_receivers.lock().await.retain(|(topic, sender)| {
                    if topic != message.topic().as_str() {
                        true
                    } else {
                        match sender.try_send(message.clone()) {
                            Ok(_) => true,
                            Err(mpsc::error::TrySendError::Full(_)) => true,
                            Err(mpsc::error::TrySendError::Closed(_)) => false,
                        }
                    }
                });
            }
        });

        // outgoing queue
        let (sender, mut outgoing_receiver) = mpsc::channel(128);
        task::spawn(async move {
            loop {
                if let Some(message) = outgoing_receiver.recv().await {
                    tracing::debug!("send: {:?}", message);
                    match serde_json::to_string(&message) {
                        Ok(text) => {
                            if let Err(e) = write.send(Message::Text(text)).await {
                                tracing::error!("{}", e);
                            }
                        }
                        Err(e) => tracing::error!("Failed to serialize message: {}", e),
                    }
                }
            }
        });

        Ok(Client {
            sender,
            receivers,
            reference: Arc::new(AtomicU32::new(1)),
        })
    }

    /// Update the acess token.
    pub async fn access_token(&mut self, token: impl Into<String>) -> Result<(), Error> {
        let reference = fetch_ref(&self.reference).to_string();
        self.sender
            .send(PhoenixMessage::AccessToken(AccessTokenMessage {
                topic: Topic::new(""),
                payload: AccessTokenPayload {
                    access_token: token.into(),
                },
                reference,
            }))
            .await?;

        Ok(())
    }

    pub async fn broadcast(
        &self,
        topic: impl Into<String>,
        payload: BroadcastPayload,
    ) -> Result<(), Error> {
        self.sender
            .send(PhoenixMessage::Broadcast(BroadcastMessage {
                topic: Topic::from(topic.into()),
                payload,
                reference: Some(self.reference.fetch_add(1, Ordering::Relaxed).to_string()),
            }))
            .await?;

        Ok(())
    }

    /// Listen to broadcast messages.
    pub async fn on_broadcast(
        &mut self,
        topic: impl Into<String>,
        config: BroadcastConfig,
    ) -> Result<Subscription<Broadcast>, Error> {
        let topic = topic.into();

        let (sender, mut receiver) = mpsc::channel(128);
        self.receivers.lock().await.push((topic.to_owned(), sender));

        let reference = fetch_ref(&self.reference).to_string();
        self.sender
            .send(PhoenixMessage::Join(JoinMessage {
                topic: topic.clone().into(),
                payload: JoinPayload {
                    config: Config::broadcast(config),
                },
                reference: reference.clone(),
            }))
            .await?;

        loop {
            let response = receiver.recv().await;

            match response {
                Some(PhoenixMessage::Reply(reply)) => {
                    if let Some(response_ref) = reply.reference {
                        if response_ref == reference {
                            if reply.payload.status == Status::Ok {
                                return Ok(Subscription::<Broadcast>::new(
                                    topic.into(),
                                    receiver,
                                    self.sender.clone(),
                                    self.reference.clone(),
                                ));
                            } else {
                                return Err(Error::from("failed to join channel"));
                            }
                        }
                    }
                }
                Some(_) => {}
                None => {
                    return Err(Error::from("stream closed"));
                }
            }
        }
    }
}

/// Fetch and increment atomic reference counter.
pub(crate) fn fetch_ref(r: &AtomicU32) -> u32 {
    r.fetch_add(1, Ordering::Relaxed)
}
