use std::io;
use std::collections::HashMap;
use std::net::TcpListener;
use std::net::{SocketAddr, ToSocketAddrs};
use std::io::{BufRead, Write};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, RwLock};
use error::AmethystNetworkError;
use std::sync::mpsc::*;

// Type alias for a thread-safe hashmap of connections
type ConnectionMap = Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<TcpClient>>>>>;
type MessageSender = Option<Box<Sender<String>>>;
type MessageReceiver = Option<Box<Receiver<String>>>;

/// Wrapper around a TcpListener
pub struct TcpServer {
    connections: ConnectionMap,
    // Control channel used to send messages to the *server* itself, *not* a specific client. Shutdown might be an example.
    rx: MessageReceiver,
    // Channel used for the server to send messages back up to the application
    tx: MessageSender,
    addr: SocketAddr,
}

impl TcpServer {
    /// Creates a new TCP server
    pub fn new(addr: SocketAddr) ->  Result<TcpServer, io::Error> {
        let tcp_server = TcpServer {
            connections: Arc::new(RwLock::new(HashMap::new())),
            rx: None,
            tx: None,
            addr: addr,
        };
        Ok(tcp_server)
    }

    /// Starts the TcpServer listening socket. When a new connection is accepted, it spawns a new thread dedicated to that client and goes back to listening for more connections.
    pub fn listen(&mut self, addr: SocketAddr) {
        // TODO: Remove unwraps once we figure out error handling
        let (tx, rx) = channel();
        self.tx = Some(Box::new(tx));
        self.rx = Some(Box::new(rx));
        thread::spawn(move || {
            let listener = TcpListener::bind(addr).unwrap();
            for stream in listener.incoming() {
                thread::spawn(|| {
                    let tcp_client = TcpClient::new(stream.unwrap());
                });
            }
        });
    }
}

/// A remote client connected via a TcpStream
pub struct TcpClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    raw_stream: TcpStream,
    tx: MessageSender,
    rx: MessageReceiver,
}

impl TcpClient {
    /// Creates and returns a TcpClient wrapper from a raw TcpStream. This is so that we can create separate BufReader and BufWriters around the stream.
    pub fn new(stream: TcpStream) -> TcpClient {
        // TODO: Handle this better
        // Note that this does not create a seperate stream for each clone, so any setting changes made on one propagates to the others.
        let reader = stream.try_clone().unwrap();
        let writer = stream.try_clone().unwrap();
        TcpClient {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            raw_stream: stream,
            tx: None,
            rx: None,
        }
    }

    /// Writes a &str to the client
    pub fn write(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => {
                match self.writer.flush() {
                    Ok(_) => {
                        true
                    }
                    Err(e) => {
                        error!("Error flushing to client: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error!("Error writing to client: {}", e);
                false
            }
        }
    }

    // Starts a thread that watches for incoming messages from the application and writes it to the client
    fn outgoing_loop(&mut self) {
        let mut writer = self.raw_stream.try_clone().unwrap();
        // We use take here because we can only have one copy of a receiver and we want to the thread to own it
        let rx = self.rx.take();
        thread::spawn(move || {
            // TODO: Remove this when error handling is figured out
            let rx = rx.unwrap();
            loop {
                match rx.recv() {
                    Ok(msg) => {
                        match writer.write_all(msg.as_bytes()) {
                            Ok(_) => {},
                            Err(_e) => {

                            }
                        };
                        match writer.flush() {
                            Ok(_) => {},
                            Err(_e) => {}
                        }
                    },
                    Err(_e) => {}
                }
            }
        });
    }

    pub fn run(&mut self) {
        self.outgoing_loop();
        let mut buf = String::new();
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    // TODO: Generate an event here with the payload?
                }
                Err(e) => {
                    error!("Error receiving: {:#?}", e);
                }
            }
        }
    }
}
