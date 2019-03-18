mod client;
mod server;

pub use self::client::Client;
pub use self::server::{Server, ServerEvent};

use std::net::SocketAddr;

pub fn client_addr() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}
