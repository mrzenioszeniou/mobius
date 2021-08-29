use chrono::NaiveDateTime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No session is currently running")]
    NoSessionRunning,
    #[error("A session is already running since {0}")]
    SessionRunning(NaiveDateTime),
    #[error("{0}")]
    ChronoParse(#[from] chrono::ParseError),
    #[error("Could not find home directory")]
    HomeDirectoryNotFound,
    #[error("IO error:{0}")]
    IO(#[from] std::io::Error),
    #[error("Error:{0}")]
    Other(String),
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

impl From<&str> for Error {
    fn from(from: &str) -> Self {
        Self::Other(from.to_string())
    }
}

impl From<String> for Error {
    fn from(from: String) -> Self {
        Self::Other(from)
    }
}
