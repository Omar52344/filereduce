// FileReduce WASM Worker (ES Module)
import initWasm, { FileReduceWasm, convert_edi_to_jsonl_simple, compress_jsonl_simple, decompress_fra_simple } from './filereduce_wasm.js';

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
    // Use async conversion for large files (> 50MB), sync for small files
    const THRESHOLD = 50 * 1024 * 1024; // 50 MB
    let resultBytes;
    
    if (fileData.length > THRESHOLD) {
      console.log(`[Worker] Large file detected (${(fileData.length / 1024 / 1024).toFixed(2)} MB), using async conversion`);
      const wasmInstance = new FileReduceWasm();
      try {
        const resultPromise = wasmInstance.convert_edi_to_jsonl_async(fileData);
        // The promise resolves to any (Uint8Array). Wait for it.
        const result = await resultPromise;
        // result should be Uint8Array
        if (result instanceof Uint8Array) {
          resultBytes = result;
        } else if (result && typeof result === 'object' && result.byteLength !== undefined) {
          resultBytes = new Uint8Array(result);
        } else {
          throw new Error(`Unexpected result type from async conversion: ${typeof result}`);
        }
      } finally {
        wasmInstance.free();
      }
    } else {
      // Use simple synchronous conversion for smaller files
      const text = new TextDecoder().decode(fileData);
      const result = convert_edi_to_jsonl_simple(text);
      resultBytes = new TextEncoder().encode(result);
    }
    
    console.log(`[Worker] Conversion complete, output size: ${resultBytes.length} bytes`);
    // Log first 100 bytes for debugging
    if (resultBytes.length > 0) {
      const hexPreview = Array.from(resultBytes.slice(0, Math.min(100, resultBytes.length)))
        .map(b => b.toString(16).padStart(2, '0'))
        .join(' ');
      console.log(`[Worker] First 100 bytes (hex): ${hexPreview}`);
      const textPreview = new TextDecoder('utf-8', { fatal: false })
        .decode(resultBytes.slice(0, Math.min(200, resultBytes.length)));
      console.log(`[Worker] First 200 chars as text: "${textPreview}"`);
    }
    
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