mod ordering;
mod sequencing;
mod throughput;
mod traits;
mod server;

pub use self::ordering::Ordering;
pub use self::sequencing::Sequencing;
pub use self::throughput::ThroughputMonitoring;
pub use self::traits::{PacketFactory, PacketAsserting};
pub use self::server::Server;
use std::net::SocketAddr;
use std::time::Duration;

pub fn server_addr() -> SocketAddr {
    "127.0.0.1:12345".parse().unwrap()
}

pub fn client_addr() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}


pub struct ClientStub {
    pub timeout_sending: Duration,
    pub endpoint: SocketAddr,
    pub packets_to_send: u32
}

impl ClientStub {
    pub fn new(
        timeout_sending: Duration,
        endpoint: SocketAddr,
        packets_to_send: u32,
    ) -> ClientStub {
        ClientStub {
            timeout_sending,
            endpoint,
            packets_to_send,
        }
    }
}