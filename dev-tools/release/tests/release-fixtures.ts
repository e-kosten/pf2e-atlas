/** Fixture construction shared by shell and PowerShell smoke tests. */
import { execFileSync } from 'node:child_process';
import { copyFileSync, createWriteStream, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { pipeline } from 'node:stream/promises';
import { ZipFile } from 'yazl';
import { generateManifest } from '../src/generate-release-manifest.js';
import { generateDesktopUpdater } from '../src/desktop-updater.js';
import { cli, object, publicAssetNames, readJson, sha256, windowsInstaller, writeJson, type Json } from '../src/release-data.js';

export const windowsRuntimeFixture = (root: string) => Object.fromEntries(
  ['msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll', 'atlas-windows-runtime.json']
    .map((name) => [`${root}/${name}`, 'Windows runtime fixture']),
);

export async function writeZip(file: string, entries: Record<string, string | Buffer>): Promise<void> {
  const zip = new ZipFile();
  for (const [name, value] of Object.entries(entries)) zip.addBuffer(typeof value === 'string' ? Buffer.from(value) : value, name);
  const done = pipeline(zip.outputStream, createWriteStream(file));
  zip.end();
  await done;
}
// Models the metadata emitted by signing. Native rehearsals verify real signatures.
export async function writeUpdaterFixture(dist: string, tag: string): Promise<void> {
  const version = tag.slice(1);
  const nightly = /-nightly\.(\d{14}\.g([0-9a-f]{12}))$/.exec(tag);
  const compatibility = { workspaceSchema: 12, foundryCommit: '1'.repeat(40), foundryVersion: '8.7.0' };
  const mac = 'atlas-desktop-aarch64-apple-darwin.app.tar.gz';
  writeFileSync(path.join(dist, mac), 'fixture Mac update package');
  for (const name of [mac, windowsInstaller]) {
    const file = path.join(dist, name);
    const descriptor = file + '.transition.json';
    writeJson(file + '.compatibility.json', { version, ...compatibility });
    writeJson(descriptor, {
      protocol: 2, channel: nightly ? 'nightly' : 'stable', tag: nightly ? `nightly/${nightly[1]}` : `release/${tag}`,
      version, commit: (nightly?.[2] ?? '').padEnd(40, '1'), asset: name, sha256: await sha256(file),
      ...compatibility,
      migrationLedger: { applicationId: 1, entries: [{ version: 12, name: 'baseline', checksum: 'a'.repeat(64) }] },
    });
    for (const signed of [file, descriptor]) {
      const metadata = `untrusted comment: fixture\nfixture\ntrusted comment: timestamp:1\tfile:${path.basename(signed)}\tversion:${version}\nfixture\n`;
      writeFileSync(signed + '.sig', Buffer.from(metadata).toString('base64'));
    }
  }
  generateDesktopUpdater(tag, dist);
}

function copyPublicRelease(dist: string, tag: string, destination: string): void {
  mkdirSync(destination);
  for (const name of publicAssetNames(tag)) copyFileSync(path.join(dist, name), path.join(destination, name));
}

export async function fixture(args: string[]): Promise<void> {
  const [action, file, extra, notices] = args;
  if (!action || !file) throw new Error('Fixture action and path are required');
  switch (action) {
    case 'updater': await writeUpdaterFixture(file, extra); break;
    case 'public-release': copyPublicRelease(file, extra, notices); break;
    case 'build-public-release': {
      if (!extra.includes('-rc.')) await writeUpdaterFixture(file, extra);
      await generateManifest(extra, file);
      copyPublicRelease(file, extra, notices);
      break;
    }
    case 'windows-archive': {
      if (!extra || !notices) throw new Error('Repository root and generated notices are required');
      const root = path.basename(file, '.zip');
      const entries: Record<string, string | Buffer> = { ...windowsRuntimeFixture(root), [`${root}/atlas.exe`]: 'atlas 9.9.9\r\n' };
      for (const name of ['LICENSE', 'README.md']) entries[`${root}/${name}`] = readFileSync(path.join(extra, name));
      entries[`${root}/THIRD-PARTY-NOTICES.md`] = readFileSync(notices);
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
    case 'target-last': {
      if (!extra) throw new Error('Target is required');
      const manifest = object(readJson(file));
      if (!Array.isArray(manifest.assets)) throw new Error('Expected assets');
      manifest.assets.sort((a, b) => Number(object(a).target === extra && object(a).variant === 'cli-only') - Number(object(b).target === extra && object(b).variant === 'cli-only'));
      writeJson(file, manifest); break;
    }
    case 'legacy-manifest': {
      const manifest = object(readJson(file));
      if (!Array.isArray(manifest.assets)) throw new Error('Expected assets');
      manifest.assets = manifest.assets.filter((asset) => object(asset).variant === 'full').map((asset) => {
        const { variant: _variant, ...legacy } = object(asset); return legacy;
      });
      writeJson(file, manifest); break;
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
