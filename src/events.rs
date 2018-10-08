use std::sync::{Arc, RwLock};

use net::{Connection, Quality};

/// Events that are generated in response to a change in state of the connected client
pub enum Event {
    /// A new client connects. Clients are uniquely identified by the ip:port combination at this layer.
    Connected(Arc<RwLock<Connection>>),
    /// A client disconnects. This can be generated from the server-side intentionally disconnecting a client,
    /// or it could be from the client disconnecting.
    Disconnected(Arc<RwLock<Connection>>),
    /// This is generated if the server has not seen traffic from a client for a configurable amount of time.
    TimedOut(Arc<RwLock<Connection>>),
    /// This is generated when there is a change in the connection quality of a client.
    QualityChange {
        conn: Arc<RwLock<Connection>>,
        from: Quality,
        to: Quality,
    },
}

#[cfg(test)]
mod test {
    use super::Event;
    use net::Connection;
    use std::net::ToSocketAddrs;
    use std::sync::{Arc, RwLock};

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_event() {
        let addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        let mut addr = addr.unwrap();
        let test_conn = Arc::new(RwLock::new(Connection::new(addr.next().unwrap())));
        let _ = Event::Connected(test_conn);
    }
}
