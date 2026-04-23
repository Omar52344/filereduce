
use bytes::Bytes;

use filereduce::serializer::EdifactSerializer;
use filereduce::storage::{Storage, MemoryStorage, UploadRequest};
#[cfg(feature = "gcs")]
use filereduce::storage::GcsStorage;
use filereduce::translations::TranslationRegistry;
use filereducelib::{FileReduceCompressor, FileReduceDecompressor};
use std::collections::HashMap;
use serde::Serialize;
use std::convert::Infallible;
use std::env;
use std::error::Error;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use warp::{Filter, Rejection, Reply, sse};





#[derive(Clone, Debug, Serialize)]
enum TaskStatus {
    Pending,
    Processing,
    Completed { file_id: Uuid },
    Failed { error: String },
}

#[derive(Clone, Debug, Serialize)]
struct TaskEvent {
    task_id: Uuid,
    status: TaskStatus,
    timestamp: DateTime<Utc>,
}

#[derive(Clone)]
struct AppState {
    registry: Arc<RwLock<TranslationRegistry>>,
    storage: Arc<dyn Storage>,
    tasks: Arc<RwLock<HashMap<Uuid, TaskStatus>>>,
    broadcast_tx: broadcast::Sender<TaskEvent>,
}

#[tokio::main]
async fn main() {
    match dotenv::dotenv() {
        Ok(path) => println!("Loaded .env from {:?}", path),
        Err(e) => println!("Failed to load .env: {}", e),
    }
    let registry = TranslationRegistry::new().expect("Failed to load translations.json");
    
    let storage: Arc<dyn Storage> = {
        #[cfg(feature = "gcs")]
        {
            println!("GCS_BUCKET env: {:?}", env::var("GCS_BUCKET"));
            if let Ok(bucket) = env::var("GCS_BUCKET") {
                match GcsStorage::new(bucket.clone(), None, None).await {
                    Ok(gcs_storage) => {
                        println!("Using GCS storage with bucket {}", bucket);
                        Arc::new(gcs_storage)
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize GCS storage: {}, falling back to MemoryStorage", e);
                        Arc::new(MemoryStorage::new())
                    }
                }
            } else {
                println!("Using MemoryStorage (GCS bucket not defined)");
                Arc::new(MemoryStorage::new())
            }
        }
        #[cfg(not(feature = "gcs"))]
        {
            Arc::new(MemoryStorage::new())
        }
    };

    let (broadcast_tx, _) = broadcast::channel(100);
    let state = AppState {
        registry: Arc::new(RwLock::new(registry)),
        storage,
        tasks: Arc::new(RwLock::new(HashMap::new())),
        broadcast_tx,
    };

    let health = warp::path!("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({ "status": "ok" })));

    let reload = warp::path!("reload-translations")
        .and(warp::post())
        .and(with_state(state.clone()))
        .and_then(reload_translations_handler);

    let process_edifact = warp::path!("process" / "edifact")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(with_state(state.clone()))
        .and_then(process_edifact_handler);

    let process_jsonl = warp::path!("process" / "jsonl")
        .and(warp::post())
        .and(warp::body::bytes())
        .and_then(process_jsonl_handler);

    let decompress_fra = warp::path!("decompress" / "fra")
        .and(warp::post())
        .and(warp::body::bytes())
        .and_then(decompress_fra_handler);

    let convert_json_to_edi = warp::path!("convert" / "json-to-edi")
        .and(warp::post())
        .and(warp::body::bytes())
        .and_then(convert_json_to_edi_handler);

    let upload_request = warp::path!("upload" / "request")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(upload_request_handler);

    let download = warp::path!("download" / Uuid)
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(download_handler);

    let status = warp::path!("status" / Uuid)
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(status_handler);

    let events = warp::path!("events")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(events_handler);
    let process_cloud = warp::path!("process" / "cloud" / Uuid)
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(process_cloud_handler);

    let routes = health
        .or(reload)
        .or(process_edifact)
        .or(process_jsonl)
        .or(decompress_fra)
        .or(convert_json_to_edi)
        .or(upload_request)
        .or(download)
        .or(status)
        .or(events)
        .or(process_cloud)
        .with(warp::cors().allow_any_origin())
        .with(warp::log("filereduce::api"));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap();
    println!("API server starting on 0.0.0.0:{}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn upload_request_handler(req: UploadRequest, state: AppState) -> Result<impl Reply, Rejection> {
    match state.storage.create_upload_url(&req).await {
        Ok(resp) => Ok(warp::reply::json(&resp).into_response()),
        Err(e) => {
            eprintln!("Upload request error: {}", e);
            Ok(warp::reply::with_status(warp::reply::json(&serde_json::json!({ "error": e.to_string() })), warp::http::StatusCode::INTERNAL_SERVER_ERROR).into_response())
        }
    }
}

async fn download_handler(file_id: Uuid, state: AppState) -> Result<impl Reply, Rejection> {
    match state.storage.create_download_url(file_id, "file.bin", "application/octet-stream").await {
        Ok(resp) => Ok(warp::reply::with_status(warp::reply::json(&resp), warp::http::StatusCode::OK).into_response()),
        Err(e) => {
            eprintln!("Download error: {}", e);
            Ok(warp::reply::with_status(warp::reply::json(&serde_json::json!({ "error": e.to_string() })), warp::http::StatusCode::NOT_FOUND).into_response())
        }
    }
}

async fn status_handler(task_id: Uuid, state: AppState) -> Result<impl Reply, Rejection> {
    let tasks = state.tasks.read().await;
    match tasks.get(&task_id) {
        Some(status) => Ok(warp::reply::json(status).into_response()),
        None => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "error": "Task not found" })),
            warp::http::StatusCode::NOT_FOUND,
        ).into_response()),
    }
}

async fn events_handler(state: AppState) -> Result<impl Reply, Rejection> {
    let mut rx = state.broadcast_tx.subscribe();
    let stream = async_stream::stream! {
        // Send initial connection event
            yield Ok::<sse::Event, std::convert::Infallible>(sse::Event::default().data("connected"));
        while let Ok(event) = rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(data) => yield Ok::<sse::Event, std::convert::Infallible>(sse::Event::default().data(data)),
                Err(e) => eprintln!("Failed to serialize event: {}", e),
            }
        }
    };
    Ok(sse::reply(stream))
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
struct CloudProcessRequest {
    operation: String, // "edifact", "jsonl", "fra"
}

async fn process_cloud_handler(file_id: Uuid, req: CloudProcessRequest, state: AppState) -> Result<impl Reply, Rejection> {
    let task_id = Uuid::new_v4();
    
    // Crear tarea con estado Pending
    {
        let mut tasks = state.tasks.write().await;
        tasks.insert(task_id, TaskStatus::Pending);
    }
    
    // Emitir evento de tarea creada
    let _ = state.broadcast_tx.send(TaskEvent {
        task_id,
        status: TaskStatus::Pending,
        timestamp: chrono::Utc::now(),
    });
    
    // Obtener el storage para verificar que el archivo existe
    let storage = state.storage.clone();
    let registry = state.registry.clone();
    let broadcast_tx = state.broadcast_tx.clone();
    let tasks_map = state.tasks.clone();
    
    // Spawn worker asíncrono
    tokio::spawn(async move {
        // Actualizar estado a Processing
        {
            let mut tasks = tasks_map.write().await;
            tasks.insert(task_id, TaskStatus::Processing);
        }
        
        // Emitir evento Processing
        let _ = broadcast_tx.send(TaskEvent {
            task_id,
            status: TaskStatus::Processing,
            timestamp: chrono::Utc::now(),
        });
        
        // Procesamiento real
        let result = process_file_cloud(file_id, &req.operation, storage, registry).await;
        
        match result {
            Ok(result_file_id) => {
                // Actualizar estado a Completed
                {
                    let mut tasks = tasks_map.write().await;
                    tasks.insert(task_id, TaskStatus::Completed { file_id: result_file_id });
                }
                
                // Emitir evento Completed
                let _ = broadcast_tx.send(TaskEvent {
                    task_id,
                    status: TaskStatus::Completed { file_id: result_file_id },
                    timestamp: chrono::Utc::now(),
                });
            }
            Err(e) => {
                // Actualizar estado a Failed
                {
                    let mut tasks = tasks_map.write().await;
                    tasks.insert(task_id, TaskStatus::Failed { error: e.to_string() });
                }
                
                // Emitir evento Failed
                let _ = broadcast_tx.send(TaskEvent {
                    task_id,
                    status: TaskStatus::Failed { error: e.to_string() },
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    });
    
    Ok(warp::reply::json(&serde_json::json!({
        "task_id": task_id,
        "message": "Processing started"
    })).into_response())
}

async fn process_file_cloud(
    file_id: Uuid,
    operation: &str,
    storage: Arc<dyn Storage>,
    registry: Arc<RwLock<TranslationRegistry>>,
) -> Result<Uuid, Box<dyn Error + Send>> {
    // Descargar archivo del storage
    let bytes = storage.retrieve_bytes(file_id).await?;
    
    match operation {
        "edifact" => {
            // EDIFACT -> JSONL
            let reader = std::io::Cursor::new(bytes.to_vec());
            let registry = registry.read().await;
            let mut processor = filereduce::core::EdifactProcessor::with_registry(registry.clone());
            let result = processor.process_to_vec(reader).map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
            
            // Guardar resultado en storage
            let result_file_id = Uuid::new_v4();
            storage.store_bytes(result_file_id, bytes::Bytes::from(result)).await?;
            
            Ok(result_file_id)
        }
        "jsonl" => {
            // JSONL -> .fra (compresión)
            use std::io::Cursor;
            let input = bytes.to_vec();
            let compressed = tokio::task::spawn_blocking(move || {
                let mut compressor = FileReduceCompressor::new();
                let mut input_cursor = Cursor::new(input);
                let mut output_cursor = Cursor::new(Vec::new());
                compressor
                    .compress(&mut input_cursor, &mut output_cursor)
                    .map(|_| output_cursor.into_inner())
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)
            }).await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)??;
            
            let result_file_id = Uuid::new_v4();
            storage.store_bytes(result_file_id, bytes::Bytes::from(compressed)).await?;
            
            Ok(result_file_id)
        }
        "fra" => {
            // .fra -> JSONL (descompresión)
            use std::io::Cursor;
            let input = bytes.to_vec();
            let decompressed = tokio::task::spawn_blocking(move || {
                let mut decompressor = FileReduceDecompressor::new();
                let mut input_cursor = Cursor::new(input);
                let mut output_cursor = Cursor::new(Vec::new());
                decompressor
                    .decompress(&mut input_cursor, &mut output_cursor)
                    .map(|_| output_cursor.into_inner())
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)
            }).await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)??;
            
            let result_file_id = Uuid::new_v4();
            storage.store_bytes(result_file_id, bytes::Bytes::from(decompressed)).await?;
            
            Ok(result_file_id)
        }
        _ => Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unsupported operation")) as Box<dyn Error + Send>),
    }
}

async fn reload_translations_handler(state: AppState) -> Result<impl Reply, Rejection> {
    let registry_arc = state.registry.clone();
    match tokio::task::spawn_blocking(move || TranslationRegistry::new()).await {
        Ok(Ok(new_registry)) => {
            *registry_arc.write().await = new_registry;
            Ok(
                warp::reply::json(&serde_json::json!({ "message": "Translations reloaded" }))
                    .into_response(),
            )
        }
        Ok(Err(e)) => {
            eprintln!("Failed to reload translations: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("{}", e) })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
        Err(join_err) => {
            eprintln!("Join error: {}", join_err);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": "Internal server error" })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}

async fn process_edifact_handler(body: Bytes, state: AppState) -> Result<impl Reply, Rejection> {
    use filereduce::core::EdifactProcessor;
    use std::io::{BufReader, Cursor};

    let task_id = Uuid::new_v4();
    let state = state.clone();
    {
        let mut tasks = state.tasks.write().await;
        tasks.insert(task_id, TaskStatus::Pending);
    }

    let input = body.to_vec();
    let registry = state.registry.read().await.clone();
    
    // Update to Processing
    {
        let mut tasks = state.tasks.write().await;
        tasks.insert(task_id, TaskStatus::Processing);
    }

    // Spawn blocking processing
    let result = tokio::task::spawn_blocking(move || {
        let mut processor = EdifactProcessor::with_registry(registry);
        let reader = BufReader::new(Cursor::new(input));
        processor.process_to_vec(reader)
    }).await;

    // Handle result and store output in storage
    match result {
        Ok(Ok(output)) => {
            // Store output in storage
            let file_id = Uuid::new_v4();
            match state.storage.store_bytes(file_id, output.into()).await {
                Ok(_) => {
                    // Update task status
                    let mut tasks = state.tasks.write().await;
                    tasks.insert(task_id, TaskStatus::Completed { file_id });
                    // Return 202 Accepted with task info
                    Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "task_id": task_id,
                            "status": "completed",
                            "download_url": format!("/download/{}", file_id),
                        })),
                        warp::http::StatusCode::ACCEPTED,
                    ).into_response())
                }
                Err(e) => {
                    let error_msg = {
                        let e = e;
                        format!("Storage error: {}", e)
                    };
                    eprintln!("{}", error_msg);
                    let mut tasks = state.tasks.write().await;
                    tasks.insert(task_id, TaskStatus::Failed { error: error_msg.clone() });
                    Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "task_id": task_id,
                            "status": "failed",
                            "error": error_msg,
                        })),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ).into_response())
                }
            }
        }
        Ok(Err(e)) => {
            let error_msg = {
                let e = e;
                e.to_string()
            };
            eprintln!("Processing error: {}", error_msg);
            let mut tasks = state.tasks.write().await;
            tasks.insert(task_id, TaskStatus::Failed { error: error_msg.clone() });
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "task_id": task_id,
                    "status": "failed",
                    "error": error_msg,
                })),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
        Err(join_err) => {
            eprintln!("Join error: {}", join_err);
            let error_msg = "Internal server error".to_string();
            let mut tasks = state.tasks.write().await;
            tasks.insert(task_id, TaskStatus::Failed { error: error_msg.clone() });
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "task_id": task_id,
                    "status": "failed",
                    "error": error_msg,
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response())
        }
    }
}

async fn process_jsonl_handler(body: Bytes) -> Result<impl Reply, Rejection> {
    use filereducelib::FileReduceCompressor;
    use std::io::Cursor;

    let input = body.to_vec();
    match tokio::task::spawn_blocking(move || {
        let mut compressor = FileReduceCompressor::new();
        let mut input_cursor = Cursor::new(input);
        let mut output_cursor = Cursor::new(Vec::new());
        compressor
            .compress(&mut input_cursor, &mut output_cursor)
            .map(|_| output_cursor.into_inner())
    })
    .await
    {
        Ok(Ok(output)) => Ok(warp::reply::with_header(
            output,
            warp::http::header::CONTENT_TYPE,
            "application/octet-stream",
        )
        .into_response()),
        Ok(Err(e)) => {
            eprintln!("Compression error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("{}", e) })),
                warp::http::StatusCode::BAD_REQUEST,
            )
            .into_response())
        }
        Err(join_err) => {
            eprintln!("Join error: {}", join_err);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": "Internal server error" })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}

async fn decompress_fra_handler(body: Bytes) -> Result<impl Reply, Rejection> {
    use filereducelib::FileReduceDecompressor;
    use std::io::Cursor;

    let input = body.to_vec();
    match tokio::task::spawn_blocking(move || {
        let mut decompressor = FileReduceDecompressor::new();
        let mut input_cursor = Cursor::new(input);
        let mut output_cursor = Cursor::new(Vec::new());
        decompressor
            .decompress(&mut input_cursor, &mut output_cursor)
            .map(|_| output_cursor.into_inner())
    })
    .await
    {
        Ok(Ok(output)) => Ok(warp::reply::with_header(
            output,
            warp::http::header::CONTENT_TYPE,
            "application/jsonl",
        )
        .into_response()),
        Ok(Err(e)) => {
            eprintln!("Decompression error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("{}", e) })),
                warp::http::StatusCode::BAD_REQUEST,
            )
            .into_response())
        }
        Err(join_err) => {
            eprintln!("Join error: {}", join_err);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": "Internal server error" })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}

async fn convert_json_to_edi_handler(body: Bytes) -> Result<impl Reply, Rejection> {
    use filereduce::model::streaming::StreamingDocument;

    let input = body.to_vec();
    match tokio::task::spawn_blocking(move || {
        let content = match String::from_utf8(input) {
            Ok(c) => c,
            Err(e) => {
                return Err(format!("Invalid UTF-8: {}", e));
            }
        };

        let registry = match TranslationRegistry::new() {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Failed to load translations: {}", e));
            }
        };
        let serializer = EdifactSerializer::new(registry);

        let mut edifact_output = String::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let doc: StreamingDocument = match serde_json::from_str(line) {
                Ok(d) => d,
                Err(e) => {
                    return Err(format!("Invalid JSONL line: {}", e));
                }
            };
            let edifact = match serializer.serialize_document(&doc) {
                Ok(e) => e,
                Err(e) => {
                    return Err(format!("Serialization error: {}", e));
                }
            };
            edifact_output.push_str(&edifact);
            edifact_output.push_str("\\n");
        }

        Ok(edifact_output)
    })
    .await
    {
        Ok(Ok(output)) => {
            Ok(
                warp::reply::with_header(output, warp::http::header::CONTENT_TYPE, "text/plain")
                    .into_response(),
            )
        }
        Ok(Err(e)) => {
            eprintln!("Conversion error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": e })),
                warp::http::StatusCode::BAD_REQUEST,
            )
            .into_response())
        }
        Err(join_err) => {
            eprintln!("Join error: {}", join_err);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": "Internal server error" })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}
