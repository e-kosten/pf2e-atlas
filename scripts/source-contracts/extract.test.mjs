import assert from 'node:assert/strict';
import { cp, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { execFileSync, spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { SOURCE_ROOTS } from './type-graph.mjs';

const command = fileURLToPath(new URL('./extract.mjs', import.meta.url));

test('command writes discoverable failures and strict mode rejects incomplete extraction', async (t) => {
  const temporary = await mkdtemp(path.join(tmpdir(), 'atlas-extract-'));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const source = path.join(temporary, 'source');
  const output = path.join(temporary, 'output');
  await cp(new URL('./fixtures/trait-catalog/', import.meta.url), source, { recursive: true });
  await mkdir(path.join(source, 'types'));
  await writeFile(path.join(source, 'package.json'), '{"version":"test"}');
  const args = [command, '--source', source, '--out', output];
  const discovered = JSON.parse(execFileSync(process.execPath, args, { encoding: 'utf8' }));
  assert.equal(discovered.complete, false);
  assert.equal(discovered.type_graph_complete, false);
  assert.equal(discovered.trait_catalog_complete, true);
  assert.equal(spawnSync(process.execPath, [...args, '--strict']).status, 1);
  const graph = JSON.parse(await readFile(path.join(output, 'type-graph.json')));
  assert.ok(graph.diagnostics.some(({ code }) => code === 'missing-root'));
  assert.deepEqual(JSON.parse(await readFile(path.join(output, 'summary.json'))), discovered);
  assert.match(discovered.dependency_lock_digest, /^[a-f0-9]{64}$/);
  for (const { file, name } of SOURCE_ROOTS) {
    const target = path.join(source, file);
    await mkdir(path.dirname(target), { recursive: true });
    await writeFile(target, `export interface ${name} { value: string }\n`);
  }
  const complete = JSON.parse(execFileSync(process.execPath, [...args, '--strict'], { encoding: 'utf8' }));
  assert.equal(complete.complete, true);
  assert.notEqual(complete.source.source_digest, discovered.source.source_digest);
  assert.equal(complete.dependency_lock_digest, discovered.dependency_lock_digest);
});
