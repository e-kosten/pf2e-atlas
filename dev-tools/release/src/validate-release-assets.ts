import { readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { validateDesktopUpdater } from './desktop-updater.js';
import { archiveEntries } from './archives.js';
import { archiveName, compare, isFile, object, publicAssetNames, readJson, sha256, targets, variants } from './release-data.js';

const noticesName = 'THIRD-PARTY-NOTICES.md';
const manifestName = 'atlas-release-manifest.json';
export const containsRuntimeData = (entry: string) => ['pf2e-index.sqlite', 'model.onnx', 'tokenizer.json', 'vendor/pf2e/', 'hf-models/']
  .some((part) => entry.replaceAll('\\', '/').includes(part));
function inspectDirectory(dist: string, directory = dist): void {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isSymbolicLink()) throw new Error(`Release asset must be a regular file: ${entry.name}`);
    const file = path.join(directory, entry.name);
    const relative = path.relative(dist, file).replaceAll('\\', '/');
    if (['pf2e-index.sqlite', 'model.onnx', 'tokenizer.json'].includes(entry.name)
      || relative.includes('vendor/pf2e/') || relative.includes('hf-models/')) {
      throw new Error('release assets contain runtime data that must not be bundled');
    }
    if (entry.isDirectory()) inspectDirectory(dist, file);
  }
}

export async function validateAssets(dist: string, expectedTag: string): Promise<void> {
  const names = publicAssetNames(expectedTag);
  inspectDirectory(dist);
  for (const name of [...names, 'dist-manifest.json']) {
    const file = path.join(dist, name);
    if (!isFile(file) || statSync(file).size === 0) throw new Error(`missing required release asset: ${name}`);
  }
  const { assets, version } = object(readJson(path.join(dist, manifestName)));
  if (version !== expectedTag) throw new Error('Release manifest version mismatch');
  if (!Array.isArray(assets)) throw new Error('Expected release manifest assets array');

  const expectedSums = new Set(names.filter((name) => name !== 'SHA256SUMS'));
  const checksums = new Map<string, string>();
  const lines = readFileSync(path.join(dist, 'SHA256SUMS'), 'utf8').split(/\r?\n/);
  if (lines.at(-1) === '') lines.pop();
  for (const line of lines) {
    const match = /^([a-f0-9]{64})  (\S+)$/.exec(line);
    if (!match) throw new Error('Malformed SHA256SUMS entry');
    const [, digest, name] = match;
    if (!expectedSums.has(name) || checksums.has(name)) throw new Error(`checksum metadata mismatch for ${name}`);
    checksums.set(name, digest);
  }
  for (const variant of variants) {
    for (const target of Object.keys(targets)) {
      const matching = assets.map(object).filter((entry) => entry.target === target && entry.variant === variant);
      const asset = matching[0];
      if (matching.length !== 1 || !asset || asset.name !== archiveName(target, variant)) {
        throw new Error(`manifest is missing required target asset or has duplicates: ${target} (${variant})`);
      }
      if (typeof asset.sha256 !== 'string' || asset.sha256 !== checksums.get(asset.name)) {
        throw new Error(`checksum metadata mismatch for ${asset.name}`);
      }
    }
  }
  if (assets.length !== variants.length * Object.keys(targets).length) throw new Error('unexpected release manifest assets');
  for (const target of Object.keys(targets)) {
    const first = assets.map(object).find((entry) => entry.target === target);
    if (first?.variant !== 'full') throw new Error(`full build must precede CLI-only for legacy installers: ${target}`);
  }
  for (const name of expectedSums) {
    if (checksums.get(name) !== await sha256(path.join(dist, name))) throw new Error(`checksum metadata mismatch for ${name}`);
  }
  validateDesktopUpdater(expectedTag, dist);
  const artifacts = object(object(readJson(path.join(dist, 'dist-manifest.json'))).artifacts ?? {});
  for (const [name, entry] of Object.entries(artifacts)) {
    const kind = object(entry).kind;
    if (typeof kind === 'string' && ['executable-zip', 'checksum', 'unified-checksum'].includes(kind) && !isFile(path.join(dist, name))) {
      throw new Error(`dist-manifest.json references missing asset: ${name}`);
    }
  }
  for (const name of readdirSync(dist).sort(compare)) {
    if (!(name.endsWith('.tar.xz') || name.endsWith('.zip'))) continue;
    const entries = await archiveEntries(path.join(dist, name));
    for (const entry of entries) {
      if (containsRuntimeData(entry)) throw new Error(`release archive contains runtime data that must not be bundled: ${name}:${entry}`);
    }
    if (!entries.some((entry) => entry === noticesName || entry.endsWith(`/${noticesName}`))) {
      throw new Error(`release archive is missing ${noticesName}: ${name}`);
    }
    if (name.endsWith('-x86_64-pc-windows-msvc.zip')) {
      const binaries = entries.filter((entry) => path.posix.basename(entry).toLowerCase() === 'atlas.exe');
      if (binaries.length !== 1) throw new Error(`expected one packaged atlas.exe: ${name}`);
      const directory = path.posix.dirname(binaries[0]);
      for (const runtimeFile of ['msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll', 'atlas-windows-runtime.json']) {
        const expected = path.posix.join(directory, runtimeFile);
        if (entries.filter((entry) => entry.toLowerCase() === expected.toLowerCase()).length !== 1) {
          throw new Error(`expected ${runtimeFile} beside atlas.exe: ${name}`);
        }
      }
    }
  }
}
