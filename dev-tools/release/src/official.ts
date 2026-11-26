import path from 'node:path';
import { macosDmg, object, officialPackageTag, readJson, windowsInstaller } from './release-data.js';
import { expectedAssetDigests, readRelease, releaseEndpoint, releases, uploadDraftAssets, verifyUploadedAssets, type Run } from './release-upload.js';

export async function publishOfficial(tag: string, sha: string, dist: string, run: Run): Promise<void> {
  const packageTag = officialPackageTag(tag);
  if (object(readJson(path.join(dist, 'atlas-release-manifest.json'))).version !== packageTag) {
    throw new Error('Release manifest version mismatch');
  }
  const listed = releases(run).find((item) => item.tag_name === tag);
  if (!listed) throw new Error(`Missing release ID for ${tag}`);
  const endpoint = `${releaseEndpoint}/${listed.id}`;
  const expected = await expectedAssetDigests(dist, packageTag);
  const release = readRelease(run, listed.id, tag, sha, expected);
  const prerelease = tag.startsWith('rc/');
  if (!release.draft && release.prerelease !== prerelease) throw new Error('Published release channel mismatch');
  if (release.draft) uploadDraftAssets(run, release, dist, expected.keys());
  const uploaded = release.draft ? readRelease(run, listed.id, tag, sha, expected) : release;
  if (release.draft && !uploaded.draft) throw new Error('Expected an official draft after uploading assets');
  verifyUploadedAssets(uploaded, expected);
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
