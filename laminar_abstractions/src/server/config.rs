use std::time::Duration;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub tickrate: Duration,
    pub udp_addr: SocketAddr,
    pub tcp_addr: SocketAddr,
    pub enable_tcp: bool,
    pub enable_udp: bool,
}

impl Default for ServerConfig
{
    fn default() -> Self {
        ServerConfig {
            tickrate: Duration::from_millis(100),
            udp_addr: "127.0.0.1:123456".parse().unwrap(),
            tcp_addr: "127.0.0.1:123457".parse().unwrap(),
            enable_tcp: false,
            enable_udp: true
        }
    }
}