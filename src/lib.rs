pub mod error;
pub mod translations;

pub mod parser {
    pub mod edifact;
    #[cfg(feature = "full")]
    pub mod json;
    pub mod parser;
    pub mod segment;
    pub mod tokenizer;
    #[cfg(feature = "full")]
    pub mod xml;
}

pub mod core;
pub mod serializer;
pub mod version_detector;

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
