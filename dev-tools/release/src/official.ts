import path from 'node:path';
import { macosDmg, object, officialPackageTag, readJson, sha256, windowsInstaller } from './release-data.js';
import { releaseAssetNames, releaseEndpoint, releases, uploadDraftAssets, verifyReleaseSource, type Release, type Run } from './release-upload.js';

export async function publishOfficial(tag: string, sha: string, dist: string, run: Run): Promise<void> {
  if (object(readJson(path.join(dist, 'atlas-release-manifest.json'))).version !== officialPackageTag(tag)) {
    throw new Error('Release manifest version mismatch');
  }
  const listed = releases(run).find((item) => item.tag_name === tag);
  if (!listed || !Number.isSafeInteger(listed.id) || listed.id <= 0) throw new Error(`Missing release ID for ${tag}`);
  const endpoint = `${releaseEndpoint}/${listed.id}`;
  const readRelease = (): Release => {
    const release: Release = JSON.parse(run('gh', ['api', endpoint]));
    if (release.id !== listed.id) throw new Error('Official release lookup returned a different ID');
    verifyReleaseSource(run, release, tag, sha);
    return release;
  };
  const names = releaseAssetNames(dist);
  const expected = new Map(await Promise.all(names.map(async (name) =>
    [name, `sha256:${await sha256(path.join(dist, name))}`] as const)));
  const release = readRelease();
  const prerelease = tag.startsWith('rc/');
  if (!release.draft && release.prerelease !== prerelease) throw new Error('Published release channel mismatch');
  if (release.draft) uploadDraftAssets(run, release, dist, names);
  const uploaded = release.draft ? readRelease() : release;
  if (release.draft && !uploaded.draft) throw new Error('Expected an official draft after uploading assets');
  for (const [name, digest] of expected) {
    const matching = uploaded.assets?.filter((asset) => asset.name === name);
    if (matching?.length !== 1 || matching[0].digest !== digest) {
      throw new Error(`Uploaded asset missing, duplicated or has a digest mismatch for ${tag}: ${name}`);
    }
  }
  if (!release.draft) return;
  const notes = uploaded.body ?? '';
  const downloads = [
    `[Download PF2e Atlas for Apple Silicon](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${macosDmg}). [First-open instructions](https://github.com/e-kosten/pf2e-atlas/blob/${sha}/docs/installation.md#macos-desktop-app).`,
    `[Download PF2e Atlas for Windows x64](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${windowsInstaller}). [Installation instructions](https://github.com/e-kosten/pf2e-atlas/blob/${sha}/docs/installation.md#windows-desktop-app).`,
  ].filter((line) => !notes.includes(line));
  const edit = ['api', '--method', 'PATCH', endpoint, '-F', 'draft=false', '-F', `prerelease=${prerelease}`, '-f', `make_latest=${!prerelease}`];
  if (downloads.length) edit.push('-f', `body=${notes.trimEnd()}\n\n${downloads.join('\n\n')}\n`);
  run('gh', edit);
}
