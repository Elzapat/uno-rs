use std::fmt;
use crate::server_result::ServerError::*;

#[derive(Debug)]
pub enum ServerError {
    IoError(std::io::Error),
    ServerError(String),
    PacketError(uno::packet::PacketError),
}

pub type ServerResult<T> = Result<T, ServerError>;

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> ServerError {
        IoError(err)
    }
}

impl From<uno::packet::PacketError> for ServerError {
    fn from(err: uno::packet::PacketError) -> ServerError {
        PacketError(err)
    }
}

impl From<String> for ServerError {
    fn from(err: String) -> ServerError {
        ServerError(err)
    }
}

impl From<&str> for ServerError {
    fn from(err: &str) -> ServerError {
        ServerError(err.to_string())
    }
}
