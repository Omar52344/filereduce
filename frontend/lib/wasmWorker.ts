// Types for WASM worker communication
export type WorkerOperation = 'process-edifact' | 'compress-jsonl' | 'decompress-fra' | 'ping';

export interface WorkerRequest {
  id: string;
  type: WorkerOperation;
  data: Uint8Array;
}

export interface WorkerResponse {
  id: string;
  success: boolean;
  result?: {
    data: Uint8Array | string;
    type: 'jsonl' | 'fra';
  };
  error?: string;
}

export interface ProcessResult {
  success: boolean;
  data?: Uint8Array | string;
  error?: string;
}

class WasmWorkerClient {
  private worker: Worker | null = null;
  private pendingRequests: Map<string, (response: WorkerResponse) => void> = new Map();
  private ready = false;
  private initPromise: Promise<void> | null = null;

  constructor() {
    this.initWorker();
  }

  private initWorker() {
    if (this.initPromise) return this.initPromise;
    
    this.initPromise = new Promise((resolve, reject) => {
      try {
        this.worker = new Worker('/worker.mjs', { type: 'module' });
        
        this.worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
          const { id } = event.data;
          const resolver = this.pendingRequests.get(id);
          if (resolver) {
            resolver(event.data);
            this.pendingRequests.delete(id);
          }
        };

        this.worker.onerror = (error) => {
          console.error('Worker error:', error);
          reject(error);
        };

        // Test worker readiness
        this.sendRequest({ id: 'ping', type: 'ping', data: new Uint8Array() })
          .then(() => {
            this.ready = true;
            resolve();
          })
          .catch(reject);
      } catch (error) {
        reject(error);
      }
    });

    return this.initPromise;
  }

  private async sendRequest(request: WorkerRequest): Promise<WorkerResponse> {
    await this.initPromise;
    
    return new Promise((resolve, reject) => {
      this.pendingRequests.set(request.id, resolve);
      this.worker?.postMessage(request, [request.data.buffer]);
      
      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(request.id)) {
          this.pendingRequests.delete(request.id);
          reject(new Error('Worker request timeout'));
        }
      }, 30000);
    });
  }

  async processEdifact(fileData: Uint8Array): Promise<ProcessResult> {
    const id = `edifact-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const response = await this.sendRequest({
      id,
      type: 'process-edifact',
      data: fileData
    });

    if (!response.success) {
      return { success: false, error: response.error };
    }

    return {
      success: true,
      data: response.result?.data
    };
  }

  async compressJsonl(fileData: Uint8Array): Promise<ProcessResult> {
    const id = `compress-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const response = await this.sendRequest({
      id,
      type: 'compress-jsonl',
      data: fileData
    });

    if (!response.success) {
      return { success: false, error: response.error };
    }

    return {
      success: true,
      data: response.result?.data
    };
  }

  async decompressFra(fileData: Uint8Array): Promise<ProcessResult> {
    const id = `decompress-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const response = await this.sendRequest({
      id,
      type: 'decompress-fra',
      data: fileData
    });

    if (!response.success) {
      return { success: false, error: response.error };
    }

    return {
      success: true,
      data: response.result?.data
    };
  }

  isReady(): boolean {
    return this.ready;
  }

  terminate() {
    this.worker?.terminate();
    this.worker = null;
    this.pendingRequests.clear();
    this.ready = false;
    this.initPromise = null;
  }
}

// Singleton instance
let workerClient: WasmWorkerClient | null = null;

export function getWasmWorkerClient(): WasmWorkerClient {
  if (!workerClient) {
    workerClient = new WasmWorkerClient();
  }
  return workerClient;
}

// React hook for using the WASM worker
export function useWasmWorker() {
  // Note: In a real hook, we would manage state and effects
  // For simplicity, we return the client and let components manage lifecycle
  const client = getWasmWorkerClient();
  
  return {
    client,
    isReady: client.isReady(),
    processEdifact: client.processEdifact.bind(client),
    compressJsonl: client.compressJsonl.bind(client),
    decompressFra: client.decompressFra.bind(client),
    terminate: client.terminate.bind(client)
  };
}