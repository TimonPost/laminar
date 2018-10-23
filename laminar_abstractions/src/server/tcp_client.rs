use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::io::{Result, Error, ErrorKind};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};

type MessageSender = Option<Sender<String>>;
type MessageReceiver = Option<Receiver<String>>;

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
    pub fn new(stream: TcpStream) -> Result<TcpClient> {
        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream.try_clone()?);
        let (tx, rx) = mpsc::channel();

        Ok(TcpClient {
            reader,
            writer,
            raw_stream: stream,
            tx: Some(tx),
            rx: Some(rx),
        })
    }

    /// Sets up the background loop that waits for data to be received on the rx channel that is meant to be sent to the remote client, then enters a loop to watch for input *from* the remote endpoint.
    pub fn run(client: Arc<Mutex<TcpClient>>) -> Result<()> {
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
                return Err(Error::new(ErrorKind::Other, ""))?
            }
        }
    }

    fn start_recv(client: Arc<Mutex<TcpClient>>) -> Result<()> {
        if let Ok(mut l) = client.lock() {
            match l.outgoing_loop() {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            return Err(Error::new(ErrorKind::Other, ""))?
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
    fn outgoing_loop(&mut self) -> Result<JoinHandle<()>> {
        let mut writer = match self.raw_stream.try_clone() {
            Ok(w) => w,
            Err(_e) => {
                return Err(Error::new(ErrorKind::Other, ""))?
            }
        };

        // We use take here because we can only have one copy of a receiver and we want to the thread to own it
        // The match is used because `std::option::NoneError` is still on nightly
        let rx = match self.rx.take() {
            Some(rx) => rx,
            None => {
                return Err(Error::new(ErrorKind::Other, ""))
            }
        };

        Ok(thread::spawn(move || {
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