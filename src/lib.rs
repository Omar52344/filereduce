pub mod error;
pub mod translations;

pub mod parser {
    pub mod edifact;
    pub mod json;
    pub mod parser;
    pub mod segment;
    pub mod tokenizer;
    pub mod xml;
}

pub mod core;
pub mod serializer;

#[cfg(feature = "full")]
pub mod processor;

#[cfg(feature = "full")]
pub mod sink;

#[cfg(any(feature = "cli", feature = "full"))]
pub mod cli;

#[cfg(feature = "full")]
pub mod config;

pub mod model {
    pub mod document;
    pub mod streaming;
}
