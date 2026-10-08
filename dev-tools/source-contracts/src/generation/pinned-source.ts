import { execFile as execFileCallback, spawnSync } from 'node:child_process';
import { cp, lstat, mkdir, mkdtemp, readFile, realpath, rename, rm, symlink } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import ts from 'typescript';
import type { SourceIdentity } from '../contracts.js';
import { sourceIdentity } from '../discovery/source-identity.js';

const execFile = promisify(execFileCallback);
export const packageRoot = fileURLToPath(new URL('../../..', import.meta.url));
export const repositoryRoot = path.resolve(packageRoot, '../..');
export const defaultCache = path.join(repositoryRoot, '.cache/source-contracts');
export interface SourcePin {
  repository: string; commit: string; systemVersion: string; sourceDigest: string;
  inputFileCount: number; typescriptVersion: string; ruleKeys: string[];
}
export async function readSourcePin(): Promise<SourcePin> {
  const pin = JSON.parse(await readFile(path.join(packageRoot, 'source-pin.json'), 'utf8')) as SourcePin;
  if (!/^https:\/\/[A-Za-z0-9./_-]+\.git$/.test(pin.repository) || !/^[a-f0-9]{40}$/.test(pin.commit)
    || !/^[a-f0-9]{64}$/.test(pin.sourceDigest) || !Number.isSafeInteger(pin.inputFileCount)
    || pin.inputFileCount <= 0 || typeof pin.systemVersion !== 'string' || pin.typescriptVersion !== ts.version
    || !Array.isArray(pin.ruleKeys) || !pin.ruleKeys.every(key => /^[A-Z][A-Za-z0-9]*$/.test(key))
    || new Set(pin.ruleKeys).size !== pin.ruleKeys.length) throw new Error('Invalid source pin or TypeScript version mismatch');
  return pin;
}
export function assertSourcePin(identity: SourceIdentity, pin: SourcePin): void {
  if (identity.source_digest !== pin.sourceDigest || identity.input_file_count !== pin.inputFileCount
    || identity.system_version !== pin.systemVersion) throw new Error('Source bytes do not match source-pin.json; inspect extraction before intentionally updating the pin');
}
export function contains(parent: string, child: string): boolean {
  const relative = path.relative(path.resolve(parent), path.resolve(child));
  return !relative || relative !== '..' && !relative.startsWith('..' + path.sep) && !path.isAbsolute(relative);
}
const sourcePaths = ['src', 'types', 'package.json', 'tsconfig.json', 'static/system.json', 'static/lang/en.json'];
async function exists(file: string): Promise<boolean> {
  try { await lstat(file); return true; } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return false;
    throw error;
  }
}
/** Use an isolated export with this package's locked declarations, never mutate a caller's checkout. */
export async function preparePinnedSource(pin: SourcePin, cache = defaultCache, suppliedSource?: string): Promise<string> {
  for (const code of ['crates', 'dev-tools', 'scripts', 'docs', '.git'])
    if (contains(path.join(repositoryRoot, code), cache) || contains(cache, path.join(repositoryRoot, code)))
      throw new Error('Source cache must be separate from tracked code');
  if (suppliedSource && (contains(cache, suppliedSource) || contains(suppliedSource, cache)))
    throw new Error('Source cache must be separate from the supplied source');
  if (suppliedSource) assertSourcePin(await sourceIdentity(suppliedSource), pin);
  await mkdir(cache, { recursive: true });
  if ((await lstat(cache)).isSymbolicLink()) throw new Error('Source cache must not be a symlink');
  const source = path.join(cache, pin.commit, 'source');
  await mkdir(path.dirname(source), { recursive: true });
  if (!await exists(source)) {
    const stage = await mkdtemp(path.join(cache, 'export-'));
    try {
      if (suppliedSource) {
        for (const relative of sourcePaths) {
          await mkdir(path.dirname(path.join(stage, relative)), { recursive: true });
          await cp(path.join(suppliedSource, relative), path.join(stage, relative), { recursive: true });
        }
      } else {
        const git = path.join(cache, 'upstream.git');
        if (!await exists(git)) await execFile('git', ['init', '--bare', git]);
        try { await execFile('git', ['--git-dir', git, 'cat-file', '-e', pin.commit + '^{commit}']); }
        catch {
          console.error('Fetching pinned Foundry source ' + pin.commit);
          await execFile('git', ['--git-dir', git, 'fetch', '--depth=1', pin.repository, pin.commit],
            { timeout: 180_000, maxBuffer: 4 * 1024 * 1024 });
        }
        const { stdout } = await execFile('git', ['--git-dir', git, 'archive', pin.commit, ...sourcePaths],
          { encoding: 'buffer', maxBuffer: 64 * 1024 * 1024 });
        const unpack = spawnSync('tar', ['-xf', '-', '-C', stage], { input: stdout, encoding: 'utf8', timeout: 60_000 });
        if (unpack.error || unpack.status !== 0) throw new Error('Cannot export pinned source: ' + (unpack.error?.message ?? unpack.stderr));
      }
      assertSourcePin(await sourceIdentity(stage), pin);
      await rename(stage, source);
    } finally { await rm(stage, { recursive: true, force: true }); }
  }
  if ((await lstat(source)).isSymbolicLink()) throw new Error('Cached source must not be a symlink');
  assertSourcePin(await sourceIdentity(source), pin);
  const dependencies = path.join(source, 'node_modules'), lockedDependencies = await realpath(path.join(packageRoot, 'node_modules'));
  if (await exists(dependencies)) {
    if (!(await lstat(dependencies)).isSymbolicLink()) throw new Error('Cached dependency path must be a tool-owned symlink');
    if (await realpath(dependencies) === lockedDependencies) return source;
    await rm(dependencies);
  }
  await symlink(lockedDependencies, dependencies, process.platform === 'win32' ? 'junction' : 'dir');
  return source;
}
