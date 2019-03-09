mod client;
mod server;

pub use self::client::Client;
pub use self::server::{Server, ServerEvent};

use std::net::SocketAddr;

pub fn server_addr() -> SocketAddr {
    "127.0.0.1:12345".parse().unwrap()
}

pub fn client_addr() -> SocketAddr {
    "127.0.0.1:12346".parse().unwrap()
}
