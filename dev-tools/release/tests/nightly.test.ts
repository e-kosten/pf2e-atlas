import { createHash } from 'node:crypto';
import { coverageArchive } from '../src/embedding-coverage.js';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { nightlyVersion, planNightly, prepareNightly, publishNightly, type Run } from '../src/nightly.js';
import { macosDmg } from '../src/release-data.js';

const sha = 'abcdef123456' + '0'.repeat(28);
const tag = 'v0.1.0-nightly.20261018092300.gabcdef123456';
const archiveBytes = 'checked CLI archive';
const archiveDigest = createHash('sha256').update(archiveBytes).digest('hex');
const dmgBytes = 'checked DMG';
const dmgAsset = { name: macosDmg, digest: `sha256:${createHash('sha256').update(dmgBytes).digest('hex')}` };
const pins = Object.fromEntries(['bge-base', 'embeddinggemma'].map((profile) => [profile, {
  tag: `embedding-seed/${profile}/v1`, asset: `${profile}.gz`, file: { sha256: 'b'.repeat(64), bytes: 1 },
}]));
function fixture(t: TestContext) {
  const root = mkdtempSync(path.join(tmpdir(), 'atlas-nightly-test-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const manifest = path.join(root, 'crates/atlas-cli/Cargo.toml'), lock = path.join(root, 'Cargo.lock');
  mkdirSync(path.dirname(manifest), { recursive: true });
  writeFileSync(manifest, '[package]\nname = "atlas-cli"\nversion = "0.1.0-rc.6"\n\n[dependencies.other]\nversion = "0.1.0-rc.6"\n');
  writeFileSync(lock, 'version = 4\n\n[[package]]\nname = "atlas-cli"\nversion = "0.1.0-rc.6"\n\n[[package]]\nname = "other"\nversion = "0.1.0-rc.6"\n');
  const dist = path.join(root, 'dist'); mkdirSync(dist);
  writeFileSync(path.join(dist, 'atlas-release-manifest.json'), JSON.stringify({
    version: tag, assets: [{ name: coverageArchive, target: 'x86_64-unknown-linux-gnu', variant: 'cli-only', sha256: archiveDigest }],
  }));
  writeFileSync(path.join(dist, coverageArchive), archiveBytes);
  writeFileSync(path.join(dist, macosDmg), dmgBytes);
  const pinPath = path.join(root, 'crates/atlas-runtime/embedding-seeds.json');
  mkdirSync(path.dirname(pinPath), { recursive: true });
  writeFileSync(pinPath, JSON.stringify(pins));
  for (const name of ['atlas-installer.sh', 'atlas-installer.ps1', 'SHA256SUMS']) writeFileSync(path.join(dist, name), 'fixture\n');
  return { root, manifest, lock, dist };
}

test('nightly preparation changes only CLI versions and is idempotent', (t) => {
  const { root, manifest, lock } = fixture(t);
  prepareNightly(root, tag); prepareNightly(root, tag);
  assert.equal(readFileSync(manifest, 'utf8'), `[package]\nname = "atlas-cli"\nversion = "${tag.slice(1)}"\n\n[dependencies.other]\nversion = "0.1.0-rc.6"\n`);
  assert.equal(readFileSync(lock, 'utf8'), `version = 4\n\n[[package]]\nname = "atlas-cli"\nversion = "${tag.slice(1)}"\n\n[[package]]\nname = "other"\nversion = "0.1.0-rc.6"\n`);
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
  for (const scenario of ['same', 'changed', 'draft', 'missing-channel', 'missing-pointer', 'missing-dmg', 'manual']) {
    const dated = { tag_name: tag, target_commitish: scenario === 'changed' ? '1'.repeat(40) : sha, draft: scenario === 'draft', prerelease: true,
      assets: scenario === 'missing-dmg' ? [] : [dmgAsset] };
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

function publisher(t: TestContext, options: { existing?: object[]; failInstall?: boolean; wrongVersion?: boolean; failValidation?: boolean; gap?: boolean; badCoverage?: boolean; badDmg?: boolean; channelTag?: string } = {}) {
  const { root, dist } = fixture(t);
  const coverage = path.join(root, 'embedding-coverage.json');
  const covered = { status: 'covered', inferred_inputs: 0, reused_inputs: 3 };
  writeFileSync(coverage, JSON.stringify({
    version: 1, source_sha: sha, tag, archive_sha256: options.badCoverage ? '0'.repeat(64) : archiveDigest,
    pins, profiles: { 'bge-base': covered, embeddinggemma: options.gap
      ? { status: 'gap', reason: 'missing_input', first_hash: 'a'.repeat(64) } : covered },
  }));
  const calls: string[][] = [];
  const releases = [...options.existing ?? []] as { tag_name: string; target_commitish?: string; draft?: boolean; prerelease?: boolean; assets?: object[] }[];
  const run: Run = (program, args) => {
    calls.push([program, ...args]);
    if (program === 'git') return releases.some((release) => release.tag_name === tag && !release.draft) ? `${sha}\trefs/tags/${tag}` : '';
    if (program === 'gh' && args[0] === 'api') return JSON.stringify([releases]);
    if (args[0] === 'release' && args[2] === tag) {
      if (args[1] === 'create') releases.push({ tag_name: tag, target_commitish: sha, draft: true, prerelease: true });
      const release = releases.find((item) => item.tag_name === tag)!;
      if (args[1] === 'upload') release.assets = options.badDmg ? [] : [dmgAsset];
      if (args[1] === 'edit' && args.includes('--draft=false')) release.draft = false;
    }
    if (args[0] === 'release' && args[1] === 'download' && args[2] === 'nightly' && options.channelTag) {
      writeFileSync(path.join(args[args.indexOf('--dir') + 1], 'nightly-version.txt'), options.channelTag);
    }
    if (program === 'sh' && options.failInstall) throw new Error('install failed');
    if (path.basename(program) === 'atlas' && args[0] === 'build-info') return JSON.stringify({ status: 'ok', data: { version: nightlyVersion(tag), variant: 'cli-only' } });
    if (path.basename(program) === 'atlas') return options.wrongVersion ? 'atlas wrong' : `atlas ${nightlyVersion(tag)}`;
    return '';
  };
  const validate = async () => { if (options.failValidation) throw new Error('invalid assets'); };
  return { calls, execute: () => publishNightly(root, tag, sha, dist, coverage, run, validate), root, dist, coverage, run, validate };
}

test('channel advances only after dated publication and successful downloaded install', async (t) => {
  const { execute, calls } = publisher(t); await execute();
  const locate = (...prefix: string[]) => calls.findIndex((call) => prefix.every((value, index) => call[index] === value));
  const datedPublication = locate('gh', 'release', 'edit', tag);
  const installation = locate('sh');
  const channelPublication = locate('gh', 'release', 'create', 'nightly');
  assert.notEqual(datedPublication, -1, 'dated release must be published');
  assert.notEqual(installation, -1, 'downloaded installer must run');
  assert.notEqual(channelPublication, -1, 'nightly channel must be published');
  assert.ok(datedPublication < installation);
  assert.ok(installation < channelPublication);
  const cliInstallation = calls.findIndex((call) => call[0] === 'sh' && call.includes('--without-web'));
  assert.ok(cliInstallation > installation && cliInstallation < channelPublication);
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
  const { execute, calls } = publisher(t, { existing: [{ tag_name: tag, target_commitish: sha, draft: false, prerelease: true, assets: [dmgAsset, { name: coverageArchive, digest: `sha256:${archiveDigest}` }] }] });
  await execute();
  assert.ok(!calls.some((call) => ['upload', 'edit', 'create'].includes(call[2]) && call[3] === tag));
  assert.ok(calls.some((call) => call[2] === 'upload' && call[3] === 'nightly'));
});

test('a bad uploaded DMG keeps the release in draft and an older retry cannot change the channel', async (t) => {
  const bad = publisher(t, { badDmg: true });
  await assert.rejects(bad.execute, /Uploaded DMG digest mismatch/);
  assert.ok(!bad.calls.some((call) => call.includes('--draft=false')));
  const old = publisher(t, { existing: [{ tag_name: 'nightly', assets: [{ name: 'nightly-version.txt' }] }],
    channelTag: tag.replace('20261018092300', '20261019092300') });
  await old.execute();
  assert.ok(!old.calls.some((call) => ['upload', 'edit', 'create'].includes(call[2])));
  const current = publisher(t, { existing: [
    { tag_name: 'nightly', draft: false, assets: [{ name: 'nightly-version.txt' }] },
    { tag_name: tag, target_commitish: sha, draft: false, prerelease: true,
      assets: [dmgAsset, { name: coverageArchive, digest: `sha256:${archiveDigest}` }] },
  ], channelTag: tag });
  await current.execute();
  assert.ok(!current.calls.some((call) => ['upload', 'edit', 'create'].includes(call[2])));
});

test('source or manifest identity mismatches cannot publish', async (t) => {
  const { root, dist, coverage, run, validate, calls } = publisher(t);
  await assert.rejects(() => publishNightly(root, tag, '1'.repeat(40), dist, coverage, run, validate), /source SHA/);
  writeFileSync(path.join(dist, 'atlas-release-manifest.json'), JSON.stringify({ version: 'v9.9.9' }));
  await assert.rejects(() => publishNightly(root, tag, sha, dist, coverage, run, validate), /manifest version/);
  assert.deepEqual(calls, []);
});


test('nightly discloses the affected profile before install instructions', async (t) => {
  const { execute, calls } = publisher(t, { gap: true });
  await execute();
  const create = calls.find((call) => call[2] === 'create' && call[3] === tag);
  assert.ok(create);
  const notes = create[create.indexOf('--notes') + 1];
  assert.match(notes, /\*\*EmbeddingGemma\*\*/);
  assert.match(notes, /--build-embeddings-locally/);
  assert.ok(notes.indexOf('Embedding coverage gap') < notes.indexOf('Install instructions'));
});

test('bad report and missing notice on a published nightly block the channel', async (t) => {
  const invalid = publisher(t, { badCoverage: true });
  await assert.rejects(invalid.execute, /archive differs/);
  assert.deepEqual(invalid.calls, []);
  const retry = publisher(t, { gap: true, existing: [{
    tag_name: tag, target_commitish: sha, draft: false, body: 'Not a stable release',
    assets: [{ name: coverageArchive, digest: `sha256:${archiveDigest}` }],
  }] });
  await assert.rejects(retry.execute, /notes differ/);
  assert.ok(!retry.calls.some((call) => call[3] === 'nightly'));
});
