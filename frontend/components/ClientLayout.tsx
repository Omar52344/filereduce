'use client';

import { usePathname } from 'next/navigation';
import { LanguageProvider } from '@/lib/i18n/LanguageContext';
import Header from './Header';
import Footer from './Footer';

interface ClientLayoutProps {
  children: React.ReactNode;
}

export default function ClientLayout({ children }: ClientLayoutProps) {
  const pathname = usePathname();

  // Determine active route based on pathname
  let activeRoute: 'edifact' | 'compression' | 'api' | 'docs' | 'faqs' | 'about' = 'edifact';
  let color: 'blue' | 'purple' = 'blue';
  
  if (pathname === '/compression') {
    activeRoute = 'compression';
    color = 'purple';
  } else if (pathname.startsWith('/api')) {
    activeRoute = 'api';
  } else if (pathname.startsWith('/docs')) {
    activeRoute = 'docs';
  } else if (pathname === '/faqs') {
    activeRoute = 'faqs';
  } else if (pathname === '/about') {
    activeRoute = 'about';
  }

  return (
    <LanguageProvider>
      <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-black font-sans">
        <Header activeRoute={activeRoute} color={color} />
        <main className="container mx-auto px-6 py-12 flex-grow">
          {children}
        </main>
        <Footer />
      </div>
    </LanguageProvider>
  );
}