import path from 'node:path';
import { macosDmg, sha256 } from './release-data.js';

export type Run = (program: string, args: string[]) => string;

export interface Release {
  id: number;
  upload_url?: string;
  tag_name: string;
  target_commitish: string;
  draft: boolean;
  prerelease: boolean;
  body?: string;
  assets?: { id: number; name: string; digest?: string }[];
}

export function releases(run: Run): Release[] {
  return (JSON.parse(run('gh', ['api', '--paginate', '--slurp', 'repos/{owner}/{repo}/releases?per_page=100'])) as Release[][]).flat();
}

export function verifyReleaseSource(run: Run, release: Release, tag: string, sha: string, nightly = false): void {
  const refs = run('git', ['ls-remote', 'origin', `refs/tags/${tag}`, `refs/tags/${tag}^{}`])
    .split('\n').filter(Boolean).map((line) => line.split('\t'));
  const commit = refs.find(([, ref]) => ref === `refs/tags/${tag}^{}`)?.[0]
    ?? refs.find(([, ref]) => ref === `refs/tags/${tag}`)?.[0];
  if (release.tag_name !== tag || (commit ? commit !== sha : !(nightly && release.draft && release.target_commitish === sha))) {
    throw new Error(`Release ${tag} does not point to build commit ${sha}`);
  }
}

export async function verifyUploadedDmg(release: Release, dist: string): Promise<void> {
  const matching = release.assets?.filter((asset) => asset.name === macosDmg);
  if (matching?.length !== 1 || matching[0].digest !== `sha256:${await sha256(path.join(dist, macosDmg))}`) {
    throw new Error(`Uploaded DMG digest mismatch for ${release.tag_name}`);
  }
}
