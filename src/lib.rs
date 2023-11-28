pub mod channel;
pub mod filter;

use channel::Channel;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http::Request, Result},
    MaybeTlsStream, WebSocketStream,
};

pub struct Realtime {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    channels: Vec<Channel>,
}

impl Realtime {
    pub async fn new<S>(url: S, api_key: S) -> Result<Realtime>
    where
        S: Into<String>,
    {
        let uri = format!("{}?apikey={}", &url.into(), &api_key.into());

        let request = Request::builder()
            .uri(uri)
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Host", "sb.aerl.cloud")
            .body(())?;

        let (stream, response) = connect_async(request).await?;

        println!("Huh? {:?}", response);

        Ok(Realtime {
            stream,
            channels: Vec::new(),
        })
    }

    /// Open a channel.
    pub async fn channel(&self) -> Result<Channel> {
        todo!("")
    }
}
