import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { coverageHeading, validateNightlyCoverage } from './embedding-coverage.js';
import { validateAssets } from './validate-release-assets.js';
import { desktopAssets, macosDmg, nightlyChannel, nightlyRef, nightlyTag, object, readJson, windowsInstaller } from './release-data.js';
import { expectedAssetDigests, readRelease, releaseEndpoint, releases, uploadDraftAssets, verifyReleaseSource, verifyUploadedAssets, type Release, type Run } from './release-upload.js';
import { cliVersion, lockPath, manifestPath, versionFiles, writeVersionFiles } from './package-version.mjs';
export type { Run } from './release-upload.js';

export function nightlyVersion(tag: string): string {
  const match = nightlyTag.exec(tag);
  if (!match || match[0] !== tag) throw new Error('Invalid nightly tag');
  return match[1];
}

/** Only the ephemeral build checkout is changed; no dependency versions move. */
export function prepareNightly(root: string, tag: string): void {
  const version = nightlyVersion(tag);
  const manifest = readFileSync(path.join(root, manifestPath), 'utf8');
  if (version.split('-')[0] !== cliVersion(manifest).split('-')[0]) throw new Error('Nightly must retain the source version core');
  const lock = readFileSync(path.join(root, lockPath), 'utf8');
  writeVersionFiles(root, versionFiles(manifest, lock, version));
}

export function commandRunner(root: string): Run {
  return (program, args) => execFileSync(program, args, {
    cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], maxBuffer: 64 * 1024 * 1024,
  }).trim();
}
function temporary<T>(work: (directory: string) => T): T {
  const directory = mkdtempSync(path.join(tmpdir(), 'atlas-nightly-'));
  try { return work(directory); } finally { rmSync(directory, { recursive: true, force: true }); }
}

function nightlyPackageTag(root: string, ref: string): string {
  const match = nightlyRef.exec(ref);
  if (!match || match[0] !== ref) throw new Error('Invalid nightly release tag');
  const core = cliVersion(readFileSync(path.join(root, manifestPath), 'utf8')).split('-')[0];
  return `v${core}-nightly.${match[1]}.g${match[2]}`;
}

export function planNightly(root: string, scheduled: boolean, run = commandRunner(root), now = new Date()): string {
  const sha = run('git', ['rev-parse', 'HEAD']).trim();
  if (!/^[0-9a-f]{40}$/.test(sha)) throw new Error('Invalid source SHA');
  const existing = releases(run);
  const channel = existing.find((release) => release.tag_name === nightlyChannel && !release.draft);
  if (scheduled && channel?.assets?.some((asset) => asset.name === 'nightly-version.txt')) {
    const tag = temporary((directory) => {
      run('gh', ['release', 'download', nightlyChannel, '--pattern', 'nightly-version.txt', '--dir', directory]);
      return readFileSync(path.join(directory, 'nightly-version.txt'), 'utf8').trim();
    });
    if (nightlyRef.test(tag) && existing.some((release) => release.tag_name === tag && release.target_commitish === sha && release.prerelease && !release.draft
      && desktopAssets.every(({ name }) => release.assets?.some((asset) =>
        asset.name === name && /^sha256:[a-f0-9]{64}$/.test(asset.digest ?? ''))))) {
      return 'build=false\n';
    }
  }
  const stamp = now.toISOString().replace(/[-:T]/g, '').slice(0, 14);
  const tag = `nightly/${stamp}.g${sha.slice(0, 12)}`;
  const packageTag = nightlyPackageTag(root, tag);
  nightlyVersion(packageTag);
  return `build=true\nsha=${sha}\ntag=${tag}\npackage-tag=${packageTag}\n`;
}

function createNightlyDraft(run: Run, tag: string, sha: string, version: string, notes: string): Release {
  const draft: Release = JSON.parse(run('gh', ['api', '--method', 'POST', releaseEndpoint,
    '-f', `tag_name=${tag}`, '-f', `target_commitish=${sha}`, '-f', `name=Nightly ${version}`,
    '-F', 'draft=true', '-F', 'prerelease=true', '-f', 'make_latest=false', '-f', `body=${notes}`]));
  if (!Number.isSafeInteger(draft.id) || draft.id <= 0 || !draft.draft || !draft.prerelease) {
    throw new Error('Expected the dated nightly draft before uploading assets');
  }
  verifyReleaseSource(run, draft, tag, sha, true);
  return draft;
}

export async function publishNightly(root: string, tag: string, sha: string, dist: string, coveragePath: string,
  run = commandRunner(root), validate: (directory: string, packageTag: string) => Promise<void> = validateAssets): Promise<void> {
  const packageTag = nightlyPackageTag(root, tag);
  const version = nightlyVersion(packageTag);
  if (!/^[0-9a-f]{40}$/.test(sha) || !tag.endsWith(`.g${sha.slice(0, 12)}`)) throw new Error('Nightly tag does not identify the source SHA');
  await validate(dist, packageTag);
  if (object(readJson(path.join(dist, 'atlas-release-manifest.json'))).version !== packageTag) throw new Error('Release manifest version mismatch');
  const coverage = await validateNightlyCoverage(root, dist, coveragePath, packageTag, sha);
  const existing = releases(run);
  const listed = existing.find((item) => item.tag_name === tag);
  const channel = existing.find((item) => item.tag_name === nightlyChannel);
  let alreadyCurrent = false;
  if (channel?.assets?.some((asset) => asset.name === 'nightly-version.txt')) {
    const current = temporary((directory) => {
      run('gh', ['release', 'download', nightlyChannel, '--pattern', 'nightly-version.txt', '--dir', directory]);
      return readFileSync(path.join(directory, 'nightly-version.txt'), 'utf8').trim();
    });
    const currentDate = nightlyRef.exec(current)?.[1];
    if (!currentDate) throw new Error('Invalid nightly channel pointer');
    if (currentDate > nightlyRef.exec(tag)![1]) return;
    alreadyCurrent = current === tag && !channel.draft;
  }
  const notes = [
    'Not a stable release',
    '',
    ...(coverage.notice ? [coverage.notice, ''] : []),
    `[Download PF2e Atlas for Apple Silicon](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${macosDmg}). [First-open instructions](https://github.com/e-kosten/pf2e-atlas/blob/${sha}/docs/installation.md#macos-desktop-app).`,
    '',
    `[Download PF2e Atlas for Windows x64](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${windowsInstaller}). [Installation instructions](https://github.com/e-kosten/pf2e-atlas/blob/${sha}/docs/installation.md#windows-desktop-app).`,
    '',
    '<details>',
    '<summary>CLI installation instructions</summary>',
    '',
    'Install with the installer for your platform:',
    '',
    '**macOS / Linux**',
    '',
    '```sh',
    '(',
    '  set -eu',
    '  installer=$(mktemp)',
    "  trap 'rm -f \"$installer\"' EXIT",
    "  curl --proto '=https' --tlsv1.2 -LsSf \\",
    `    https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/atlas-installer.sh -o "$installer"`,
    '  test -s "$installer"',
    `  sh "$installer" --version ${tag}`,
    ')',
    '```',
    '',
    '**Windows (PowerShell)**',
    '',
    '```powershell',
    `Invoke-WebRequest https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/atlas-installer.ps1 -OutFile atlas-installer.ps1`,
    `./atlas-installer.ps1 -Version ${tag}`,
    '```',
    '',
    'For first-time setup, see the [Quick Start](https://github.com/e-kosten/pf2e-atlas#quick-start).',
    '',
    '</details>',
    '',
    `Built from main commit ${sha}.`,
    '',
  ].join('\n');
  const expected = await expectedAssetDigests(dist, packageTag);
  const { id } = listed ?? createNightlyDraft(run, tag, sha, version, notes);
  // The listing may lag creation or publication. Bind every check to the draft's ID.
  const readDated = () => readRelease(run, id, tag, sha, expected);
  let release = readDated();
  if (!release.prerelease) throw new Error('Expected a dated nightly prerelease');
  if (release.draft) {
    uploadDraftAssets(run, release, dist, expected.keys());
    release = readDated();
    if (!release.draft || !release.prerelease) throw new Error('Expected the dated nightly draft after uploading assets');
    verifyUploadedAssets(release, expected);
    const published: Release = JSON.parse(run('gh', ['api', '--method', 'PATCH', `${releaseEndpoint}/${id}`,
      '-F', 'draft=false', '-F', 'prerelease=true', '-f', 'make_latest=false', '-f', `body=${notes}`]));
    if (published.id !== id) throw new Error('Published nightly ID differs from the draft');
    release = readDated();
  }
  if (release.draft || !release.prerelease) throw new Error('Expected a published dated nightly');
  verifyUploadedAssets(release, expected);
  if (coverage.notice ? !release.body?.includes(coverage.notice) : release.body?.includes(coverageHeading)) {
    throw new Error('Published nightly notes differ from the validated coverage report');
  }
  const channelNotes = [
    'Latest verified Nightly',
    '',
    `[Download PF2e Atlas for Apple Silicon](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${macosDmg})`,
    '',
    `[Download PF2e Atlas for Windows x64](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${windowsInstaller})`,
    '',
    `[Dated release notes and checksums](https://github.com/e-kosten/pf2e-atlas/releases/tag/${tag})`,
    '',
    '[Standalone CLI installation](https://github.com/e-kosten/pf2e-atlas/blob/main/docs/installation.md#nightly-builds)',
  ].join('\n');
  const publishChannelPage = () => run('gh', ['release', 'edit', nightlyChannel,
    '--draft=false', '--prerelease', '--latest=false', '--notes', channelNotes]);
  if (alreadyCurrent) {
    if (channel?.body !== channelNotes) publishChannelPage();
    return;
  }
  // Published dated assets stay immutable. Check actual downloads before advancing
  // the channel so a failed install preserves the last successful nightly.
  temporary((directory) => {
    run('gh', ['release', 'download', tag, '--pattern', 'atlas-installer.sh', '--dir', directory]);
    run('sh', [path.join(directory, 'atlas-installer.sh'), '--version', tag, '--install-dir', directory, '--yes']);
    const actual = run(path.join(directory, 'atlas'), ['--version']).trim();
    if (actual !== `atlas ${version}`) throw new Error(`Downloaded binary version mismatch: ${actual}`);
    run('sh', [path.join(directory, 'atlas-installer.sh'), '--version', tag, '--install-dir', directory, '--without-web', '--yes']);
    const cliVersion = run(path.join(directory, 'atlas'), ['--version']).trim();
    const cliInfo = JSON.parse(run(path.join(directory, 'atlas'), ['build-info', '--json']));
    if (cliVersion !== `atlas ${version}` || cliInfo.status !== 'ok' || cliInfo.data.version !== version || cliInfo.data.variant !== 'cli-only') {
      throw new Error('Downloaded CLI-only binary version or build variant mismatch');
    }
    if (!channel) {
      run('gh', ['release', 'create', nightlyChannel, '--target', sha, '--title', 'Nightly installer channel', '--notes',
        'Installers and a pointer to the newest validated nightly. Dated releases contain the binaries and source commit.',
        '--draft', '--prerelease', '--latest=false']);
    }
    run('gh', ['release', 'upload', nightlyChannel, path.join(dist, 'atlas-installer.sh'), path.join(dist, 'atlas-installer.ps1'), '--clobber']);
    const pointer = path.join(directory, 'nightly-version.txt');
    writeFileSync(pointer, tag + '\n');
    run('gh', ['release', 'upload', nightlyChannel, pointer, '--clobber']);
    publishChannelPage();
  });
}
