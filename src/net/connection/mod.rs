mod connection_pool;
mod quality;
mod virtual_connection;
mod timeout_thread;

pub use self::connection_pool::ConnectionPool;
pub use self::quality::{NetworkQuality, RttMeasurer};
pub use self::virtual_connection::VirtualConnection;
pub use self::timeout_thread::TimeoutThread;

use std::sync::{Arc, RwLock};
use std::net::SocketAddr;
use std::collections::HashMap;

pub type Connection = Arc<RwLock<VirtualConnection>>;
pub type Connections = HashMap<SocketAddr, Connection>;
pub type ConnectionsCollection = Arc<RwLock<Connections>>;