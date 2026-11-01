import path from 'node:path';
import { readdirSync, unlinkSync } from 'node:fs';
import { isFile, object, readJson, writeJson } from './release-data.js';

export function pruneManifest(file: string, dist: string): void {
  const manifest = object(readJson(file));
  const artifacts = object(manifest.artifacts ?? {});
  // Merge actual CLI-only build metadata before filtering the assembled asset set.
  const releases = manifest.releases ?? [];
  if (!Array.isArray(releases)) throw new Error('Expected releases array');
  for (const name of readdirSync(dist).filter((name) => /^cli-only-.*\.json$/.test(name))) {
    const extra = object(readJson(path.join(dist, name)));
    Object.assign(artifacts, object(extra.artifacts ?? {}));
    if (extra.assets) manifest.assets = { ...object(manifest.assets ?? {}), ...object(extra.assets) };
    if (extra.systems) manifest.systems = { ...object(manifest.systems ?? {}), ...object(extra.systems) };
    if (extra.linkage) {
      const linkage = manifest.linkage ?? [];
      if (!Array.isArray(linkage) || !Array.isArray(extra.linkage)) throw new Error('Expected linkage arrays');
      manifest.linkage = [...linkage, ...extra.linkage];
    }
    if (!Array.isArray(extra.releases)) throw new Error('Expected CLI-only releases array');
    for (const value of extra.releases) {
      const release = object(value);
      const destination = releases.map(object).find((item) => item.app_name === release.app_name && item.app_version === release.app_version);
      if (!destination) throw new Error('CLI-only release does not match the full release');
      if (!Array.isArray(destination.artifacts) || !Array.isArray(release.artifacts)) throw new Error('Expected release artifacts');
      destination.artifacts = [...new Set([...destination.artifacts, ...release.artifacts])];
    }
    unlinkSync(path.join(dist, name));
  }

  const kept = Object.fromEntries(Object.entries(artifacts).filter(([name]) => isFile(path.join(dist, name))));
  manifest.artifacts = kept;
  for (const value of releases) {
    const release = object(value);
    const names = release.artifacts ?? [];
    if (!Array.isArray(names)) throw new Error('Expected release artifacts array');
    release.artifacts = names.filter((name) => typeof name === 'string' && Object.hasOwn(kept, name));
  }
  writeJson(file, manifest);
}
