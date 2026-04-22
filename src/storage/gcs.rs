use crate::storage::{DownloadResponse, Storage, UploadRequest, UploadResponse};
use async_trait::async_trait;
use bytes::Bytes;
use google_cloud_storage::client::Client;
use google_cloud_storage::client::ClientConfig;
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{UploadObjectRequest, UploadType};
use google_cloud_storage::http::objects::upload::Media;
use google_cloud_storage::sign::{SignedURLMethod, SignedURLOptions};
use std::error::Error;
use std::time::Duration;
use uuid::Uuid;

pub struct GcsStorage {
client: Client,
bucket: String,
upload_prefix: String,
download_prefix: String,
}

impl GcsStorage {
    pub async fn new(bucket: String, upload_prefix: Option<String>, download_prefix: Option<String>) -> Result<Self, Box<dyn Error>> {
        let config = ClientConfig::default().with_auth().await.map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let client = Client::new(config);

        Ok(Self {
            client,
            bucket,
            upload_prefix: upload_prefix.unwrap_or_else(|| "uploads".to_string()),
            download_prefix: download_prefix.unwrap_or_else(|| "results".to_string()),
        })
    }

    fn upload_path(&self, file_id: &Uuid) -> String {
        format!("{}/{}", self.upload_prefix, file_id)
    }

    fn download_path(&self, file_id: &Uuid) -> String {
        format!("{}/{}", self.download_prefix, file_id)
    }
}

#[async_trait]
impl Storage for GcsStorage {
async fn create_upload_url(&self, request: &UploadRequest) -> Result<UploadResponse, Box<dyn Error + Send>> {
let object_name = self.upload_path(&request.file_id);

        let url = self.client
            .signed_url(
                &self.bucket,
                &object_name,
                None,
                None,
                SignedURLOptions {
                    method: SignedURLMethod::PUT,
                    expires: Duration::from_secs(3600), // 1 hour
                    content_type: Some("application/octet-stream".to_string()),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

Ok(UploadResponse {
upload_url: url,
file_id: request.file_id,
})
}

async fn store_bytes(&self, file_id: Uuid, data: Bytes) -> Result<String, Box<dyn Error + Send>> {
let object_name = self.upload_path(&file_id);

let req = UploadObjectRequest {
bucket: self.bucket.clone(),
..Default::default()
};

let media = Media::new(object_name.clone());

        self.client
            .upload_object(
                &req,
                data,
                &UploadType::Simple(media),
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

Ok(format!("gs://{}/{}", self.bucket, object_name))
}

async fn retrieve_bytes(&self, file_id: Uuid) -> Result<Bytes, Box<dyn Error + Send>> {
let object_name = self.upload_path(&file_id);

let req = GetObjectRequest {
bucket: self.bucket.clone(),
object: object_name,
..Default::default()
};

let range = Range(None, None); // Rango completo

        let data = self.client
            .download_object(&req, &range)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

Ok(Bytes::from(data))
}

async fn create_download_url(&self, file_id: Uuid, file_name: &str, content_type: &str) -> Result<DownloadResponse, Box<dyn Error + Send>> {
let object_name = self.download_path(&file_id);

        let url = self.client
            .signed_url(
                &self.bucket,
                &object_name,
                None,
                None,
                SignedURLOptions {
                    method: SignedURLMethod::GET,
                    expires: Duration::from_secs(3600), // 1 hour
                    content_type: Some(content_type.to_string()),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

Ok(DownloadResponse {
download_url: url,
file_name: file_name.to_string(),
content_type: content_type.to_string(),
})
}

async fn delete(&self, file_id: Uuid) -> Result<(), Box<dyn Error + Send>> {
let upload_object = self.upload_path(&file_id);
let download_object = self.download_path(&file_id);

// Try to delete both, ignore errors if objects don't exist
let req = DeleteObjectRequest { 
bucket: self.bucket.clone(), 
object: upload_object, 
..Default::default() 
};
let _ = self.client.delete_object(&req).await;

let req2 = DeleteObjectRequest { 
bucket: self.bucket.clone(), 
object: download_object, 
..Default::default() 
};
let _ = self.client.delete_object(&req2).await;

Ok(())
}
}