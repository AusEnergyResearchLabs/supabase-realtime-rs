//! # Supabase Realtime
//!

mod channel;
mod protocol;

pub use channel::{Channel, PresenceMessage};
pub use protocol::{
    BroadcastConfig, BroadcastPayload, ChannelConfig, Payload, PostgresConfig, PostgresEvent,
    PresenceConfig,
};

use futures::{SinkExt, StreamExt};
use protocol::*;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use tokio::{
    sync::{mpsc, Mutex},
    task,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Supabase realtime client.
#[derive(Debug)]
pub struct Client {
    sender: mpsc::Sender<PhoenixMessage>,
    receivers: Arc<Mutex<Vec<(String, mpsc::Sender<PhoenixMessage>)>>>,
    reference: Arc<AtomicU32>,
    incoming_handle: task::AbortHandle,
    outgoing_handle: task::AbortHandle,
}

impl Client {
    /// Create a new realtime client.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use supabase_realtime::Client;
    /// # tokio_test::block_on(async {
    /// let url = "ws://localhost:54321/realtime/v1/websocket";
    /// let anon_token = "...";
    /// let client = Client::connect(url, anon_token).await.unwrap();
    /// # })
    /// ```
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

        // incoming event listener
        let incoming_handle = task::spawn(async move {
            while let Some(message) = read.next().await {
                let message = match message {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<PhoenixMessage>(&text) {
                            Ok(message) => {
                                tracing::debug!("recv: {:?}", message);
                                message
                            }
                            Err(e) => {
                                tracing::error!("failed to parse: {} error: {}", text, e);
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
        })
        .abort_handle();

        // outgoing queue
        let (sender, mut outgoing_receiver) = mpsc::channel(128);
        let outgoing_handle = task::spawn(async move {
            while let Some(message) = outgoing_receiver.recv().await {
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
        })
        .abort_handle();

        Ok(Client {
            sender,
            receivers,
            reference: Arc::new(AtomicU32::new(1)),
            incoming_handle,
            outgoing_handle,
        })
    }

    /// Update the acess token.
    pub async fn access_token(&self, token: impl Into<String>) -> Result<(), Error> {
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

    /// Open a new channel.
    pub async fn channel(
        &mut self,
        topic: impl Into<String>,
        config: ChannelConfig,
    ) -> Result<Channel, Error> {
        let topic = topic.into();

        let (sender, mut receiver) = mpsc::channel(128);
        self.receivers.lock().await.push((topic.to_owned(), sender));

        let reference = fetch_ref(&self.reference).to_string();
        self.sender
            .send(PhoenixMessage::Join(JoinMessage {
                topic: topic.clone().into(),
                payload: JoinPayload { config },
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
                                return Ok(Channel::new(
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

impl Drop for Client {
    fn drop(&mut self) {
        // close background tasks
        self.incoming_handle.abort();
        self.outgoing_handle.abort();
    }
}

/// Fetch and increment atomic reference counter.
pub(crate) fn fetch_ref(r: &AtomicU32) -> u32 {
    r.fetch_add(1, Ordering::Relaxed)
}
