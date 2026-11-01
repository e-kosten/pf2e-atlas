import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import type { Variant } from './release-data.js';

export function smokePackagedCli(binary: string, version: string, variant: Variant): void {
  const cwd = mkdtempSync(path.join(os.tmpdir(), 'atlas-cli-smoke-'));
  const run = (...args: string[]) => execFileSync(binary, args, { cwd, encoding: 'utf8' });
  try {
    if (run('--version').trim() !== `atlas ${version}`) throw new Error('Packaged version mismatch');
    const info = JSON.parse(run('build-info', '--json'));
    if (info.status !== 'ok' || info.data.version !== version || info.data.variant !== variant) throw new Error('Packaged build information mismatch');
    const help = run('--help');
    if (/^\s+web\s/m.test(help) !== (variant === 'full')) throw new Error('Packaged help has the wrong web surface');
    const web = spawnSync(binary, ['web', '--help'], { cwd, encoding: 'utf8' });
    if (web.status !== (variant === 'full' ? 0 : 2)) throw new Error('Packaged web command has the wrong availability');
    if (variant === 'cli-only' && !web.stderr.includes("unrecognized subcommand 'web'")) throw new Error('Packaged CLI-only build did not reject web');
    for (const command of ['setup', 'search', 'record', 'similar', 'lists']) run(command, '--help');
    const roll = JSON.parse(run('roll', '1+2', '--json'));
    if (roll.status !== 'ok' || roll.data.total !== 3) throw new Error('Packaged CLI JSON command failed');
    const skill = JSON.parse(run('agent', 'skills', 'install', '--target', 'agents', '--scope', 'workspace', '--yes', '--json'));
    if (skill.status !== 'ok') throw new Error('Packaged skill installation failed');
  } finally { rmSync(cwd, { recursive: true, force: true }); }
}
