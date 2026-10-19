import { readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { compare, isFile, sha256, targets, writeJson } from './release-data.js';

export async function generateManifest(tag: string, dist: string): Promise<void> {
  const assets = [];
  const sums: string[] = [];
  for (const name of readdirSync(dist).sort(compare)) {
    const file = path.join(dist, name);
    if (!isFile(file) || !(name.endsWith('.tar.xz') || name.endsWith('.zip'))) continue;
    const target = (Object.keys(targets) as (keyof typeof targets)[]).find((candidate) => name.includes(candidate));
    if (!target) continue;
    const digest = await sha256(file);
    const [os, arch] = targets[target];
    assets.push({ target, os, arch, name, sha256: digest });
    sums.push(`${digest}  ${name}`);
  }
  writeJson(path.join(dist, 'atlas-release-manifest.json'), { version: tag, assets });
  for (const name of ['atlas-installer.sh', 'atlas-installer.ps1', 'atlas-release-manifest.json', 'THIRD-PARTY-NOTICES.md']) {
    const file = path.join(dist, name);
    if (isFile(file)) sums.push(`${await sha256(file)}  ${name}`);
  }
  writeFileSync(path.join(dist, 'SHA256SUMS'), sums.sort(compare).join('\n') + '\n');
}
