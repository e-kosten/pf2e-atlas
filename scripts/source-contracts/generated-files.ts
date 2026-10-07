import { mkdir, readFile, readdir, writeFile, unlink, lstat } from 'node:fs/promises';
import path from 'node:path';

/** Only generator-owned artifact directories; refuse symlinks and unrelated files. */
export async function artifactFiles(directory: string, relative = ''): Promise<string[]> {
  let entries;
  try {
    if ((await lstat(path.join(directory, relative))).isSymbolicLink())
      throw new Error(`Symlink in generated artifact directory: ${relative || directory}`);
    entries = await readdir(path.join(directory, relative), { withFileTypes: true });
  }
  catch (error) { if ((error as NodeJS.ErrnoException).code === 'ENOENT') return []; throw error; }
  const result: string[] = [];
  for (const entry of entries) {
    const file = relative ? `${relative}/${entry.name}` : entry.name;
    if (entry.isSymbolicLink()) throw new Error(`Symlink in generated artifact directory: ${file}`);
    if (entry.isDirectory()) result.push(...await artifactFiles(directory, file));
    else if (entry.isFile()) result.push(file);
    else throw new Error(`Unsupported artifact directory entry: ${file}`);
  }
  return result.sort();
}
export async function prepareGeneratedFiles(directory: string, expected: Record<string, string>, check: boolean,
  owned: (text: string) => boolean): Promise<() => Promise<void>> {
  const existing = await artifactFiles(directory);
  for (const file of Object.keys(expected)) {
    if (file.split('/').some(part => !part || part === '.' || part === '..') || path.isAbsolute(file) || file.includes('\\'))
      throw new Error(`Invalid generated artifact path: ${file}`);
    try {
      if (!(await lstat(path.join(directory, file))).isFile())
        throw new Error(`Expected generated file path is not a regular file: ${file}`);
    } catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
  }
  // Inspect everything before changing anything, including stale output.
  for (const file of existing) {
    const text = await readFile(path.join(directory, file), 'utf8');
    if (!owned(text)) throw new Error(`Unmanaged file in generated artifact directory: ${file}`);
    if (check && expected[file] !== text) throw new Error(`Generated artifact is stale: ${file}`);
  }
  if (check) {
    const missing = Object.keys(expected).filter(file => !existing.includes(file));
    if (missing.length) throw new Error(`Generated artifacts missing: ${missing.join(', ')}`);
    return async () => {};
  }
  return async () => {
    for (const [file, text] of Object.entries(expected)) {
      await mkdir(path.dirname(path.join(directory, file)), { recursive: true });
      await writeFile(path.join(directory, file), text);
    }
    for (const file of existing.filter(file => !(file in expected))) await unlink(path.join(directory, file));
  };
}
