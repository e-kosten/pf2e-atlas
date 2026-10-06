import { execFileSync } from 'node:child_process';
import path from 'node:path';
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
  // GNU tar treats a drive-letter archive path as a remote host. Keep it local
  // using a relative filename, which also works with BSD tar.
  return execFileSync('tar', ['-tf', `./${path.basename(file)}`], {
    cwd: path.dirname(file), encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
  }).split(/\r?\n/).filter(Boolean);
}
