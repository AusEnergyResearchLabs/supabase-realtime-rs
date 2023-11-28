use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::protocol::Message;
use tungstenite::Result;

use crate::Realtime;

pub struct Channel {
    topic: String,
}

impl Channel {
    pub async fn new(client: &mut Realtime) -> Result<Channel> {
        client
            .stream
            .send(Message::Text(Event::Join.as_str().to_string()))
            .await?;

        let res = client.stream.next().await;

        Ok(Channel {
            topic: "".to_string(),
        })
    }
}

enum Event {
    Close,
    Error,
    Join,
    Reply,
    Leave,
    AccessToken,
}

impl Event {
    fn as_str(&self) -> &'static str {
        match self {
            Event::Close => "phx_close",
            Event::Error => "phx_error",
            Event::Join => "phx_join",
            Event::Reply => "phx_reply",
            Event::Leave => "phx_leave",
            Event::AccessToken => "access_token",
        }
    }
}

enum Type {
    Broadcast {
        self_: bool,
    },
    Presence {
        key: String,
    },
    PostgresChanges {
        event: PostgresEvent,
        schema: String,
        table: String,
        filter: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum PostgresEvent {
    #[serde(rename = "*")]
    All,
    Insert,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum SubscribeState {
    Subscribed,
    TimedOut,
    Closed,
    ChannelError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_subscribe_state() {
        let output = serde_json::to_string(&SubscribeState::Closed).unwrap();
        assert_eq!(output, "\"CLOSED\"")
    }
}
