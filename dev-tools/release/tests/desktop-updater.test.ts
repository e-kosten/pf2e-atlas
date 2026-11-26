import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { validateDesktopUpdater } from '../src/desktop-updater.js';
import { writeUpdaterFixture } from './release-fixtures.js';
import { publicAssetNames, sha256, windowsInstaller } from '../src/release-data.js';

test('desktop update feeds bind both packages to the dated release before publication', async () => {
  const directory = mkdtempSync(path.join(os.tmpdir(), 'atlas-update-release-'));
  const version = '0.2.3-nightly.20261116092300.gabcdef123456';
  const tag = `v${version}`;
  const mac = 'atlas-desktop-aarch64-apple-darwin.app.tar.gz';
  try {
    writeFileSync(path.join(directory, windowsInstaller), 'package fixture');
    await writeUpdaterFixture(directory, tag);
    const feed = JSON.parse(readFileSync(path.join(directory, 'atlas-desktop-update.json'), 'utf8'));
    assert.equal(feed.atlasCompatibility.workspaceSchema, 12);
    const published = new Set(publicAssetNames(tag));
    assert.ok(published.has('atlas-desktop-update.json'));
    for (const [platform, name] of [['darwin-aarch64', mac], ['windows-x86_64', windowsInstaller]]) {
      const update = feed.platforms[platform];
      assert.equal(update.url, `https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/20261116092300.gabcdef123456/${name}`);
      assert.equal(update.signature, readFileSync(path.join(directory, name + '.sig'), 'utf8'));
      assert.ok(!published.has(name + '.sig'));
      const descriptor = name + '.transition.json';
      for (const asset of [name, descriptor, descriptor + '.sig']) assert.ok(published.has(asset), asset);
      const transition = JSON.parse(readFileSync(path.join(directory, descriptor), 'utf8'));
      assert.equal(transition.sha256, await sha256(path.join(directory, name)));
      assert.equal(transition.asset, name);
    }

    writeFileSync(path.join(directory, windowsInstaller), 'other package');
    assert.throws(() => validateDesktopUpdater(tag, directory), /Invalid desktop transition descriptor/);
    rmSync(path.join(directory, windowsInstaller + '.sig'));
    assert.throws(() => validateDesktopUpdater(tag, directory), /Missing desktop update asset/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
