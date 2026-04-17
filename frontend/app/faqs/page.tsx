'use client';

import { useTranslation } from '@/lib/i18n/LanguageContext';

export default function FaqsPage() {
  const { t } = useTranslation();

  const whatsappNumber = '+57 3175604331';
  const email = 'omarjaramillo8@gmail.com';

  return (
    <div className="container mx-auto px-6 py-12 max-w-5xl">
      {/* Hero section */}
      <div className="text-center mb-12">
        <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-4">
          {t('faqs.title')}
        </h1>
        <p className="text-lg text-gray-600 dark:text-gray-300">
          {t('faqs.subtitle')}
        </p>
      </div>

      {/* FAQ sections */}
      <div className="space-y-12">
        {/* General */}
        <section>
          <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200 mb-6">
            {t('faqs.sections.general.title')}
          </h2>
          <div className="space-y-6">
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.general.q1')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.general.a1')}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.general.q2')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.general.a2')}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.general.q3')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.general.a3')}
              </p>
            </div>
          </div>
        </section>

        {/* Usage */}
        <section>
          <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200 mb-6">
            {t('faqs.sections.usage.title')}
          </h2>
          <div className="space-y-6">
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.usage.q1')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.usage.a1')}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.usage.q2')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.usage.a2')}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.usage.q3')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.usage.a3')}
              </p>
            </div>
          </div>
        </section>

        {/* Technical */}
        <section>
          <h2 className="text-2xl font-bold text-gray-800 dark:text-gray-200 mb-6">
            {t('faqs.sections.technical.title')}
          </h2>
          <div className="space-y-6">
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.technical.q1')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.technical.a1')}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.technical.q2')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.technical.a2')}
              </p>
            </div>
            <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                {t('faqs.sections.technical.q3')}
              </h3>
              <p className="text-gray-700 dark:text-gray-300">
                {t('faqs.sections.technical.a3')}
              </p>
            </div>
          </div>
        </section>
      </div>

      {/* Contact section */}
      <div className="mt-16 bg-gradient-to-r from-blue-50 to-purple-50 dark:from-gray-800 dark:to-gray-900 rounded-2xl p-8 border border-blue-200 dark:border-gray-700">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
          {t('faqs.contactTitle')}
        </h2>
        <p className="text-gray-700 dark:text-gray-300 mb-8">
          {t('faqs.contactDescription')}
        </p>
        <div className="flex flex-col sm:flex-row gap-6">
          <a
            href={`https://wa.me/${whatsappNumber.replace('+', '')}?text=Hola%20FileReduce,%20tengo%20una%20consulta`}
            target="_blank"
            rel="noopener noreferrer"
            className="flex-1 bg-green-600 hover:bg-green-700 text-white font-medium py-4 px-6 rounded-xl text-center transition-colors flex items-center justify-center gap-3"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347m-5.421 7.403h-.004a9.87 9.87 0 01-5.031-1.378l-.361-.214-3.76.982.998-3.675-.236-.374a9.86 9.86 0 01-1.51-5.26c.001-5.45 4.436-9.884 9.888-9.884 2.64 0 5.122 1.03 6.988 2.898a9.825 9.825 0 012.9 6.994c-.004 5.45-4.438 9.88-9.888 9.88m8.413-18.297A11.815 11.815 0 0012.05 0C5.495 0 .16 5.333.157 11.892c0 2.096.547 4.142 1.588 5.945L.057 24l6.304-1.654a11.882 11.882 0 005.683 1.447h.005c6.554 0 11.89-5.333 11.893-11.892 0-3.18-1.24-6.162-3.495-8.411"/>
            </svg>
            <span>{t('faqs.whatsapp')}: {whatsappNumber}</span>
          </a>
          <a
            href={`mailto:${email}?subject=FileReduce%20Inquiry`}
            className="flex-1 bg-blue-600 hover:bg-blue-700 text-white font-medium py-4 px-6 rounded-xl text-center transition-colors flex items-center justify-center gap-3"
          >
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
            </svg>
            <span>{t('faqs.email')}: {email}</span>
          </a>
        </div>
      </div>
    </div>
  );
}