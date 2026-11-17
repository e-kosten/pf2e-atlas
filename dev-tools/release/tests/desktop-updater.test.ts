import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { generateDesktopUpdater, validateDesktopUpdater } from '../src/desktop-updater.js';
import { windowsInstaller } from '../src/release-data.js';

test('desktop update feeds bind both packages to the dated release before publication', () => {
  const directory = mkdtempSync(path.join(os.tmpdir(), 'atlas-update-release-'));
  const version = '0.2.3-nightly.20261116092300.gabcdef123456';
  const tag = `v${version}`;
  const mac = 'atlas-desktop-aarch64-apple-darwin.app.tar.gz';
  try {
    for (const name of [mac, windowsInstaller]) {
      writeFileSync(path.join(directory, name), 'package fixture');
      writeFileSync(path.join(directory, name + '.compatibility.json'), JSON.stringify({
        version, workspaceSchema: 12, foundryCommit: '1'.repeat(40), foundryVersion: '8.7.0',
      }));
      const metadata = Buffer.from(`untrusted comment: fixture\nfixture\ntrusted comment: timestamp:1\tfile:${name}\tversion:${version}\nfixture\n`);
      writeFileSync(path.join(directory, name + '.sig'), metadata.toString('base64'));
      const descriptor = name + '.transition.json';
      writeFileSync(path.join(directory, descriptor), JSON.stringify({
        protocol: 1, channel: 'nightly', tag: 'nightly/20261116092300.gabcdef123456',
        version, commit: 'abcdef123456' + '1'.repeat(28), asset: name,
        sha256: createHash('sha256').update('package fixture').digest('hex'),
        workspaceSchema: 12, foundryCommit: '1'.repeat(40), foundryVersion: '8.7.0',
        migrationLedger: { applicationId: 1, entries: [{ version: 12, name: 'baseline', checksum: 'a'.repeat(64) }] },
      }));
      writeFileSync(path.join(directory, descriptor + '.sig'), metadata.toString('base64'));
    }
    generateDesktopUpdater(tag, directory);
    const feed = JSON.parse(readFileSync(path.join(directory, 'atlas-desktop-update.json'), 'utf8'));
    assert.equal(feed.platforms['darwin-aarch64'].url,
      `https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/20261116092300.gabcdef123456/${mac}`);
    assert.equal(feed.atlasCompatibility.workspaceSchema, 12);

    writeFileSync(path.join(directory, windowsInstaller), 'other package');
    assert.throws(() => validateDesktopUpdater(tag, directory), /Invalid desktop transition descriptor/);
    rmSync(path.join(directory, windowsInstaller + '.sig'));
    assert.throws(() => validateDesktopUpdater(tag, directory), /Missing desktop update asset/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
