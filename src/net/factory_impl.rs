use crate::config::Config;
use crate::net::{Connection, ConnectionFactory};
use crate::packet::Packet;

use super::{ConnectionImpl, VirtualConnection};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Interval before connection is banned if not accepted
const NON_ACCEPT_TIMEOUT: Duration = Duration::from_secs(1);
/// Interval how long connection is banned, after it was not accepted
const TEMPORARY_BAN_TIMEOUT: Duration = Duration::from_secs(60);

/// Provides simple DDoS protection by auto disconnecting non accepted connections, and ban them for some time.
#[derive(Debug)]
pub struct FactoryImpl {
    config: Config,
    temporary_banned: HashMap<SocketAddr, Instant>,
}

impl FactoryImpl {
    pub fn new(config: Config) -> Self {
        FactoryImpl {
            config,
            temporary_banned: Default::default(),
        }
    }

    fn should_accept(
        &mut self,
        time: Instant,
        address: SocketAddr,
        non_accepted_timeout: Option<Instant>,
    ) -> Option<ConnectionImpl> {
        if let Some(banned_until) = self.temporary_banned.get(&address).copied() {
            if non_accepted_timeout.is_none() {
                self.temporary_banned.remove(&address);
            } else if banned_until > time {
                return None;
            }
        }
        Some(ConnectionImpl {
            non_accepted_timeout,
            conn: VirtualConnection::new(address, &self.config, time),
        })
    }
}

impl ConnectionFactory for FactoryImpl {
    type Connection = ConnectionImpl;

    fn address_from_user_event<'s, 'a>(&'s self, event: &'a Packet) -> Option<&'a SocketAddr>
    where
        's: 'a,
    {
        Some(&event.addr)
    }

    /// Accepts connection if not banned.
    fn should_accept_remote(
        &mut self,
        time: Instant,
        address: SocketAddr,
        _data: &[u8],
    ) -> Option<Self::Connection> {
        self.should_accept(time, address, Some(time + NON_ACCEPT_TIMEOUT))
    }

    /// Accepts connection if not banned.
    fn should_accept_local(
        &mut self,
        time: Instant,
        address: SocketAddr,
        _event: &<Self::Connection as Connection>::UserEvent,
    ) -> Option<Self::Connection> {
        self.should_accept(time, address, None)
    }

    /// Removes addresses from ban list.
    fn update(&mut self, time: Instant, _connections: &mut HashMap<SocketAddr, Self::Connection>) {
        self.temporary_banned
            .retain(|_, banned_until| *banned_until > time);
    }

    /// Discards connection and ban it if it was banned due to non accepted timeout.
    fn should_discard(&mut self, time: Instant, connection: &Self::Connection) -> bool {
        if connection
            .non_accepted_timeout
            .map_or(false, |timeout| timeout < time)
        {
            self.temporary_banned
                .insert(connection.conn.remote_address, time + TEMPORARY_BAN_TIMEOUT);
            return true;
        }
        connection.conn.should_drop(time)
    }
}

#[cfg(test)]
mod tests {
    use super::{ConnectionFactory, FactoryImpl, NON_ACCEPT_TIMEOUT, TEMPORARY_BAN_TIMEOUT};
    use crate::packet::Packet;
    use std::net::SocketAddr;
    use std::time::{Duration, Instant};

    /// The socket address of where the server is located.
    const ADDR: &str = "127.0.0.1:10001";

    fn address() -> SocketAddr {
        ADDR.parse().unwrap()
    }

    fn user_event() -> Packet {
        Packet::unreliable(address(), Default::default())
    }

    #[test]
    fn accepting_local_connection_do_not_set_accept_timeout() {
        let mut factory = FactoryImpl::new(Default::default());

        let conn = factory
            .should_accept_local(Instant::now(), address(), &user_event())
            .unwrap();

        assert_eq!(conn.non_accepted_timeout, None);
    }

    #[test]
    fn accepting_remote_connection_sets_accept_timeout() {
        let time = Instant::now();
        let mut factory = FactoryImpl::new(Default::default());

        let conn = factory.should_accept_remote(time, address(), &[]).unwrap();

        assert_eq!(conn.non_accepted_timeout, Some(time + NON_ACCEPT_TIMEOUT));
    }

    #[test]
    fn when_non_accept_timeout_expires_connection_is_discarded_and_banned() {
        let time = Instant::now();
        let mut factory = FactoryImpl::new(Default::default());

        let conn = factory.should_accept_remote(time, address(), &[]).unwrap();
        let time = time + NON_ACCEPT_TIMEOUT + Duration::from_nanos(1);
        let is_discarded = factory.should_discard(time, &conn);
        let banned = factory.temporary_banned.iter().nth(0).unwrap();

        assert_eq!(is_discarded, true);
        assert_eq!(*banned.0, address());
        assert_eq!(*banned.1, time + TEMPORARY_BAN_TIMEOUT);
    }

    #[test]
    fn accepting_remote_while_banned_returns_none() {
        let time = Instant::now();
        let mut factory = FactoryImpl::new(Default::default());

        factory
            .temporary_banned
            .insert(address(), time + Duration::from_secs(100));
        let conn = factory.should_accept_remote(time, address(), &[]);

        assert_eq!(conn.is_none(), true);
    }

    #[test]
    fn accepting_local_while_banned_removes_ban() {
        let time = Instant::now();
        let mut factory = FactoryImpl::new(Default::default());

        factory
            .temporary_banned
            .insert(address(), time + Duration::from_secs(100));
        let conn = factory.should_accept_local(time, address(), &user_event());

        assert_eq!(conn.is_some(), true);
        assert_eq!(factory.temporary_banned.is_empty(), true);
    }
}
