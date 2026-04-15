use filereduce::serializer::EdifactSerializer;
use filereduce::translations::TranslationRegistry;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use bytes::Bytes;
use warp::{Filter, Rejection, Reply};

#[derive(Clone)]
struct AppState {
    registry: Arc<RwLock<TranslationRegistry>>,
}

#[tokio::main]
async fn main() {
    let registry = TranslationRegistry::new()
        .expect("Failed to load translations.json");
    let state = AppState {
        registry: Arc::new(RwLock::new(registry)),
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

    let routes = health
        .or(reload)
        .or(process_edifact)
        .or(process_jsonl)
        .or(decompress_fra)
        .or(convert_json_to_edi)
        .with(warp::cors().allow_any_origin())
        .with(warp::log("filereduce::api"));

    println!("API server starting on 0.0.0.0:8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn reload_translations_handler(state: AppState) -> Result<impl Reply, Rejection> {
    match TranslationRegistry::new() {
        Ok(new_registry) => {
            *state.registry.write().await = new_registry;
            Ok(warp::reply::json(&serde_json::json!({ "message": "Translations reloaded" })).into_response())
        }
        Err(e) => {
            eprintln!("Failed to reload translations: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("{}", e) })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response())
        }
    }
}

async fn process_edifact_handler(body: Bytes, state: AppState) -> Result<impl Reply, Rejection> {
    use filereduce::core::EdifactProcessor;
    use std::io::{BufReader, Cursor};

    let input = body.to_vec();
    let registry = state.registry.read().await;
    let mut processor = EdifactProcessor::with_registry(registry.clone());
    let reader = BufReader::new(Cursor::new(input));
    match processor.process_to_vec(reader) {
        Ok(output) => Ok(warp::reply::with_header(
            output,
            warp::http::header::CONTENT_TYPE,
            "application/jsonl",
        ).into_response()),
        Err(e) => {
            eprintln!("Processing error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("{}", e) })),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

async fn process_jsonl_handler(body: Bytes) -> Result<impl Reply, Rejection> {
    use filereducelib::FileReduceCompressor;
    use std::io::Cursor;

    let input = body.to_vec();
    let mut compressor = FileReduceCompressor::new();
    let mut input_cursor = Cursor::new(input);
    let mut output_cursor = Cursor::new(Vec::new());
    match compressor.compress(&mut input_cursor, &mut output_cursor) {
        Ok(_) => Ok(warp::reply::with_header(
            output_cursor.into_inner(),
            warp::http::header::CONTENT_TYPE,
            "application/octet-stream",
        ).into_response()),
        Err(e) => {
            eprintln!("Compression error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("{}", e) })),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

async fn decompress_fra_handler(body: Bytes) -> Result<impl Reply, Rejection> {
    use filereducelib::FileReduceDecompressor;
    use std::io::Cursor;

    let input = body.to_vec();
    let mut decompressor = FileReduceDecompressor::new();
    let mut input_cursor = Cursor::new(input);
    let mut output_cursor = Cursor::new(Vec::new());
    match decompressor.decompress(&mut input_cursor, &mut output_cursor) {
        Ok(_) => Ok(warp::reply::with_header(
            output_cursor.into_inner(),
            warp::http::header::CONTENT_TYPE,
            "application/jsonl",
        ).into_response()),
        Err(e) => {
            eprintln!("Decompression error: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("{}", e) })),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response())
        }
    }
}

async fn convert_json_to_edi_handler(body: Bytes) -> Result<impl Reply, Rejection> {
    use filereduce::model::streaming::StreamingDocument;
    
    let content = match String::from_utf8(body.to_vec()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Invalid UTF-8: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("Invalid UTF-8: {}", e) })),
                warp::http::StatusCode::BAD_REQUEST,
            ).into_response());
        }
    };
    
    let registry = match TranslationRegistry::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load translations: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({ "error": format!("Failed to load translations: {}", e) })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ).into_response());
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
                eprintln!("Failed to parse JSONL line: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({ "error": format!("Invalid JSONL line: {}", e) })),
                    warp::http::StatusCode::BAD_REQUEST,
                ).into_response());
            }
        };
        let edifact = match serializer.serialize_document(&doc) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Serialization error: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({ "error": format!("Serialization error: {}", e) })),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ).into_response());
            }
        };
        edifact_output.push_str(&edifact);
        edifact_output.push('\n');
    }
    
    Ok(warp::reply::with_header(
        edifact_output,
        warp::http::header::CONTENT_TYPE,
        "text/plain",
    ).into_response())
}