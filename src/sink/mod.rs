pub mod db;
pub mod file;

use crate::error::Result;
use crate::processor::StreamingDocument;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(untagged)]
pub enum SinkItem {
    Document(StreamingDocument),
    Raw(Value),
}

#[async_trait]
pub trait DataSink: Send + Sync {
    async fn send(&mut self, item: SinkItem) -> Result<()>;
    async fn flush(&mut self) -> Result<()>;
}
