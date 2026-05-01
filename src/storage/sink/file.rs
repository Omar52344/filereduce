use super::{DataSink, SinkItem};
use crate::error::Result;
use async_trait::async_trait;
use std::io::Write;

pub struct FileDataSink<W: Write + Send + Sync> {
    writer: W,
}

impl<W: Write + Send + Sync> FileDataSink<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

#[async_trait]
impl<W: Write + Send + Sync> DataSink for FileDataSink<W> {
    async fn send(&mut self, item: SinkItem) -> Result<()> {
        let json = serde_json::to_string(&item)?;
        writeln!(self.writer, "{}", json).map_err(|e| e.into())
    }

    async fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(|e| e.into())
    }
}
