use super::{DataSink, SinkItem};
use crate::config::IngestSection;
use crate::error::Result;
use crate::processor::StreamingDocument;
use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::Config;

pub struct DbDataSink {
    pool: Pool<ConnectionManager>,
    config: IngestSection,
    buffer: Vec<StreamingDocument>,
    sql_statement: String,
    success_count: u64,
    fail_count: u64,
}

impl DbDataSink {
    pub async fn new(config: IngestSection) -> Result<Self> {
        let manager = ConnectionManager::new(Config::from_ado_string(&config.connection_string)?);
        let pool = Pool::builder().max_size(10).build(manager).await?;

        let sql_statement = format!("EXEC {} {} = @P1", config.procedure_name, config.json_param);

        Ok(Self {
            pool,
            config,
            buffer: Vec::new(),
            sql_statement,
            success_count: 0,
            fail_count: 0,
        })
    }

    async fn flush_batch(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get().await?;

        let batch_json = serde_json::to_string(&self.buffer)?;

        match conn
            .execute(self.sql_statement.as_str(), &[&batch_json])
            .await
        {
            Ok(_) => self.success_count += self.buffer.len() as u64,
            Err(e) => {
                eprintln!(
                    "Error inserting batch of {} records: {}",
                    self.buffer.len(),
                    e
                );
                self.fail_count += self.buffer.len() as u64;
                return Err(crate::error::FileReduceError::Db(e));
            }
        }

        self.buffer.clear();
        Ok(())
    }
}

#[async_trait]
impl DataSink for DbDataSink {
    async fn send(&mut self, item: SinkItem) -> Result<()> {
        if let SinkItem::Document(doc) = item {
            self.buffer.push(doc);
            if self.buffer.len() >= self.config.batch_size {
                self.flush_batch().await?;
            }
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        self.flush_batch().await?;
        println!(
            "Ingest Summary: Success={}, Failed={}",
            self.success_count, self.fail_count
        );
        Ok(())
    }
}
