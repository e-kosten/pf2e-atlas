import { readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { archiveEntries } from './archives.js';
import { argumentPath, cli, compare, isFile, object, readJson, sha256, targets } from './release-data.js';

const required = [
  'atlas-cli-aarch64-apple-darwin.tar.xz', 'atlas-cli-x86_64-unknown-linux-gnu.tar.xz',
  'atlas-cli-aarch64-unknown-linux-gnu.tar.xz', 'atlas-cli-x86_64-pc-windows-msvc.zip',
  'SHA256SUMS', 'dist-manifest.json', 'atlas-installer.sh', 'atlas-installer.ps1',
  'atlas-release-manifest.json', 'THIRD-PARTY-NOTICES.md',
];
export const containsRuntimeData = (entry: string) => ['pf2e-index.sqlite', 'model.onnx', 'tokenizer.json', 'vendor/pf2e/', 'hf-models/']
  .some((part) => entry.replaceAll('\\', '/').includes(part));
export async function validateAssets(dist: string): Promise<void> {
  const missing = required.filter((name) => !isFile(path.join(dist, name)) || statSync(path.join(dist, name)).size === 0);
  if (missing.length) throw new Error(missing.map((name) => `missing required release asset: ${name}`).join('\n'));
  const assets = object(readJson(path.join(dist, 'atlas-release-manifest.json'))).assets;
  if (!Array.isArray(assets)) throw new Error('Expected release manifest assets array');
  const checksums = readFileSync(path.join(dist, 'SHA256SUMS'), 'utf8').split(/\r?\n/);
  for (const target of Object.keys(targets)) {
    const asset = assets.map(object).find((entry) => entry.target === target);
    if (!asset || typeof asset.name !== 'string' || !isFile(path.join(dist, asset.name)) || statSync(path.join(dist, asset.name)).size === 0) {
      throw new Error(`manifest is missing required target asset: ${target}`);
    }
    const actual = await sha256(path.join(dist, asset.name));
    const sums = checksums.filter((line) => line.trim().split(/\s+/)[1] === asset.name).map((line) => line.trim().split(/\s+/)[0]);
    if (asset.sha256 !== actual || sums.length !== 1 || sums[0] !== actual) throw new Error(`checksum metadata mismatch for ${asset.name}`);
  }
  const artifacts = object(object(readJson(path.join(dist, 'dist-manifest.json'))).artifacts ?? {});
  for (const [name, entry] of Object.entries(artifacts)) {
    const kind = object(entry).kind;
    if (typeof kind === 'string' && ['executable-zip', 'checksum', 'unified-checksum'].includes(kind) && !isFile(path.join(dist, name))) {
      throw new Error(`dist-manifest.json references missing asset: ${name}`);
    }
  }
  function inspectDirectory(directory: string): void {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      const relative = path.relative(dist, file).replaceAll('\\', '/');
      if (['pf2e-index.sqlite', 'model.onnx', 'tokenizer.json'].includes(entry.name)
        || relative.includes('vendor/pf2e/') || relative.includes('hf-models/')) {
        throw new Error('release assets contain runtime data that must not be bundled');
      }
      if (entry.isDirectory()) inspectDirectory(file);
    }
  }
  inspectDirectory(dist);
  for (const name of readdirSync(dist).sort(compare)) {
    if (!(name.endsWith('.tar.xz') || name.endsWith('.zip'))) continue;
    for (const entry of await archiveEntries(path.join(dist, name))) {
      if (containsRuntimeData(entry)) throw new Error(`release archive contains runtime data that must not be bundled: ${name}:${entry}`);
    }
  }
}
cli(import.meta.url, async (args) => {
  if (args.length !== 2) { console.error('usage: npm --prefix scripts/release run validate -- <tag> <dist-dir>'); return 2; }
  await validateAssets(argumentPath(args[1]));
  return 0;
});
