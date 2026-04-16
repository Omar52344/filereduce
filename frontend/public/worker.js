// FileReduce WASM Worker
// Loads WASM module and exposes processing functions

let wasm = null;
let isInitialized = false;

// Load WASM module
async function initWasm() {
  if (isInitialized) return;
  
  console.log('[Worker] Initializing WASM module...');
  
  try {
    // Import the wasm-bindgen generated module
    // Using dynamic import with importScripts alternative
    // Since we can't use ES modules in classic workers, we use importScripts
    // The generated JS file expects to be loaded in global scope
    importScripts('/filereduce_wasm.js');
    
    // Check if the module is available
    if (typeof self.__wbg_init === 'function') {
      // Use the default init function which loads the WASM file
      const wasmModule = await self.__wbg_init({ module_or_path: '/filereduce_wasm_bg.wasm' });
      wasm = wasmModule;
      console.log('[Worker] WASM module initialized successfully');
    } else if (typeof self.initSync === 'function') {
      // Fallback: fetch WASM and init synchronously
      const response = await fetch('/filereduce_wasm_bg.wasm');
      const bytes = await response.arrayBuffer();
      wasm = self.initSync(bytes);
      console.log('[Worker] WASM module initialized synchronously');
    } else {
      throw new Error('WASM initialization functions not found');
    }
    
    isInitialized = true;
  } catch (error) {
    console.error('[Worker] Failed to initialize WASM:', error);
    throw error;
  }
}

// Process EDIFACT file
async function processEdifact(fileData) {
  await initWasm();
  
  try {
    console.log('[Worker] Converting EDIFACT to JSONL...');
    const text = new TextDecoder().decode(fileData);
    
    // Use the simple conversion function
    const result = self.convert_edi_to_jsonl_simple(text);
    
    // Convert string to Uint8Array for consistent response
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
  await initWasm();
  
  try {
    console.log('[Worker] Compressing JSONL to .fra...');
    
    // Use the simple compression function
    const result = self.compress_jsonl_simple(fileData);
    
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
  await initWasm();
  
  try {
    console.log('[Worker] Decompressing .fra to JSONL...');
    
    // Use the simple decompression function
    const result = self.decompress_fra_simple(fileData);
    
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
        result = { success: true, message: 'Worker ready' };
        break;
      default:
        throw new Error(`Unknown operation: ${type}`);
    }
    
    // Send response back to main thread
    self.postMessage({
      id,
      success: true,
      result
    }, result.data ? [result.data.buffer] : []);
  } catch (error) {
    console.error('[Worker] Error processing message:', error);
    self.postMessage({
      id,
      success: false,
      error: error.message
    });
  }
});

console.log('[Worker] FileReduce WASM worker loaded');