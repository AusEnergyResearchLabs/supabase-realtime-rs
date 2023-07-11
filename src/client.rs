use std::{collections::HashMap, time::Duration};

use tokio_tungstenite::tungstenite::http::HeaderMap;

///! Client builder
#[derive(Default)]
pub struct ClientBuilder {
    timeout: Option<Duration>,
    heartbeat_interval: Option<Duration>,
    reconnection_delay: Option<Duration>,
    headers: HeaderMap,
    params: HashMap<String, String>,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            ..Default::default()
        }
    }
}

pub struct Client {}

impl Client {
    pub fn builder() -> ClientBuilder {
        return ClientBuilder::new();
    }
}
