/** Fixture construction shared by shell and PowerShell smoke tests. */
import { execFileSync } from 'node:child_process';
import { createWriteStream, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { pipeline } from 'node:stream/promises';
import { ZipFile } from 'yazl';
import { cli, object, readJson, writeJson, type Json } from './release-data.js';

export async function writeZip(file: string, entries: Record<string, string | Buffer>): Promise<void> {
  const zip = new ZipFile();
  for (const [name, value] of Object.entries(entries)) zip.addBuffer(typeof value === 'string' ? Buffer.from(value) : value, name);
  const done = pipeline(zip.outputStream, createWriteStream(file));
  zip.end();
  await done;
}
export async function fixture(args: string[]): Promise<void> {
  const [action, file, extra] = args;
  if (!action || !file) throw new Error('Fixture action and path are required');
  switch (action) {
    case 'windows-archive': {
      if (!extra) throw new Error('Repository root is required');
      const root = path.basename(file, '.zip');
      const entries: Record<string, string | Buffer> = { [`${root}/atlas.exe`]: 'atlas 9.9.9\r\n' };
      for (const name of ['LICENSE', 'README.md', 'THIRD-PARTY-NOTICES.md']) entries[`${root}/${name}`] = readFileSync(path.join(extra, name));
      await writeZip(file, entries); break;
    }
    case 'unix-archive': {
      if (!extra) throw new Error('Archive source root is required');
      const source = path.resolve(extra);
      execFileSync('tar', ['-cJf', `./${path.basename(file)}`, '-C', path.dirname(source), path.basename(source)], {
        cwd: path.dirname(file),
      }); break;
    }
    case 'bad-zip': await writeZip(file, { 'atlas-cli-x86_64-pc-windows-msvc/vendor/pf2e/source.json': '{}\n' }); break;
    case 'dist-manifest': {
      const dist = path.dirname(file);
      const artifacts: Record<string, Json> = {};
      for (const name of readdirSync(dist).sort()) {
        if (!(name.endsWith('.tar.xz') || name.endsWith('.zip'))) continue;
        artifacts[name] = { kind: 'executable-zip' };
        artifacts[`${name}.sha256`] = { kind: 'checksum' };
        writeFileSync(path.join(dist, `${name}.sha256`), 'fixture checksum\n');
      }
      artifacts.SHA256SUMS = { kind: 'unified-checksum' };
      writeJson(file, { artifacts }); break;
    }
    case 'remove-target': {
      const manifest = object(readJson(file));
      if (!Array.isArray(manifest.assets)) throw new Error('Expected assets');
      manifest.assets = manifest.assets.filter((asset) => object(asset).target !== 'aarch64-apple-darwin');
      writeJson(file, manifest); break;
    }
    case 'bad-manifest-checksum': {
      const manifest = object(readJson(file));
      if (!Array.isArray(manifest.assets)) throw new Error('Expected assets');
      object(manifest.assets[0]).sha256 = '0'.repeat(64); writeJson(file, manifest); break;
    }
    case 'missing-artifact': {
      const manifest = object(readJson(file));
      object(manifest.artifacts)['missing.tar.xz'] = { kind: 'executable-zip' }; writeJson(file, manifest); break;
    }
    default: throw new Error(`Unknown release fixture action: ${action}`);
  }
}
cli(import.meta.url, async (args) => { await fixture(args); return 0; });
