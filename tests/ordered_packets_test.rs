mod common;

use common::{Server, Ordering, ClientStub, client_addr};

use std::time::Duration;

#[test]
fn ordered_packets() {
    let mut server = Server::new();

    let ordering = Ordering::new(client_addr());

    let server_handle = server.start_receiving(Box::from(ordering.clone()));

    server_handle.spawn_client(ClientStub::new(Duration::from_secs(1), client_addr(), 10000), Box::from(ordering));

    server_handle.wait_until_finished();
}