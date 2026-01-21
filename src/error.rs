use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileReduceError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid segment format: {0}")]
    InvalidSegment(String),

    #[error("Missing expected segment: {0}")]
    MissingSegment(String),

    #[error("Document is incomplete")]
    IncompleteDocument,
}

pub type Result<T> = std::result::Result<T, FileReduceError>;
