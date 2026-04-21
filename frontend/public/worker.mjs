// FileReduce WASM Worker (ES Module)
import initWasm, { convert_edi_to_jsonl_simple, compress_jsonl_simple, decompress_fra_simple } from './filereduce_wasm.js';

let wasmInitialized = false;

 // Initialize WASM module
async function init() {
  if (wasmInitialized) return;
  
  console.log('[Worker] Initializing WASM module...');
  try {
    // Load WASM with cache-busting query parameter
    const wasmUrl = './filereduce_wasm_bg.wasm?v=' + Date.now();
    await initWasm(wasmUrl);
    wasmInitialized = true;
    console.log('[Worker] WASM module initialized');
  } catch (error) {
    console.error('[Worker] WASM initialization error:', error);
    console.error('[Worker] Error stack:', error.stack);
    throw error;
  }
}

// Process EDIFACT file
async function processEdifact(fileData) {
  await init();
  
  try {
    console.log('[Worker] Converting EDIFACT to JSONL...');
    const text = new TextDecoder().decode(fileData);
    const result = convert_edi_to_jsonl_simple(text);
    const resultBytes = new TextEncoder().encode(result);
    
    return {
      success: true,
      data: resultBytes,
      type: 'jsonl'
    };
  } catch (error) {
    console.error('[Worker] EDIFACT conversion error:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

// Process JSONL compression
async function compressJsonl(fileData) {
  await init();
  
  try {
    console.log('[Worker] Compressing JSONL to .fra...');
    const result = compress_jsonl_simple(fileData);
    
    return {
      success: true,
      data: result,
      type: 'fra'
    };
  } catch (error) {
    console.error('[Worker] JSONL compression error:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

// Process .fra decompression
async function decompressFra(fileData) {
  await init();
  
  try {
    console.log('[Worker] Decompressing .fra to JSONL...');
    const result = decompress_fra_simple(fileData);
    
    return {
      success: true,
      data: result,
      type: 'jsonl'
    };
  } catch (error) {
    console.error('[Worker] .fra decompression error:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

// Handle messages from main thread
self.addEventListener('message', async (event) => {
  const { id, type, data } = event.data;
  
  try {
    let result;
    
    switch (type) {
      case 'process-edifact':
        result = await processEdifact(data);
        break;
      case 'compress-jsonl':
        result = await compressJsonl(data);
        break;
      case 'decompress-fra':
        result = await decompressFra(data);
        break;
      case 'ping':
        result = undefined;
        break;
      default:
        throw new Error(`Unknown operation: ${type}`);
    }
    
    // Send response back to main thread
    self.postMessage({
      id,
      success: true,
      result
    }, result && result.data ? [result.data.buffer] : []);
  } catch (error) {
    console.error('[Worker] Error processing message:', error);
    self.postMessage({
      id,
      success: false,
      error: error.message
    });
  }
});

console.log('[Worker] FileReduce WASM worker (ES module) loaded');