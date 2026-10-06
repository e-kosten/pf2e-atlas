import assert from 'node:assert/strict';
import { cp, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { execFileSync, spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const command = fileURLToPath(new URL('./extract.js', import.meta.url));

test('command writes discoverable failures and strict mode rejects incomplete extraction', async (t) => {
  const temporary = await mkdtemp(path.join(tmpdir(), 'atlas-extract-'));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const source = path.join(temporary, 'source');
  const output = path.join(temporary, 'output');
  await cp(new URL('../fixtures/trait-catalog/', import.meta.url), source, { recursive: true });
  await mkdir(path.join(source, 'types'));
  await writeFile(path.join(source, 'package.json'), '{"version":"test"}');
  await cp(new URL('../fixtures/type-graph/static/system.json', import.meta.url), path.join(source, 'static/system.json'));
  const args = [command, '--source', source, '--out', output];
  const discovered = JSON.parse(execFileSync(process.execPath,
    [command, '--source', 'source', '--out', 'output'],
    { encoding: 'utf8', env: { ...process.env, INIT_CWD: temporary } }));
  assert.equal(discovered.complete, false);
  assert.equal(discovered.type_graph_complete, false);
  assert.equal(discovered.trait_catalog_complete, true);
  assert.equal(spawnSync(process.execPath, [...args, '--strict']).status, 1);
  const graph = JSON.parse(await readFile(path.join(output, 'type-graph.json'), 'utf8'));
  assert.ok(graph.diagnostics.some(({ code }: { code: string }) => code === 'missing-root'));
  assert.deepEqual(JSON.parse(await readFile(path.join(output, 'summary.json'), 'utf8')), discovered);
  assert.match(discovered.dependency_lock_digest, /^[a-f0-9]{64}$/);
  for (const directory of ['src/module/actor', 'src/module/item/base', 'src/module/rules', 'types']) {
    await cp(new URL(`../fixtures/type-graph/${directory}/`, import.meta.url), path.join(source, directory), { recursive: true });
  }
  const configFile = path.join(source, 'src/scripts/config/index.ts');
  const portfolioConfig = await readFile(new URL('../fixtures/type-graph/src/scripts/config/index.ts', import.meta.url), 'utf8');
  await writeFile(configFile, `${await readFile(configFile, 'utf8')}\n${portfolioConfig}`);
  const tsconfig = JSON.parse(await readFile(new URL('../fixtures/type-graph/tsconfig.json', import.meta.url), 'utf8'));
  tsconfig.compilerOptions.paths['@item/*'] = ['src/module/item/*'];
  await writeFile(path.join(source, 'tsconfig.json'), JSON.stringify(tsconfig));
  const complete = JSON.parse(execFileSync(process.execPath, [...args, '--strict'], { encoding: 'utf8' }));
  assert.equal(complete.complete, true);
  const completeGraph = JSON.parse(await readFile(path.join(output, 'type-graph.json'), 'utf8'));
  assert.equal(completeGraph.roots.length, 7);
  assert.deepEqual(completeGraph.portfolio.ruleKeys, ['Example', 'Inherited']);
  assert.notEqual(complete.source.source_digest, discovered.source.source_digest);
  assert.equal(complete.dependency_lock_digest, discovered.dependency_lock_digest);
});
