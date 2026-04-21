import { readdir } from 'fs/promises';
import { join } from 'path';
import GenerateClient from './GenerateClient';

async function getAvailableVersions(): Promise<string[]> {
  try {
    // standards directory is at project root, one level up from frontend
    const standardsDir = join(process.cwd(), '..', 'standards');
    const files = await readdir(standardsDir);
    const versions = files
      .filter(file => file.endsWith('.json'))
      .map(file => file.replace('.json', ''))
      .sort();
    if (versions.length === 0) {
      console.warn('No standard JSON files found in', standardsDir);
    }
    return versions;
  } catch (error) {
    console.error('Failed to read standards directory:', error);
    // Fallback to some common versions
    return ['D96A', 'D01B', 'D95A', 'D94A', 'D93A', 'D921'];
  }
}

export default async function GeneratePage() {
  const availableVersions = await getAvailableVersions();
  return <GenerateClient availableVersions={availableVersions} />;
}