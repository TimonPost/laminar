use super::ConnectionPool;
use error::{NetworkError, NetworkErrorKind, NetworkResult};
use events::Event;
use log::error;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Default time between checks of all clients for timeouts in seconds
pub const TIMEOUT_POLL_INTERVAL: u64 = 1;

/// Thread responsible for checking out if clients have timed out.
/// Once a connection has timed out the user of this thread will be notified on the passed sender when constructing this instance.
///
/// This thread does the following:
/// 1. Gets a read lock on the HashMap containing all the connections.
/// 2. Iterate through each one.
/// 3. Check if the last time we have heard from them (received a packet from them) is greater than the amount of time considered to be a timeout.
/// 4. If they have timed out, send a notification up the stack.
pub struct TimeoutThread {
    shutdown_signal: Arc<AtomicBool>,
    poll_interval: Duration,
    timeout_check_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<Event>,
    connection_pool: Arc<ConnectionPool>,
}

impl TimeoutThread {
    pub fn new(
        events_sender: Sender<Event>,
        connection_pool: Arc<ConnectionPool>,
    ) -> TimeoutThread {
        let poll_interval = Duration::from_secs(TIMEOUT_POLL_INTERVAL);

        TimeoutThread {
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            poll_interval,
            timeout_check_thread: None,
            sender: events_sender,
            connection_pool,
        }
    }

    /// Start the timeout thread which will check for idling clients.
    ///
    /// This will return a `Receiver` on witch error messages will be send.
    pub fn start(&mut self) -> NetworkResult<Receiver<NetworkError>> {
        let connection_pool = self.connection_pool.clone();
        let poll_interval = self.poll_interval;
        let sender = self.sender.clone();
        let (tx, rx) = channel();

        let shutdown_signal = self.shutdown_signal.clone();
        let thread = thread::spawn(move || loop {
            if shutdown_signal.load(Ordering::Relaxed) {
                break;
            }

            match connection_pool.check_for_timeouts(poll_interval, &sender) {
                Ok(timed_out_clients) => {
                    for timed_out_client in timed_out_clients {
                        if let Err(e) = connection_pool.remove_connection(&timed_out_client) {
                            tx.send(e)
                                    .expect("The corresponding receiver for the error message channel has already been deallocated.");
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(e);
                }
            }

            thread::sleep(poll_interval);
        });

        self.timeout_check_thread = Some(thread);
        Ok(rx)
    }

    /// Stops the thread, note that this is an blocking call until the timeout thread fails.
    pub fn stop(&mut self) -> NetworkResult<()> {
        // Notify the thread that it should exit its loop at the next available opportunity.
        self.shutdown_signal.store(true, Ordering::Relaxed);

        // Retrieve the handler from ourselves. `take` is used to move the join handle
        // out of `TimeoutThread`; this allows us to `join` without consuming the instance
        // inside `TimeoutThread`.
        let handler = self.timeout_check_thread.take();
        if let Some(handle) = handler {
            handle
                .join()
                .map_err(|_| NetworkErrorKind::JoiningThreadFailed)?;
        }
        Ok(())
    }
}

impl Drop for TimeoutThread {
    fn drop(&mut self) {
        let result = self.stop();
        // As we are in a `drop`, we cannot return the error.
        // Instead, we use error! as a best-effort way of notifying the user
        // that there was an issue.
        if let Err(err) = result {
            error!("Error while dropping TimeoutThread: {}", err);
        }
    }
}
