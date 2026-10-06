import assert from 'node:assert/strict';
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { sourceIdentity } from './source-identity.js';

test('source identity follows source bytes and relative names, not location', async () => {
  const directories = await Promise.all([mkdtemp(path.join(tmpdir(), 'atlas-source-a-')), mkdtemp(path.join(tmpdir(), 'atlas-source-b-'))]);
  try {
    for (const root of directories) {
      for (const directory of ['src', 'types', 'static/lang']) await mkdir(path.join(root, directory), { recursive: true });
      for (const [name, bytes] of [['src/source.ts', 'export interface Source { value: number }'], ['types/base.d.ts', 'interface Base {}'], ['package.json', '{"version":"6.12.4"}'], ['tsconfig.json', '{}'], ['static/system.json', '{"packs":[]}'], ['static/lang/en.json', '{}']]) await writeFile(path.join(root, name), bytes);
    }
    const first = await sourceIdentity(directories[0]);
    assert.deepEqual(first, await sourceIdentity(directories[1]));
    assert.equal(first.git_commit, null);
    assert.equal(first.system_version, '6.12.4');
    await writeFile(path.join(directories[1], 'static/system.json'), '{"packs":[{"type":"Item"}]}');
    assert.notEqual(first.source_digest, (await sourceIdentity(directories[1])).source_digest);
    await writeFile(path.join(directories[1], 'package.json'), '{"version":42}');
    await assert.rejects(sourceIdentity(directories[1]), /version must be a string/);
    await writeFile(path.join(directories[1], 'package.json'), '{"version":"6.12.4"}');
    await writeFile(path.join(directories[1], 'static/system.json'), '{"packs":[]}');
    await writeFile(path.join(directories[1], 'src/source.ts'), 'export interface Source { value: string }');
    assert.notEqual(first.source_digest, (await sourceIdentity(directories[1])).source_digest);
  } finally {
    for (const root of directories) await rm(root, { recursive: true, force: true });
  }
});
