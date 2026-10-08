import path from 'node:path';
import { isFile, object, readJson, writeJson } from './release-data.js';

export function pruneManifest(file: string, dist: string): void {
  const manifest = object(readJson(file));
  const artifacts = object(manifest.artifacts ?? {});
  const kept = Object.fromEntries(Object.entries(artifacts).filter(([name]) => isFile(path.join(dist, name))));
  manifest.artifacts = kept;
  const releases = manifest.releases ?? [];
  if (!Array.isArray(releases)) throw new Error('Expected releases array');
  for (const value of releases) {
    const release = object(value);
    const names = release.artifacts ?? [];
    if (!Array.isArray(names)) throw new Error('Expected release artifacts array');
    release.artifacts = names.filter((name) => typeof name === 'string' && Object.hasOwn(kept, name));
  }
  writeJson(file, manifest);
}
