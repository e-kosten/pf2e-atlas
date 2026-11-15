import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { coverageArchive, coverageHeading, validateNightlyCoverage } from './embedding-coverage.js';
import { validateAssets } from './validate-release-assets.js';
import { macosDmg, object, readJson } from './release-data.js';
import { releases, verifyReleaseSource, verifyUploadedDmg, type Release, type Run } from './release-upload.js';
export type { Run } from './release-upload.js';

const nightlyTag = /^v([0-9]+\.[0-9]+\.[0-9]+-nightly\.([0-9]{14})\.g[0-9a-f]{12})$/;
export function nightlyVersion(tag: string): string {
  const match = nightlyTag.exec(tag);
  if (!match || match[0] !== tag) throw new Error('Invalid nightly tag');
  return match[1];
}

function cliVersion(manifest: string): string {
  const section = manifest.split(/^\[/m).find((part) => part.startsWith('package]'));
  if (!section || !/^name = "atlas-cli"\r?$/m.test(section)) throw new Error('Expected atlas-cli package');
  const versions = [...section.matchAll(/^version = "([^"]+)"\r?$/gm)];
  if (versions.length !== 1) throw new Error('Expected one explicit CLI version');
  return versions[0][1];
}

/** Only the ephemeral build checkout is changed; no dependency versions move. */
export function prepareNightly(root: string, tag: string): void {
  const version = nightlyVersion(tag);
  const manifestFile = path.join(root, 'crates/atlas-cli/Cargo.toml');
  const lockFile = path.join(root, 'Cargo.lock');
  const manifest = readFileSync(manifestFile, 'utf8');
  const lock = readFileSync(lockFile, 'utf8');
  const old = cliVersion(manifest);
  if (version.split('-')[0] !== old.split('-')[0]) throw new Error('Nightly must retain the source version core');
  const sections = lock.split(/(?=^\[\[package\]\])/m);
  const matches = sections.map((section, index) => /^name = "atlas-cli"\r?$/m.test(section) ? index : -1).filter((index) => index >= 0);
  if (matches.length !== 1) throw new Error('Expected one CLI lockfile entry');
  const index = matches[0];
  const locked = /^version = "([^"]+)"\r?$/m.exec(sections[index]);
  if (locked?.[1] !== old) throw new Error('CLI manifest and lockfile disagree');
  sections[index] = sections[index].replace(/^(version = ")[^"]+("\r?)$/m, `$1${version}$2`);
  // Restrict replacement to the package section, leaving dependencies untouched.
  const manifestSections = manifest.split(/(?=^\[)/m);
  const packageIndex = manifestSections.findIndex((section) => section.startsWith('[package]'));
  manifestSections[packageIndex] = manifestSections[packageIndex].replace(/^(version = ")[^"]+("\r?)$/m, `$1${version}$2`);
  writeFileSync(manifestFile, manifestSections.join(''));
  writeFileSync(lockFile, sections.join(''));
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

export function planNightly(root: string, scheduled: boolean, run = commandRunner(root), now = new Date()): string {
  const sha = run('git', ['rev-parse', 'HEAD']).trim();
  if (!/^[0-9a-f]{40}$/.test(sha)) throw new Error('Invalid source SHA');
  const existing = releases(run);
  const channel = existing.find((release) => release.tag_name === 'nightly' && !release.draft);
  if (scheduled && channel?.assets?.some((asset) => asset.name === 'nightly-version.txt')) {
    const tag = temporary((directory) => {
      run('gh', ['release', 'download', 'nightly', '--pattern', 'nightly-version.txt', '--dir', directory]);
      return readFileSync(path.join(directory, 'nightly-version.txt'), 'utf8').trim();
    });
    if (nightlyTag.test(tag) && existing.some((release) => release.tag_name === tag && release.target_commitish === sha && release.prerelease && !release.draft
      && release.assets?.some((asset) => asset.name === macosDmg && /^sha256:[a-f0-9]{64}$/.test(asset.digest ?? '')))) {
      return 'build=false\n';
    }
  }
  const core = cliVersion(readFileSync(path.join(root, 'crates/atlas-cli/Cargo.toml'), 'utf8')).split('-')[0];
  const stamp = now.toISOString().replace(/[-:T]/g, '').slice(0, 14);
  const tag = `v${core}-nightly.${stamp}.g${sha.slice(0, 12)}`;
  nightlyVersion(tag);
  return `build=true\nsha=${sha}\ntag=${tag}\n`;
}

const releaseEndpoint = 'repos/{owner}/{repo}/releases';

function nightlyDraft(run: Run, tag: string, sha: string, version: string, notes: string, existing?: Release): Release {
  const request = existing
    ? ['--method', 'PATCH', `${releaseEndpoint}/${existing.id}`]
    : ['--method', 'POST', releaseEndpoint, '-f', `tag_name=${tag}`, '-f', `target_commitish=${sha}`,
      '-f', `name=Nightly ${version}`, '-F', 'draft=true', '-F', 'prerelease=true', '-f', 'make_latest=false'];
  const draft: Release = JSON.parse(run('gh', ['api', ...request, '-f', `body=${notes}`]));
  if (!Number.isSafeInteger(draft.id) || draft.id <= 0 || (existing && existing.id !== draft.id)
    || !draft.draft || !draft.prerelease) {
    throw new Error('Expected the dated nightly draft before uploading assets');
  }
  verifyReleaseSource(run, draft, tag, sha, true);
  return draft;
}

async function uploadNightlyDraft(run: Run, draft: Release, dist: string): Promise<void> {
  const upload = new URL(draft.upload_url?.replace(/\{\?name,label\}$/, '') ?? '');
  if (upload.origin !== 'https://uploads.github.com' || upload.username || upload.password
    || !upload.pathname.endsWith(`/releases/${draft.id}/assets`)) {
    throw new Error('Invalid dated nightly upload URL');
  }
  const assets = readdirSync(dist, { withFileTypes: true }).filter((entry) => entry.isFile())
    .map((entry) => entry.name).sort();
  const uploaded = assets.map((name) => {
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
  await verifyUploadedDmg({ ...draft, assets: uploaded }, dist);
}

export async function publishNightly(root: string, tag: string, sha: string, dist: string, coveragePath: string,
  run = commandRunner(root), validate: (directory: string) => Promise<void> = validateAssets): Promise<void> {
  const version = nightlyVersion(tag);
  if (!/^[0-9a-f]{40}$/.test(sha) || !tag.endsWith(`.g${sha.slice(0, 12)}`)) throw new Error('Nightly tag does not identify the source SHA');
  await validate(dist);
  if (object(readJson(path.join(dist, 'atlas-release-manifest.json'))).version !== tag) throw new Error('Release manifest version mismatch');
  const coverage = await validateNightlyCoverage(root, dist, coveragePath, tag, sha);
  const existing = releases(run);
  const listed = existing.find((item) => item.tag_name === tag);
  const channel = existing.find((item) => item.tag_name === 'nightly');
  let alreadyCurrent = false;
  if (channel?.assets?.some((asset) => asset.name === 'nightly-version.txt')) {
    const current = temporary((directory) => {
      run('gh', ['release', 'download', 'nightly', '--pattern', 'nightly-version.txt', '--dir', directory]);
      return readFileSync(path.join(directory, 'nightly-version.txt'), 'utf8').trim();
    });
    const currentDate = nightlyTag.exec(current)?.[2];
    if (!currentDate) throw new Error('Invalid nightly channel pointer');
    if (currentDate > nightlyTag.exec(tag)![2]) return;
    alreadyCurrent = current === tag && !channel.draft;
  }
  // Recheck drafts by ID before mutating them. The list may lag publication too.
  const release: Release | undefined = listed?.draft
    ? JSON.parse(run('gh', ['api', `${releaseEndpoint}/${listed.id}`]))
    : listed;
  if (release) {
    if (release.id !== listed?.id) throw new Error('Dated nightly lookup returned a different release ID');
    verifyReleaseSource(run, release, tag, sha, true);
  }
  const notes = [
    'Not a stable release',
    '',
    ...(coverage.notice ? [coverage.notice, ''] : []),
    `[Download PF2e Atlas for Apple Silicon](https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/${macosDmg}). [First-open instructions](https://github.com/e-kosten/pf2e-atlas/blob/${sha}/docs/installation.md#macos-desktop-app).`,
    '',
    '<details>',
    '<summary>Install instructions</summary>',
    '',
    'Install with the installer for your platform:',
    '',
    '**macOS / Linux**',
    '',
    '```sh',
    "curl --proto '=https' --tlsv1.2 -LsSf \\",
    `  https://github.com/e-kosten/pf2e-atlas/releases/download/${tag}/atlas-installer.sh \\`,
    `  | sh -s -- --version ${tag}`,
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
  let published: Release;
  if (!release || release.draft) {
    // Carry GitHub's draft ID forward even if the release list still omits it.
    const draft = nightlyDraft(run, tag, sha, version, notes, release);
    await uploadNightlyDraft(run, draft, dist);
    published = JSON.parse(run('gh', ['api', '--method', 'PATCH', `${releaseEndpoint}/${draft.id}`,
      '-F', 'draft=false', '-F', 'prerelease=true', '-f', 'make_latest=false']));
    if (published.id !== draft.id) throw new Error('Published nightly ID differs from the draft');
  } else {
    const remote = release.assets?.find((asset) => asset.name === coverageArchive);
    if (remote?.digest !== `sha256:${coverage.archiveSha256}`) {
      throw new Error('Published nightly archive differs from the validated coverage report');
    }
    if (coverage.notice ? !release.body?.includes(coverage.notice) : release.body?.includes(coverageHeading)) {
      throw new Error('Published nightly notes differ from the validated coverage report');
    }
    published = release;
  }
  if (published.draft || !published.prerelease) throw new Error('Expected a published dated nightly');
  verifyReleaseSource(run, published, tag, sha, true);
  await verifyUploadedDmg(published, dist);
  if (alreadyCurrent) return;
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
      run('gh', ['release', 'create', 'nightly', '--target', sha, '--title', 'Nightly installer channel', '--notes',
        'Installers and a pointer to the newest validated nightly. Dated releases contain the binaries and source commit.',
        '--draft', '--prerelease', '--latest=false']);
    }
    run('gh', ['release', 'upload', 'nightly', path.join(dist, 'atlas-installer.sh'), path.join(dist, 'atlas-installer.ps1'), '--clobber']);
    const pointer = path.join(directory, 'nightly-version.txt');
    writeFileSync(pointer, tag + '\n');
    run('gh', ['release', 'upload', 'nightly', pointer, '--clobber']);
    run('gh', ['release', 'edit', 'nightly', '--draft=false', '--prerelease', '--latest=false']);
  });
}
