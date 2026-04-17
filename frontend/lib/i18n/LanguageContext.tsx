'use client';

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

// Import translation files
import enTranslations from './en.json';
import esTranslations from './es.json';

export type Language = 'en' | 'es';

export interface Translations {
  common: {
    appName: string;
    loading: string;
    error: string;
    success: string;
    remove: string;
    download: string;
    processing: string;
    upload: string;
    cancel: string;
    save: string;
    close: string;
  };
  header: {
    title: string;
    nav: {
      edifactProcessor: string;
      fraCompression: string;
      api: string;
      docs: string;
      faqs: string;
      about: string;
    };
  };
  footer: {
    copyright: string;
    github: string;
  };
  home: {
    title: string;
    subtitle: string;
    dropzone: {
      active: string;
      inactive: string;
      supportedFormats: string;
    };
    fileInfo: {
      type: string;
      size: string;
      unknownFileType: string;
    };
    processing: {
      mode: {
        local: string;
        backend: string;
        workerLoading: string;
        workerReady: string;
        localDescription: string;
        backendDescription: string;
      };
      compressToFra: string;
      processButton: string;
      processingButton: string;
    };
    results: {
      complete: string;
      dataPreview: string;
      downloadResults: string;
      originalFile: string;
      downloadJSONL: string;
      downloadCSV: string;
      downloadFRA: string;
    };
    features: {
      edifactToJsonl: {
        title: string;
        description: string;
      };
      jsonlFra: {
        title: string;
        description: string;
      };
      dynamicTranslations: {
        title: string;
        description: string;
      };
    };
  };
  compression: {
    title: string;
    subtitle: string;
    dropzone: {
      active: string;
      inactive: string;
      supportedFormats: string;
    };
     fileInfo: {
       type: string;
       size: string;
       operation: string;
       invalidFileType: string;
     };
     actions: {
       decompressToJSONL: string;
       compressToFRA: string;
     };
     processing: {
      processButton: string;
      processingButton: string;
    };
     results: {
       complete: string;
       downloadResults: string;
       downloadJSONL: string;
       downloadFRA: string;
       originalFile: string;
       processedFile: string;
     };
     features: {
       jsonlToFra: {
         title: string;
         description: string;
       };
       fraToJsonl: {
         title: string;
         description: string;
       };
     };
  };
  dashboard: {
    title: string;
    originalSize: string;
    processedSize: string;
    compressionRatio: string;
    savings: string;
    spaceSaved: string;
    reduction: string;
    cloudCost: string;
    cloudCostProjected: string;
    costSaving: string;
    perMonth: string;
    perYear: string;
    save: string;
    beforeCompression: string;
    inputFile: string;
    afterCompression: string;
    outputFile: string;
    bytes: string;
    kb: string;
    mb: string;
    gb: string;
  };
  errors: {
    unsupportedFileType: string;
    processingFailed: string;
    workerNotReady: string;
  };
}

// Type guard for translations
const translations: Record<Language, Translations> = {
  en: enTranslations,
  es: esTranslations,
};

interface LanguageContextType {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: Translations;
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useLanguage must be used within a LanguageProvider');
  }
  return context;
}

interface LanguageProviderProps {
  children: ReactNode;
}

export function LanguageProvider({ children }: LanguageProviderProps) {
  // Initialize language from localStorage or default to 'en'
  const [language, setLanguageState] = useState<Language>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('filereduce-language');
      if (saved === 'en' || saved === 'es') {
        return saved;
      }
      // Try to detect browser language
      const browserLang = navigator.language.split('-')[0];
      if (browserLang === 'es') return 'es';
    }
    return 'en';
  });

  // Update localStorage when language changes
  const setLanguage = (lang: Language) => {
    setLanguageState(lang);
    if (typeof window !== 'undefined') {
      localStorage.setItem('filereduce-language', lang);
      // Update html lang attribute
      document.documentElement.lang = lang;
    }
  };

  // Set html lang attribute on initial load
  useEffect(() => {
    if (typeof window !== 'undefined') {
      document.documentElement.lang = language;
    }
  }, [language]);

  const value: LanguageContextType = {
    language,
    setLanguage,
    t: translations[language],
  };

  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

// Helper hook for translations with optional interpolation
export function useTranslation() {
  const { t } = useLanguage();
  
  const translate = (key: string, params?: Record<string, string | number>) => {
    // Simple key path resolution (e.g., 'common.appName')
    const keys = key.split('.');
    let value: any = t;
    
    for (const k of keys) {
      if (value && typeof value === 'object' && k in value) {
        value = value[k];
      } else {
        console.warn(`Translation key not found: ${key}`);
        return key;
      }
    }
    
    if (typeof value === 'string' && params) {
      return value.replace(/\{\{(\w+)\}\}/g, (match, paramKey) => {
        return params[paramKey] !== undefined ? String(params[paramKey]) : match;
      });
    }
    
    return value;
  };
  
  return { t: translate };
}