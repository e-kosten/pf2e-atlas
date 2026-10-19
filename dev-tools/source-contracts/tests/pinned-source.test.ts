import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import ts from 'typescript';
import { sourceIdentity } from '../src/discovery/source-identity.js';
import { assertSourcePin, preparePinnedSource, type SourcePin } from '../src/generation/pinned-source.js';

async function fixture() {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas pinned source '));
  const source = path.join(directory, 'input');
  for (const relative of ['src', 'types', 'static/lang']) await mkdir(path.join(source, relative), { recursive: true });
  for (const [relative, text] of Object.entries({
    'src/example.ts': 'export type Example = string;',
    'types/example.d.ts': 'declare type Example = string;',
    'package.json': '{"version":"fixture"}',
    'tsconfig.json': '{}', 'static/system.json': '{}', 'static/lang/en.json': '{}',
  })) await writeFile(path.join(source, relative), text);
  const identity = await sourceIdentity(source);
  const pin: SourcePin = { repository: 'https://github.com/foundryvtt/pf2e.git', commit: 'a'.repeat(40),
    systemVersion: 'fixture', sourceDigest: identity.source_digest, inputFileCount: identity.input_file_count,
    typescriptVersion: ts.version, ruleKeys: [] };
  return { directory, source, pin, identity };
}

test('source admission checks bytes, file count and version independently', async t => {
  const { directory, identity, pin } = await fixture();
  t.after(() => rm(directory, { recursive: true, force: true }));
  assertSourcePin(identity, pin);
  for (const change of [{ source_digest: 'changed' }, { input_file_count: 1 }, { system_version: 'other' }])
    assert.throws(() => assertSourcePin({ ...identity, ...change }, pin), /do not match/);
});

test('local source is isolated, a warm cache works, and corrupt source never authorizes output', async t => {
  const { directory, source, pin } = await fixture();
  t.after(() => rm(directory, { recursive: true, force: true }));
  const cache = path.join(directory, 'cache');
  const exported = await preparePinnedSource(pin, cache, source);
  assert.equal((await sourceIdentity(exported)).source_digest, pin.sourceDigest);
  await assert.rejects(readFile(path.join(source, 'node_modules')), /ENOENT/);
  assert.equal(await preparePinnedSource(pin, cache), exported);
  await assert.rejects(preparePinnedSource(pin, source, source), /separate/);
  await writeFile(path.join(exported, 'src/example.ts'), 'changed cached declaration');
  await assert.rejects(preparePinnedSource(pin, cache), /do not match/);
  await writeFile(path.join(source, 'src/example.ts'), 'changed caller declaration');
  await assert.rejects(preparePinnedSource(pin, path.join(directory, 'new cache'), source), /do not match/);
});

test('cold Git acquisition exports the exact pinned commit and reuses it without a remote', async t => {
  const { directory, source, pin } = await fixture();
  t.after(() => rm(directory, { recursive: true, force: true }));
  // Identity is scoped to this disposable test repository; no configuration is written.
  const git = (...args: string[]) => execFileSync('git', ['-C', source,
    '-c', 'user.name=Atlas source fixture', '-c', 'user.email=source-fixture@example.invalid',
    '-c', 'commit.gpgsign=false', '-c', 'core.hooksPath=/dev/null', ...args], { encoding: 'utf8' }).trim();
  git('init', '-b', 'main'); git('add', '.'); git('commit', '-m', 'fixture');
  pin.commit = git('rev-parse', 'HEAD'); pin.repository = source;
  const cache = path.join(directory, 'cache');
  const exported = await preparePinnedSource(pin, cache);
  assert.equal((await sourceIdentity(exported)).source_digest, pin.sourceDigest);
  pin.repository = path.join(directory, 'absent remote');
  assert.equal(await preparePinnedSource(pin, cache), exported);
});
