import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { nightlyVersion, planNightly, prepareNightly, publishNightly, type Run } from '../src/nightly.js';

const sha = 'abcdef123456' + '0'.repeat(28);
const tag = 'v0.1.0-nightly.20261018092300.gabcdef123456';
function fixture(t: TestContext) {
  const root = mkdtempSync(path.join(tmpdir(), 'atlas-nightly-test-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const manifest = path.join(root, 'crates/atlas-cli/Cargo.toml'), lock = path.join(root, 'Cargo.lock');
  mkdirSync(path.dirname(manifest), { recursive: true });
  writeFileSync(manifest, '[package]\nname = "atlas-cli"\nversion = "0.1.0-rc.6"\n\n[dependencies.other]\nversion = "0.1.0-rc.6"\n');
  writeFileSync(lock, 'version = 4\n\n[[package]]\nname = "atlas-cli"\nversion = "0.1.0-rc.6"\n\n[[package]]\nname = "other"\nversion = "0.1.0-rc.6"\n');
  const dist = path.join(root, 'dist'); mkdirSync(dist);
  writeFileSync(path.join(dist, 'atlas-release-manifest.json'), JSON.stringify({ version: tag }));
  for (const name of ['atlas-installer.sh', 'atlas-installer.ps1', 'SHA256SUMS']) writeFileSync(path.join(dist, name), 'fixture\n');
  return { root, manifest, lock, dist };
}

test('nightly preparation changes only CLI versions and is idempotent', (t) => {
  const { root, manifest, lock } = fixture(t);
  prepareNightly(root, tag); prepareNightly(root, tag);
  for (const file of [manifest, lock]) {
    const content = readFileSync(file, 'utf8');
    assert.equal(content.split(tag.slice(1)).length - 1, 1);
    assert.equal(content.split('version = "0.1.0-rc.6"').length - 1, 1);
  }
});

test('invalid versions and inconsistent locks fail before changing either file', (t) => {
  const { root, manifest, lock } = fixture(t);
  for (const invalid of ['nightly', 'v0.1.0', tag + '\n', tag.replace('0.1.0', '9.0.0')]) {
    const before = [readFileSync(manifest, 'utf8'), readFileSync(lock, 'utf8')];
    assert.throws(() => prepareNightly(root, invalid));
    assert.deepEqual([readFileSync(manifest, 'utf8'), readFileSync(lock, 'utf8')], before);
  }
  writeFileSync(lock, readFileSync(lock, 'utf8').replace('name = "atlas-cli"', 'name = "missing"'));
  assert.throws(() => prepareNightly(root, tag), /one CLI lockfile entry/);
  assert.match(readFileSync(manifest, 'utf8'), /0.1.0-rc.6/);
});

test('scheduled skip requires the published channel to point to this exact source', (t) => {
  const { root } = fixture(t);
  for (const scenario of ['same', 'changed', 'draft', 'missing-channel', 'missing-pointer', 'manual']) {
    const dated = { tag_name: tag, target_commitish: scenario === 'changed' ? '1'.repeat(40) : sha, draft: scenario === 'draft', prerelease: true };
    const channel = { tag_name: 'nightly', draft: false, assets: scenario === 'missing-pointer' ? [] : [{ name: 'nightly-version.txt' }] };
    const run: Run = (program, args) => {
      if (program === 'git') return sha;
      if (args[0] === 'api') return JSON.stringify([[dated, ...(scenario === 'missing-channel' ? [] : [channel])]]);
      if (args[1] === 'download') writeFileSync(path.join(args[args.indexOf('--dir') + 1], 'nightly-version.txt'), tag + '\n');
      return '';
    };
    const result = planNightly(root, scenario !== 'manual', run, new Date('2026-10-18T09:23:00Z'));
    assert.equal(result, scenario === 'same' ? 'build=false\n' : `build=true\nsha=${sha}\ntag=${tag}\n`);
  }
});

function publisher(t: TestContext, options: { existing?: object[]; failInstall?: boolean; wrongVersion?: boolean; failValidation?: boolean } = {}) {
  const { root, dist } = fixture(t);
  const calls: string[][] = [];
  const run: Run = (program, args) => {
    calls.push([program, ...args]);
    if (program === 'gh' && args[0] === 'api') return JSON.stringify([options.existing ?? []]);
    if (program === 'sh' && options.failInstall) throw new Error('install failed');
    if (path.basename(program) === 'atlas') return options.wrongVersion ? 'atlas wrong' : `atlas ${nightlyVersion(tag)}`;
    return '';
  };
  const validate = async () => { if (options.failValidation) throw new Error('invalid assets'); };
  return { calls, execute: () => publishNightly(root, tag, sha, dist, run, validate), root, dist, run, validate };
}

test('channel advances only after dated publication and successful downloaded install', async (t) => {
  const { execute, calls } = publisher(t); await execute();
  const locate = (...prefix: string[]) => calls.findIndex((call) => prefix.every((value, index) => call[index] === value));
  assert.ok(locate('gh', 'release', 'edit', tag) < locate('sh'));
  assert.ok(locate('sh') < locate('gh', 'release', 'create', 'nightly'));
  const uploads = calls.filter((call) => call.slice(0, 4).join(' ') === 'gh release upload nightly');
  assert.equal(path.basename(uploads.at(-1)![4]), 'nightly-version.txt');
  for (const call of calls.filter((call) => call[1] === 'release' && ['create', 'edit'].includes(call[2]))) {
    assert.ok(call.includes('--latest=false')); assert.ok(call.includes('--prerelease'));
  }
});

test('validation, installation and binary-version failures preserve the channel', async (t) => {
  for (const options of [{ failValidation: true }, { failInstall: true }, { wrongVersion: true }]) {
    const { execute, calls } = publisher(t, options);
    await assert.rejects(execute);
    assert.ok(!calls.some((call) => call[3] === 'nightly'));
    if ('failValidation' in options) assert.deepEqual(calls, []);
  }
});

test('retry preserves published dated assets while repairing the channel', async (t) => {
  const { execute, calls } = publisher(t, { existing: [{ tag_name: tag, target_commitish: sha, draft: false }] });
  await execute();
  assert.ok(!calls.some((call) => ['upload', 'edit', 'create'].includes(call[2]) && call[3] === tag));
  assert.ok(calls.some((call) => call[2] === 'upload' && call[3] === 'nightly'));
});

test('source or manifest identity mismatches cannot publish', async (t) => {
  const { root, dist, run, validate, calls } = publisher(t);
  await assert.rejects(() => publishNightly(root, tag, '1'.repeat(40), dist, run, validate), /source SHA/);
  writeFileSync(path.join(dist, 'atlas-release-manifest.json'), JSON.stringify({ version: 'v9.9.9' }));
  await assert.rejects(() => publishNightly(root, tag, sha, dist, run, validate), /manifest version/);
  assert.deepEqual(calls, []);
});
