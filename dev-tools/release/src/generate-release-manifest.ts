import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { archiveName, compare, isFile, sha256, targets, variants, writeJson } from './release-data.js';

export async function generateManifest(tag: string, dist: string): Promise<void> {
  const assets = [];
  const sums: string[] = [];
  // Legacy installers select the first matching target, so full builds come first.
  for (const variant of variants) {
    for (const target of Object.keys(targets).sort(compare) as (keyof typeof targets)[]) {
      const name = archiveName(target, variant);
      const file = path.join(dist, name);
      if (!isFile(file)) continue;
      const digest = await sha256(file);
      const [os, arch] = targets[target];
      assets.push({ target, os, arch, name, variant, sha256: digest });
      sums.push(`${digest}  ${name}`);
    }
  }
  writeJson(path.join(dist, 'atlas-release-manifest.json'), { version: tag, assets });
  for (const name of ['atlas-installer.sh', 'atlas-installer.ps1', 'atlas-release-manifest.json', 'THIRD-PARTY-NOTICES.md']) {
    const file = path.join(dist, name);
    if (isFile(file)) sums.push(`${await sha256(file)}  ${name}`);
  }
  writeFileSync(path.join(dist, 'SHA256SUMS'), sums.sort(compare).join('\n') + '\n');
}
