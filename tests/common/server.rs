use crate::common::server_addr;
use crossbeam_channel::{Receiver, Sender, TryIter};
use laminar::{Config, Packet, Socket, SocketEvent, ThroughputMonitoring};

use log::error;
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug)]
pub enum ServerNotify {
    Shutdown,
}

#[derive(Debug)]
pub enum ServerEvent {
    Throughput(u32),
    AverageThroughput(u32),
    TotalSent(u32),
    SocketEvent(SocketEvent),
}

pub struct Server {
    throughput_monitor: ThroughputMonitoring,
}

impl Server {
    pub fn new() -> Server {
        Server {
            throughput_monitor: ThroughputMonitoring::new(Duration::from_secs(1)),
        }
    }

    pub fn start_receiving<F>(self, packet_assert: F) -> ServerHandle
    where
        F: Fn(Packet) + Send + Sized + 'static,
    {
        let (mut socket, _, packet_receiver) =
            Socket::bind(server_addr(), Config::default()).unwrap();
        let _ = thread::spawn(move || socket.start_polling());

        let (notify_tx, notify_rx) = crossbeam_channel::unbounded();
        let (events_tx, events_rx) = crossbeam_channel::unbounded();
        let mut throughput_monitor = self.throughput_monitor;

        let serve_handle = thread::spawn(move || loop {
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
                            error!("Client can not send packet {:?}", e);
                        }
                    }
                },
                Err(e) => {
                    if !e.is_empty() {
                        error!("An error has occurred: {}", e);
                    }
                }
            }

            match notify_rx.try_recv() {
                Ok(notify) => match notify {
                    ServerNotify::Shutdown => {
                        let result = || -> Result<(), crossbeam_channel::SendError<ServerEvent>> {
                            events_tx.send(ServerEvent::AverageThroughput(
                                throughput_monitor.average(),
                            ))?;
                            events_tx.send(ServerEvent::TotalSent(
                                throughput_monitor.total_measured(),
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
        });

        ServerHandle::new(serve_handle, notify_tx, events_rx)
    }
}

pub struct ServerHandle {
    server_handle: JoinHandle<()>,
    notify_tx: Sender<ServerNotify>,
    events_rx: Receiver<ServerEvent>,
}

impl ServerHandle {
    pub fn new(
        server_handle: JoinHandle<()>,
        notify_tx: Sender<ServerNotify>,
        events_rx: Receiver<ServerEvent>,
    ) -> ServerHandle {
        ServerHandle {
            server_handle,
            notify_tx,
            events_rx,
        }
    }

    pub fn shutdown(&self) {
        self.notify_tx.send(ServerNotify::Shutdown).unwrap();
    }

    pub fn wait_until_finished(self) {
        self.server_handle.join().unwrap();
    }

    pub fn iter_events(&self) -> TryIter<ServerEvent> {
        self.events_rx.try_iter()
    }
}
