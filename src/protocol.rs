use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(rename = "system")]
    System(SystemMessage),
    #[serde(rename = "heartbeat")]
    Heartbeat(HeartbeatMessage),
    #[serde(rename = "access_token")]
    AccessToken(AccessTokenMessage),
    #[serde(rename = "postgres_changes")]
    PostgresChanges(PostgresChangesMessage),
    #[serde(rename = "broadcast")]
    Broadcast(BroadcastMessage),
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
            PhoenixMessage::System(sys) => &sys.topic,
            PhoenixMessage::Heartbeat(heart) => &heart.topic,
            PhoenixMessage::AccessToken(acc) => &acc.topic,
            PhoenixMessage::PostgresChanges(pg) => &pg.topic,
            PhoenixMessage::Broadcast(bcast) => &bcast.topic,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinMessage {
    pub topic: Topic,
    pub payload: JoinPayload,
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveMessage {
    pub topic: Topic,
    pub payload: HashMap<String, String>,
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseMessage {
    pub topic: Topic,
    pub payload: HashMap<String, String>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyMessage {
    pub topic: Topic,
    pub payload: ReplyPayload,
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub topic: Topic,
    pub payload: SystemPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub topic: Topic,
    pub payload: HashMap<String, String>,
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
pub struct PostgresChangesMessage {
    pub topic: Topic,
    pub payload: PostgresChangesPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub topic: Topic,
    pub payload: BroadcastPayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceStateMessage {
    pub topic: Topic,
    pub payload: HashMap<String, PresenceMetas>,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceDiffMessage {
    pub topic: Topic,
    pub payload: PresencePayload,
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMetas {
    pub metas: Vec<PresenceMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresencePayload {
    pub joins: HashMap<String, PresenceMetas>,
    pub leaves: HashMap<String, PresenceMetas>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinPayload {
    pub config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyPayload {
    pub response: ReplyResponse,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPayload {
    pub channel: String,
    pub extension: String,
    pub message: String,
    pub status: Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenPayload {
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresChangesPayload {
    pub data: PostgresChangeData,
    pub ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastPayload {
    pub event: String,
    pub payload: HashMap<String, serde_json::Value>,
    #[serde(rename = "type")]
    pub broadcast_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub broadcast: Option<BroadcastConfig>,
    pub presence: Option<PresenceConfig>,
    pub postgres: Option<Vec<PostgresConfig>>,
}

impl Config {
    pub fn broadcast(config: BroadcastConfig) -> Self {
        Self {
            broadcast: Some(config),
            ..Default::default()
        }
    }

    pub fn presence(config: PresenceConfig) -> Self {
        Self {
            presence: Some(config),
            ..Default::default()
        }
    }

    pub fn postgres(config: Vec<PostgresConfig>) -> Self {
        Self {
            postgres: Some(config),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BroadcastConfig {
    pub ack: bool,
    #[serde(rename = "self")]
    pub self_broadcast: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresenceConfig {
    pub key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub id: Option<i64>,
    pub event: PostgresEvent,
    pub schema: String,
    pub table: String,
    pub filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresChangeData {
    pub schema: String,
    pub table: String,
    pub commit_timestamp: String,
    #[serde(rename = "eventType")]
    pub event_type: PostgresEvent,
    pub new: HashMap<String, serde_json::Value>,
    pub old: HashMap<String, serde_json::Value>,
    pub errors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMeta {
    pub phx_ref: String,
    pub name: String,
    pub t: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyResponse {
    pub postgres_changes: Vec<PostgresChangeData>,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}
