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

// Cloud processing types
interface UploadRequest {
  file_id: string;
  file_name: string;
  file_size: number;
}

interface UploadResponse {
  upload_url: string;
  file_id: string;
}

interface CloudProcessRequest {
  operation: string;
}

interface CloudProcessResponse {
  task_id: string;
}

interface ProcessingTask {
  id: string;
  file_name: string;
  file_size: number;
  status: 'Pending' | 'Processing' | 'Completed' | 'Failed';
  created_at: string;
  updated_at: string;
  result_url?: string;
  error_message?: string;
}

export default function FileUpload() {
  const [file, setFile] = useState<File | null>(null);
  const [fileType, setFileType] = useState<FileType>('unknown');
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [workerReady, setWorkerReady] = useState(false);
  const [alsoCompressToFra, setAlsoCompressToFra] = useState(false);
  const [processingMode, setProcessingMode] = useState<'local' | 'cloud'>('local');
  const [cloudProcessing, setCloudProcessing] = useState(false);
  const [cloudStatus, setCloudStatus] = useState<string>('');
  const [cloudProgress, setCloudProgress] = useState<number>(0);
  const MAX_COMPRESSION_SIZE = 50 * 1024 * 1024; // 50 MB limit for compression
  const CLOUD_THRESHOLD_BYTES = 100 * 1024 * 1024; // 100 MB threshold for cloud processing

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

  useEffect(() => {
    if (file) {
      const mode = file.size >= CLOUD_THRESHOLD_BYTES ? 'cloud' : 'local';
      setProcessingMode(mode);
    } else {
      setProcessingMode('local');
    }
  }, [file]);

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

  const parseJsonlPartial = (
    jsonlBytes: Uint8Array,
    maxDocuments: number = 1000
  ): { documents: any[]; totalLines: number; linesProcessed: number; jsonParseErrors: number } => {
    const decoder = new TextDecoder('utf-8', { fatal: false });
    const documents: any[] = [];
    let lineStart = 0;
    let totalLines = 0;
    let linesProcessed = 0;
    let linesWithContent = 0;
    let jsonParseErrors = 0;
    
    console.log(`[parseJsonlPartial] Starting parse of ${jsonlBytes.length} bytes`);
    const MAX_SCAN_BYTES = 10 * 1024 * 1024; // Limit scanning to first 10MB for performance
    const scanLimit = Math.min(jsonlBytes.length, MAX_SCAN_BYTES);
    
    for (let i = 0; i < scanLimit && documents.length < maxDocuments; i++) {
      if (jsonlBytes[i] === 0x0A) { // newline character
        totalLines++;
        if (lineStart < i) {
          // Check if previous character is carriage return
          const end = jsonlBytes[i - 1] === 0x0D ? i - 1 : i;
          if (end > lineStart) {
            linesProcessed++;
            const lineBytes = jsonlBytes.subarray(lineStart, end);
            const lineText = decoder.decode(lineBytes, { stream: false });
            if (lineText.trim()) {
              linesWithContent++;
              try {
                const doc = JSON.parse(lineText);
                documents.push(doc);
              } catch (e: any) {
                jsonParseErrors++;
                // Log first few errors for debugging
                if (jsonParseErrors <= 3) {
                  console.debug(`[parseJsonlPartial] JSON parse error on line ${totalLines}:`, e.message, 'Text preview:', lineText.substring(0, 100));
                }
              }
            }
          }
        }
        lineStart = i + 1;
      }
    }
    
    // Count last line within scan limit if no trailing newline
    if (lineStart < scanLimit) {
      totalLines++;
      // Try to process the last line if we didn't encounter a newline
      if (documents.length < maxDocuments) {
        linesProcessed++;
        const lineBytes = jsonlBytes.subarray(lineStart, scanLimit);
        const lineText = decoder.decode(lineBytes, { stream: false });
        if (lineText.trim()) {
          linesWithContent++;
          try {
            const doc = JSON.parse(lineText);
            documents.push(doc);
          } catch (e: any) {
            jsonParseErrors++;
            if (jsonParseErrors <= 3) {
              console.debug(`[parseJsonlPartial] JSON parse error on last line:`, e.message, 'Text preview:', lineText.substring(0, 100));
            }
          }
        }
      }
    }
    
    console.log(`[parseJsonlPartial] Result: totalLines=${totalLines}, linesProcessed=${linesProcessed}, linesWithContent=${linesWithContent}, documentsFound=${documents.length}, jsonParseErrors=${jsonParseErrors}`);
    
    // If no newlines found and no documents parsed, try to parse entire scanned content as single JSON
    // Skip this if the file is larger than scan limit (i.e., we haven't seen the full content)
    if (documents.length === 0 && linesWithContent === 0 && scanLimit > 0 && scanLimit === jsonlBytes.length) {
      const entireText = decoder.decode(jsonlBytes.subarray(0, Math.min(scanLimit, 1024 * 1024)), { stream: false });
      if (entireText.trim()) {
        linesWithContent++;
        linesProcessed++;
        try {
          const doc = JSON.parse(entireText);
          documents.push(doc);
          totalLines = 1;
          console.log(`[parseJsonlPartial] Parsed as single JSON document (no newlines detected)`);
        } catch (e: any) {
          jsonParseErrors++;
          console.debug(`[parseJsonlPartial] Failed to parse as single JSON:`, e.message, 'Preview:', entireText.substring(0, 200));
        }
      }
    }
    
    if (linesProcessed > 0 && documents.length === 0) {
      // Try to decode and log first line for debugging
      const firstNewline = jsonlBytes.indexOf(0x0A);
      if (firstNewline !== -1 && firstNewline > 0) {
        const firstLineBytes = jsonlBytes.subarray(0, Math.min(firstNewline, 200));
        const firstLineText = decoder.decode(firstLineBytes, { stream: false });
        console.log(`[parseJsonlPartial] First line preview (${firstLineBytes.length} bytes):`, firstLineText);
      }
    }
    
    return { documents, totalLines, linesProcessed, jsonParseErrors };
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
      if (!jsonlData) {
        throw new Error('No output data received from conversion');
      }
       const jsonlBytes = jsonlData instanceof Uint8Array ? jsonlData : new TextEncoder().encode(jsonlData as string);
      processedSize = jsonlBytes.length;
      console.log(`[processWithWorker] jsonlData type: ${typeof jsonlData}, isUint8Array: ${jsonlData instanceof Uint8Array}, jsonlBytes size: ${jsonlBytes.length}`);
      // Log first 100 bytes for debugging
      if (jsonlBytes.length > 0) {
        const preview = Array.from(jsonlBytes.slice(0, Math.min(100, jsonlBytes.length)))
          .map(b => b.toString(16).padStart(2, '0'))
          .join(' ');
        console.log(`[processWithWorker] First 100 bytes (hex): ${preview}`);
        // Also try to decode as text
        const decoder = new TextDecoder('utf-8', { fatal: false });
        const textPreview = decoder.decode(jsonlBytes.slice(0, Math.min(200, jsonlBytes.length)));
        console.log(`[processWithWorker] First 200 chars as text: "${textPreview}"`);
      }
      if (processedSize === 0) {
        throw new Error('Output file is empty. The EDIFACT file may not contain valid segments.');
      }
      
      // For large files, only parse a subset for preview
      const LARGE_FILE_THRESHOLD = 10 * 1024 * 1024; // 10 MB
      const MAX_PREVIEW_DOCUMENTS = 1000;
      
        const { documents, totalLines, linesProcessed, jsonParseErrors } = parseJsonlPartial(jsonlBytes, MAX_PREVIEW_DOCUMENTS);
      processedData = documents;
      console.log(`[processWithWorker] Parse results: documents=${documents.length}, totalLines=${totalLines}, linesProcessed=${linesProcessed}`);
      
      // Only validate if we got zero documents but we attempted to parse some lines
      const isLargeFile = jsonlBytes.length > LARGE_FILE_THRESHOLD;
      if (processedData.length === 0 && linesProcessed > 0) {
        // For large files, parsing may fail due to scan limit; don't treat as error
        if (!isLargeFile || jsonParseErrors < linesProcessed) {
          // Only throw if it's a small file OR not all lines failed to parse (some may be valid)
          throw new Error('No valid JSONL lines produced. The EDIFACT file may not be compatible.');
        } else {
          console.warn(`[processWithWorker] Large file (${jsonlBytes.length} bytes) could not be parsed partially, but may still be valid JSONL.`);
        }
      }
      
      contentType = 'application/jsonl';
      processedBlob = new Blob([jsonlBytes.slice()], { type: 'application/jsonl' });
    } else if (fileType === 'jsonl') {
      // Result is .fra (Uint8Array)
      if (!result.data) {
        throw new Error('No output data received from compression');
      }
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

  const processWithCloud = async (file: File, fileType: FileType, onProgress?: (pct: number, status: string) => void): Promise<ProcessResult> => {
    const CLOUD_API_BASE = 'http://localhost:8080';
    let endpoint: string;
    let contentType: string;
    let operation: Operation;
    
    if (fileType === 'edifact') {
      endpoint = '/process/edifact';
      contentType = 'application/jsonl';
      operation = 'conversion';
    } else if (fileType === 'jsonl') {
      endpoint = '/process/jsonl';
      contentType = 'application/octet-stream';
      operation = 'compression';
    } else if (fileType === 'fra') {
      endpoint = '/decompress/fra';
      contentType = 'application/jsonl';
      operation = 'decompression';
    } else {
      throw new Error('Unsupported file type for cloud processing');
    }
    
    onProgress?.(5, 'Reading file...');
    const arrayBuffer = await file.arrayBuffer();
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5 * 60 * 1000); // 5 minutes timeout for large files
    
    onProgress?.(15, 'Sending file to cloud...');
    let response: Response;
    try {
      response = await fetch(`${CLOUD_API_BASE}${endpoint}`, {
        method: 'POST',
        body: arrayBuffer,
        headers: {
          'Content-Type': 'application/octet-stream',
        },
        signal: controller.signal,
      });
    } catch (err: any) {
      if (err.name === 'AbortError') {
        throw new Error('Cloud processing timeout: The request took too long. Please try again or use a smaller file.');
      } else {
        throw new Error(`Network error: ${err.message}`);
      }
    } finally {
      clearTimeout(timeoutId);
    }
    
    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Cloud processing failed: ${response.status} ${response.statusText} - ${errorText}`);
    }

    onProgress?.(50, 'Processing on server...');
    // The /process/edifact endpoint returns JSON with task status and download URL
    const processResult = await response.json();
    
    if (processResult.status !== 'completed') {
      throw new Error(`Cloud processing failed: ${processResult.status} - ${processResult.error || 'Unknown error'}`);
    }
    
    // Extract file_id from download_url: "/download/{file_id}" -> file_id
    const fileId = processResult.download_url?.split('/').pop();
    if (!fileId) {
      throw new Error('Cloud processing: missing file_id in response');
    }
    
    onProgress?.(65, 'Downloading processed data...');
    // Fetch the actual processed data
    const dataResponse = await fetch(`${CLOUD_API_BASE}/data/${fileId}`);
    if (!dataResponse.ok) {
      throw new Error(`Failed to download processed data: ${dataResponse.status}`);
    }
    
    const resultBlob = await dataResponse.blob();
    const processedSize = resultBlob.size;
    
    onProgress?.(85, 'Parsing results...');
    let processedData: any[] | undefined;
    let processedBlob: Blob | undefined;
    
    if (fileType === 'edifact' || fileType === 'fra') {
      // Parse JSONL for preview
      const jsonlBytes = new Uint8Array(await resultBlob.arrayBuffer());
      const { documents } = parseJsonlPartial(jsonlBytes, 1000);
      processedData = documents;
      processedBlob = resultBlob;
    } else {
      processedBlob = resultBlob;
    }
    
    onProgress?.(95, 'Finalizing...');
    return {
      originalSize: file.size,
      processedSize,
      processedData,
      processedBlob,
      fileName: file.name.replace(/\.[^/.]+$/, ''),
      fileType,
      operation,
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

    // Determine processing method based on file size (Smart Switching)
    if (processingMode === 'local') {
      if (!workerReady) {
        setError(t('errors.workerNotReady'));
        setProcessing(false);
        return;
      }
    } else {
      // Cloud processing mode - set cloud processing state
      setCloudProcessing(true);
      setCloudStatus('Preparing upload...');
      setCloudProgress(0);
    }
    let processedResult: ProcessResult;

    try {
      if (processingMode === 'local') {
        processedResult = await processWithWorker(file, fileType);
      } else {
        // Cloud processing
        setCloudStatus('Uploading file to cloud...');
        setCloudProgress(10);
        processedResult = await processWithCloud(file, fileType, (pct, status) => {
          setCloudProgress(pct);
          setCloudStatus(status);
        });
        setCloudStatus('Processing complete');
        setCloudProgress(100);
      }


      // Optional compression to .fra for EDIFACT files (local only)
      if (processingMode === 'local' && alsoCompressToFra && fileType === 'edifact' && processedResult.processedBlob) {
        try {
          const jsonlBytes = new Uint8Array(await processedResult.processedBlob.arrayBuffer());
          // Check if JSONL is too large for compression in browser
          if (jsonlBytes.length > MAX_COMPRESSION_SIZE) {
            console.warn(`JSONL too large for compression (${jsonlBytes.length} bytes > ${MAX_COMPRESSION_SIZE} bytes). Skipping compression.`);
            // Optionally set a warning message for the user
          } else {
            const client = getWasmWorkerClient();
            const compressResult = await client.compressJsonl(jsonlBytes);
            if (compressResult.success && compressResult.data) {
              processedResult.processedFraBlob = new Blob([compressResult.data.slice()], { type: 'application/octet-stream' });
            }
          }
        } catch (err) {
          console.warn('Failed to compress to .fra:', err);
          // Continue without .fra blob
        }
      }

      setResult(processedResult);
      if (processingMode === 'cloud') {
        setCloudProcessing(false);
        setCloudStatus('Completed');
        setCloudProgress(100);
      }
    } catch (err: any) {
      setError(err.message || 'Processing failed');
      if (processingMode === 'cloud') {
        setCloudProcessing(false);
        setCloudStatus('Error: ' + err.message);
        setCloudProgress(0);
      }
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
    setCloudProcessing(false);
    setCloudStatus('');
    setCloudProgress(0);
    setProcessingMode('local');
  };

  const downloadFile = (blob: Blob, filename: string) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.style.display = 'none';
    document.body.appendChild(a);
    a.click();
    // Keep blob URL alive long enough for browser to start the download
    setTimeout(() => {
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }, 3000);
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
    if (!result) return;

    const generateAndDownloadCSV = (data: any[], fileName: string) => {
      try {
        const headers = Object.keys(data[0]);
        const csvRows = [
          headers.join(','),
          ...data.map((row: any) =>
            headers.map(header => `"${(row[header] || '').toString().replace(/"/g, '""')}"`).join(',')
          ),
        ];
        const csv = csvRows.join('\n');
        const blob = new Blob([csv], { type: 'text/csv' });
        downloadFile(blob, `${fileName}.csv`);
      } catch (err) {
        console.error('Error generating CSV:', err);
        alert('Error al generar el CSV. Revisa la consola para más detalles.');
      }
    };

    // If parsed data is available, use it directly
    if (result.processedData && result.processedData.length > 0) {
      generateAndDownloadCSV(result.processedData, result.fileName || 'output');
      return;
    }

    // Fallback: if processedBlob exists but processedData was empty (e.g. cloud flow
    // where the blob wasn't parsed as JSONL), try to parse it on-the-fly
    if (result.processedBlob) {
      result.processedBlob.text().then(text => {
        const lines = text.split('\n').filter((l: string) => l.trim());
        const parsed = lines.map((l: string) => {
          try { return JSON.parse(l); } catch { return null; }
        }).filter((d: any) => d !== null);

        if (parsed.length > 0) {
          generateAndDownloadCSV(parsed, result.fileName || 'output');
        } else {
          alert('No se pudieron extraer datos tabulares del archivo procesado.');
        }
      }).catch(err => {
        console.error('Error reading blob for CSV:', err);
        alert('Error al leer el archivo procesado para generar CSV.');
      });
    }
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
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Processing Mode:</span>
                <div className={`px-2 py-1 rounded text-xs font-medium ${processingMode === 'local' ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300' : 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300'}`}>
                  {processingMode === 'local' ? 'Local (WASM)' : 'Cloud (Google Cloud Run)'}
                </div>
                {processingMode === 'local' && !workerReady && (
                  <span className="text-xs text-amber-600 dark:text-amber-400">(Worker loading...)</span>
                )}
                {processingMode === 'local' && workerReady && (
                  <span className="text-xs text-green-600 dark:text-green-400">✓ Worker ready</span>
                )}
                {processingMode === 'cloud' && (
                  <span className="text-xs text-blue-600 dark:text-blue-400">(File size ≥ 100 MB)</span>
                )}
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {processingMode === 'local' ? 'Processes files locally in your browser' : 'Files are sent to cloud for processing'}
              </div>
            </div>
            {cloudProcessing && processingMode === 'cloud' && (
              <div className="mt-2 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-medium text-blue-800 dark:text-blue-300">Cloud Processing Status</span>
                  <span className="text-xs text-blue-600 dark:text-blue-400">{cloudStatus}</span>
                </div>
                <div className="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-2">
                  <div 
                    className="bg-blue-600 h-2 rounded-full transition-all duration-300" 
                    style={{ width: `${cloudProgress}%` }}
                  />
                </div>
                <p className="text-xs text-blue-700 dark:text-blue-400 mt-2">
                  {cloudProgress < 100 ? 'Processing in cloud...' : 'Ready for download'}
                </p>
              </div>
            )}
            {fileType === 'edifact' && (
              <>
                <div className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    checked={alsoCompressToFra}
                    onChange={(e) => setAlsoCompressToFra(e.target.checked)}
                    disabled={file.size > MAX_COMPRESSION_SIZE}
                    id="compress-to-fra"
                    className={`h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 ${file.size > MAX_COMPRESSION_SIZE ? 'opacity-50 cursor-not-allowed' : ''}`}
                  />
                  <label htmlFor="compress-to-fra" className={`text-sm ${file.size > MAX_COMPRESSION_SIZE ? 'text-gray-400 dark:text-gray-500' : 'text-gray-700 dark:text-gray-300'}`}>
                     {t('home.processing.compressToFra')}
                     {file.size > MAX_COMPRESSION_SIZE && ' (Disabled - file too large)'}
                  </label>
                </div>
                {file.size > MAX_COMPRESSION_SIZE && (
                  <div className="mt-2 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-2">
                    <p className="text-yellow-800 dark:text-yellow-300 text-xs">
                      File is too large ({Math.round(file.size / (1024 * 1024))} MB) for compression in browser. Maximum size for compression is 50 MB. Use the CLI tool for larger files.
                    </p>
                  </div>
                )}
                {alsoCompressToFra && file && file.size <= MAX_COMPRESSION_SIZE && file.size > 10 * 1024 * 1024 && (
                  <div className="mt-2 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-2">
                    <p className="text-yellow-800 dark:text-yellow-300 text-xs">
                      File is large ({Math.round(file.size / (1024 * 1024))} MB). Compression may take longer.
                    </p>
                  </div>
                )}
              </>
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
              {(result.processedData || (result.processedBlob && result.contentType?.includes('application/jsonl'))) && (
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