#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate laminar;

use std::process::exit;
use std::thread;
use std::time::{Instant, Duration};
use std::net::SocketAddr;

use clap::App;

use laminar::net;
use laminar::packet::Packet;
use laminar::infrastructure::DeliveryMethod;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(m) = matches.subcommand_matches("server") {
        process_server_subcommand(m);
    }

    if let Some(m) = matches.subcommand_matches("client") {
        process_client_subcommand(m);
    }
}

fn process_server_subcommand(m: &clap::ArgMatches<'_>) {
    let host = m.value_of("LISTEN_HOST").unwrap();
    let port = m.value_of("LISTEN_PORT").unwrap();
    let st = m.value_of("SHUTDOWN_TIMER").unwrap();

    let shutdown_timer = match st.parse::<u64>() {
        Ok(parsed_st) => {
            parsed_st
        }
        Err(_) => {
            error!("Invalid shutdown timer value");
            exit(1);
        }
    };

    let timeout = Duration::from_secs(shutdown_timer);
    let socket_addr = host.to_string() + ":" + port;
    thread::spawn(move || {
        info!("Server started");
        run_server(&socket_addr);
    });
    info!("Main thread sleeping");
    thread::sleep(timeout);
    info!("Shutting down...");
    exit(0);
}

fn process_client_subcommand(m: &clap::ArgMatches<'_>) {
    let connect_host = m.value_of("CONNECT_HOST").unwrap();
    let connect_port = m.value_of("CONNECT_PORT").unwrap();
    let listen_host = m.value_of("LISTEN_HOST").unwrap();
    let listen_port = m.value_of("LISTEN_PORT").unwrap();
    let test_name = m.value_of("TEST_TO_RUN").unwrap();
    let pps = m.value_of("PACKETS_PER_SECOND").unwrap();
    let test_duration = m.value_of("TEST_DURATION").unwrap();
    let endpoint = listen_host.to_string() + ":" + listen_port;
    let destination = connect_host.to_string() + ":" + connect_port;
    run_client(&test_name, &endpoint, &destination, &pps, &test_duration);
    exit(0);
}

fn run_server(socket_addr: &str) {
    let network_config = net::NetworkConfig::default();
    let mut udp_server = net::UdpSocket::bind(socket_addr, network_config).unwrap();
    let mut packet_throughput = 0;
    let mut packets_total_received = 0;
    let mut second_counter = Instant::now();
    loop {
        let result = udp_server.recv();
        match result {
            Ok(Some(_packet)) => {
                packets_total_received += 1;
                packet_throughput += 1;
            }
            Ok(None) => {}
            Err(e) => {
                error!("Error receiving packet: {:?}", e);
            }
        }
        if second_counter.elapsed().as_secs() >= 10 {
            second_counter = Instant::now();
            debug!("Packet throughput is: {}", packet_throughput);
            debug!("Total packets is: {}", packets_total_received);
            packet_throughput = 0;
        }
    }
}

fn run_client(test_name: &str, destination: &str, endpoint: &str, pps: &str, test_duration: &str) {
    let network_config = net::NetworkConfig::default();
    let mut client = net::UdpSocket::bind(endpoint, network_config.clone()).unwrap();
    client.set_nonblocking(true);

    // See which test we want to run
    match test_name {
        "steady-stream" => {
            test_steady_stream(&mut client, destination, pps, test_duration);
            exit(0);
        },
        _ => {
            error!("Invalid test name");
            exit(1);
        }
    }
}

// Basic test where the client sends packets at a steady rate to the server
fn test_steady_stream(client: &mut net::UdpSocket, target: &str, pps: &str, test_duration: &str) {
    info!("Beginning steady-state test");
    let data_to_send = String::from("steady-state test packet");
    let server_addr: SocketAddr = target.parse().unwrap();
    let pps = pps.parse::<u64>().unwrap();
    let test_duration = test_duration.parse::<u64>().unwrap();
    let test_duration = Duration::from_secs(test_duration);
    let test_packet = Packet::new(server_addr, data_to_send.clone().into(), DeliveryMethod::ReliableOrdered);
    let time_quantum = 1000 / pps;
    let start_time = Instant::now();
    let mut packets_sent = 0;
    loop {
        client.send(test_packet.clone());
        packets_sent += 1;
        let now = Instant::now();
        let d = now - start_time;
        if d >= test_duration {
            info!("Ending test!");
            info!("Sent: {} packets", packets_sent);
            return;
        }
        thread::sleep(Duration::from_millis(time_quantum))
    }
}
