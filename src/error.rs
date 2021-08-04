use chrono::NaiveDateTime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No session is currently running")]
    NoSessionRunning,
    #[error("A session is already running since {0}")]
    SessionRunning(NaiveDateTime),
    #[error("{0}")]
    Parse(#[from] chrono::format::ParseError),
    #[error("Could not find home directory")]
    HomeDirectoryNotFound,
    #[error("IO error:{0}")]
    IO(#[from] std::io::Error),
}
