use crate::{
    fetch_ref,
    protocol::{
        BroadcastMessage, HeartbeatMessage, PhoenixMessage, PostgresChangesMessage,
        PresenceDiffMessage, PresenceStateMessage, Topic,
    },
};
use serde_json::Map;
use std::{
    marker::PhantomData,
    sync::{atomic::AtomicU32, Arc},
    task::Poll,
};
use tokio::{sync::mpsc, task};

#[derive(Debug)]
pub struct Broadcast;

#[derive(Debug)]
pub struct Presence;

#[derive(Debug)]
pub struct Postgres;

/// Subscription instance.
pub struct Subscription<T> {
    pub(crate) _t: PhantomData<T>,
    pub(crate) receiver: mpsc::Receiver<PhoenixMessage>,
    pub(crate) heartbeat_handle: task::AbortHandle,
}

impl<T> Subscription<T> {
    pub(crate) fn new(
        topic: Topic,
        receiver: mpsc::Receiver<PhoenixMessage>,
        sender: mpsc::Sender<PhoenixMessage>,
        reference: Arc<AtomicU32>,
    ) -> Self {
        // spawn heartbeat task. cleaned up on drop.
        let heartbeat_handle = task::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(25)).await;
                if let Err(e) = sender
                    .send(PhoenixMessage::Heartbeat(HeartbeatMessage {
                        topic: topic.clone(),
                        payload: Map::new(),
                        reference: fetch_ref(&reference).to_string(),
                    }))
                    .await
                {
                    tracing::error!("failed to send heartbeat: {}", e);
                }
            }
        })
        .abort_handle();

        Self {
            _t: PhantomData::default(),
            receiver,
            heartbeat_handle,
        }
    }
}

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        self.heartbeat_handle.abort();
    }
}

impl futures::Stream for Subscription<Broadcast> {
    type Item = BroadcastMessage;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(msg) => match msg {
                Some(PhoenixMessage::Broadcast(bcast)) => Poll::Ready(Some(bcast)),
                Some(_) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                None => Poll::Ready(None),
            },
            Poll::Pending => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

#[derive(Debug)]
pub enum PresenceMessage {
    State(PresenceStateMessage),
    Diff(PresenceDiffMessage),
}

impl futures::Stream for Subscription<Presence> {
    type Item = PresenceMessage;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(msg) => match msg {
                Some(PhoenixMessage::PresenceState(state)) => {
                    Poll::Ready(Some(PresenceMessage::State(state)))
                }
                Some(PhoenixMessage::PresenceDiff(diff)) => {
                    Poll::Ready(Some(PresenceMessage::Diff(diff)))
                }
                _ => Poll::Pending,
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl futures::Stream for Subscription<Postgres> {
    type Item = PostgresChangesMessage;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(msg) => match msg {
                Some(PhoenixMessage::PostgresChanges(changes)) => Poll::Ready(Some(changes)),
                _ => Poll::Pending,
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
