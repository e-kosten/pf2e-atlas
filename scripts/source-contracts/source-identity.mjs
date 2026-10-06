import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';

const inputDirectories = ['src', 'types'];
const inputFiles = ['package.json', 'tsconfig.json', 'static/system.json', 'static/lang/en.json'];

export async function sourceIdentity(sourceRoot) {
  const names = [...inputFiles];
  async function walk(relative) {
    const entries = await readdir(path.join(sourceRoot, relative), { withFileTypes: true });
    for (const entry of entries.sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
      const name = `${relative}/${entry.name}`;
      if (entry.isDirectory()) await walk(name);
      else if (entry.isFile()) names.push(name);
      else throw new Error(`Source input must be a regular file or directory: ${name}`);
    }
  }
  for (const directory of inputDirectories) await walk(directory);
  names.sort();
  const hash = createHash('sha256');
  for (const name of names) {
    const bytes = await readFile(path.join(sourceRoot, name));
    hash.update(`${Buffer.byteLength(name)}:${name}:${bytes.length}:`);
    hash.update(bytes);
  }
  const manifest = JSON.parse(await readFile(path.join(sourceRoot, 'package.json'), 'utf8'));
  let gitCommit = null;
  let gitClean = null;
  try {
    const gitRoot = execFileSync('git', ['-C', sourceRoot, 'rev-parse', '--show-toplevel'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
    if (path.resolve(gitRoot) === path.resolve(sourceRoot)) {
      gitCommit = execFileSync('git', ['-C', sourceRoot, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
      gitClean = execFileSync('git', ['-C', sourceRoot, 'status', '--porcelain', '--untracked-files=normal'], { encoding: 'utf8' }).trim() === '';
    }
  } catch {
    // An exported source directory has no Git identity; its bytes still identify it.
  }
  return { system_version: manifest.version ?? null, source_digest: hash.digest('hex'), input_file_count: names.length, git_commit: gitCommit, git_clean: gitClean };
}
