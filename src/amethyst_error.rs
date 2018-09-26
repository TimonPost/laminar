use std::fmt;

pub enum AmethystNetworkError {
    AddConnectionToManagerFailed{reason: String}
}

impl fmt::Display for AmethystNetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &AmethystNetworkError::AddConnectionToManagerFailed{reason} => {
                write!(f, "That {} combination already existed in the manager", reason)
            }
        }
    }
}
