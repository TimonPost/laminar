mod connection_pool;
mod quality;
mod timeout_thread;
mod virtual_connection;

pub use self::connection_pool::ConnectionPool;
pub use self::quality::{NetworkQuality, RttMeasurer};
pub use self::timeout_thread::TimeoutThread;
pub use self::virtual_connection::VirtualConnection;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

pub type Connection = Arc<RwLock<VirtualConnection>>;
pub type Connections = HashMap<SocketAddr, Connection>;
pub type ConnectionsCollection = Arc<RwLock<Connections>>;
