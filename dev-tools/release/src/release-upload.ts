import { lstatSync } from 'node:fs';
import path from 'node:path';
import { publicAssetNames, sha256 } from './release-data.js';

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

export async function expectedAssetDigests(dist: string, packageTag: string): Promise<Map<string, string>> {
  return new Map(await Promise.all(publicAssetNames(packageTag).map(async (name) => {
    const file = path.join(dist, name);
    if (!lstatSync(file).isFile()) throw new Error(`Release asset must be a regular file: ${name}`);
    return [name, `sha256:${await sha256(file)}`] as const;
  })));
}

export function readRelease(run: Run, id: number, tag: string, sha: string, expected: ReadonlyMap<string, string>): Release {
  if (!Number.isSafeInteger(id) || id <= 0) throw new Error(`Missing release ID for ${tag}`);
  const endpoint = `${releaseEndpoint}/${id}`;
  const release: Release = JSON.parse(run('gh', ['api', endpoint]));
  if (release.id !== id) throw new Error('Release lookup returned a different ID');
  verifyReleaseSource(run, release, tag, sha, tag.startsWith('nightly/'));
  const pages: NonNullable<Release['assets']>[] = JSON.parse(run('gh', [
    'api', '--paginate', '--slurp', endpoint + '/assets?per_page=100',
  ]));
  const assets = pages.flat();
  const extras = assets.filter(({ name }) => !expected.has(name));
  if (extras.length) throw new Error(`Unexpected assets for ${tag}: ${extras.map(({ name }) => name).join(', ')}`);
  return { ...release, assets };
}

export function verifyUploadedAssets(release: Release, expected: ReadonlyMap<string, string>): void {
  for (const [name, digest] of expected) {
    const matching = release.assets?.filter((asset) => asset.name === name);
    if (matching?.length !== 1 || matching[0].digest !== digest) {
      throw new Error(`Uploaded asset missing, duplicated or has a digest mismatch for ${release.tag_name}: ${name}`);
    }
  }
}

export function uploadDraftAssets(run: Run, draft: Release, dist: string, names: Iterable<string>): void {
  if (!draft.draft) throw new Error('Refusing to upload assets to a published release');
  const upload = new URL(draft.upload_url?.replace(/\{\?name,label\}$/, '') ?? '');
  if (upload.origin !== 'https://uploads.github.com' || upload.username || upload.password
    || !upload.pathname.endsWith(`/releases/${draft.id}/assets`)) {
    throw new Error('Invalid release draft upload URL');
  }
  for (const name of names) {
    const previous = draft.assets?.find((asset) => asset.name === name);
    if (previous) run('gh', ['api', '--method', 'DELETE', `${releaseEndpoint}/assets/${previous.id}`]);
    upload.searchParams.set('name', name);
    const asset: NonNullable<Release['assets']>[number] = JSON.parse(run('gh', [
      'api', '--method', 'POST', upload.href, '--header', 'Content-Type: application/octet-stream',
      '--input', path.join(dist, name),
    ]));
    if (asset.name !== name) throw new Error(`Unexpected uploaded asset for ${name}`);
  }
}
