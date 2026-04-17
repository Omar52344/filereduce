'use client';

import { useTranslation } from '@/lib/i18n/LanguageContext';

export default function AboutPage() {
  const { t } = useTranslation();

  const whatsappNumber = '+57 3175604331';
  const email = 'omarjaramillo8@gmail.com';

  const features = [
    { key: 'dynamic', emoji: '🔄' },
    { key: 'compression', emoji: '📦' },
    { key: 'local', emoji: '🔒' },
    { key: 'zeroConfig', emoji: '⚡' },
    { key: 'open', emoji: '🔓' },
  ];

  return (
    <div className="container mx-auto px-6 py-12 max-w-5xl">
      {/* Hero section */}
      <div className="text-center mb-16">
        <h1 className="text-5xl font-bold text-gray-900 dark:text-white mb-6">
          {t('about.title')}
        </h1>
        <p className="text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto">
          {t('about.subtitle')}
        </p>
      </div>

      {/* Mission & Vision */}
      <div className="grid md:grid-cols-2 gap-12 mb-16">
        <div className="bg-gradient-to-br from-blue-50 to-white dark:from-gray-800 dark:to-gray-900 rounded-2xl p-8 border border-blue-100 dark:border-gray-700">
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-3">
            <span className="text-blue-600 dark:text-blue-400">🌟</span>
            {t('about.missionTitle')}
          </h2>
          <p className="text-lg text-gray-700 dark:text-gray-300 leading-relaxed">
            {t('about.mission')}
          </p>
        </div>
        <div className="bg-gradient-to-br from-purple-50 to-white dark:from-gray-800 dark:to-gray-900 rounded-2xl p-8 border border-purple-100 dark:border-gray-700">
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-3">
            <span className="text-purple-600 dark:text-purple-400">🚀</span>
            {t('about.visionTitle')}
          </h2>
          <p className="text-lg text-gray-700 dark:text-gray-300 leading-relaxed">
            {t('about.vision')}
          </p>
        </div>
      </div>

      {/* Key Features */}
      <section className="mb-16">
        <h2 className="text-3xl font-bold text-gray-900 dark:text-white mb-12 text-center">
          {t('about.featuresTitle')}
        </h2>
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map(({ key, emoji }) => (
            <div
              key={key}
              className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700 hover:shadow-md transition-shadow"
            >
              <div className="text-3xl mb-4">{emoji}</div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                {t(`about.features.${key}`)}
              </h3>
            </div>
          ))}
        </div>
      </section>

      {/* Story/Description */}
      <div className="bg-gradient-to-r from-gray-50 to-white dark:from-gray-800 dark:to-gray-900 rounded-2xl p-10 border border-gray-200 dark:border-gray-700 mb-16">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
          The FileReduce Story
        </h2>
        <div className="space-y-4 text-gray-700 dark:text-gray-300">
          <p>
            FileReduce started as a solution to a common problem in EDI processing: static, hard‑coded translation tables that couldn't adapt to new EDIFACT versions. By introducing dynamic translation dictionaries and automatic version detection, we eliminated the need for manual updates and configuration.
          </p>
          <p>
            The second breakthrough came with the .fra compression format. While working with large JSONL datasets derived from EDIFACT, we noticed extreme redundancy in the structure. By developing a specialized compression algorithm tailored to this pattern, we achieved consistent savings of 95% or more.
          </p>
          <p>
            Today, FileReduce combines these innovations with a modern web interface, WebAssembly for local processing, and a zero‑config architecture that makes EDIFACT processing accessible to everyone—from logistics managers to data engineers.
          </p>
        </div>
      </div>

      {/* Contact section */}
      <div className="bg-gradient-to-r from-blue-50 to-purple-50 dark:from-gray-800 dark:to-gray-900 rounded-2xl p-10 border border-blue-200 dark:border-gray-700">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">
          {t('about.contactTitle')}
        </h2>
        <p className="text-gray-700 dark:text-gray-300 mb-8">
          {t('about.contactDescription')}
        </p>
        <div className="flex flex-col sm:flex-row gap-6">
          <a
            href={`https://wa.me/${whatsappNumber.replace('+', '')}?text=Hola%20FileReduce,%20me%20interesa%20saber%20más`}
            target="_blank"
            rel="noopener noreferrer"
            className="flex-1 bg-green-600 hover:bg-green-700 text-white font-medium py-4 px-6 rounded-xl text-center transition-colors flex items-center justify-center gap-3"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.76.982.998-3.675-.236-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.9 6.994c-.004 5.45-4.438 9.88-9.888 9.88m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.333.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.304-1.654a11.882 11.882 0 005.683 1.447h.005c6.554 0 11.89-5.333 11.893-11.892 0-3.18-1.24-6.162-3.495-8.411"/>
            </svg>
            <span>{t('about.whatsapp')}: {whatsappNumber}</span>
          </a>
          <a
            href={`mailto:${email}?subject=FileReduce%20Inquiry`}
            className="flex-1 bg-blue-600 hover:bg-blue-700 text-white font-medium py-4 px-6 rounded-xl text-center transition-colors flex items-center justify-center gap-3"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
            </svg>
            <span>{t('about.email')}: {email}</span>
          </a>
        </div>
      </div>
    </div>
  );
}