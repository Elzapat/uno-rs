#[derive(Debug)]
pub enum UnoError {
    Disconnected,
    MessageNotBinary,
}

impl std::fmt::Display for UnoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for UnoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
