'use client';

import { useState, useEffect } from 'react';
import { useTranslation } from '@/lib/i18n';

export default function GeneratePage() {
  const { t } = useTranslation();
  const [version, setVersion] = useState('D96A');
  const [sizeMB, setSizeMB] = useState(10);
  const [isGenerating, setIsGenerating] = useState(false);
  const [progress, setProgress] = useState(0);
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [downloadName, setDownloadName] = useState('');

  // Available versions (could be fetched from standards directory)
  const availableVersions = ['D96A', 'D01B'];

  // Generate EDIFACT content
  const generateEdifact = (version: string, targetBytes: number): string => {
    const lines = [];
    
    // Helper to format segment with elements separated by '+'
    const segment = (code: string, ...elements: string[]) => {
      return code + (elements.length > 0 ? '+' + elements.join('+') : '') + "'";
    };

    // UNB: Interchange header
    const interchangeId = `INT${Date.now()}`;
    lines.push(segment('UNB', 'UNOC', '3', 'SENDER', '14', 'RECEIVER', '14', interchangeId, '0'));
    
    // UNH: Message header with version
    lines.push(segment('UNH', '1', version, 'ORDERS'));
    
    // BGM: Beginning of message
    lines.push(segment('BGM', '220', `ORDER${Date.now().toString().slice(-6)}`));
    
    // DTM: Date/time
    const now = new Date();
    const dateStr = now.getFullYear().toString() + 
                    (now.getMonth()+1).toString().padStart(2, '0') + 
                    now.getDate().toString().padStart(2, '0');
    lines.push(segment('DTM', '137', dateStr));
    
    // NAD: Parties (buyer/seller)
    lines.push(segment('NAD', 'BY', 'BUYER001'));
    lines.push(segment('NAD', 'SU', 'SELLER001'));
    
    // CUX: Currency
    lines.push(segment('CUX', '2', 'USD'));
    
    // Estimate bytes per line (including newline)
    const bytesPerLine = 100; // approximate
    const targetLines = Math.ceil(targetBytes / bytesPerLine);
    const linesPerItem = 3; // LIN + QTY + PRI
    
    const items = Math.floor(targetLines / linesPerItem);
    
    for (let i = 1; i <= items; i++) {
      // LIN: Line item
      lines.push(segment('LIN', i.toString(), '', '', `SKU${i.toString().padStart(6, '0')}`));
      
      // QTY: Quantity
      const qty = Math.floor(Math.random() * 1000) + 1;
      lines.push(segment('QTY', '1', `${qty}`, 'PCE'));
      
      // PRI: Price
      const price = (Math.random() * 100).toFixed(2);
      lines.push(segment('PRI', 'AAA', price));
    }
    
    // CNT: Control count
    lines.push(segment('CNT', '2', items.toString()));
    
    // UNT: Message trailer (segment count)
    const segmentCount = lines.length + 1; // including UNT
    lines.push(segment('UNT', segmentCount.toString(), '1'));
    
    // UNZ: Interchange trailer
    lines.push(segment('UNZ', '1', interchangeId));
    
    return lines.join('\n');
  };

  const handleGenerate = async () => {
    setIsGenerating(true);
    setProgress(0);
    setDownloadUrl(null);
    
    // Simulate progress updates
    const interval = setInterval(() => {
      setProgress(prev => Math.min(prev + 5, 90));
    }, 100);
    
    // Generate in background (could use Web Worker for large files)
    setTimeout(() => {
      const bytes = sizeMB * 1024 * 1024;
      const content = generateEdifact(version, bytes);
      
      clearInterval(interval);
      setProgress(100);
      
      // Create downloadable blob
      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      setDownloadUrl(url);
      setDownloadName(`edifact-${version}-${sizeMB}MB.edi`);
      
      setIsGenerating(false);
    }, 500); // Simulate some processing time
  };

  useEffect(() => {
    return () => {
      if (downloadUrl) {
        URL.revokeObjectURL(downloadUrl);
      }
    };
  }, [downloadUrl]);

  return (
    <div className="container mx-auto px-4 py-8 max-w-4xl">
      <h1 className="text-3xl font-bold mb-6">{t('generate.title')}</h1>
      <p className="text-gray-600 mb-8">
        {t('generate.description')}
      </p>
      
      <div className="bg-white rounded-lg shadow-md p-6 mb-8">
        <div className="space-y-6">
          {/* Version selection */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
               {t('generate.versionLabel')}
            </label>
            <div className="flex flex-wrap gap-2">
              {availableVersions.map(v => (
                <button
                  key={v}
                  type="button"
                  className={`px-4 py-2 rounded-md ${version === v ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-700 hover:bg-gray-200'}`}
                  onClick={() => setVersion(v)}
                >
                  {v}
                </button>
              ))}
              <div className="relative flex-1 max-w-xs">
                <input
                  type="text"
                  value={version}
                  onChange={(e) => setVersion(e.target.value.toUpperCase())}
                  className="w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="e.g. D96A"
                />
              </div>
            </div>
          </div>
          
          {/* Size slider */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
               {t('generate.sizeLabel', { sizeMB })}
            </label>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-gray-500">1 MB</span>
              <input
                type="range"
                min="1"
                max="200"
                value={sizeMB}
                onChange={(e) => setSizeMB(parseInt(e.target.value))}
                className="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              />
              <span className="text-sm text-gray-500">200 MB</span>
              <div className="w-24">
                <input
                  type="number"
                  min="1"
                  max="200"
                  value={sizeMB}
                  onChange={(e) => setSizeMB(Math.min(200, Math.max(1, parseInt(e.target.value) || 1)))}
                  className="w-full px-3 py-1 border border-gray-300 rounded-md text-center"
                />
              </div>
            </div>
            <p className="text-sm text-gray-500 mt-2">
               {t('generate.sizeHint')}
            </p>
          </div>
          
          {/* Generate button */}
          <div className="pt-4">
            <button
              onClick={handleGenerate}
              disabled={isGenerating}
              className={`w-full py-3 px-4 rounded-md font-medium ${isGenerating ? 'bg-blue-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700'} text-white transition-colors`}
            >
               {isGenerating ? t('generate.generating') : t('generate.button')}
            </button>
          </div>
          
          {/* Progress bar */}
          {isGenerating && (
            <div className="pt-2">
              <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-green-500 transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="text-sm text-gray-600 mt-2 text-center">
                 {t('generate.progress', { progress })}
              </p>
            </div>
          )}
          
          {/* Download link */}
          {downloadUrl && (
            <div className="pt-6 border-t border-gray-200">
              <div className="bg-green-50 border border-green-200 rounded-md p-4">
                <h3 className="text-lg font-medium text-green-800 mb-2">
                   {t('generate.ready')}
                </h3>
                <p className="text-green-700 mb-4">
                   {t('generate.readyDescription')}
                </p>
                <a
                  href={downloadUrl}
                  download={downloadName}
                  className="inline-flex items-center px-4 py-2 bg-green-600 text-white font-medium rounded-md hover:bg-green-700 transition-colors"
                >
                  <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                   {t('generate.download', { filename: downloadName })}
                </a>
                <p className="text-sm text-green-600 mt-2">
                   {t('generate.downloadHint')}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
      
      <div className="bg-yellow-50 border border-yellow-200 rounded-md p-4">
        <h3 className="text-lg font-medium text-yellow-800 mb-2">
           {t('generate.noteTitle')}
        </h3>
        <p className="text-yellow-700">
           {t('generate.noteText')}
        </p>
      </div>
    </div>
  );
}