import { readdirSync } from 'node:fs';
import path from 'node:path';
import { desktopAssets, sha256 } from './release-data.js';

export type Run = (program: string, args: string[]) => string;

export const releaseEndpoint = 'repos/{owner}/{repo}/releases';

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
  return (JSON.parse(run('gh', ['api', '--paginate', '--slurp', `${releaseEndpoint}?per_page=100`])) as Release[][]).flat();
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

export async function verifyUploadedDesktop(release: Release, dist: string): Promise<void> {
  for (const { name, label } of desktopAssets) {
    const matching = release.assets?.filter((asset) => asset.name === name);
    if (matching?.length !== 1 || matching[0].digest !== `sha256:${await sha256(path.join(dist, name))}`) {
      throw new Error(`Uploaded ${label} digest mismatch for ${release.tag_name}`);
    }
  }
}

export function releaseAssetNames(dist: string): string[] {
  const entries = readdirSync(dist, { withFileTypes: true });
  const link = entries.find((entry) => entry.isSymbolicLink());
  if (link) throw new Error(`Release asset must be a regular file: ${link.name}`);
  return entries.filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
}

export function uploadDraftAssets(run: Run, draft: Release, dist: string, names: string[]): NonNullable<Release['assets']> {
  if (!draft.draft) throw new Error('Refusing to upload assets to a published release');
  const upload = new URL(draft.upload_url?.replace(/\{\?name,label\}$/, '') ?? '');
  if (upload.origin !== 'https://uploads.github.com' || upload.username || upload.password
    || !upload.pathname.endsWith(`/releases/${draft.id}/assets`)) {
    throw new Error('Invalid release draft upload URL');
  }
  return names.map((name) => {
    const previous = draft.assets?.find((asset) => asset.name === name);
    if (previous) run('gh', ['api', '--method', 'DELETE', `${releaseEndpoint}/assets/${previous.id}`]);
    upload.searchParams.set('name', name);
    const asset: NonNullable<Release['assets']>[number] = JSON.parse(run('gh', [
      'api', '--method', 'POST', upload.href, '--header', 'Content-Type: application/octet-stream',
      '--input', path.join(dist, name),
    ]));
    if (asset.name !== name) throw new Error(`Unexpected uploaded asset for ${name}`);
    return asset;
  });
}
