import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { validateAssets } from './validate-release-assets.js';
import { object, readJson } from './release-data.js';

const nightlyTag = /^v([0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{14}\.g[0-9a-f]{12})$/;
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

export type Run = (program: string, args: string[]) => string;
export function commandRunner(root: string): Run {
  return (program, args) => execFileSync(program, args, {
    cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], maxBuffer: 64 * 1024 * 1024,
  }).trim();
}
interface Release {
  tag_name: string;
  target_commitish: string;
  draft: boolean;
  prerelease: boolean;
  assets?: { name: string }[];
}
function releases(run: Run): Release[] {
  return (JSON.parse(run('gh', ['api', '--paginate', '--slurp', 'repos/{owner}/{repo}/releases?per_page=100'])) as Release[][]).flat();
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
    if (nightlyTag.test(tag) && existing.some((release) => release.tag_name === tag && release.target_commitish === sha && release.prerelease && !release.draft)) {
      return 'build=false\n';
    }
  }
  const core = cliVersion(readFileSync(path.join(root, 'crates/atlas-cli/Cargo.toml'), 'utf8')).split('-')[0];
  const stamp = now.toISOString().replace(/[-:T]/g, '').slice(0, 14);
  const tag = `v${core}-nightly.${stamp}.g${sha.slice(0, 12)}`;
  nightlyVersion(tag);
  return `build=true\nsha=${sha}\ntag=${tag}\n`;
}

export async function publishNightly(root: string, tag: string, sha: string, dist: string,
  run = commandRunner(root), validate: (directory: string) => Promise<void> = validateAssets): Promise<void> {
  const version = nightlyVersion(tag);
  if (!/^[0-9a-f]{40}$/.test(sha) || !tag.endsWith(`.g${sha.slice(0, 12)}`)) throw new Error('Nightly tag does not identify the source SHA');
  await validate(dist);
  if (object(readJson(path.join(dist, 'atlas-release-manifest.json'))).version !== tag) throw new Error('Release manifest version mismatch');
  const existing = releases(run);
  const release = existing.find((item) => item.tag_name === tag);
  if (release && release.target_commitish !== sha) throw new Error('Existing nightly points at another commit');
  if (!release) {
    run('gh', ['release', 'create', tag, '--target', sha, '--title', `Nightly ${version}`, '--notes',
      [
        'Not a stable release',
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
        `Built from main commit ${sha}.`,
        '',
      ].join('\n'),
      '--draft', '--prerelease', '--latest=false']);
  }
  if (!release || release.draft) {
    const assets = readdirSync(dist, { withFileTypes: true }).filter((entry) => entry.isFile()).map((entry) => path.join(dist, entry.name)).sort();
    run('gh', ['release', 'upload', tag, ...assets, '--clobber']);
    run('gh', ['release', 'edit', tag, '--draft=false', '--prerelease', '--latest=false']);
  }
  // Published dated assets stay immutable. Check actual downloads before advancing
  // the channel so a failed install preserves the last successful nightly.
  temporary((directory) => {
    run('gh', ['release', 'download', tag, '--pattern', 'atlas-installer.sh', '--dir', directory]);
    run('sh', [path.join(directory, 'atlas-installer.sh'), '--version', tag, '--install-dir', directory, '--yes']);
    const actual = run(path.join(directory, 'atlas'), ['--version']).trim();
    if (actual !== `atlas ${version}`) throw new Error(`Downloaded binary version mismatch: ${actual}`);
    if (!existing.some((item) => item.tag_name === 'nightly')) {
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
