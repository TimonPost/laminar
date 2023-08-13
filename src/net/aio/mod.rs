mod channel;
mod peer;
mod peers;

pub mod udp;
pub use channel::*;
pub use peer::*;
pub use peers::*;

/// Forwards all messages from one reciever to a sender until either the sender or reciever are
/// closed.
pub async fn forward<T>(
    input: async_channel::Receiver<T>,
    output: async_channel::Sender<T>,
) {
    while let Ok(message) = input.recv().await {
        if let Err(_) = output.send(message).await {
            break;
        }
    }
}
