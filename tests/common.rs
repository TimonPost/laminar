extern crate laminar;

use laminar::{NetworkConfig, Packet, UdpSocket};
use laminar::net::constants;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};
use std::net::{SocketAddr};
use std::time::{Duration, Instant};

/// This is an test server we use to receive data from clients.
pub struct ServerMoq {
    config: NetworkConfig,
    client: Vec<ClientStub>,
    host: SocketAddr,
    non_blocking: bool,
}

impl ServerMoq {
    pub fn new(config: NetworkConfig, non_blocking: bool, host: SocketAddr) -> Self {

        ServerMoq { config, client: Vec::new(), host, non_blocking }
    }

    pub fn start_receiving(&mut self, cancellation_channel: Receiver<bool>, expected_payload: Vec<u8>) -> JoinHandle<u32>
    {
        let mut udp_socket: UdpSocket = UdpSocket::bind(self.host, self.config.clone()).unwrap();
        udp_socket.set_nonblocking(self.non_blocking);

        let mut packets_total_received = 0;
        let mut packet_throughput = 0;
        let mut packets_total_received = 0;
        let mut second_counter = Instant::now();

        thread::spawn(move || {
            loop {
                let result = udp_socket.recv();

                match result {
                    Ok(Some(packet)) => {
                        assert_eq!(packet.payload(), expected_payload.as_slice());

                        packets_total_received += 1;
                        packet_throughput += 1;
                    }
                    Ok(None) => {}
                    Err(_) => {
                        // if no packets are send we try to detect if the client has send us an notifier to stop the receive loop.
                        match cancellation_channel.try_recv() {
                            Ok(val) => if val == true { return packets_total_received },
                            Err(_) => {}
                        }
                    }
                }

                if second_counter.elapsed().as_secs() >= 1 {
                    // reset counter
                    second_counter = Instant::now();

                    println!("Packet throughput: {} p/s", packet_throughput);
                    packet_throughput = 0;
                }
            }
        })
    }

    pub fn add_client(&self, data: Vec<u8>, client_stub: ClientStub) -> JoinHandle<()>
    {
        let packets_to_send = client_stub.packets_to_send;
        let host = self.host;
        let data_to_send = data;
        let config = self.config.clone();
        thread::spawn(move || {
            let mut client = UdpSocket::bind(client_stub.endpoint, config.clone()).unwrap();

            let len = data_to_send.len();

            for i in 0..packets_to_send {
                let send_result = client.send(Packet::new(host, data_to_send.clone()));

                if len <= config.fragment_size as usize {
                    assert_eq!(send_result.unwrap(), len + constants::PACKET_HEADER_SIZE as usize);
                }else {
                    // if fragment, todo: add size assert.
                    send_result.is_ok();
                }

                thread::sleep(client_stub.timeout_sending);
            }
        })
    }
}

pub struct ClientStub
{
    timeout_sending: Duration,
    endpoint: SocketAddr,
    packets_to_send: u32
}

impl ClientStub
{
    pub fn new(timeout_sending: Duration, endpoint: SocketAddr, packets_to_send: u32) -> ClientStub
    {
        ClientStub { timeout_sending, endpoint, packets_to_send }
    }
}