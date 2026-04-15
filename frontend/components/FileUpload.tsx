'use client';

import { useCallback, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import DataGrid from './DataGrid';
import Dashboard from './Dashboard';

type FileType = 'edifact' | 'jsonl' | 'fra' | 'unknown';
type Operation = 'conversion' | 'compression' | 'decompression';

interface ProcessResult {
  originalSize: number;
  processedSize?: number;
  processedData?: any[]; // parsed JSONL documents
  processedBlob?: Blob;
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

  const detectFileType = (filename: string): FileType => {
    const ext = filename.split('.').pop()?.toLowerCase();
    if (ext === 'edi' || ext === 'edifact' || ext === 'txt') return 'edifact';
    if (ext === 'jsonl' || ext === 'json') return 'jsonl';
    if (ext === 'fra') return 'fra';
    return 'unknown';
  };

  const onDrop = useCallback((acceptedFiles: File[]) => {
    const file = acceptedFiles[0];
    if (!file) return;
    setFile(file);
    setFileType(detectFileType(file.name));
    setError(null);
    setResult(null);
  }, []);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'text/plain': ['.edi', '.edifact', '.txt', '.jsonl', '.json', '.fra'],
    },
    maxFiles: 1,
  });

  const handleProcess = async () => {
    if (!file) return;
    setProcessing(true);
    setError(null);
    const formData = new FormData();
    formData.append('file', file);

    let endpoint = '';
    let operation: Operation = 'conversion';
    if (fileType === 'edifact') {
      endpoint = '/api/process/edifact';
      operation = 'conversion';
    } else if (fileType === 'jsonl') {
      endpoint = '/api/process/jsonl';
      operation = 'compression';
    } else if (fileType === 'fra') {
      endpoint = '/api/decompress/fra';
      operation = 'decompression';
    } else {
      setError('Unsupported file type');
      setProcessing(false);
      return;
    }

    try {
      const response = await fetch(`http://localhost:8080${endpoint}`, {
        method: 'POST',
        body: formData,
      });
      if (!response.ok) {
        // Try to parse error as JSON, fallback to text
        try {
          const err = await response.json();
          throw new Error(err.error || `HTTP ${response.status}`);
        } catch {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
      }

      const contentType = response.headers.get('content-type') || '';
      let processedData: any[] | undefined;
      let processedBlob: Blob | undefined;
      let processedSize: number | undefined;

      if (contentType.includes('application/jsonl') || contentType.includes('application/json')) {
        const text = await response.text();
        processedSize = new TextEncoder().encode(text).length;
        // Parse JSONL (one JSON object per line)
        const lines = text.split('\n').filter(line => line.trim());
        processedData = lines.map(line => {
          try {
            return JSON.parse(line);
          } catch (e) {
            console.warn('Failed to parse JSONL line:', line);
            return null;
          }
        }).filter(obj => obj !== null);
      } else if (contentType.includes('application/octet-stream')) {
        processedBlob = await response.blob();
        processedSize = processedBlob.size;
      } else {
        // Fallback: try as blob
        processedBlob = await response.blob();
        processedSize = processedBlob.size;
      }

      setResult({
        originalSize: file.size,
        processedSize,
        processedData,
        processedBlob,
        fileName: file.name.replace(/\.[^/.]+$/, ''), // remove extension
        fileType,
        operation,
        contentType,
      });
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
    if (!result?.processedBlob) return;
    downloadFile(result.processedBlob, `${result.fileName || 'output'}.fra`);
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
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">FileReduce Processor</h1>
        <p className="text-gray-600 dark:text-gray-300 mt-2">
          Upload EDIFACT, JSONL, or .fra files for conversion and compression
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
          <p className="text-blue-600 dark:text-blue-400">Drop the file here ...</p>
        ) : (
          <>
            <p className="text-gray-700 dark:text-gray-300">
              Drag & drop a file, or click to select
            </p>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
              Supported: .edi, .edifact, .jsonl, .json, .fra
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
                Type: {fileType} • Size: {(file.size / 1024).toFixed(2)} KB
              </p>
            </div>
            <button
              onClick={() => setFile(null)}
              className="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300"
            >
              Remove
            </button>
          </div>
          <div className="mt-4">
            <button
              onClick={handleProcess}
              disabled={processing || fileType === 'unknown'}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {processing ? 'Processing...' : `Process as ${fileType.toUpperCase()}`}
            </button>
            {fileType === 'unknown' && (
              <p className="text-red-600 dark:text-red-400 text-sm mt-2">
                Unknown file type. Please upload a supported format.
              </p>
            )}
          </div>
        </div>
      )}

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <h4 className="font-medium text-red-800 dark:text-red-300">Error</h4>
          <p className="text-red-600 dark:text-red-400">{error}</p>
        </div>
      )}

      {result && (
        <div className="space-y-6">
          <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-6">
            <h4 className="font-medium text-green-800 dark:text-green-300 text-lg mb-4">Processing Complete</h4>
            <Dashboard
              originalSize={result.originalSize}
              processedSize={result.processedSize}
              fileType={result.fileType}
              operation={result.operation}
            />
          </div>

          {result.processedData && result.processedData.length > 0 && (
            <div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
              <h4 className="font-bold text-gray-900 dark:text-white text-lg mb-4">Data Preview</h4>
              <DataGrid data={result.processedData} />
            </div>
          )}

          <div className="bg-gray-50 dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-6">
            <h4 className="font-bold text-gray-900 dark:text-white text-lg mb-4">Download Results</h4>
            <div className="flex flex-wrap gap-3">
              {result.processedData && (
                <>
                  <button
                    onClick={handleDownloadJSONL}
                    className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
                  >
                    Download JSONL
                  </button>
                  <button
                    onClick={handleDownloadCSV}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                  >
                    Download CSV
                  </button>
                </>
              )}
              {result.processedBlob && result.contentType?.includes('application/octet-stream') && (
                <button
                  onClick={handleDownloadFRA}
                  className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700"
                >
                  Download .fra
                </button>
              )}
              {result.processedBlob && result.contentType?.includes('application/jsonl') && (
                <button
                  onClick={handleDownloadJSONL}
                  className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700"
                >
                  Download JSONL
                </button>
              )}
            </div>
            <p className="text-gray-600 dark:text-gray-400 text-sm mt-4">
              Original file: {result.fileName}.{result.fileType} ({Math.round(result.originalSize / 1024)} KB)
            </p>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-8">
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
          <h3 className="font-bold text-gray-900 dark:text-white">EDIFACT → JSONL</h3>
          <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
            Convert EDIFACT files to structured JSONL using dynamic translations.
          </p>
        </div>
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
          <h3 className="font-bold text-gray-900 dark:text-white">JSONL ↔ .fra</h3>
          <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
            Compress JSONL to .fra format (95%+ savings) or decompress back.
          </p>
        </div>
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
          <h3 className="font-bold text-gray-900 dark:text-white">Dynamic Translations</h3>
          <p className="text-gray-600 dark:text-gray-300 text-sm mt-2">
            Mapping driven by translations.json; supports unknown segment telemetry.
          </p>
        </div>
      </div>
    </div>
  );
}