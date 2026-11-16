import { readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { isFile, nightlyTag, object, readJson, windowsInstaller, writeJson } from './release-data.js';

const manifest = 'atlas-desktop-update.json';
const platforms = {
  'darwin-aarch64': 'atlas-desktop-aarch64-apple-darwin.app.tar.gz',
  'windows-x86_64': windowsInstaller,
};
const packages = Object.values(platforms);
export const desktopUpdaterAssets = [
  manifest, packages[0], ...packages.map((name) => name + '.sig'),
];

function updaterRef(tag: string): string {
  const nightly = nightlyTag.exec(tag);
  if (nightly) return `nightly/${nightly[2]}.${tag.split('.').at(-1)}`;
  if (/^v\d+\.\d+\.\d+$/.test(tag)) return `release/${tag}`;
  throw new Error('Desktop updater manifests require an official stable or nightly version');
}

export function generateDesktopUpdater(tag: string, dist: string): void {
  const ref = updaterRef(tag);
  const readCompatibility = (name: string) => {
    const info = object(readJson(path.join(dist, name + '.compatibility.json')));
    if (info.version !== tag.slice(1)
      || typeof info.workspaceSchema !== 'number' || !Number.isSafeInteger(info.workspaceSchema)
      || typeof info.foundryCommit !== 'string' || !/^[a-f0-9]{40}$/.test(info.foundryCommit)
      || typeof info.foundryVersion !== 'string' || !/^\d+\.\d+\.\d+$/.test(info.foundryVersion)) {
      throw new Error(`Invalid updater compatibility metadata for ${name}`);
    }
    return { workspaceSchema: info.workspaceSchema, foundryCommit: info.foundryCommit, foundryVersion: info.foundryVersion };
  };
  const [mac, windows] = packages.map(readCompatibility);
  if (JSON.stringify(mac) !== JSON.stringify(windows)) throw new Error('Desktop update compatibility metadata disagrees between platforms');
  writeJson(path.join(dist, manifest), {
    version: tag.slice(1),
    atlasCompatibility: mac,
    platforms: Object.fromEntries(Object.entries(platforms).map(([target, name]) => [target, {
      url: `https://github.com/e-kosten/pf2e-atlas/releases/download/${ref}/${name}`,
      signature: readFileSync(path.join(dist, name + '.sig'), 'utf8').trim(),
    }])),
  });
  validateDesktopUpdater(tag, dist);
}

export function validateDesktopUpdater(tag: string, dist: string): string[] {
  if (!desktopUpdaterAssets.some((name) => isFile(path.join(dist, name)))) return [];
  const ref = updaterRef(tag);
  for (const name of desktopUpdaterAssets) {
    const file = path.join(dist, name);
    if (!isFile(file) || statSync(file).size === 0) throw new Error(`Missing desktop update asset: ${name}`);
  }
  const info = object(readJson(path.join(dist, manifest)));
  if (info.version !== tag.slice(1)) throw new Error('Desktop update manifest version mismatch');
  const targets = object(info.platforms);
  if (Object.keys(targets).length !== Object.keys(platforms).length) throw new Error('Unexpected desktop update targets');
  for (const [target, name] of Object.entries(platforms)) {
    const asset = object(targets[target]);
    const signature = readFileSync(path.join(dist, name + '.sig'), 'utf8').trim();
    const signedComment = Buffer.from(signature, 'base64').toString('utf8').split(/\r?\n/)[2];
    if (asset.url !== `https://github.com/e-kosten/pf2e-atlas/releases/download/${ref}/${name}`
      || signature !== asset.signature
      || !signedComment?.split('\t').includes(`version:${tag.slice(1)}`)) {
      throw new Error(`Desktop update manifest has mismatched package identity for ${target}`);
    }
  }
  return desktopUpdaterAssets;
}
