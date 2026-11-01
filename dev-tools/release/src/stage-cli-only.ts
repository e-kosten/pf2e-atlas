import { renameSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { archiveName, isFile, object, readJson, sha256, targets, writeJson, type Json } from './release-data.js';

/** Keep cargo-dist packaging while giving the CLI-only build distinct artifact identities. */
export async function stageCliOnly(target: string, dist: string, manifestFile: string): Promise<void> {
  if (!Object.hasOwn(targets, target)) throw new Error(`Unsupported target: ${target}`);
  const original = archiveName(target, 'full');
  const renamed = archiveName(target, 'cli-only');
  if (!isFile(path.join(dist, original))) throw new Error(`Missing archive: ${original}`);
  const manifest = object(readJson(manifestFile));
  const artifacts = object(manifest.artifacts ?? {});
  if (!artifacts[original]) throw new Error(`cargo-dist manifest is missing ${original}`);
  const systems = new Set(Object.keys(object(manifest.systems ?? {})));
  const rewriteIdentity = (value: string) => systems.has(value) ? `cli-only:${target}:${value}`
    : value.replaceAll(`atlas-cli-${target}`, `atlas-cli-only-${target}`);
  function rewrite(value: Json): Json {
    if (typeof value === 'string') return rewriteIdentity(value);
    if (Array.isArray(value)) return value.map(rewrite);
    if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([key, member]) =>
      [rewriteIdentity(key), rewrite(member)]));
    return value;
  }
  renameSync(path.join(dist, original), path.join(dist, renamed));
  const digest = await sha256(path.join(dist, renamed));
  const originalChecksum = `${original}.sha256`;
  if (isFile(path.join(dist, originalChecksum))) renameSync(path.join(dist, originalChecksum), path.join(dist, `${renamed}.sha256`));
  writeFileSync(path.join(dist, `${renamed}.sha256`), `${digest}  ${renamed}\n`);
  writeJson(path.join(dist, `cli-only-${target}.json`), rewrite(manifest));
}
