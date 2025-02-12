use std::collections::HashMap;

use futures::StreamExt;
use supabase_realtime::{BroadcastConfig, BroadcastPayload, Client};

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::init();

    let url = std::env::var("SUPABASE_URL").unwrap();
    let token = std::env::var("SUPABASE_ANON_KEY").unwrap();
    let mut client = Client::connect(format!("ws://{}/realtime/v1/websocket", url), token)
        .await
        .unwrap();

    let mut channel = client
        .on_broadcast("test", BroadcastConfig::default())
        .await
        .unwrap();

    /*client
    .broadcast(
        "test",
        BroadcastPayload {
            event: "Wow!".to_string(),
            payload: HashMap::new(),
            broadcast_type: "broadcast".to_string(),
        },
    )
    .await
    .unwrap();*/

    println!("Ready");

    while let Some(msg) = channel.next().await {
        println!("Got message: {:?}", msg);
    }
}
