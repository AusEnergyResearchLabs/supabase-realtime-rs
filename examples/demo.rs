use futures::StreamExt;
use serde_json::Map;
use supabase_realtime::{BroadcastConfig, BroadcastPayload, Client};

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::init();

    let url = std::env::var("SUPABASE_URL").unwrap();
    let token = std::env::var("SUPABASE_ANON_KEY").unwrap();
    let mut client = Client::connect(format!("ws://{}/realtime/v1/websocket", url), token)
        .await
        .unwrap();

    // Open a channel
    let mut channel = client
        .on_broadcast(
            "test",
            BroadcastConfig {
                ack: true,
                self_broadcast: true,
            },
        )
        .await
        .unwrap();

    // Send a broadcast message to the channel "test"
    let mut payload = Map::new();
    payload.insert("Test message".to_owned(), "Hello world".into());
    channel
        .broadcast(BroadcastPayload {
            event: "Test message".to_string(),
            payload,
            broadcast_type: "broadcast".to_string(),
        })
        .await
        .unwrap();

    println!("Ready");

    while let Some(msg) = channel.next().await {
        println!("Got message: {:?}", msg);
    }
}
