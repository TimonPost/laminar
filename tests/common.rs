use laminar::{Config, DeliveryMethod, Packet, Socket, SocketEvent};
use log::{debug, error};
use std::{
    net::SocketAddr,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// This is an test server we use to receive data from clients.
pub struct ServerMoq {
    config: Config,
    host: SocketAddr,
}

impl ServerMoq {
    pub fn new(config: Config, host: SocketAddr) -> Self {
        ServerMoq { config, host }
    }

    pub fn start_receiving(
        &mut self,
        cancellation_channel: Receiver<bool>,
        expected_payload: Vec<u8>,
    ) -> JoinHandle<u32> {
        let (mut socket, packet_sender, event_receiver) =
            Socket::bind(self.host, self.config.clone()).unwrap();

        let mut packet_throughput = 0;
        let mut packets_total_received = 0;
        let mut second_counter = Instant::now();

        thread::spawn(move || {
            let _polling_thread = thread::spawn(move || socket.start_polling());
            loop {
                match event_receiver.try_recv() {
                    Ok(SocketEvent::Packet(packet)) => {
                        debug!("Received Packet Event");
                        assert_eq!(packet.payload(), expected_payload.as_slice());
                        packets_total_received += 1;
                        packet_throughput += 1;
                        packet_sender.send(packet).unwrap();
                    }
                    _ => {
                        debug!("Received unexpected event");
                    }
                }

                match cancellation_channel.try_recv() {
                    Ok(cancelled) => {
                        debug!("Received cancel message");
                        if cancelled {
                            return packets_total_received;
                        }
                    }
                    Err(e) => {
                        error!("Received error on cancelleation channel: {:?}", e);
                    }
                }

                if second_counter.elapsed().as_secs() >= 1 {
                    // reset counter
                    second_counter = Instant::now();
                    packet_throughput = 0;
                }
            }
        })
    }

    pub fn add_client(&self, data: Vec<u8>, client_stub: ClientStub) -> JoinHandle<()> {
        let packets_to_send = client_stub.packets_to_send;
        let host = self.host;
        let data_to_send = data;
        let config = self.config.clone();
        thread::spawn(move || {
            println!("[{}] - Starting thread", client_stub.endpoint);
            let mut stopwatch = Instant::now();

            let (mut client, packet_sender, _event_receiver) =
                Socket::bind(client_stub.endpoint, config.clone()).unwrap();
            let _thread = thread::spawn(move || client.start_polling());

            println!(
                "[{}] - Binding and starting the polling thread took {} ms.",
                client_stub.endpoint,
                stopwatch.elapsed().as_millis()
            );
            stopwatch = Instant::now();

            let len = data_to_send.len();

            for _ in 0..packets_to_send {
                let send_result = packet_sender.send(Packet::new(
                    host,
                    data_to_send.clone().into_boxed_slice(),
                    client_stub.packet_delivery,
                ));

                if len <= config.fragment_size as usize {
                    send_result.is_ok();
                } else {
                    // if fragment, todo: add size assert.
                    send_result.is_ok();
                }

                //                thread::sleep(client_stub.timeout_sending);
            }

            println!(
                "[{}] - Sending the packets took {} ms.",
                client_stub.endpoint,
                stopwatch.elapsed().as_millis()
            );
        })
    }
}

pub struct ClientStub {
    timeout_sending: Duration,
    endpoint: SocketAddr,
    packets_to_send: u32,
    packet_delivery: DeliveryMethod,
}

impl ClientStub {
    pub fn new(
        timeout_sending: Duration,
        endpoint: SocketAddr,
        packets_to_send: u32,
        packet_delivery: DeliveryMethod,
    ) -> ClientStub {
        ClientStub {
            timeout_sending,
            endpoint,
            packets_to_send,
            packet_delivery,
        }
    }
}
