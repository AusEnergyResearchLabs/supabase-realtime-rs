mod channel;
mod client;

pub struct RealtimeMessage<T> {
    topic: String,
    event: String,
    payload: T,
    reference: String,
    join_reference: Option<String>,
}
