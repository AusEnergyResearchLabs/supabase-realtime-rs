use crate::{fetch_ref, protocol::*, Error};
use std::{
    sync::{atomic::AtomicU32, Arc},
    task::Poll,
};
use tokio::{
    sync::{mpsc, Mutex},
    task,
};

type SharedSenders<T> = Arc<Mutex<Vec<T>>>;

/// Subscription instance.
pub struct Channel {
    topic: Topic,
    sender: mpsc::Sender<PhoenixMessage>,
    reference: Arc<AtomicU32>,
    heartbeat_handle: task::AbortHandle,
    incoming_handle: task::AbortHandle,
    broadcast_subscriptions: SharedSenders<(String, mpsc::Sender<BroadcastMessage>)>,
    presence_subscriptions: SharedSenders<mpsc::Sender<PresenceMessage>>,
    postgres_subscriptions: SharedSenders<mpsc::Sender<PostgresMessage>>,
}

impl Channel {
    pub(crate) fn new(
        topic: Topic,
        receiver: mpsc::Receiver<PhoenixMessage>,
        sender: mpsc::Sender<PhoenixMessage>,
        reference: Arc<AtomicU32>,
    ) -> Self {
        // spawn heartbeat task. cleaned up on drop.
        let heartbeat_topic = topic.clone();
        let heartbeat_sender = sender.clone();
        let heartbeat_reference = reference.clone();
        let heartbeat_handle = task::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(25)).await;
                if let Err(e) = heartbeat_sender
                    .send(PhoenixMessage::Heartbeat(HeartbeatMessage {
                        topic: heartbeat_topic.clone(),
                        payload: Payload::new(),
                        reference: fetch_ref(&heartbeat_reference).to_string(),
                    }))
                    .await
                {
                    tracing::error!("failed to send heartbeat: {}", e);
                }
            }
        })
        .abort_handle();

        let broadcast_subscriptions = SharedSenders::new(Mutex::new(Vec::new()));
        let presence_subscriptions = SharedSenders::new(Mutex::new(Vec::new()));
        let postgres_subscriptions = SharedSenders::new(Mutex::new(Vec::new()));

        let mut receiver = receiver;
        let broadcast = broadcast_subscriptions.clone();
        let presence = presence_subscriptions.clone();
        let postgres = postgres_subscriptions.clone();
        let incoming_handle =
            tokio::spawn(async move {
                while let Some(message) = receiver.recv().await {
                    match message {
                        PhoenixMessage::Broadcast(bcast) => {
                            broadcast.lock().await.retain(
                                |(event, sender): &(String, mpsc::Sender<BroadcastMessage>)| {
                                    if event != bcast.payload.event.as_str() {
                                        true
                                    } else {
                                        match sender.try_send(bcast.clone()) {
                                            Ok(_) => true,
                                            Err(mpsc::error::TrySendError::Full(_)) => true,
                                            Err(mpsc::error::TrySendError::Closed(_)) => false,
                                        }
                                    }
                                },
                            );
                        }
                        PhoenixMessage::PresenceState(state) => {
                            presence.lock().await.retain(
                                |sender: &mpsc::Sender<PresenceMessage>| match sender
                                    .try_send(PresenceMessage::State(state.clone()))
                                {
                                    Ok(_) => true,
                                    Err(mpsc::error::TrySendError::Full(_)) => true,
                                    Err(mpsc::error::TrySendError::Closed(_)) => false,
                                },
                            );
                        }
                        PhoenixMessage::PresenceDiff(diff) => {
                            presence.lock().await.retain(
                                |sender: &mpsc::Sender<PresenceMessage>| match sender
                                    .try_send(PresenceMessage::Diff(diff.clone()))
                                {
                                    Ok(_) => true,
                                    Err(mpsc::error::TrySendError::Full(_)) => true,
                                    Err(mpsc::error::TrySendError::Closed(_)) => false,
                                },
                            );
                        }
                        PhoenixMessage::Postgres(pg) => {
                            postgres.lock().await.retain(
                                |sender: &mpsc::Sender<PostgresMessage>| match sender
                                    .try_send(pg.clone())
                                {
                                    Ok(_) => true,
                                    Err(mpsc::error::TrySendError::Full(_)) => true,
                                    Err(mpsc::error::TrySendError::Closed(_)) => false,
                                },
                            );
                        }
                        _ => {}
                    }
                }
            })
            .abort_handle();

        Self {
            topic,
            sender,
            reference,
            heartbeat_handle,
            incoming_handle,
            broadcast_subscriptions,
            presence_subscriptions,
            postgres_subscriptions,
        }
    }

    /// Create a broadcast subscriber.
    pub async fn on_broadcast(
        &mut self,
        event: impl Into<String>,
    ) -> Subscription<BroadcastMessage> {
        let (sender, receiver) = mpsc::channel(128);
        self.broadcast_subscriptions
            .lock()
            .await
            .push((event.into(), sender));
        Subscription { receiver }
    }

    /// Send broadcast message.
    pub async fn broadcast(&self, event: impl Into<String>, payload: Payload) -> Result<(), Error> {
        self.sender
            .send(PhoenixMessage::Broadcast(BroadcastMessage {
                topic: self.topic.clone(),
                payload: BroadcastPayload {
                    event: event.into(),
                    payload: Some(payload),
                    broadcast_type: BroadcastType::Broadcast,
                },
                reference: Some(fetch_ref(&self.reference).to_string()),
            }))
            .await?;
        Ok(())
    }

    /// Create a broadcast subscriber.
    pub async fn on_presence(&mut self) -> Subscription<PresenceMessage> {
        let (sender, receiver) = mpsc::channel(128);
        self.presence_subscriptions.lock().await.push(sender);
        Subscription { receiver }
    }

    /// Send state to subscribers.
    pub async fn track(&self, state: Payload) -> Result<(), Error> {
        self.sender
            .send(PhoenixMessage::Broadcast(BroadcastMessage {
                topic: self.topic.clone(),
                payload: BroadcastPayload {
                    event: "track".to_owned(),
                    payload: Some(state),
                    broadcast_type: BroadcastType::Presence,
                },
                reference: Some(fetch_ref(&self.reference).to_string()),
            }))
            .await?;
        Ok(())
    }

    /// Stop listening to presence events.
    pub async fn untrack(&self) -> Result<(), Error> {
        self.sender
            .send(PhoenixMessage::Broadcast(BroadcastMessage {
                topic: self.topic.clone(),
                payload: BroadcastPayload {
                    event: "untrack".to_owned(),
                    payload: None,
                    broadcast_type: BroadcastType::Presence,
                },
                reference: Some(fetch_ref(&self.reference).to_string()),
            }))
            .await?;
        Ok(())
    }

    /// Create a broadcast subscriber.
    pub async fn on_postgres(&mut self) -> Subscription<PostgresMessage> {
        let (sender, receiver) = mpsc::channel(128);
        self.postgres_subscriptions.lock().await.push(sender);
        Subscription { receiver }
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        self.heartbeat_handle.abort();
        self.incoming_handle.abort();
    }
}

/// Message subscription.
#[derive(Debug)]
pub struct Subscription<T> {
    receiver: mpsc::Receiver<T>,
}

impl<T> futures::Stream for Subscription<T> {
    type Item = T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(msg) => match msg {
                Some(msg) => Poll::Ready(Some(msg)),
                None => Poll::Ready(None),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Presence message.
#[derive(Debug)]
pub enum PresenceMessage {
    State(PresenceStateMessage),
    Diff(PresenceDiffMessage),
}
