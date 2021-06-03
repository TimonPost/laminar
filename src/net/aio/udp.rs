use crate::Config;
use async_channel::TrySendError;
use async_net::{SocketAddr, UdpSocket};
use super::{Peer, Peers};
use bevy_tasks::TaskPool;
use std::convert::TryFrom;
use std::net::{Ipv4Addr, SocketAddrV4,ToSocketAddrs, UdpSocket as BlockingUdpSocket};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use tracing::{debug, error};

const CLEANUP_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug)]
pub struct UdpManager {
    peers: Arc<Peers<SocketAddr>>,
    config: Config,
    socket: UdpSocket,
    task_pool: TaskPool,
}

impl UdpManager {
    /// Binds a [UdpSocket] and starts listening on it.
    ///
    /// # Errors
    /// Returns a [std::io::Error] if it fails to bind to the provided socket addresses
    /// or start an async poll on the socket.
    ///
    /// [UdpSocket]: async_net::UdpSocket
    pub fn bind(pool: TaskPool, addrs: impl ToSocketAddrs) -> std::io::Result<Self> {
        Self::bind_with_config(pool, addrs, Config::default())
    }

    /// Binds to any local port on the system, if available.
    ///
    /// # Errors
    /// Returns a [std::io::Error] if it fails to bind to the provided socket addresses
    /// or start an async poll on the socket.
    pub fn bind_any(pool: TaskPool) -> std::io::Result<Self> {
        Self::bind_any_with_config(pool, Config::default())
    }

    /// Binds to any local port on the system, if available, with a given config.
    ///
    /// # Errors
    /// Returns a [std::io::Error] if it fails to bind to the provided socket addresses
    /// or start an async poll on the socket.
    pub fn bind_any_with_config(
        pool: TaskPool,
        config: Config
    ) -> std::io::Result<Self> {
        let loopback = Ipv4Addr::new(127, 0, 0, 1);
        let address = SocketAddrV4::new(loopback, 0);
        let blocking = BlockingUdpSocket::bind(address)?;
        let socket = UdpSocket::try_from(blocking)?;
        Ok(Self::bind_internal(pool, socket, config))
    }

    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not
    ///
    /// This function allows you to configure the socket with the passed configuration.
    ///
    /// # Errors
    /// Returns a [std::io::Error] if it fails to bind to the provided socket addresses
    /// or start an async poll on the socket.
    pub fn bind_with_config(
        pool: TaskPool,
        addrs: impl ToSocketAddrs,
        config: Config
    ) -> std::io::Result<Self> {
        let blocking = BlockingUdpSocket::bind(addrs)?;
        let socket = UdpSocket::try_from(blocking)?;
        Ok(Self::bind_internal(pool, socket, config))
    }

    fn bind_internal(pool: TaskPool, socket: UdpSocket, config: Config) -> Self {
        let peers = Arc::new(Peers::default());
        let read_buffer_len = config.receive_buffer_max_size;
        let manager = Self {
            peers: peers.clone(),
            config,
            socket: socket.clone(),
            task_pool: pool.clone(),
        };

        pool.spawn(Self::recv(
                Arc::downgrade(&peers),
                socket,
                read_buffer_len
            ))
            .detach();

        manager
    }

    /// Creates a [Peer] bound to a specific target [SocketAddr].
    ///
    /// Note this does not block or send any I/O. It simply creates the tasks for reading and
    /// sending.
    ///
    /// [Peer]: super::Peer
    /// [SocketAddr]: std::net::SocketAddr
    pub fn connect(&self, remote: SocketAddr) -> Peer {
        let peer = self.peers.create_bounded(remote, self.config.socket_event_buffer_size);
        let other = self.peers.get(&remote).unwrap().clone();
        let socket = self.socket.clone();
        let task = Self::send(other, remote, socket);
        self.task_pool.spawn(task).detach();
        peer
    }

    /// Disconnects the connection to a given [SocketAddr] if available.
    ///
    /// [SocketAddr]: std::net::SocketAddr
    pub fn disconnect(&self, addr: SocketAddr) {
        self.peers.disconnect(&addr);
    }

    async fn send(peer: Peer, target_addr: SocketAddr, socket: UdpSocket) {
        while let Ok(message) = peer.recv().await {
            if let Err(err) = socket.send_to(message.as_ref(), target_addr).await {
                error!(
                    "Error while sending message to {:?}: {:?}",
                    target_addr, err
                );
            }
        }

        if let Ok(addr) = socket.local_addr() {
            debug!(
                "Stopping sender to {} from UDP socket on {}",
                target_addr,
                addr
            );
        }
    }

    async fn recv(
        peers: Weak<Peers<SocketAddr>>,
        socket: UdpSocket,
        read_buffer_len: usize
    ) {
        let mut read_buf = vec![0u8; read_buffer_len];
        let last_flush = Instant::now();

        while let Some(peers) = peers.upgrade() {
            match socket.recv_from(&mut read_buf).await {
                Ok((len, addr)) => {
                    debug_assert!(len < read_buffer_len);
                    if let Some(peer) = peers.get(&addr) {
                        Self::forward_packet(addr, peer, &read_buf[0..len]);
                    }
                }
                Err(err) => {
                    error!("Error while receiving UDP packets: {:?}", err);
                }
            }

            // Periodically cleanup the peers.
            if Instant::now() - last_flush > CLEANUP_INTERVAL {
                peers.flush_disconnected();
            }
        }

        if let Ok(addr) = socket.local_addr() {
            debug!(
                "Stopping reciever for UDP socket on {}",
                addr
            );
        }
    }

    fn forward_packet(addr: SocketAddr, peer: Peer, data: &[u8]) {
        match peer.try_send(data.into()) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                debug!(
                    "Dropped packet due to the packet queue for {} being full",
                    addr
                );
            }
            Err(TrySendError::Closed(_)) => {
                debug!("Dropped packet for disconnected packet queue: {} ", addr);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[serial_test::serial]
    pub fn test_basic_connect() {
        const ADDR_A: &str = "127.0.0.1:10000";
        const ADDR_B: &str = "127.0.0.1:10001";
        let pool = TaskPool::new();

        let socket_a = UdpManager::bind(pool.clone(), ADDR_A).unwrap();
        let socket_b = UdpManager::bind(pool.clone(), ADDR_B).unwrap();

        let peer_a = socket_b.connect(ADDR_A.parse().unwrap());
        let peer_b = socket_a.connect(ADDR_B.parse().unwrap());

        let msg_a: Box<[u8]> = b"Hello A!"[0..].into();
        let msg_b: Box<[u8]> = b"Hello B!"[0..].into();

        peer_a.try_send(msg_b.clone()).unwrap();
        peer_b.try_send(msg_a.clone()).unwrap();

        let recv_msg_a = futures::executor::block_on(peer_a.recv()).unwrap();
        let recv_msg_b = futures::executor::block_on(peer_b.recv()).unwrap();

        assert_eq!(msg_a, recv_msg_a);
        assert_eq!(msg_b, recv_msg_b);
    }

    #[test]
    #[serial_test::serial]
    pub fn test_multiple_send() {
        const ADDR_A: &str = "127.0.0.1:10000";
        const ADDR_B: &str = "127.0.0.1:10001";
        let pool = TaskPool::new();

        let socket_a = UdpManager::bind(pool.clone(), ADDR_A).unwrap();
        let socket_b = UdpManager::bind(pool.clone(), ADDR_B).unwrap();

        let peer_a = socket_b.connect(ADDR_A.parse().unwrap());
        let peer_b = socket_a.connect(ADDR_B.parse().unwrap());

        peer_a.try_send(b"100"[0..].into()).unwrap();
        peer_a.try_send(b"101"[0..].into()).unwrap();
        peer_a.try_send(b"102"[0..].into()).unwrap();
        peer_a.try_send(b"103"[0..].into()).unwrap();
        peer_a.try_send(b"104"[0..].into()).unwrap();
        peer_a.try_send(b"105"[0..].into()).unwrap();

        assert_eq!(
            futures::executor::block_on(peer_b.recv()),
            Ok(b"100"[0..].into())
        );
        assert_eq!(
            futures::executor::block_on(peer_b.recv()),
            Ok(b"101"[0..].into())
        );
        assert_eq!(
            futures::executor::block_on(peer_b.recv()),
            Ok(b"102"[0..].into())
        );
        assert_eq!(
            futures::executor::block_on(peer_b.recv()),
            Ok(b"103"[0..].into())
        );
        assert_eq!(
            futures::executor::block_on(peer_b.recv()),
            Ok(b"104"[0..].into())
        );
        assert_eq!(
            futures::executor::block_on(peer_b.recv()),
            Ok(b"105"[0..].into())
        );
    }
}
