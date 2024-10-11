use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Remote,
    Unauthorized,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
