use crate::config::Config;

use std::{self, collections::HashMap, fmt::Debug, net::SocketAddr, time::Instant};

/// Allows connection to send packet, send event and get global configuration.
pub trait ConnectionMessenger<ConnectionEvent: Debug> {
    /// Returns global configuration.
    fn config(&self) -> &Config;
    /// Sends a connection event.
    fn send_event(&mut self, event: ConnectionEvent);
    /// Sends a packet.
    fn send_packet(&mut self, address: &SocketAddr, payload: &[u8]);
}

/// Allows to implement actual connection.
/// Defines types of user and connection events that will be used by a connection.
pub trait Connection: Debug {
    /// Defines a user event type.
    type UserEvent: Debug;
    /// Defines a connection event type.
    type ConnectionEvent: Debug;

    /// Initial call with a payload, when connection is created by accepting remote packet.
    fn after_remote_accepted(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        payload: &[u8],
    );

    /// Initial call with a event, when connection is created by accepting user event.
    fn after_local_accepted(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        event: Self::UserEvent,
    );

    /// Processes a received packet: parse it and emit an event.
    fn process_packet(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        payload: &[u8],
    );

    /// Processes a received event and send a packet.
    fn process_event(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        event: Self::UserEvent,
    );

    /// Processes various connection-related tasks: resend dropped packets, send heartbeat packet, etc...
    /// This function gets called frequently.
    fn update(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
    );

    /// Last call before connection is destroyed.
    fn before_discarded(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
    );
}

/// Decides when to create and destroy connections, and provides a way for `ConnectionManager` to get connection from an user event.
pub trait ConnectionFactory: Debug {
    /// An actual connection type that is created by a factory.
    type Connection: Connection;

    /// Provides a mapping from user event to an actual physical address.
    /// If `None` is returned, event is ignored. If address doesn't exists in the active connections list, then `should_accept_local` will be invoked.
    /// Being factory method, it supports connections that are not necessary identified by `SocketAddr`.
    /// E.g. QUIC use ConnectionId to identify the connection.
    fn address_from_user_event<'s, 'a>(
        &'s self,
        event: &'a <Self::Connection as Connection>::UserEvent,
    ) -> Option<&'a SocketAddr>
    where
        's: 'a;

    /// Determines if remote connection can be accepted.
    /// If connection is accepted, then `after_remote_accepted` will be invoked on it.
    fn should_accept_remote(
        &mut self,
        time: Instant,
        address: SocketAddr,
        data: &[u8],
    ) -> Option<Self::Connection>;

    /// Determines if local connection can be accepted.
    /// If connection is accepted, then `after_remote_accepted` will be invoked on it.
    fn should_accept_local(
        &mut self,
        time: Instant,
        address: SocketAddr,
        event: &<Self::Connection as Connection>::UserEvent,
    ) -> Option<Self::Connection>;

    /// This allows to implement all sorts of things, a few examples include:
    /// * Banning a connection.
    /// * Disconnect a connection, if there are too many connections in "connecting" state.
    fn update(&mut self, time: Instant, connections: &mut HashMap<SocketAddr, Self::Connection>);

    /// Determines if connection should be discarded.
    /// If connection is discarded, then `before_discarded` will be invoked on it.
    fn should_discard(&mut self, time: Instant, connection: &Self::Connection) -> bool;
}
