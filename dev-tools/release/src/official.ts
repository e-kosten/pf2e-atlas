import { readdirSync } from 'node:fs';
import path from 'node:path';
import { macosDmg, object, officialPackageTag, readJson, windowsInstaller } from './release-data.js';
import { releases, verifyReleaseSource, verifyUploadedDesktop, type Run } from './release-upload.js';

export async function publishOfficial(tag: string, sha: string, dist: string, run: Run): Promise<void> {
  if (object(readJson(path.join(dist, 'atlas-release-manifest.json'))).version !== officialPackageTag(tag)) {
    throw new Error('Release manifest version mismatch');
  }
  const find = () => {
    const release = releases(run).find((item) => item.tag_name === tag);
    if (!release) throw new Error(`Missing release ${tag}`);
    verifyReleaseSource(run, release, tag, sha);
    return release;
  };
  const release = find();
  const prerelease = tag.startsWith('rc/');
  if (!release.draft) {
    if (release.prerelease !== prerelease) throw new Error('Published release channel mismatch');
    await verifyUploadedDesktop(release, dist);
    return;
  }
  const assets = readdirSync(dist, { withFileTypes: true }).filter((entry) => entry.isFile())
    .map((entry) => path.join(dist, entry.name)).sort();
  run('gh', ['release', 'upload', tag, ...assets, '--clobber']);
  const uploaded = find();
  if (!uploaded.draft) throw new Error('Expected an official draft after uploading assets');
  await verifyUploadedDesktop(uploaded, dist);
  const notes = uploaded.body ?? '';
  const downloads = [
    `[Download PF2e Atlas for Apple Silicon](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${macosDmg}). [First-open instructions](https://github.com/e-kosten/pf2e-atlas/blob/${sha}/docs/installation.md#macos-desktop-app).`,
    `[Download PF2e Atlas for Windows x64](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${windowsInstaller}). [Installation instructions](https://github.com/e-kosten/pf2e-atlas/blob/${sha}/docs/installation.md#windows-desktop-app).`,
  ].filter((line) => !notes.includes(line));
  const edit = ['release', 'edit', tag, '--draft=false', `--prerelease=${prerelease}`, `--latest=${!prerelease}`];
  if (downloads.length) edit.push('--notes', `${notes.trimEnd()}\n\n${downloads.join('\n\n')}\n`);
  run('gh', edit);
}
