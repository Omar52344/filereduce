'use client';

import { useTranslation } from '@/lib/i18n/LanguageContext';

interface DashboardProps {
  originalSize: number; // bytes
  processedSize?: number; // bytes, optional for compression
  fileType: string;
  operation: string;
}

export default function Dashboard({
  originalSize,
  processedSize,
  fileType,
  operation,
}: DashboardProps) {
  const { t } = useTranslation();
  
  const formatBytes = (bytes: number): string => {
    const absBytes = Math.abs(bytes);
    if (absBytes === 0) return `0 ${t('dashboard.bytes')}`;
    const k = 1024;
    const sizes = ['bytes', 'kb', 'mb', 'gb'];
    const i = Math.floor(Math.log(absBytes) / Math.log(k));
    const value = parseFloat((absBytes / Math.pow(k, i)).toFixed(2));
    const unit = t(`dashboard.${sizes[i]}`);
    return `${value} ${unit}`;
  };

  const calculateSavings = () => {
    if (!processedSize) return null;
    const savedBytes = originalSize - processedSize;
    const percentage = originalSize > 0
      ? ((savedBytes / originalSize) * 100).toFixed(2)
      : '0.00';
    return { savedBytes, percentage };
  };

  const calculateCloudCost = (bytes: number) => {
    // Estimated cost: $0.023 per GB per month (AWS S3 Standard)
    const costPerGBPerMonth = 0.023;
    const gb = bytes / (1024 * 1024 * 1024);
    const monthly = gb * costPerGBPerMonth;
    const yearly = monthly * 12;
    return { monthly, yearly };
  };

  const savings = calculateSavings();
  const originalCost = calculateCloudCost(originalSize);
  const processedCost = processedSize ? calculateCloudCost(processedSize) : null;

  const isExpansion = savings && Number(savings.percentage) < 0;
  const barPct = savings ? Math.min(Math.abs(Number(savings.percentage)), 100) : 0;

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
           {t('dashboard.originalSize')}
        </h3>
        <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
          {formatBytes(originalSize)}
        </p>
        <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
           {operation === 'compression' ? t('dashboard.beforeCompression') : t('dashboard.inputFile')}
        </p>
      </div>

      {processedSize && (
        <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
             {t('dashboard.processedSize')}
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {formatBytes(processedSize)}
          </p>
          <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
             {operation === 'compression' ? t('dashboard.afterCompression') : t('dashboard.outputFile')}
          </p>
        </div>
      )}

      {savings && (
        <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
            {t('dashboard.spaceSaved')}
          </h3>
          <p className={`mt-2 text-3xl font-bold ${isExpansion ? 'text-orange-600 dark:text-orange-400' : 'text-green-600 dark:text-green-400'}`}>
            {isExpansion ? '+' : ''}{savings.percentage}%
          </p>
          <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
            {formatBytes(savings.savedBytes)} {isExpansion ? `(${t('dashboard.expansion')})` : t('dashboard.reduction')}
          </p>
          <div className="mt-4 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              className={`h-2 rounded-full ${isExpansion ? 'bg-orange-500' : 'bg-green-500'}`}
              style={{ width: `${barPct}%` }}
            ></div>
          </div>
        </div>
      )}

      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
             {t('dashboard.cloudCostProjected')}
        </h3>
        <p className="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
          ${originalCost.monthly.toFixed(3)}
          <span className="text-sm font-normal text-gray-500 dark:text-gray-400">
            {' '}{t('dashboard.perMonth')}
          </span>
        </p>
        <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
          ${originalCost.yearly.toFixed(2)}            {t('dashboard.perYear')}
        </p>
        {processedCost && savings && (
          <p className="mt-2 text-sm text-green-600 dark:text-green-400">
             {t('dashboard.save')} ${(originalCost.monthly - processedCost.monthly).toFixed(3)}/mo
          </p>
        )}
      </div>
    </div>
  );
}
