use std::{
    net::{SocketAddr, ToSocketAddrs},
    process::exit,
    thread,
    time::{Duration, Instant},
};

use clap::{load_yaml, App, AppSettings};
use crossbeam_channel::Sender;
use laminar::{Config, DeliveryMethod, Packet, Socket, SocketEvent};
use log::{debug, error, info};

fn main() {
    env_logger::init();
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

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
        Ok(parsed_st) => parsed_st,
        Err(_) => {
            error!("Invalid shutdown timer value");
            exit(1);
        }
    };

    let timeout = Duration::from_secs(shutdown_timer);
    let socket_addr = host.to_string() + ":" + port;
    thread::spawn(move || {
        info!("Server started");
        info!("Server listening on: {:?}", socket_addr);
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
    debug!("Endpoint is: {:?}", endpoint);
    debug!("Client destination is: {:?}", destination);
    run_client(&test_name, &destination, &endpoint, &pps, &test_duration);
    exit(0);
}

fn run_server(socket_addr: &str) {
    let network_config = Config::default();
    let (mut socket, _packet_sender, event_receiver) =
        Socket::bind(socket_addr, network_config).unwrap();
    let _thread = thread::spawn(move || socket.start_polling());

    let mut packet_throughput = 0;
    let mut packets_total_received = 0;
    let mut second_counter = Instant::now();
    loop {
        let result = event_receiver.recv();
        match result {
            Ok(SocketEvent::Packet(_packet)) => {
                packets_total_received += 1;
                packet_throughput += 1;
            }
            Ok(_) => {}
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
    let network_config = Config::default();
    let (mut socket, packet_sender, _event_receiver) =
        match Socket::bind(endpoint, network_config.clone()) {
            Ok((socket, sender, receiver)) => (socket, sender, receiver),
            Err(e) => {
                println!("Error binding was: {:?}", e);
                exit(1);
            }
        };
    let _thread = thread::spawn(move || socket.start_polling());

    // See which test we want to run
    match test_name {
        "steady-stream" => {
            test_steady_stream(&packet_sender, destination, pps, test_duration);
            exit(0);
        }
        _ => {
            error!("Invalid test name");
            exit(1);
        }
    }
}

// Basic test where the client sends packets at a steady rate to the server
fn test_steady_stream(sender: &Sender<Packet>, target: &str, pps: &str, test_duration: &str) {
    info!("Beginning steady-state test");
    let data_to_send = String::from("steady-state test packet");
    let server_addr: SocketAddr = target.to_socket_addrs().unwrap().next().unwrap();
    let pps = pps.parse::<u64>().unwrap();
    let test_duration = test_duration.parse::<u64>().unwrap();
    let test_duration = Duration::from_secs(test_duration);
    let test_packet = Packet::new(
        server_addr,
        data_to_send.into_bytes().into_boxed_slice(),
        DeliveryMethod::ReliableUnordered,
    );
    let time_quantum = 1000 / pps;
    let start_time = Instant::now();
    let mut packets_sent = 0;
    loop {
        sender
            .send(test_packet.clone())
            .expect("Unable to send a client packet");
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
