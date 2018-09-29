use failure;
use std::result;

pub type Error = failure::Error;
pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum NetworkError {
    // TODO: write more informative error
    #[fail(display = "Lock posioned")]
    AddConnectionToManagerFailed,
}
