use std::io;
use std::collections::HashMap;
use std::net::TcpListener;
use std::net::{SocketAddr, ToSocketAddrs};
use std::io::{BufRead, Write};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::thread;
use std::sync::{Arc, RwLock};
use amethyst_error::AmethystNetworkError;


type ConnectionMap = Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<TcpClient>>>>>;

pub struct TcpSocketState {
    connections: ConnectionMap
}

pub struct TcpServer {
    socket: TcpListener
}

impl TcpServer {
    pub fn bind<A: ToSocketAddrs>(addr: A) ->  Result<TcpServer, io::Error> {
        let socket = TcpListener::bind(addr)?;
        Ok(TcpServer {
            socket
        })
    }

    pub fn listen(&mut self) {
        for stream in self.socket.incoming() {
            let stream = stream.unwrap();
            thread::spawn(|| {
                let mut client = TcpClient::new(stream);
                client.run();
            });
        }
    }
}

pub struct TcpClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    raw_stream: TcpStream,
}

impl TcpClient {
    pub fn new(stream: TcpStream) -> TcpClient {
        // TODO: Handle this better
        let reader = stream.try_clone().unwrap();
        let writer = stream.try_clone().unwrap();

        TcpClient {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            raw_stream: stream,
        }
    }

    fn write(&mut self, msg: &str) -> bool {
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

    fn recv_loop(&mut self) {
        // TODO: This will need to be uncommented out and made to work with the mpsc channels we setup
        // let mut writer = self.raw_stream.try_clone().unwrap();
        // let _t = thread::spawn(move || {
        //     let chan = rx.unwrap();
        //     loop {
        //         match chan.recv() {
        //             Ok(msg) => {
        //                 match writer.write_all(msg.as_bytes()) {
        //                     Ok(_) => {},
        //                     Err(_e) => {
        //
        //                     }
        //                 };
        //                 match writer.flush() {
        //                     Ok(_) => {},
        //                     Err(_e) => {}
        //                 }
        //             },
        //             Err(_e) => {}
        //         }
        //     }
        // });
    }

    pub fn run(&mut self) {
        self.recv_loop();
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
