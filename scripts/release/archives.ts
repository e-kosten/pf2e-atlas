import { execFileSync } from 'node:child_process';
import { open } from 'yauzl';

/** Inspect ZIP central-directory names without decompressing release binaries. */
export function zipEntries(file: string): Promise<string[]> {
  return new Promise((resolve, reject) => {
    open(file, { lazyEntries: true }, (error, zip) => {
      if (error || !zip) { reject(error ?? new Error('ZIP could not be opened')); return; }
      const names: string[] = [];
      zip.on('error', (error: Error) => { zip.close(); reject(error); });
      zip.on('entry', (entry: { fileName: string }) => { names.push(entry.fileName); zip.readEntry(); });
      zip.on('end', () => resolve(names));
      zip.readEntry();
    });
  });
}
export async function archiveEntries(file: string): Promise<string[]> {
  if (file.endsWith('.zip')) return zipEntries(file);
  return execFileSync('tar', ['-tf', file], { encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 }).split(/\r?\n/).filter(Boolean);
}
