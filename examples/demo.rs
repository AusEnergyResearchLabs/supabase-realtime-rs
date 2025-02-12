use futures::StreamExt;
use supabase_realtime::{ChannelConfig, Client, Error};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let url = std::env::var("SUPABASE_URL").unwrap();
    let token = std::env::var("SUPABASE_ANON_KEY").unwrap();
    let mut client = Client::connect(format!("ws://{}/realtime/v1/websocket", url), token).await?;

    // Open a channel
    let mut channel = client
        .channel(
            "test",
            ChannelConfig {
                ..Default::default()
            },
        )
        .await?;

    // Subscribe to broadcast messages
    let mut broadcast_subscriber = channel.on_broadcast("Test message");
    tokio::spawn(async move {
        while let Some(msg) = broadcast_subscriber.next().await {
            println!("{:?}", msg);
        }
    });

    // Subscribe to presence messages
    let mut presence_subscriber = channel.on_presence();
    tokio::spawn(async move {
        while let Some(msg) = presence_subscriber.next().await {
            println!("{:?}", msg);
        }
    });

    // Start listening for messages
    channel.subscribe().await
}
