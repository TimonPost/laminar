use std;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum AmethystNetworkError {
    AddConnectionToManagerFailed { err: String },
    Unknown,
}

impl fmt::Display for AmethystNetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &AmethystNetworkError::AddConnectionToManagerFailed { ref err } => write!(
                f,
                "That {} combination already existed in the manager",
                err
            ),
            &AmethystNetworkError::Unknown => write!(f, "Unknown error")
        }
    }
}
