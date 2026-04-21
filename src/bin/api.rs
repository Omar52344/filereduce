use bytes::Bytes;
use filereduce::serializer::EdifactSerializer;
use filereduce::translations::TranslationRegistry;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply};

#[derive(Clone)]
struct AppState {
    registry: Arc<RwLock<TranslationRegistry>>,
}

#[tokio::main]
async fn main() {
    let registry = TranslationRegistry::new().expect("Failed to load translations.json");
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

    let input = body.to_vec();
    let registry = state.registry.read().await.clone();
    match tokio::task::spawn_blocking(move || {
        let mut processor = EdifactProcessor::with_registry(registry);
        let reader = BufReader::new(Cursor::new(input));
        processor.process_to_vec(reader)
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
            eprintln!("Processing error: {}", e);
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
            edifact_output.push('\n');
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
