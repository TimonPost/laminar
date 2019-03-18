use crossbeam_channel::{Receiver, Sender, TryIter};
use laminar::{Config, Packet, Socket, SocketEvent, ThroughputMonitoring};

use log::error;
use std::net::SocketAddr;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Enum with commands you can send to the server.
#[derive(Debug)]
pub enum ServerCommand {
    Shutdown,
}

/// Enums which events you can receive from the server.
#[derive(Debug)]
pub enum ServerEvent {
    Throughput(u32),
    AverageThroughput(u32),
    TotalSent(u32),
    SocketEvent(SocketEvent),
}

/// Represents a server which receives packets from some endpoint.
pub struct Server {
    throughput_monitor: ThroughputMonitoring,
    listening_host: SocketAddr,
}

impl Server {
    /// Constructs a new `Server` instance.
    pub fn new(listening_host: SocketAddr) -> Server {
        Server {
            throughput_monitor: ThroughputMonitoring::new(Duration::from_millis(1000)),
            listening_host,
        }
    }

    /// Start to receive packets from some endpoint.
    /// This function takes in a closure with which a packet contents will be asserted.
    pub fn start_receiving<F>(self, packet_assert: F) -> ServerHandle
    where
        F: Fn(Packet) + Send + Sized + 'static,
    {
        let (mut socket, _, packet_receiver) =
            Socket::bind(self.listening_host, Config::default()).unwrap();

        let _ = thread::spawn(move || socket.start_polling());

        let (notify_tx, notify_rx) = crossbeam_channel::unbounded();
        let (events_tx, events_rx) = crossbeam_channel::unbounded();
        let mut throughput_monitor = self.throughput_monitor;

        let serve_handle = thread::spawn(move || {
            loop {
                match packet_receiver.try_recv() {
                    Ok(result) => match result {
                        SocketEvent::Packet(p) => {
                            packet_assert(p);
                            if throughput_monitor.tick() {
                                if let Err(e) = events_tx.send(ServerEvent::Throughput(
                                    throughput_monitor.last_throughput(),
                                )) {
                                    error!("Client can not send packet {:?}", e);
                                }
                            }
                        }
                        _ => {
                            if let Err(e) = events_tx.send(ServerEvent::SocketEvent(result)) {
                                error!("Client cannot send packet {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        if !e.is_empty() {
                            error!("An error has occurred: {}", e);
                        } else {
                            // check if we received a notify to close the server.
                            match notify_rx.try_recv() {
                                Ok(notify) => match notify {
                                    ServerCommand::Shutdown => {
                                        let result = || -> Result<(), crossbeam_channel::SendError<ServerEvent>> {
                                            events_tx.send(ServerEvent::AverageThroughput(
                                                throughput_monitor.average(),
                                            ))?;
                                            events_tx.send(ServerEvent::TotalSent(
                                                throughput_monitor.total_measured_ticks(),
                                            ))?;
                                            Ok(())
                                        };

                                        if let Err(e) = result() {
                                            error!("Unable to sent an event {:?}", e);
                                        };

                                        return;
                                    }
                                },
                                Err(e) => {
                                    if !e.is_empty() {
                                        error!("Error occurred when trying to receive on notify channel");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        ServerHandle::new(serve_handle, notify_tx, events_rx, self.listening_host)
    }
}

/// Handle to the running server.
pub struct ServerHandle {
    server_handle: JoinHandle<()>,
    notify_tx: Sender<ServerCommand>,
    events_rx: Receiver<ServerEvent>,
    pub listening_host: SocketAddr,
}

impl ServerHandle {
    /// Construct a new `ServerHandle`
    pub fn new(
        server_handle: JoinHandle<()>,
        notify_tx: Sender<ServerCommand>,
        events_rx: Receiver<ServerEvent>,
        listening_host: SocketAddr,
    ) -> ServerHandle {
        ServerHandle {
            server_handle,
            notify_tx,
            events_rx,
            listening_host,
        }
    }

    /// Send the shutdown signal to the server.
    pub fn shutdown(&self) {
        self.notify_tx.send(ServerCommand::Shutdown).unwrap();
    }

    /// Wait until this server is finished, if no shutdown signal is send or no error has been thrown then this will be a blocking call.
    pub fn wait_until_finished(self) {
        self.server_handle.join();
    }

    /// Iterate over the events that have happened on the server.
    pub fn iter_events(&self) -> TryIter<ServerEvent> {
        self.events_rx.try_iter()
    }

    pub fn event_receiver(&self) -> Receiver<ServerEvent> {
        self.events_rx.clone()
    }
}
