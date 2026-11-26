import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { archiveName, compare, publicAssetNames, sha256, targets, variants, writeJson } from './release-data.js';

export async function generateManifest(tag: string, dist: string): Promise<void> {
  const assets = [];
  const names = publicAssetNames(tag);
  const archiveDigests = new Map<string, string>();
  // Legacy installers select the first matching target, so full builds come first.
  for (const variant of variants) {
    for (const target of Object.keys(targets).sort(compare) as (keyof typeof targets)[]) {
      const name = archiveName(target, variant);
      const file = path.join(dist, name);
      const digest = await sha256(file);
      const [os, arch] = targets[target];
      assets.push({ target, os, arch, name, variant, sha256: digest });
      archiveDigests.set(name, digest);
    }
  }
  writeJson(path.join(dist, 'atlas-release-manifest.json'), { version: tag, assets });
  const sums: string[] = [];
  for (const name of names) {
    if (name === 'SHA256SUMS') continue;
    const digest = archiveDigests.get(name) ?? await sha256(path.join(dist, name));
    sums.push(`${digest}  ${name}`);
  }
  writeFileSync(path.join(dist, 'SHA256SUMS'), sums.sort(compare).join('\n') + '\n');
}
