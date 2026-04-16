use filereduce::core::EdifactProcessor;
use filereducelib::{FileReduceCompressor, FileReduceDecompressor};
use js_sys::{Uint8Array, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::console;

#[wasm_bindgen]
pub struct FileReduceWasm {
    processor: EdifactProcessor,
}

#[wasm_bindgen]
impl FileReduceWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console::log_1(&"FileReduceWasm initialized".into());
        Self {
            processor: EdifactProcessor::new(),
        }
    }

    /// Convert EDIFACT text to JSONL string (synchronous)
    #[wasm_bindgen]
    pub fn convert_edi_to_jsonl(&mut self, edi_text: &str) -> Result<String, JsValue> {
        console::log_1(&"Converting EDIFACT to JSONL".into());
        self.processor.process_to_string(edi_text)
            .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
    }

    /// Convert EDIFACT bytes to JSONL bytes (synchronous)
    #[wasm_bindgen]
    pub fn convert_edi_to_jsonl_bytes(&mut self, edi_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        console::log_1(&"Converting EDIFACT bytes to JSONL bytes".into());
        let reader = std::io::Cursor::new(edi_bytes);
        self.processor.process_to_vec(reader)
            .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
    }

    /// Convert EDIFACT to JSONL asynchronously (for large files)
    #[wasm_bindgen]
    pub fn convert_edi_to_jsonl_async(&mut self, edi_bytes: &[u8]) -> Promise {
        let bytes = edi_bytes.to_vec();
        let mut processor = EdifactProcessor::new();
        
        future_to_promise(async move {
            // Use spawn_local or web worker? For now, just synchronous in async wrapper
            let result = processor.process_to_vec(std::io::Cursor::new(&bytes))
                .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))?;
            Ok(JsValue::from(Uint8Array::from(&result[..])))
        })
    }

    /// Compress JSONL data to .fra format (synchronous)
    #[wasm_bindgen]
    pub fn compress_to_fra(&mut self, jsonl_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        console::log_1(&"Compressing JSONL to .fra".into());
        let mut compressor = FileReduceCompressor::new();
        let mut output = std::io::Cursor::new(Vec::new());
        let input = std::io::Cursor::new(jsonl_bytes);
        compressor.compress(input, &mut output)
            .map_err(|e| JsValue::from_str(&format!("Compression error: {}", e)))?;
        Ok(output.into_inner())
    }

    /// Decompress .fra data to JSONL (synchronous)
    #[wasm_bindgen]
    pub fn decompress_from_fra(&mut self, fra_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        console::log_1(&"Decompressing .fra to JSONL".into());
        let mut decompressor = FileReduceDecompressor::new();
        let mut output = std::io::Cursor::new(Vec::new());
        let input = std::io::Cursor::new(fra_bytes);
        decompressor.decompress(input, &mut output)
            .map_err(|e| JsValue::from_str(&format!("Decompression error: {}", e)))?;
        Ok(output.into_inner())
    }

    /// Convert JSONL to EDIFACT (placeholder - not yet implemented)
    #[wasm_bindgen]
    pub fn convert_jsonl_to_edi(&mut self, _jsonl_bytes: &[u8]) -> Result<String, JsValue> {
        Err(JsValue::from_str("JSONL to EDIFACT conversion not yet implemented"))
    }
}

#[wasm_bindgen]
pub fn convert_edi_to_jsonl_simple(edi_text: &str) -> Result<String, JsValue> {
    let mut processor = EdifactProcessor::new();
    processor.process_to_string(edi_text)
        .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

#[wasm_bindgen]
pub fn compress_jsonl_simple(jsonl_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
    let mut compressor = FileReduceCompressor::new();
    let mut output = std::io::Cursor::new(Vec::new());
    let input = std::io::Cursor::new(jsonl_bytes);
    compressor.compress(input, &mut output)
        .map_err(|e| JsValue::from_str(&format!("Compression error: {}", e)))?;
    Ok(output.into_inner())
}

#[wasm_bindgen]
pub fn decompress_fra_simple(fra_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
    let mut decompressor = FileReduceDecompressor::new();
    let mut output = std::io::Cursor::new(Vec::new());
    let input = std::io::Cursor::new(fra_bytes);
    decompressor.decompress(input, &mut output)
        .map_err(|e| JsValue::from_str(&format!("Decompression error: {}", e)))?;
    Ok(output.into_inner())
}