import FileUpload from '@/components/FileUpload';

export default function Home() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-black font-sans">
      <header className="border-b border-gray-200 dark:border-gray-800 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm">
        <div className="container mx-auto px-6 py-4 flex justify-between items-center">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-blue-600 rounded-lg"></div>
            <h1 className="text-xl font-bold text-gray-900 dark:text-white">FileReduce</h1>
          </div>
          <nav className="flex gap-6">
            <a href="#" className="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400">Processor</a>
            <a href="#" className="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400">API</a>
            <a href="#" className="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400">Docs</a>
          </nav>
        </div>
      </header>
      <main className="container mx-auto px-6 py-12">
        <FileUpload />
      </main>
      <footer className="border-t border-gray-200 dark:border-gray-800 mt-12 py-8 text-center text-gray-600 dark:text-gray-400">
        <p>FileReduce v0.1.0 • Dynamic EDIFACT/JSONL processing with WebAssembly • <a href="https://github.com/Omar52344/filereduce" className="text-blue-600 dark:text-blue-400 hover:underline">GitHub</a></p>
      </footer>
    </div>
  );
}
