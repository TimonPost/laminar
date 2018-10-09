use std::sync::{Arc, Mutex, RwLock};
use std::net::{SocketAddr};
use std::collections::HashMap;

mod virtual_connection;
mod connection_pool;
mod quality;

pub use self::virtual_connection::VirtualConnection;
pub use self::connection_pool::ConnectionPool;
pub use self::quality::NetworkQuality;

