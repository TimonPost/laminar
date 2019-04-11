use std::{
    clone::Clone,
    net::SocketAddr,
    process::exit,
    thread,
    time::{Duration, Instant},
};

use clap::{load_yaml, App, AppSettings};
use crossbeam_channel::Sender;
use laminar::{Config, Packet, Result, Socket, SocketEvent, ThroughputMonitoring};
use log::{debug, error, info};

fn main() {
    env_logger::init();
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    if let Some(m) = matches.subcommand_matches("server") {
        process_server_subcommand(m.to_owned());
    }
    if let Some(m) = matches.subcommand_matches("client") {
        process_client_subcommand(m.to_owned());
    }
}

struct ClientConfiguration {
    listen_host: SocketAddr,
    destination: SocketAddr,
    _run_duration: Duration,
    packet_ps: u64,
    maximal_duration: Duration,
    test_name: String,
}

impl From<clap::ArgMatches<'_>> for ClientConfiguration {
    fn from(args: clap::ArgMatches<'_>) -> Self {
        ClientConfiguration {
            listen_host: args
                .value_of("LISTEN_ADDR")
                .expect("No `LISTEN_ADDR` argument provided!")
                .parse()
                .expect("Could not parse `LISTEN_ADDR` argument!"),
            destination: args
                .value_of("CONNECT_ADDR")
                .expect("No `CONNECT_ADDR` argument provided!")
                .parse()
                .expect("Could not parse `CONNECT_ADDR` argument!"),
            _run_duration: Duration::from_secs(
                args.value_of("SHUTDOWN_TIMER")
                    .expect("No `SHUTDOWN_TIMER` argument provided!")
                    .parse()
                    .expect("Could not parse `SHUTDOWN_TIMER` argument!"),
            ),
            packet_ps: args
                .value_of("PACKETS_PER_SECOND")
                .expect("No `PACKETS_PER_SECOND` argument provided!")
                .parse()
                .expect("Could not parse `PACKETS_PER_SECOND` argument!"),
            maximal_duration: Duration::from_secs(
                args.value_of("TEST_DURATION")
                    .expect("No `TEST_DURATION` argument provided!")
                    .parse()
                    .expect("Could not parse `TEST_DURATION` argument!"),
            ),
            test_name: String::from(
                args.value_of("TEST_TO_RUN")
                    .expect("No `TEST_TO_RUN` argument provided!"),
            ),
        }
    }
}

#[derive(Clone)]
struct ServerConfiguration {
    listen_host: SocketAddr,
    run_duration: Duration,
}

impl From<clap::ArgMatches<'_>> for ServerConfiguration {
    fn from(args: clap::ArgMatches<'_>) -> Self {
        ServerConfiguration {
            listen_host: args
                .value_of("LISTEN_ADDR")
                .expect("No `LISTEN_ADDR` argument provided!")
                .parse()
                .expect("Could not parse `LISTEN_ADDR` argument!"),
            run_duration: Duration::from_secs(
                args.value_of("SHUTDOWN_TIMER")
                    .expect("No `SHUTDOWN_TIMER` argument provided!")
                    .parse()
                    .expect("Could not parse `SHUTDOWN_TIMER` argument!"),
            ),
        }
    }
}

fn process_server_subcommand(m: clap::ArgMatches<'_>) {
    let config = ServerConfiguration::from(m);

    let run_duration = config.run_duration;

    thread::spawn(move || {
        info!("Server started");
        info!("Server listening on: {:?}", config.listen_host);
        run_server(config).expect("Server should run.");
    });

    info!("Main thread sleeping");
    thread::sleep(run_duration);
    info!("Shutting down...");
    exit(0);
}

fn process_client_subcommand(m: clap::ArgMatches<'_>) {
    let client_config = ClientConfiguration::from(m);
    debug!("Endpoint is: {:?}", client_config.listen_host);
    debug!("Client destination is: {:?}", client_config.destination);
    run_client(client_config).expect("Client should run.");
    exit(0);
}

fn run_server(server_config: ServerConfiguration) -> Result<()> {
    let (mut socket, _packet_sender, event_receiver) = Socket::bind(server_config.listen_host)?;

    let _thread = thread::spawn(move || socket.start_polling());

    let mut throughput = ThroughputMonitoring::new(Duration::from_secs(1));

    loop {
        match event_receiver.recv() {
            Ok(SocketEvent::Packet(_)) => {
                throughput.tick();
            }
            Err(e) => {
                error!("Error receiving packet: {:?}", e);
            }
            _ => error!("Event not handled yet."),
        }

        info!("{}", throughput);
    }
}

fn run_client(config: ClientConfiguration) -> Result<()> {
    let (mut socket, packet_sender, _) = Socket::bind(config.listen_host)?;

    let _thread = thread::spawn(move || socket.start_polling());

    // See which test we want to run
    match config.test_name.as_str() {
        "steady-stream" => {
            test_steady_stream(&packet_sender, config);
            exit(0);
        }
        _ => {
            error!("Invalid test name");
            exit(1);
        }
    }
}

// Basic test where the client sends packets at a steady rate to the server
fn test_steady_stream(sender: &Sender<Packet>, config: ClientConfiguration) {
    info!("Beginning steady-state test");

    let test_packet = Packet::reliable_unordered(config.listen_host, config.test_name.into_bytes());

    let time_quantum = 1000 / config.packet_ps as u64;
    let start_time = Instant::now();
    let mut packets_sent = 0;

    loop {
        sender
            .send(test_packet.clone())
            .expect("Unable to send a client packet");

        packets_sent += 1;

        if start_time.elapsed() >= config.maximal_duration {
            info!("Ending test!");
            info!("Sent: {} packets", packets_sent);
            return;
        }

        thread::sleep(Duration::from_millis(time_quantum))
    }
}
