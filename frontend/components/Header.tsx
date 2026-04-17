'use client';

import { useState, useEffect } from 'react';
import { useLanguage, useTranslation } from '@/lib/i18n/LanguageContext';

interface HeaderProps {
  activeRoute?: 'edifact' | 'compression' | 'api' | 'docs' | 'faqs' | 'about';
  color?: 'blue' | 'purple';
}

export default function Header({ activeRoute = 'edifact', color = 'blue' }: HeaderProps) {
  const { t } = useTranslation();
  const { language, setLanguage } = useLanguage();
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const logoColor = color === 'blue' ? 'bg-blue-600' : 'bg-purple-600';
  const activeColor = color === 'blue' ? 'text-blue-600 dark:text-blue-400' : 'text-purple-600 dark:text-purple-400';

  // Close mobile menu when route changes
  useEffect(() => {
    setIsMenuOpen(false);
  }, [activeRoute]);

  const navLinks = [
    { href: '/', label: t('header.nav.edifactProcessor'), route: 'edifact' },
    { href: '/compression', label: t('header.nav.fraCompression'), route: 'compression' },
    { href: '#', label: t('header.nav.api'), route: 'api' },
    { href: '#', label: t('header.nav.docs'), route: 'docs' },
    { href: '/faqs', label: t('header.nav.faqs'), route: 'faqs' },
    { href: '/about', label: t('header.nav.about'), route: 'about' },
  ];

  return (
    <header className="border-b border-gray-200 dark:border-gray-800 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm">
      <div className="container mx-auto px-6 py-4 flex justify-between items-center">
        {/* Logo */}
        <div className="flex items-center gap-2">
          <div className={`w-8 h-8 ${logoColor} rounded-lg`}></div>
          <h1 className="text-xl font-bold text-gray-900 dark:text-white">{t('header.title')}</h1>
        </div>

        {/* Desktop navigation */}
        <nav className="hidden lg:flex items-center gap-6">
          {navLinks.map((link) => (
            <a
              key={link.route}
              href={link.href}
              className={
                activeRoute === link.route
                  ? `${activeColor} font-medium`
                  : 'text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400'
              }
            >
              {link.label}
            </a>
          ))}
          
          {/* Language selector desktop */}
          <div className="relative inline-block">
            <select
              value={language}
              onChange={(e) => setLanguage(e.target.value as 'en' | 'es')}
              className="appearance-none bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 py-1 px-3 pr-8 rounded-md text-sm border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="en">🇺🇸 EN</option>
              <option value="es">🇪🇸 ES</option>
            </select>
            <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700 dark:text-gray-300">
              <svg className="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z" />
              </svg>
            </div>
          </div>
        </nav>

        {/* Mobile menu button and language selector */}
        <div className="flex items-center gap-4 lg:hidden">
          {/* Language selector mobile */}
          <div className="relative inline-block">
            <select
              value={language}
              onChange={(e) => setLanguage(e.target.value as 'en' | 'es')}
              className="appearance-none bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 py-1 px-3 pr-8 rounded-md text-sm border border-gray-300 dark:border-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="en">🇺🇸 EN</option>
              <option value="es">🇪🇸 ES</option>
            </select>
            <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700 dark:text-gray-300">
              <svg className="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z" />
              </svg>
            </div>
          </div>

          {/* Hamburger button */}
          <button
            onClick={() => setIsMenuOpen(!isMenuOpen)}
            className="p-2 rounded-md text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800"
            aria-label="Toggle menu"
          >
            <svg
              className="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              {isMenuOpen ? (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              ) : (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              )}
            </svg>
          </button>
        </div>
      </div>

      {/* Mobile menu dropdown */}
      {isMenuOpen && (
        <div className="lg:hidden border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900">
          <div className="container mx-auto px-6 py-4 flex flex-col gap-4">
            {navLinks.map((link) => (
              <a
                key={link.route}
                href={link.href}
                className={
                  activeRoute === link.route
                    ? `${activeColor} font-medium py-2`
                    : 'text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 py-2'
                }
                onClick={() => setIsMenuOpen(false)}
              >
                {link.label}
              </a>
            ))}
          </div>
        </div>
      )}
    </header>
  );
}