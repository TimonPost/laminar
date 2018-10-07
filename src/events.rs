use std::sync::Arc;

use net::{VirtualConnection, NetworkQuality};

/// Events that are generated in response to a change in state of the connected client
pub enum ConnectionEvent {
    /// A new client connects. Clients are uniquely identified by the ip:port combination at this layer.
    Connected{ conn: Arc<VirtualConnection> },
    /// A client disconnects. This can be generated from the server-side intentionally disconnecting a client,
    /// or it could be from the client disconnecting.
    Disconnected{ conn: Arc<VirtualConnection> },
    /// This is generated if the server has not seen traffic from a client for a configurable amount of time.
    TimedOut{ conn: Arc<VirtualConnection> },
    /// This is generated when there is a change in the connection quality of a client.
    QualityChange{ conn: Arc<VirtualConnection>, from: NetworkQuality, to: NetworkQuality },
}

#[cfg(test)]
mod test {
    use super::ConnectionEvent;
    use net::VirtualConnection;
    use std::sync::Arc;
    use std::net::ToSocketAddrs;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_event() {
        let addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        let mut addr = addr.unwrap();
        let test_conn = Arc::new(VirtualConnection::new(addr.next().unwrap()));
        let _ = ConnectionEvent::Connected{conn: test_conn};
    }
}
