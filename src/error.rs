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

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Invalid segment format: {0}")]
    InvalidSegment(String),

    #[error("Missing expected segment: {0}")]
    MissingSegment(String),

    #[error("Document is incomplete")]
    IncompleteDocument,

    #[error("Database error: {0}")]
    Db(#[from] tiberius::error::Error),

    #[error("Connection manager error: {0}")]
    Manager(#[from] bb8_tiberius::Error),

    #[error("Connection pool error: {0}")]
    Pool(#[from] bb8::RunError<bb8_tiberius::Error>),
}

pub type Result<T> = std::result::Result<T, FileReduceError>;
