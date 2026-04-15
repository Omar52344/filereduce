'use client';

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
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  const calculateSavings = () => {
    if (!processedSize) return null;
    const savedBytes = originalSize - processedSize;
    const percentage = ((savedBytes / originalSize) * 100).toFixed(2);
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

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
          Original Size
        </h3>
        <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
          {formatBytes(originalSize)}
        </p>
        <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
          {operation === 'compression' ? 'Before compression' : 'Input file'}
        </p>
      </div>

      {processedSize && (
        <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
            Processed Size
          </h3>
          <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
            {formatBytes(processedSize)}
          </p>
          <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
            {operation === 'compression' ? 'After compression' : 'Output file'}
          </p>
        </div>
      )}

      {savings && (
        <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
          <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
            Space Saved
          </h3>
          <p className="mt-2 text-3xl font-bold text-green-600 dark:text-green-400">
            {savings.percentage}%
          </p>
          <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
            {formatBytes(savings.savedBytes)} reduction
          </p>
          <div className="mt-4 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              className="bg-green-500 h-2 rounded-full"
              style={{ width: `${savings.percentage}%` }}
            ></div>
          </div>
        </div>
      )}

      <div className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
        <h3 className="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
          Cloud Cost (Projected)
        </h3>
        <p className="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
          ${originalCost.monthly.toFixed(3)}
          <span className="text-sm font-normal text-gray-500 dark:text-gray-400">
            {' '}/ month
          </span>
        </p>
        <p className="mt-1 text-sm text-gray-600 dark:text-gray-300">
          ${originalCost.yearly.toFixed(2)} / year
        </p>
        {processedCost && savings && (
          <p className="mt-2 text-sm text-green-600 dark:text-green-400">
            Save ${(originalCost.monthly - processedCost.monthly).toFixed(3)}/mo
          </p>
        )}
      </div>
    </div>
  );
}