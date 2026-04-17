'use client';

import { useTranslation } from '@/lib/i18n/LanguageContext';

export default function Footer() {
  const { t } = useTranslation();

  return (
    <footer className="border-t border-gray-200 dark:border-gray-800 mt-12 py-8 text-center text-gray-600 dark:text-gray-400">
      <p>{t('footer.copyright')}</p>
    </footer>
  );
}