use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::*;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use error::{NetworkResult, TcpErrorKind};

/* Summary of How This Works
This module has three main components:
1. The connections hash
2. The TcpServer struct
3. The TcpClient

The desired flow is:
1. Asynchronously listen for new client connections in a background thread. This thread blocks until a new connection is attempted.
2. Create a TCP client and add it to the connections hash
3. The TCP client starts a background thread that listens for incoming data, which it can then do whatever it wants with
4. The TCP client starts a background thread that listens for incoming data on the *Rust mpsc channel*, and sends it out to the client. This is an important distinction. The TCP client has incoming data from both the application (game) and the remote endpoint. How each of those send data to the client is different.
*/

// Type alias for a thread-safe hashmap of connections
type Connections = Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<TcpClient>>>>>;
type MessageSender = Option<Sender<String>>;
type MessageReceiver = Option<Receiver<String>>;

/// Container struct that keeps the hash map of connections
pub struct TcpSocketState {
    connections: Connections,
}

impl TcpSocketState {
    /// Creates and returns a new TcpSocketState
    pub fn new() -> TcpSocketState {
        TcpSocketState {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// This starts a TCP server on the provided SocketAddr. It is important to note that it also passes an Arc reference down to the server.
    pub fn start(&mut self, addr: SocketAddr) -> NetworkResult<JoinHandle<()>> {
        TcpServer::listen(addr, self.connections.clone())
    }
}

/// Wrapper around a TcpListener
pub struct TcpServer;

/// Using `self` to do deal with the threading proved to be very complicated. That is why these functions do use `self`.
impl TcpServer {
    /// Starts the TcpServer listening socket. When a new connection is accepted, it spawns a new thread dedicated to that client and goes back to listening for more connections.
    pub fn listen(addr: SocketAddr, connections: Connections) -> NetworkResult<JoinHandle<()>> {
        Ok(thread::spawn(move || {
            let listener = match TcpListener::bind(addr) {
                Ok(l) => l,
                Err(e) => {
                    error!("Error binding to listening socket: {}", e);
                    return;
                }
            };
            // This is a blocking call, so the thread waits here until it gets a new connection
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        // Now we call a function and pass it the stream, and a clone of the connections hash
                        match TcpServer::handle_connection(stream, connections.clone()) {
                            Ok(c) => {
                                debug!("New TCP connection: {:?}", c);
                            }
                            Err(e) => {
                                error!("Error accepting new TCP connection: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        debug!("Error accepting new TCP stream: {}", e);
                    }
                };
            }
        }))
    }

    /// This function inserts a reference to the connection into the connections hash
    pub fn handle_connection(stream: TcpStream, connections: Connections) -> NetworkResult<()> {
        let peer_addr = stream.peer_addr()?;
        let tmp_stream = stream.try_clone()?;
        let tcp_client = Arc::new(Mutex::new(TcpClient::new(stream)?));

        if !connections.is_poisoned() {
            if let Ok(mut locked_connections) = connections.lock() {
                locked_connections.insert(peer_addr, tcp_client.clone());
                // Pass it off to a function to handle setting up the client-specific background threads
                TcpClient::run(tcp_client)?;
                Ok(())
            } else {
                // If we can't get the lock, send a shutdown to the client and they will have to try again
                tmp_stream.shutdown(Shutdown::Both)?;
                Ok(())
            }
        } else {
            tmp_stream.shutdown(Shutdown::Both)?;
            Err(TcpErrorKind::TcpClientConnectionsHashPoisoned)?
        }
    }
}

/// A remote client connected via a TcpStream
#[derive(Debug)]
pub struct TcpClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    raw_stream: TcpStream,
    tx: MessageSender,
    rx: MessageReceiver,
}

impl TcpClient {
    /// Creates and returns a new TcpClient. It makes a few references to the raw stream and wraps them in BufReader and BufWriter for convenience.
    pub fn new(stream: TcpStream) -> NetworkResult<TcpClient> {
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);
        let (tx, rx) = channel();
        Ok(TcpClient {
            reader,
            writer,
            raw_stream: stream,
            tx: Some(tx),
            rx: Some(rx),
        })
    }

    /// Sets up the background loop that waits for data to be received on the rx channel that is meant to be sent to the remote client, then enters a loop to watch for input *from* the remote endpoint.
    pub fn run(client: Arc<Mutex<TcpClient>>) -> NetworkResult<()> {
        TcpClient::start_recv(client.clone())?;
        let mut buf = String::new();
        loop {
            if let Ok(mut l) = client.lock() {
                match l.reader.read_line(&mut buf) {
                    Ok(_) => {
                        // TODO: Generate an event here with the payload?
                    }
                    Err(e) => {
                        error!("Error receiving: {:#?}", e);
                    }
                }
            } else {
                Err(TcpErrorKind::TcpClientLockFailed)?
            }
        }
    }

    fn start_recv(client: Arc<Mutex<TcpClient>>) -> NetworkResult<()> {
        if let Ok(mut l) = client.lock() {
            match l.outgoing_loop() {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            Err(TcpErrorKind::TcpClientLockFailed)?
        }
    }

    pub fn write(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => match self.writer.flush() {
                Ok(_) => true,
                Err(e) => {
                    error!("Error flushing to client: {}", e);
                    false
                }
            },
            Err(e) => {
                error!("Error writing to client: {}", e);
                false
            }
        }
    }

    // Starts a thread that watches for incoming messages from the application and writes it to the client
    fn outgoing_loop(&mut self) -> NetworkResult<JoinHandle<()>> {
        let mut writer = match self.raw_stream.try_clone() {
            Ok(w) => w,
            Err(_e) => {
                Err(TcpErrorKind::TcpStreamCloneFailed)?
            }
        };

        // We use take here because we can only have one copy of a receiver and we want to the thread to own it
        // The match is used because `std::option::NoneError` is still on nightly
        let rx = match self.rx.take() {
            Some(rx) => rx,
            None => {
                Err(TcpErrorKind::TcpSteamFailedTakeRx)?
            }
        };

        Ok(thread::spawn(move || {
            // TODO: Remove this when error handling is figured out
            loop {
                match rx.recv() {
                    Ok(msg) => {
                        match writer.write_all(msg.as_bytes()) {
                            Ok(_) => {}
                            Err(_e) => {}
                        };
                        match writer.flush() {
                            Ok(_) => {}
                            Err(_e) => {}
                        }
                    }
                    Err(_e) => {}
                }
            }
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread;

    #[test]
    fn test_create_tcp_socket_state() {
        let _test_state = TcpSocketState::new();
    }

    #[test]
    fn test_lock_poisoning() {
        let addr: SocketAddr = ("127.0.0.1".to_string() + ":" + "27000").parse().unwrap();
        let mut test_state = TcpSocketState::new();
        test_state.start(addr);
        let test_lock = test_state.connections.clone();
        let _ = thread::spawn(move || {
            let _lock = test_lock.lock().unwrap();
            panic!();
        })
        .join();
        assert_eq!(test_state.connections.is_poisoned(), true);
    }
}
