use Error::*;

#[derive(Debug)]
pub enum UnoError {
    Disconnected
}

#[derive(Debug)]
pub enum Error {
    UnoError(UnoError),
    IoError(std::io::Error),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IoError(e) => f.write_str(&e.to_string()),
            UnoError(e) => match e {
                UnoError::Disconnected => f.write_str("Host disconnected"),
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IoError(e) => e.source(),
            UnoError(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        IoError(e)
    }
}

impl From<UnoError> for Error {
    fn from(e: UnoError) -> Error {
        UnoError(e)
    }
}
