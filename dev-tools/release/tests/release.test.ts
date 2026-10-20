import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test, { type TestContext } from 'node:test';
import { fileURLToPath } from 'node:url';
import { archiveEntries } from '../src/archives.js';
import { generateManifest } from '../src/generate-release-manifest.js';
import { notices } from '../src/generate-notices.js';
import { pruneManifest } from '../src/prune-dist-manifest.js';
import { fixture, writeZip } from './release-fixtures.js';
import { jsonText, object, readJson, repoRoot, sha256, targets, writeJson } from '../src/release-data.js';
import { validateAssets } from '../src/validate-release-assets.js';

function temporary(t: TestContext): string {
  const root = mkdtempSync(path.join(os.tmpdir(), 'atlas release tools '));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  return root;
}
const noticeFixturePin = { commit: 'fixture-pin', systemVersion: '1.0' };
const noticeFixture = 'Source commit: fixture-pin (PF2e 1.0)\n';
const command = (name: string) => fileURLToPath(new URL(`../src/cli/${name}.js`, import.meta.url));

test('manifest and checksums cover supported packages and release extras deterministically', async (t) => {
  const dist = temporary(t);
  for (const target of Object.keys(targets)) writeFileSync(path.join(dist, `atlas-cli-${target}${target.includes('windows') ? '.zip' : '.tar.xz'}`), target);
  writeFileSync(path.join(dist, 'unrecognized.zip'), 'ignored');
  writeFileSync(path.join(dist, 'atlas-installer.sh'), 'installer\n');
  await generateManifest('v9.9.9', dist);
  const manifestText = readFileSync(path.join(dist, 'atlas-release-manifest.json'), 'utf8');
  const manifest = object(readJson(path.join(dist, 'atlas-release-manifest.json')));
  assert.equal(manifest.version, 'v9.9.9');
  assert.ok(Array.isArray(manifest.assets));
  assert.equal(manifest.assets.length, 4);
  assert.ok(manifestText.indexOf('"arch"') < manifestText.indexOf('"target"'));
  const sums = readFileSync(path.join(dist, 'SHA256SUMS'), 'utf8');
  for (const item of manifest.assets.map(object)) {
    assert.equal(item.sha256, createHash('sha256').update(String(item.target)).digest('hex'));
    assert.ok(sums.includes(`${item.sha256}  ${item.name}\n`));
  }
  assert.ok(sums.includes(`${createHash('sha256').update(manifestText).digest('hex')}  atlas-release-manifest.json\n`));
  assert.ok(!sums.includes('unrecognized'));
  await generateManifest('v9.9.9', dist);
  assert.equal(readFileSync(path.join(dist, 'atlas-release-manifest.json'), 'utf8'), manifestText);
  assert.equal(readFileSync(path.join(dist, 'SHA256SUMS'), 'utf8'), sums);
});

test('hashing reads more than one chunk and handles binary bytes', async (t) => {
  const root = temporary(t);
  const bytes = Buffer.alloc(2 * 1024 * 1024 + 17, 0xa7);
  const file = path.join(root, 'binary.bin'); writeFileSync(file, bytes);
  assert.equal(await sha256(file), createHash('sha256').update(bytes).digest('hex'));
});

test('pruning keeps unknown metadata and removes missing package references', (t) => {
  const root = temporary(t), file = path.join(root, 'manifest.json');
  writeFileSync(path.join(root, 'keep.zip'), 'present');
  writeJson(file, { artifacts: { 'keep.zip': { kind: 'executable-zip', description: 'é' }, 'missing.zip': { kind: 'executable-zip' } },
    releases: [{ artifacts: ['missing.zip', 'keep.zip'], extra: true }], future: { retained: true } });
  pruneManifest(file, root);
  const result = object(readJson(file));
  assert.deepEqual(result.artifacts, { 'keep.zip': { kind: 'executable-zip', description: 'é' } });
  assert.deepEqual(result.releases, [{ artifacts: ['keep.zip'], extra: true }]);
  assert.deepEqual(result.future, { retained: true });
  assert.ok(readFileSync(file, 'utf8').includes('\\u00e9'));
  assert.equal(jsonText({ z: 1, a: 'é' }), '{\n  "a": "\\u00e9",\n  "z": 1\n}\n');
  assert.equal(jsonText({ '2': true, '10': false }), '{\n  "10": false,\n  "2": true\n}\n');
});

test('notices preserve dependency ordering, unknown licenses and native notices', () => {
  const content = notices({ packages: [
    { name: 'zeta', version: '1.0', license: null, source: null },
    { name: 'Alpha', version: '2.0', license: 'MIT', source: 'registry' },
    { name: 'Alpha', version: '1.0', license: 'Apache-2.0', source: null },
  ] }, noticeFixture, noticeFixturePin);
  assert.ok(content.indexOf('- Alpha 1.0: Apache-2.0 (workspace)') < content.indexOf('- Alpha 2.0: MIT (registry)'));
  assert.ok(content.includes('- zeta 1.0: UNKNOWN (workspace)'));
  assert.ok(content.includes('- PF2e Atlas: Apache-2.0'));
  assert.ok(content.includes('- ONNX Runtime: MIT'));
  assert.ok(content.endsWith('section before publishing the release.\n'));
});

test('release notices preserve both PF2e provenances and the complete applicable license text', () => {
  const upstream = readFileSync(path.join(repoRoot, 'web/atlas-ui/public/third-party-notices.txt'), 'utf8');
  const pin = readJson(path.join(repoRoot, 'dev-tools/source-contracts/source-pin.json'));
  assert.ok(upstream.includes(`Source commit: ${object(pin).commit}`), 'generated-model attribution must track the source pin');
  assert.ok(upstream.includes('Copyright 2019 Hooking and contributors'));
  assert.ok(upstream.includes('extracts and transforms'));
  assert.ok(upstream.includes('localization-key mappings'));
  assert.ok(upstream.includes('does not cover separately acquired game rules descriptions'));
  assert.ok(upstream.includes('Source commit: 4cbdaa37d6c33e9519561bae2c59a23e0288cbce'), 'retain the distinct action-font source pin');
  assert.ok(upstream.includes('235ad1a4540736c4521e07d5c8d3445b68dbe1398e8b32b7d7a5b2551a15fcee'));
  assert.ok(upstream.includes('Version 2.0, January 2004'));
  assert.ok(upstream.includes('9. Accepting Warranty or Additional Liability.'));
  assert.ok(upstream.includes('END OF TERMS AND CONDITIONS'));
  const composed = notices({ packages: [] }, upstream, pin);
  assert.ok(composed.includes(upstream.trimEnd()), 'release recipients receive the entire committed upstream notice');
  assert.throws(() => notices({ packages: [] }, upstream, { ...object(pin), commit: 'different-pin' }), /does not match source-contracts/);
  assert.throws(() => notices({ packages: [] }, upstream, { ...object(pin), systemVersion: 'different-version' }), /does not match source-contracts/);
});

test('release notice composition rejects empty upstream notices', () => {
  assert.throws(() => notices({ packages: [] }, '', noticeFixturePin), /PF2e notices are empty/);
  assert.throws(() => notices({ packages: [] }, ' \n\t', noticeFixturePin), /PF2e notices are empty/);
});

test('notice CLI fails without writing output when upstream notices are missing or empty', (t) => {
  const root = realpathSync(temporary(t)), modules = path.join(root, 'dev-tools/release/dist/src');
  mkdirSync(path.join(modules, 'cli'), { recursive: true });
  for (const name of ['generate-notices.js', 'release-data.js', 'cli/generate-notices.js']) {
    cpSync(fileURLToPath(new URL(`../src/${name}`, import.meta.url)), path.join(modules, name));
  }
  writeFileSync(path.join(root, 'package.json'), '{"type":"module"}');
  writeFileSync(path.join(root, 'Cargo.toml'), '[package]\nname = "notice-fixture"\nversion = "0.1.0"\nedition = "2024"\n');
  mkdirSync(path.join(root, 'src'));
  writeFileSync(path.join(root, 'src/main.rs'), 'fn main() {}\n');
  writeFileSync(path.join(root, 'Cargo.lock'), 'version = 4\n[[package]]\nname = "notice-fixture"\nversion = "0.1.0"\n');
  mkdirSync(path.join(root, 'dev-tools/source-contracts'), { recursive: true });
  writeFileSync(path.join(root, 'dev-tools/source-contracts/source-pin.json'), JSON.stringify(noticeFixturePin));
  const notice = path.join(root, 'web/atlas-ui/public/third-party-notices.txt');
  mkdirSync(path.dirname(notice), { recursive: true });
  for (const empty of [false, true]) {
    if (empty) writeFileSync(notice, ' \n\t');
    const result = spawnSync(process.execPath, [path.join(modules, 'cli/generate-notices.js')],
      { cwd: root, encoding: 'utf8', env: { ...process.env, CARGO_NET_OFFLINE: 'true' } });
    assert.equal(result.status, 1, result.stderr);
    assert.match(result.stderr, empty ? /PF2e notices are empty/ : /ENOENT.*third-party-notices.txt/);
    assert.equal(existsSync(path.join(root, 'THIRD-PARTY-NOTICES.md')), false);
  }
});

test('commands preserve usage exit 2 and fatal-input exit 1', (t) => {
  const root = temporary(t);
  for (const name of ['generate-notices', 'generate-release-manifest', 'prune-dist-manifest', 'validate-release-assets']) {
    const result = spawnSync(process.execPath, [command(name), '--bad-argument'], { encoding: 'utf8' });
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /usage:/);
  }
  const malformed = path.join(root, 'malformed.json'); writeFileSync(malformed, '{oops');
  assert.equal(spawnSync(process.execPath, [command('prune-dist-manifest'), malformed, root]).status, 1);
  assert.equal(spawnSync(process.execPath, [command('validate-release-assets'), 'v9.9.9', root]).status, 1);
  const dist = path.join(root, 'relative dist'); mkdirSync(dist);
  execFileSync(process.execPath, [command('generate-release-manifest'), 'v9.9.9', 'relative dist'],
    { env: { ...process.env, INIT_CWD: root } });
  assert.equal(object(readJson(path.join(dist, 'atlas-release-manifest.json'))).version, 'v9.9.9');
});

test('ZIP names are inspected without extraction; malformed ZIP fails', async (t) => {
  const root = temporary(t), file = path.join(root, 'with spaces.zip');
  await writeZip(file, { 'atlas/name with spaces': 'contents', 'atlas/é.txt': 'text' });
  assert.deepEqual(await archiveEntries(file), ['atlas/name with spaces', 'atlas/é.txt']);
  writeFileSync(file, 'not a ZIP');
  await assert.rejects(archiveEntries(file));
});

test('asset checks reject missing metadata, bad hashes, runtime data in archives and loose files', async (t) => {
  const root = temporary(t), dist = path.join(root, 'dist'); mkdirSync(dist);
  const archiveRoot = path.join(root, 'atlas'); mkdirSync(archiveRoot); writeFileSync(path.join(archiveRoot, 'atlas'), 'stub');
  for (const target of Object.keys(targets)) {
    const file = path.join(dist, `atlas-cli-${target}${target.includes('windows') ? '.zip' : '.tar.xz'}`);
    if (target.includes('windows')) await writeZip(file, { 'atlas/atlas.exe': 'stub' });
    else await fixture(['unix-archive', file, archiveRoot]);
  }
  for (const name of ['atlas-installer.sh', 'atlas-installer.ps1', 'THIRD-PARTY-NOTICES.md']) writeFileSync(path.join(dist, name), 'fixture\n');
  writeJson(path.join(dist, 'dist-manifest.json'), {});
  await generateManifest('v9.9.9', dist);
  await validateAssets(dist);
  const bad = path.join(root, 'bad');
  async function reset(): Promise<void> { rmSync(bad, { recursive: true, force: true }); cpSync(dist, bad, { recursive: true }); }
  await reset(); await fixture(['remove-target', path.join(bad, 'atlas-release-manifest.json')]);
  await assert.rejects(validateAssets(bad), /missing required target/);
  await reset(); await fixture(['bad-manifest-checksum', path.join(bad, 'atlas-release-manifest.json')]);
  await assert.rejects(validateAssets(bad), /checksum metadata mismatch/);
  await reset(); writeJson(path.join(bad, 'dist-manifest.json'), { artifacts: { 'missing.zip': { kind: 'executable-zip' } } });
  await assert.rejects(validateAssets(bad), /references missing asset/);
  await reset(); await fixture(['bad-zip', path.join(bad, 'atlas-cli-x86_64-pc-windows-msvc.zip')]);
  await generateManifest('v9.9.9', bad); // Hashes must pass before testing archive contents.
  await assert.rejects(validateAssets(bad), /release archive contains runtime data.*zip/);
  await reset(); writeFileSync(path.join(archiveRoot, 'model.onnx'), 'forbidden');
  await fixture(['unix-archive', path.join(bad, 'atlas-cli-aarch64-apple-darwin.tar.xz'), archiveRoot]);
  await generateManifest('v9.9.9', bad);
  await assert.rejects(validateAssets(bad), /release archive contains runtime data.*model.onnx/);
  await reset(); writeFileSync(path.join(bad, 'pf2e-index.sqlite'), 'forbidden');
  await assert.rejects(validateAssets(bad), /release assets contain runtime data/);
});
