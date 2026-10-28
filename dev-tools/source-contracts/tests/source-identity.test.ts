import assert from 'node:assert/strict';
import { mkdtemp, mkdir, rename, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { sourceIdentity } from '../src/discovery/source-identity.js';

const files = [
  ['src/source.ts', 'export interface Source { value: number }'],
  ['types/base.d.ts', 'interface Base {}'],
  ['package.json', '{"version":"6.12.4"}'],
  ['tsconfig.json', '{}'],
  ['system.pf2e.json', '{"packs":[]}'],
  ['static/lang/en.json', '{}'],
] as const;

async function fixture(t: TestContext) {
  const root = await mkdtemp(path.join(tmpdir(), 'atlas-source-identity-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  for (const [name, bytes] of files) {
    await mkdir(path.dirname(path.join(root, name)), { recursive: true });
    await writeFile(path.join(root, name), bytes);
  }
  return root;
}

test('source identity ignores checkout location and files outside contract inputs', async (t) => {
  const firstRoot = await fixture(t);
  const secondRoot = await fixture(t);
  const first = await sourceIdentity(firstRoot);
  await writeFile(path.join(secondRoot, 'README.md'), 'Contributor notes');
  assert.deepEqual(await sourceIdentity(secondRoot), first);
  assert.equal(first.git_commit, null);
  assert.equal(first.git_clean, null);
  assert.equal(first.system_version, '6.12.4');
  assert.equal(first.input_file_count, 6);
});

for (const [name, changed] of [
  ['src/source.ts', 'export interface Source { value: string }'],
  ['types/base.d.ts', 'interface Base { label: string }'],
  ['package.json', '{"version":"6.12.5"}'],
  ['tsconfig.json', '{"compilerOptions":{"strict":true}}'],
  ['system.pf2e.json', '{"packs":[{"type":"Item"}]}'],
  ['static/lang/en.json', '{"PF2E.Label":"Changed"}'],
]) {
  test(`source digest changes when ${name} bytes change`, async (t) => {
    const root = await fixture(t);
    const before = await sourceIdentity(root);
    await writeFile(path.join(root, name), changed);
    const after = await sourceIdentity(root);
    assert.notEqual(after.source_digest, before.source_digest);
    assert.equal(after.input_file_count, 6);
  });
}

test('source digest binds relative names even when file contents are unchanged', async (t) => {
  const root = await fixture(t);
  const before = await sourceIdentity(root);
  await rename(path.join(root, 'src/source.ts'), path.join(root, 'src/renamed.ts'));
  const after = await sourceIdentity(root);
  assert.notEqual(after.source_digest, before.source_digest);
  assert.equal(after.input_file_count, 6);
});

test('source identity rejects a non-string package version', async (t) => {
  const root = await fixture(t);
  await writeFile(path.join(root, 'package.json'), '{"version":42}');
  await assert.rejects(sourceIdentity(root), /version must be a string/);
});
