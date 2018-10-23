use std::net::SocketAddr;

mod server;
mod config;
mod tcp_server;
mod udp_server;
mod tcp_client;

pub use self::tcp_server::TcpServer;
pub use self::tcp_client::TcpClient;
pub use self::udp_server::UdpServer;
pub use self::server::Server;
pub use self::config::ServerConfig;

pub trait ProtocolServer {
    /// Start receiving data.
    fn start_receiving(&mut self);
    /// This will send all data.
    fn send_all(&mut self);
    // Find client by its socket adders.
    fn find_client_by_addr(&self, addr: &SocketAddr) -> Option<&()>;
    /// Find client by its id.
    fn find_client_by_id<'a>(&self, client_id: u64) -> Option<&'a mut SocketAddr>;
}