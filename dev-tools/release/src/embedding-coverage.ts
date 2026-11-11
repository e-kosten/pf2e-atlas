import path from 'node:path';
import { archiveName, jsonText, object, readJson, sha256, type Json } from './release-data.js';

const profiles = ['bge-base', 'embeddinggemma'] as const;
const labels = { 'bge-base': 'BGE Base', embeddinggemma: 'EmbeddingGemma' } as const;
const target = 'x86_64-unknown-linux-gnu';
export const coverageArchive = archiveName(target, 'cli-only');
export const coverageHeading = '### Embedding coverage gap';

function exactKeys(value: { [key: string]: Json }, keys: readonly string[]): void {
  if (Object.keys(value).length !== keys.length || keys.some((key) => !Object.hasOwn(value, key))) {
    throw new Error('Unexpected embedding coverage report fields');
  }
}

export async function validateNightlyCoverage(
  root: string, dist: string, receipt: string, tag: string, sourceSha: string,
): Promise<{ archiveSha256: string; notice: string }> {
  const report = object(readJson(receipt));
  exactKeys(report, ['version', 'source_sha', 'tag', 'archive_sha256', 'pins', 'profiles']);
  if (report.version !== 1 || report.tag !== tag || report.source_sha !== sourceSha) {
    throw new Error('Embedding coverage report refers to a different nightly');
  }
  const pins = object(report.pins);
  exactKeys(pins, profiles);
  if (jsonText(pins) !== jsonText(readJson(path.join(root, 'crates/atlas-runtime/embedding-seeds.json')))) {
    throw new Error('Embedding coverage report pins differ from the candidate pins');
  }
  const archiveSha256 = await sha256(path.join(dist, coverageArchive));
  const assets = object(readJson(path.join(dist, 'atlas-release-manifest.json'))).assets;
  const archive = Array.isArray(assets)
    ? assets.map(object).find((asset) => asset.name === coverageArchive
        && asset.target === target && asset.variant === 'cli-only')
    : undefined;
  if (report.archive_sha256 !== archiveSha256 || archive?.sha256 !== archiveSha256) {
    throw new Error('Embedding coverage report archive differs from the assembled Linux CLI');
  }
  const results = object(report.profiles);
  exactKeys(results, profiles);
  const gaps = [];
  for (const profile of profiles) {
    const row = object(results[profile]);
    if (row.status === 'covered') {
      exactKeys(row, ['status', 'reused_inputs', 'inferred_inputs']);
      if (typeof row.reused_inputs !== 'number' || !Number.isSafeInteger(row.reused_inputs)
          || row.reused_inputs <= 0 || row.inferred_inputs !== 0) {
        throw new Error(`Invalid embedding coverage for ${profile}`);
      }
    } else if (row.status === 'gap') {
      if (row.reason === 'missing_input') {
        exactKeys(row, ['status', 'reason', 'first_hash']);
        if (typeof row.first_hash !== 'string' || !/^[0-9a-f]{64}$/.test(row.first_hash)) {
          throw new Error(`Invalid missing input hash for ${profile}`);
        }
      } else if (row.reason === 'reviewed_model_transition') {
        exactKeys(row, ['status', 'reason']);
      } else {
        throw new Error(`Unrecognized embedding gap for ${profile}`);
      }
      gaps.push(labels[profile]);
    } else {
      throw new Error(`Invalid embedding coverage status for ${profile}`);
    }
  }
  const notice = gaps.length ? [
    coverageHeading,
    '',
    `The pinned precomputed embeddings do not fully cover **${gaps.join(' and ')}** in this nightly.`,
    'If setup needs these embeddings, choose **Build embeddings locally** or run `atlas setup --build-embeddings-locally`.',
    'This can take an hour or more. You can also wait for a nightly with updated embeddings.',
  ].join('\n') : '';
  return { archiveSha256, notice };
}
