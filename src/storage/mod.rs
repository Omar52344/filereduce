use async_trait::async_trait;
use bytes::Bytes;
use serde::Serialize;
use std::error::Error;
use uuid::Uuid;

#[cfg(feature = "gcs")]
pub mod gcs;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct UploadRequest {
pub file_id: Uuid,
pub file_name: String,
pub file_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadResponse {
pub upload_url: String,
pub file_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadResponse {
pub download_url: String,
pub file_name: String,
pub content_type: String,
}

#[async_trait]
pub trait Storage: Send + Sync {
/// Create a pre-signed URL for direct upload from client
async fn create_upload_url(&self, request: &UploadRequest) -> Result<UploadResponse, Box<dyn Error + Send>>;

/// Store bytes directly (for small files or server-side uploads)
async fn store_bytes(&self, file_id: Uuid, data: Bytes) -> Result<String, Box<dyn Error + Send>>;

/// Retrieve bytes (for small files or server-side downloads)
async fn retrieve_bytes(&self, file_id: Uuid) -> Result<Bytes, Box<dyn Error + Send>>;

/// Create a pre-signed URL for direct download by client
async fn create_download_url(&self, file_id: Uuid, file_name: &str, content_type: &str) -> Result<DownloadResponse, Box<dyn Error + Send>>;

/// Delete stored file
async fn delete(&self, file_id: Uuid) -> Result<(), Box<dyn Error + Send>>;
}

/// In-memory storage for development and testing
pub struct MemoryStorage {
store: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Bytes>>>,
}

impl MemoryStorage {
pub fn new() -> Self {
Self {
store: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
}
}
}

#[async_trait]
impl Storage for MemoryStorage {
async fn create_upload_url(&self, request: &UploadRequest) -> Result<UploadResponse, Box<dyn Error + Send>> {
// Memory storage doesn't support pre-signed URLs, so we return a mock URL
// The actual upload will happen via store_bytes
Ok(UploadResponse {
upload_url: format!("memory://uploads/{}", request.file_id),
file_id: request.file_id,
})
}

async fn store_bytes(&self, file_id: Uuid, data: Bytes) -> Result<String, Box<dyn Error + Send>> {
self.store.write().await.insert(file_id, data);
Ok(format!("memory://storage/{}", file_id))
}

    async fn retrieve_bytes(&self, file_id: Uuid) -> Result<Bytes, Box<dyn Error + Send>> {
        self.store.read().await
            .get(&file_id)
            .cloned()
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File {} not found", file_id))) as Box<dyn std::error::Error + Send>)
    }

async fn create_download_url(&self, file_id: Uuid, file_name: &str, content_type: &str) -> Result<DownloadResponse, Box<dyn Error + Send>> {
// Memory storage doesn't support pre-signed URLs
Ok(DownloadResponse {
download_url: format!("memory://downloads/{}", file_id),
file_name: file_name.to_string(),
content_type: content_type.to_string(),
})
}

async fn delete(&self, file_id: Uuid) -> Result<(), Box<dyn Error + Send>> {
self.store.write().await.remove(&file_id);
Ok(())
}
}

#[cfg(feature = "gcs")]
pub use gcs::GcsStorage;