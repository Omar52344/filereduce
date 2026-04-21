'use client';

import { useCallback, useEffect, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import DataGrid from './DataGrid';
import Dashboard from './Dashboard';
import { getWasmWorkerClient } from '@/lib/wasmWorker';
import { useTranslation } from '@/lib/i18n/LanguageContext';

type FileType = 'edifact' | 'jsonl' | 'fra' | 'unknown';
type Operation = 'conversion' | 'compression' | 'decompression';

interface ProcessResult {
  originalSize: number;
  processedSize?: number;
  processedData?: any[]; // parsed JSONL documents
  processedBlob?: Blob;
  processedFraBlob?: Blob;
  fileName?: string;
  fileType: FileType;
  operation: Operation;
  contentType?: string;
}

export default function FileUpload() {
  const [file, setFile] = useState<File | null>(null);
  const [fileType, setFileType] = useState<FileType>('unknown');
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [workerReady, setWorkerReady] = useState(false);
  const [alsoCompressToFra, setAlsoCompressToFra] = useState(false);

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
    if (ext === 'edi' || ext === 'edifact' || ext === 'txt') return 'edifact';
    // Only EDIFACT files allowed on this page; JSONL and .fra are not accepted
    return 'unknown';
  };

  const validateEdifactContent = async (file: File): Promise<boolean> => {
    // Read first 1024 bytes to check for EDIFACT markers
    const slice = file.slice(0, Math.min(1024, file.size));
    const text = await slice.text();
    // EDIFACT files typically start with UNA, UNB, UNH, or at least contain segment terminators "'"
    // Also check for common EDIFACT service segments
    if (text.includes("'") && (text.startsWith('UNA') || text.startsWith('UNB') || text.startsWith('UNH'))) {
      return true;
    }
    // Fallback: check for segment-like patterns (three uppercase letters followed by '+')
    const segmentPattern = /^[A-Z]{3}\+/m;
    if (segmentPattern.test(text)) {
      return true;
    }
    return false;
  };

  const processWithWorker = async (file: File, fileType: FileType): Promise<ProcessResult> => {
    const client = getWasmWorkerClient();
    const arrayBuffer = await file.arrayBuffer();
    const data = new Uint8Array(arrayBuffer);

    let result;
    if (fileType === 'edifact') {
      result = await client.processEdifact(data);
    } else if (fileType === 'jsonl') {
      result = await client.compressJsonl(data);
    } else if (fileType === 'fra') {
      result = await client.decompressFra(data);
    } else {
      throw new Error('Unsupported file type');
    }

    if (!result.success) {
      throw new Error(result.error);
    }

    let processedData: any[] | undefined;
    let processedBlob: Blob | undefined;
    let processedSize: number | undefined;
    let contentType = '';

    if (fileType === 'edifact' || fileType === 'fra') {
      // Result is JSONL (Uint8Array or string)
      const jsonlData = result.data;
      const jsonlBytes = jsonlData instanceof Uint8Array ? jsonlData : new TextEncoder().encode(jsonlData as string);
      processedSize = jsonlBytes.length;
      if (processedSize === 0) {
        throw new Error('Output file is empty. The EDIFACT file may not contain valid segments.');
      }
      const text = new TextDecoder().decode(jsonlBytes);
      const lines = text.split('\n').filter(line => line.trim());
      processedData = lines.map(line => {
        try {
          return JSON.parse(line);
        } catch (e) {
          console.warn('Failed to parse JSONL line:', line);
          return null;
        }
      }).filter(obj => obj !== null);
      if (processedData.length === 0) {
        throw new Error('No valid JSONL lines produced. The EDIFACT file may not be compatible.');
      }
      contentType = 'application/jsonl';
      processedBlob = new Blob([jsonlBytes.slice()], { type: 'application/jsonl' });
    } else if (fileType === 'jsonl') {
      // Result is .fra (Uint8Array)
      processedBlob = new Blob([(result.data as any).slice()], { type: 'application/octet-stream' });
      processedSize = processedBlob.size;
      if (processedSize === 0) {
        throw new Error('Output .fra file is empty. Compression may have failed.');
      }
      contentType = 'application/octet-stream';
    }

    return {
      originalSize: file.size,
      processedSize,
      processedData,
      processedBlob,
      fileName: file.name.replace(/\.[^/.]+$/, ''),
      fileType,
      operation: fileType === 'edifact' ? 'conversion' : fileType === 'jsonl' ? 'compression' : 'decompression',
      contentType,
    };
  };

  const onDrop = useCallback((acceptedFiles: File[]) => {
    const file = acceptedFiles[0];
    if (!file) return;
    
    const detectedType = detectFileType(file.name);
    setFile(file);
    setFileType(detectedType);
    setError(null);
    setResult(null);

    // Validate EDIFACT content if extension suggests EDIFACT
    if (detectedType === 'edifact') {
      validateEdifactContent(file).then(isValid => {
        if (!isValid) {
          setError(t('home.fileInfo.unknownFileType') + ' (File does not appear to be valid EDIFACT)');
          setFile(null);
          setFileType('unknown');
        }
      }).catch(() => {
        // Ignore validation errors
      });
    }
  }, [t]);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'text/plain': ['.edi', '.edifact', '.txt'],
    },
    maxFiles: 1,
  });

  const handleProcess = async () => {
    if (!file) return;
    setProcessing(true);
    setError(null);

    // Validate file type - only EDIFACT allowed on this page
    if (fileType !== 'edifact') {
      setError(t('home.fileInfo.unknownFileType'));
      setProcessing(false);
      return;
    }

    // Determine processing method - serverless architecture, only local WASM
    if (!workerReady) {
      setError(t('errors.workerNotReady'));
      setProcessing(false);
      return;
    }
    let processedResult: ProcessResult;

    try {
      processedResult = await processWithWorker(file, fileType);


      // Optional compression to .fra for EDIFACT files
      if (alsoCompressToFra && fileType === 'edifact' && processedResult.processedBlob) {
        try {
          const jsonlBytes = new Uint8Array(await processedResult.processedBlob.arrayBuffer());
          const client = getWasmWorkerClient();
          const compressResult = await client.compressJsonl(jsonlBytes);
          if (compressResult.success && compressResult.data) {
            processedResult.processedFraBlob = new Blob([compressResult.data.slice()], { type: 'application/octet-stream' });
          }
        } catch (err) {
          console.warn('Failed to compress to .fra:', err);
          // Continue without .fra blob
        }
      }

      setResult(processedResult);
    } catch (err: any) {
      setError(err.message || 'Processing failed');
    } finally {
      setProcessing(false);
    }
  };

  const handleRemove = () => {
    setFile(null);
    setFileType('unknown');
    setResult(null);
    setError(null);
    setAlsoCompressToFra(false);
  };

  const downloadFile = (blob: Blob, filename: string) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleDownloadJSONL = () => {
    if (!result) return;
    if (result.processedBlob && result.contentType?.includes('application/jsonl')) {
      // Blob is already JSONL
      downloadFile(result.processedBlob, `${result.fileName || 'output'}.jsonl`);
    } else if (result.processedData) {
      // Convert processedData back to JSONL
      const jsonl = result.processedData.map((doc: any) => JSON.stringify(doc)).join('\n');
      const blob = new Blob([jsonl], { type: 'application/jsonl' });
      downloadFile(blob, `${result.fileName || 'output'}.jsonl`);
    }
  };

  const handleDownloadFRA = () => {
    const blob = result?.processedFraBlob || result?.processedBlob;
    if (!blob) return;
    downloadFile(blob, `${result.fileName || 'output'}.fra`);
  };

  const handleDownloadCSV = () => {
    if (!result?.processedData || result.processedData.length === 0) return;
    // Simple CSV conversion (flatten first document)
    const headers = Object.keys(result.processedData[0]);
    const csvRows = [
      headers.join(','),
      ...result.processedData.map((row: any) =>
        headers.map(header => `"${(row[header] || '').toString().replace(/"/g, '""')}"`).join(',')
      ),
    ];
    const csv = csvRows.join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    downloadFile(blob, `${result.fileName || 'output'}.csv`);
  };

  return (
    <div className="w-full max-w-4xl mx-auto p-6 space-y-6">
       <div className="text-center">
         <h1 className="text-3xl font-bold text-gray-900 dark:text-white">{t('home.title')}</h1>
         <p className="text-gray-600 dark:text-gray-300 mt-2">
           {t('home.subtitle')}
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
           <p className="text-blue-600 dark:text-blue-400">{t('home.dropzone.active')}</p>
        ) : (
          <>
             <p className="text-gray-700 dark:text-gray-300">
               {t('home.dropzone.inactive')}
             </p>
             <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
               {t('home.dropzone.supportedFormats')}
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
                 {t('home.fileInfo.type')}: {fileType} • {t('home.fileInfo.size')}: {(file.size / 1024).toFixed(2)} KB
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
            {fileType === 'edifact' && (
              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={alsoCompressToFra}
                  onChange={(e) => setAlsoCompressToFra(e.target.checked)}
                  id="compress-to-fra"
                  className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <label htmlFor="compress-to-fra" className="text-sm text-gray-700 dark:text-gray-300">
                   {t('home.processing.compressToFra')}
                </label>
              </div>
            )}
            <button
              onClick={handleProcess}
              disabled={processing || fileType === 'unknown'}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
               {processing ? t('home.processing.processingButton') : t('home.processing.processButton', { fileType: fileType.toUpperCase() })}
            </button>
            {fileType === 'unknown' && (
              <p className="text-red-600 dark:text-red-400 text-sm mt-2">
                 {t('home.fileInfo.unknownFileType')}
              </p>
            )}
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
             <h4 className="font-medium text-green-800 dark:text-green-300 text-lg mb-4">{t('home.results.complete')}</h4>
            <Dashboard
              originalSize={result.originalSize}
              processedSize={result.processedSize}
              fileType={result.fileType}
              operation={result.operation}
            />
          </div>

          {result.processedData && result.processedData.length > 0 && (
            <div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
               <h4 className="font-bold text-gray-900 dark:text-white text-lg mb-4">{t('home.results.dataPreview')}</h4>
              <DataGrid data={result.processedData} />
            </div>
          )}

          <div className="bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
             <h4 className="font-bold text-gray-900 dark:text-white text-lg mb-4">{t('home.results.downloadResults')}</h4>
            <div className="flex flex-wrap gap-3">
              {result.processedData && (
                <>
                  <button
                    onClick={handleDownloadJSONL}
                    className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
                  >
                     {t('home.results.downloadJSONL')}
                  </button>
                  <button
                    onClick={handleDownloadCSV}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                  >
                     {t('home.results.downloadCSV')}
                  </button>
                </>
              )}
              {(result.processedFraBlob || (result.processedBlob && result.contentType?.includes('application/octet-stream'))) && (
                <button
                  onClick={handleDownloadFRA}
                  className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700"
                >
                   {t('home.results.downloadFRA')}
                </button>
              )}
              {result.processedBlob && result.contentType?.includes('application/jsonl') && (
                <button
                  onClick={handleDownloadJSONL}
                  className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
                >
                   {t('home.results.downloadJSONL')}
                </button>
              )}
            </div>
            <p className="text-gray-600 dark:text-gray-400 text-sm mt-4">
               {t('home.results.originalFile', { fileName: result.fileName || 'output', fileType: result.fileType, size: Math.round(result.originalSize / 1024) })}
            </p>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-8">
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
           <h3 className="font-bold text-gray-900 dark:text-white">{t('home.features.edifactToJsonl.title')}</h3>
           <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
             {t('home.features.edifactToJsonl.description')}
           </p>
        </div>
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
           <h3 className="font-bold text-gray-900 dark:text-white">{t('home.features.jsonlFra.title')}</h3>
           <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
             {t('home.features.jsonlFra.description')}
           </p>
        </div>
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
           <h3 className="font-bold text-gray-900 dark:text-white">{t('home.features.dynamicTranslations.title')}</h3>
           <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
             {t('home.features.dynamicTranslations.description')}
           </p>
        </div>
      </div>
    </div>
  );
}