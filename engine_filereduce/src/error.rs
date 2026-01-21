use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Row error: {0}")]
    Row(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
