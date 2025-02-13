use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum PhoenixMessage {
    #[serde(rename = "phx_join")]
    Join(JoinMessage),
    #[serde(rename = "phx_leave")]
    Leave(LeaveMessage),
    #[serde(rename = "phx_close")]
    Close(CloseMessage),
    #[serde(rename = "phx_reply")]
    Reply(ReplyMessage),
    #[serde(rename = "phx_error")]
    Error(ErrorMessage),
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "heartbeat")]
    Heartbeat(HeartbeatMessage),
    #[serde(rename = "access_token")]
    AccessToken(AccessTokenMessage),
    #[serde(rename = "postgres_changes")]
    Postgres(PostgresMessage),
    #[serde(rename = "broadcast")]
    Broadcast(BroadcastMessage),
    #[serde(rename = "presence")]
    Presence(Presence),
    #[serde(rename = "presence_state")]
    PresenceState(PresenceStateMessage),
    #[serde(rename = "presence_diff")]
    PresenceDiff(PresenceDiffMessage),
}

impl PhoenixMessage {
    pub fn topic(&self) -> &Topic {
        match self {
            PhoenixMessage::Join(join) => &join.topic,
            PhoenixMessage::Leave(leave) => &leave.topic,
            PhoenixMessage::Close(close) => &close.topic,
            PhoenixMessage::Reply(reply) => &reply.topic,
            PhoenixMessage::Error(err) => &err.topic,
            PhoenixMessage::System(sys) => &sys.topic,
            PhoenixMessage::Heartbeat(heart) => &heart.topic,
            PhoenixMessage::AccessToken(acc) => &acc.topic,
            PhoenixMessage::Postgres(pg) => &pg.topic,
            PhoenixMessage::Broadcast(bcast) => &bcast.topic,
            PhoenixMessage::Presence(pres) => &pres.topic,
            PhoenixMessage::PresenceState(pstate) => &pstate.topic,
            PhoenixMessage::PresenceDiff(pdiff) => &pdiff.topic,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic(String);

impl Topic {
    pub fn new(s: impl Into<String>) -> Self {
        Topic(s.into())
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Topic {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Serialize for Topic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let prefixed = format!("realtime:{}", self.0);
        serializer.serialize_str(&prefixed)
    }
}

impl<'de> Deserialize<'de> for Topic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        if let Some(stripped) = string.strip_prefix("realtime:") {
            Ok(Topic(stripped.to_string()))
        } else {
            Err(serde::de::Error::custom(
                "topic must start with 'realtime:'",
            ))
        }
    }
}

/// Generic loosely-typed payload.
pub type Payload = BTreeMap<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinMessage {
    pub topic: Topic,
    pub payload: JoinPayload,
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(rename = "join_ref", skip_serializing_if = "Option::is_none")]
    pub join_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinPayload {
    pub config: ChannelConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub broadcast: Option<BroadcastConfig>,
    pub presence: Option<PresenceConfig>,
    pub postgres: Option<Vec<PostgresConfig>>,
    pub private: bool,
}

/// Broadcast channel configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BroadcastConfig {
    /// Server confirms each message has been received.
    pub ack: bool,
    /// Receive own messages.
    #[serde(rename = "self")]
    pub self_broadcast: bool,
}

/// Presence channel configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresenceConfig {
    pub key: String,
}

/// Postgres CDC channel configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub id: Option<i64>,
    pub event: PostgresEvent,
    pub schema: String,
    pub table: String,
    pub filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveMessage {
    pub topic: Topic,
    pub payload: Payload,
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseMessage {
    pub topic: Topic,
    pub payload: Payload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyMessage {
    pub topic: Topic,
    pub payload: ReplyPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub topic: Topic,
    pub payload: Payload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyPayload {
    pub response: ReplyResponse,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyResponse {
    pub postgres_changes: Option<Vec<PostgresChangeData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub topic: Topic,
    pub payload: SystemPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPayload {
    pub channel: String,
    pub extension: String,
    pub message: String,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub topic: Topic,
    pub payload: Payload,
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenMessage {
    pub topic: Topic,
    pub payload: AccessTokenPayload,
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresMessage {
    pub topic: Topic,
    pub payload: PostgresChangesPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresChangesPayload {
    pub data: PostgresChangeData,
    pub ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresChangeData {
    pub schema: String,
    pub table: String,
    pub commit_timestamp: String,
    #[serde(rename = "eventType")]
    pub event_type: PostgresEvent,
    pub new: Payload,
    pub old: Payload,
    pub errors: Option<String>,
}

/// Postgres event kind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PostgresEvent {
    #[default]
    #[serde(rename = "*")]
    All,
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub topic: Topic,
    pub payload: BroadcastPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    #[serde(rename = "join_ref", skip_serializing_if = "Option::is_none")]
    pub join_reference: Option<String>,
}

/// Broadcast message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastPayload {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Payload>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub broadcast_type: Option<BroadcastType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BroadcastType {
    Broadcast,
    Presence,
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    pub topic: Topic,
    pub payload: PresencePayload,
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(rename = "join_ref", skip_serializing_if = "Option::is_none")]
    pub join_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresencePayload {
    #[serde(rename = "type")]
    pub payload_type: String,
    pub event: String,
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceStateMessage {
    pub topic: Topic,
    pub payload: BTreeMap<String, PresenceMetas>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceDiffMessage {
    pub topic: Topic,
    pub payload: PresenceDiffPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceDiffPayload {
    pub joins: BTreeMap<String, PresenceMetas>,
    pub leaves: BTreeMap<String, PresenceMetas>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMetas {
    pub metas: Vec<PresenceMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMeta {
    pub phx_ref: String,
    pub phx_ref_prev: Option<String>,
    pub name: Option<String>,
    pub t: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}
