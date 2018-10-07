// TODO: Congestion avoidance.
// To prevent congestion avoidance we need ot decide whether the connection is good or bad.
// If the network of the client is bad we do not flood the router with small packets.
// When network conditions are `Good` we send 30 packets per-second, and when network conditions are `Bad` we drop to 10 packets per-second.

/// Represents the quality of an network.
pub enum NetworkQuality {
    Good,
    Bad,
}

#[cfg(test)]
mod test {
    use net::connection::VirtualConnection;
    use std::net::ToSocketAddrs;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_BAD_HOST_IP: &'static str = "800.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_connection() {
        let mut addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT)
            .to_socket_addrs()
            .unwrap();
        let _new_conn = VirtualConnection::new(addr.next().unwrap());
    }
}