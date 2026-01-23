use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct IngestConfig {
    pub ingest: IngestSection,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IngestSection {
    pub connection_string: String,
    pub procedure_name: String,
    pub json_param: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    1000
}
