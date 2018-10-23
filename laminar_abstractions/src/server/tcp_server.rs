//! All TCP reflated logic for getting and sending data out to the other side.

use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use std::io::{Error, Result, ErrorKind};
use net_events::NetEvent;
use super::{ProtocolServer, ServerConfig, TcpClient};

pub type Connections = Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<TcpClient>>>>>;
type MessageSender = Option<Sender<String>>;
type MessageReceiver = Option<Receiver<String>>;

pub struct TcpServer {
    tx: Sender<NetEvent>,
    rx: Receiver<NetEvent>,
    pub tcp_clients: Connections,
    pub config: ServerConfig,
}

impl TcpServer {
    pub fn new(config: ServerConfig, tx: Sender<NetEvent>, rx: Receiver<NetEvent>) -> TcpServer
    {
        let connections = Arc::new(Mutex::new(HashMap::new()));

        TcpServer { tx, rx, tcp_clients: connections, config }
    }

    /// This will start accepting connections.
    pub fn start_accepting(connections: Connections, config: ServerConfig) {

        thread::spawn(move || {
            let listener = match TcpListener::bind(config.tcp_addr) {
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
        });
    }

    /// This function inserts a reference to the connection into the connections hash
    pub fn handle_connection(stream: TcpStream, connections: Connections) -> Result<()> {
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
            Err(Error::new(ErrorKind::Other, ""))
        }
    }
}

impl ProtocolServer for TcpServer
{
    fn start_receiving(&mut self) {
        // 1. Poll all clients
        // 2. Put data into tx buffer.
        unimplemented!()
    }

    fn send_all(&mut self) {
        // 1. check rx buffer
        // 2. send data to x clients depending on the event type.
        unimplemented!()
    }

    fn find_client_by_addr(&self, addr: &SocketAddr) -> Option<&()> {
        unimplemented!()
    }

    fn find_client_by_id<'a>(&self, client_id: u64) -> Option<&'a mut SocketAddr> {
        unimplemented!()
    }
}
