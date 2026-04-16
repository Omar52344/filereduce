use std::io;
use thiserror::Error;

#[cfg(feature = "db")]
use bb8::RunError;
#[cfg(feature = "db")]
use bb8_tiberius;
#[cfg(feature = "db")]
use tiberius;

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

    #[cfg(feature = "full")]
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Invalid segment format: {0}")]
    InvalidSegment(String),

    #[error("Missing expected segment: {0}")]
    MissingSegment(String),

    #[error("Document is incomplete")]
    IncompleteDocument,

    #[cfg(feature = "db")]
    #[error("Database error: {0}")]
    Db(#[from] tiberius::error::Error),

    #[cfg(feature = "db")]
    #[error("Connection manager error: {0}")]
    Manager(#[from] bb8_tiberius::Error),

    #[cfg(feature = "db")]
    #[error("Connection pool error: {0}")]
    Pool(#[from] RunError<bb8_tiberius::Error>),
}

pub type Result<T> = std::result::Result<T, FileReduceError>;
