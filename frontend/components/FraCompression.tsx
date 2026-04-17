'use client';

import { useCallback, useEffect, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import Dashboard from './Dashboard';
import { getWasmWorkerClient } from '@/lib/wasmWorker';
import { useTranslation } from '@/lib/i18n/LanguageContext';

type FileType = 'jsonl' | 'fra';
type Operation = 'compression' | 'decompression';

interface ProcessResult {
  originalSize: number;
  processedSize?: number;
  processedBlob?: Blob;
  fileName?: string;
  fileType: FileType;
  operation: Operation;
  contentType?: string;
}

export default function FraCompression() {
  const [file, setFile] = useState<File | null>(null);
  const [fileType, setFileType] = useState<FileType>('jsonl');
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [workerReady, setWorkerReady] = useState(false);

  const { t } = useTranslation();

  useEffect(() => {
    const client = getWasmWorkerClient();
    const checkReady = () => {
      if (client.isReady()) {
        setWorkerReady(true);
      } else {
        setTimeout(checkReady, 100);
      }
    };
    checkReady();
  }, []);

  const detectFileType = (filename: string): FileType => {
    const ext = filename.split('.').pop()?.toLowerCase();
    if (ext === 'fra') return 'fra';
    // Default to jsonl for .jsonl, .json, or unknown
    return 'jsonl';
  };

  const validateJsonlContent = async (file: File): Promise<boolean> => {
    // Read first 1024 bytes to check for JSONL structure
    const slice = file.slice(0, Math.min(1024, file.size));
    const text = await slice.text();
    // JSONL files have each line as a JSON object; at least one line should be valid JSON
    const lines = text.split('\n').filter(line => line.trim());
    if (lines.length === 0) return false;
    // Check first few lines
    for (let i = 0; i < Math.min(lines.length, 3); i++) {
      try {
        JSON.parse(lines[i]);
        return true; // At least one valid JSON line found
      } catch {
        continue;
      }
    }
    return false;
  };

  const validateFraContent = async (file: File): Promise<boolean> => {
    // .fra custom format: check for magic bytes "FRA" at start
    const slice = file.slice(0, 3);
    const bytes = new Uint8Array(await slice.arrayBuffer());
    // Check if first three bytes are 'F', 'R', 'A' (ASCII)
    return bytes.length >= 3 && bytes[0] === 70 && bytes[1] === 82 && bytes[2] === 65;
  };

  const processWithWorker = async (file: File, fileType: FileType): Promise<ProcessResult> => {
    const client = getWasmWorkerClient();
    const arrayBuffer = await file.arrayBuffer();
    const data = new Uint8Array(arrayBuffer);

    let result;
    if (fileType === 'fra') {
      result = await client.decompressFra(data);
    } else {
      result = await client.compressJsonl(data);
    }

    if (!result.success) {
      throw new Error(result.error);
    }

    const processedBlob = new Blob([(result.data as any).slice()], {
      type: fileType === 'fra' ? 'application/jsonl' : 'application/octet-stream'
    });
    const processedSize = processedBlob.size;

    return {
      originalSize: file.size,
      processedSize,
      processedBlob,
      fileName: file.name.replace(/\.[^/.]+$/, ''),
      fileType,
      operation: fileType === 'fra' ? 'decompression' : 'compression',
      contentType: fileType === 'fra' ? 'application/jsonl' : 'application/octet-stream',
    };
  };

  const handleRemove = () => {
    setFile(null);
    setFileType('jsonl'); // default
    setResult(null);
    setError(null);
  };
  const onDrop = useCallback((acceptedFiles: File[]) => {
    const file = acceptedFiles[0];
    if (!file) return;
    
    const detectedType = detectFileType(file.name);
    setFile(file);
    setFileType(detectedType);
    setError(null);
    setResult(null);

    // Validate file content based on detected type
    if (detectedType === 'jsonl') {
      validateJsonlContent(file).then(isValid => {
        if (!isValid) {
          setError(t('compression.fileInfo.invalidFileType') + ' (File does not appear to be valid JSONL)');
          setFile(null);
          setFileType('jsonl'); // keep as jsonl but mark as invalid
        }
      }).catch(() => {});
    } else if (detectedType === 'fra') {
      validateFraContent(file).then(isValid => {
        if (!isValid) {
          setError(t('compression.fileInfo.invalidFileType') + ' (File does not appear to be a valid .fra archive)');
          setFile(null);
          setFileType('fra');
        }
      }).catch(() => {});
    }
  }, [t]);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'application/jsonl': ['.jsonl', '.json'],
      'application/octet-stream': ['.fra'],
    },
    maxFiles: 1,
  });

  const handleProcess = async () => {
    if (!file) return;
    setProcessing(true);
    setError(null);

    const useLocal = processingMode === 'local' && workerReady;
    let processedResult: ProcessResult;

    try {
      if (useLocal) {
        processedResult = await processWithWorker(file, fileType);
      } else {
        // Fallback to backend API
        const formData = new FormData();
        formData.append('file', file);

        let endpoint = '';
        if (fileType === 'fra') {
          endpoint = '/api/decompress/fra';
        } else {
          endpoint = '/api/process/jsonl';
        }

        const response = await fetch(`http://localhost:8080${endpoint}`, {
          method: 'POST',
          body: formData,
        });
        if (!response.ok) {
          try {
            const err = await response.json();
            throw new Error(err.error || `HTTP ${response.status}`);
          } catch {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
          }
        }

        const contentType = response.headers.get('content-type') || '';
        const processedBlob = await response.blob();
        const processedSize = processedBlob.size;

        processedResult = {
          originalSize: file.size,
          processedSize,
          processedBlob,
          fileName: file.name.replace(/\.[^/.]+$/, ''),
          fileType,
          operation: fileType === 'fra' ? 'decompression' : 'compression',
          contentType,
        };
      }

      setResult(processedResult);
    } catch (err: any) {
      setError(err.message || 'Processing failed');
    } finally {
      setProcessing(false);
    }
  };

  const downloadFile = (blob: Blob, filename: string) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleDownload = () => {
    if (!result?.processedBlob) return;
    const extension = result.operation === 'compression' ? '.fra' : '.jsonl';
    downloadFile(result.processedBlob, `${result.fileName || 'output'}${extension}`);
  };

  return (
    <div className="w-full max-w-4xl mx-auto p-6 space-y-6">
       <div className="text-center">
         <h1 className="text-3xl font-bold text-gray-900 dark:text-white">{t('compression.title')}</h1>
         <p className="text-gray-600 dark:text-gray-300 mt-2">
           {t('compression.subtitle')}
         </p>
       </div>

      <div
        {...getRootProps()}
        className={`border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-colors ${
          isDragActive
            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
            : 'border-gray-300 hover:border-gray-400 dark:border-gray-700 dark:hover:border-gray-600'
        }`}
      >
        <input {...getInputProps()} />
        {isDragActive ? (
           <p className="text-blue-600 dark:text-blue-400">{t('compression.dropzone.active')}</p>
        ) : (
          <>
             <p className="text-gray-700 dark:text-gray-300">
               {t('compression.dropzone.inactive')}
             </p>
             <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
               {t('compression.dropzone.supportedFormats')}
             </p>
          </>
        )}
      </div>

      {file && (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium text-gray-900 dark:text-white">{file.name}</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                 {t('compression.fileInfo.type')}: {fileType} • {t('compression.fileInfo.size')}: {(file.size / 1024).toFixed(2)} KB • {t('compression.fileInfo.operation')}:{' '}
                 {fileType === 'fra' ? t('compression.actions.decompressToJSONL') : t('compression.actions.compressToFRA')}
              </p>
            </div>
            <button
              onClick={handleRemove}
              className="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300"
            >
                {t('common.remove')}
            </button>
          </div>
          <div className="mt-4 space-y-3">
            {/*<div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Processing Mode:</span>
                <button
                  onClick={() => setProcessingMode(mode => mode === 'local' ? 'backend' : 'local')}
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${processingMode === 'local' ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-700'}`}
                >
                  <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${processingMode === 'local' ? 'translate-x-6' : 'translate-x-1'}`} />
                </button>
                <span className="text-sm text-gray-700 dark:text-gray-300">
                  {processingMode === 'local' ? 'Local (WASM)' : 'Backend (API)'}
                </span>
                {processingMode === 'local' && !workerReady && (
                  <span className="text-xs text-amber-600 dark:text-amber-400">(Worker loading...)</span>
                )}
                {processingMode === 'local' && workerReady && (
                  <span className="text-xs text-green-600 dark:text-green-400">✓ Worker ready</span>
                )}
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {processingMode === 'local' ? 'Processes files locally in your browser' : 'Sends files to backend server'}
              </div>
            </div>*/}
            <button
              onClick={handleProcess}
              disabled={processing}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
               {processing ? t('common.processing') : (fileType === 'fra' ? t('compression.actions.decompressToJSONL') : t('compression.actions.compressToFRA'))}
            </button>
          </div>
        </div>
      )}

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
           <h4 className="font-medium text-red-800 dark:text-red-300">{t('common.error')}</h4>
          <p className="text-red-600 dark:text-red-400">{error}</p>
        </div>
      )}

      {result && (
        <div className="space-y-6">
          <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-6">
             <h4 className="font-medium text-green-800 dark:text-green-300 text-lg mb-4">{t('compression.results.complete')}</h4>
            <Dashboard
              originalSize={result.originalSize}
              processedSize={result.processedSize}
              fileType={result.fileType}
              operation={result.operation}
            />
          </div>

          <div className="bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
             <h4 className="font-bold text-gray-900 dark:text-white text-lg mb-4">{t('compression.results.downloadResults')}</h4>
            <div className="flex flex-wrap gap-3">
              <button
                onClick={handleDownload}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700"
              >
                 {result.operation === 'compression' ? t('compression.results.downloadFRA') : t('compression.results.downloadJSONL')}
              </button>
            </div>
            <p className="text-gray-600 dark:text-gray-400 text-sm mt-4">
               {t('compression.results.originalFile', { fileName: result.fileName || 'output', fileType: result.fileType, size: Math.round(result.originalSize / 1024) })}
               {result.processedSize && (
                 <span>
                   {' • '}{t('compression.results.processedFile', { fileName: result.fileName || 'output', extension: result.operation === 'compression' ? 'fra' : 'jsonl', size: Math.round(result.processedSize / 1024) })}
                 </span>
               )}
            </p>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-8">
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
           <h3 className="font-bold text-gray-900 dark:text-white">{t('compression.features.jsonlToFra.title')}</h3>
           <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
             {t('compression.features.jsonlToFra.description')}
           </p>
        </div>
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
           <h3 className="font-bold text-gray-900 dark:text-white">{t('compression.features.fraToJsonl.title')}</h3>
           <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
             {t('compression.features.fraToJsonl.description')}
           </p>
        </div>
      </div>
    </div>
  );
}