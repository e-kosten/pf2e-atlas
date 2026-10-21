import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, writeFile, mkdir, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test, { type TestContext } from 'node:test';

import type { Catalog, TraitCatalog } from '../src/contracts.js';
import { compareTraitSnapshots, compareTraits, readTraitSnapshot, type TraitSnapshot } from '../src/comparison/compare-traits.js';
import { renderTraitChangeReport } from '../src/comparison/trait-change-report.js';

const source = { file: 'src/scripts/config/traits.ts', line: 1, column: 1 };
const entry = (identifier: string, description: string | null = 'Definition'): Catalog['entries'][number] => ({
  identifier, labelKey: `PF2E.Trait${identifier}`, label: identifier,
  descriptionKey: description === null ? null : 'PF2E.SharedDefinition', description, source,
});
function group(name: string, entries: Catalog['entries'], namespace: Catalog['namespace'] = 'trait'): Catalog {
  return { name, namespace, entries, complete: true, source, exposedInConfig: true, exportedFromTraits: true, familyBindings: [] };
}
function snapshot(groups: Catalog[]): TraitSnapshot {
  return {
    source: { system_version: 'fixture', source_digest: 'a'.repeat(64), input_file_count: 1, git_commit: null, git_clean: null },
    evidence: { summarySha256: 'b'.repeat(64), catalogSha256: 'c'.repeat(64) },
    catalog: { complete: true, catalogs: groups, descriptions: [], diagnostics: [] },
  };
}
async function temporary(t: TestContext) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-trait-changes-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  return directory;
}
async function writeSnapshot(directory: string, value: TraitSnapshot) {
  await mkdir(directory, { recursive: true });
  await writeFile(path.join(directory, 'summary.json'), JSON.stringify({
    format: 'atlas-source-extraction/v1', source: value.source, trait_catalog_complete: true,
    // This command owns trait comparison; unrelated declaration incompleteness is explicit.
    complete: false, type_graph_complete: false,
  }));
  await writeFile(path.join(directory, 'trait-catalog.json'), JSON.stringify(value.catalog));
}

test('reports exact additions/removals, shared-definition changes and catalog memberships independently', () => {
  const old = snapshot([group('weapons', [entry('deadly-d8'), entry('deadly-d12'), entry('old')])]);
  const next = snapshot([
    group('weapons', [entry('deadly-d8', 'Changed'), entry('deadly-d12', 'Changed'), entry('MiXeD')]),
    group('attacks', [entry('deadly-d8', 'Changed')]),
  ]);
  const result = compareTraitSnapshots(old, next);
  assert.deepEqual(result.counts, { added: 1, removed: 1, changed: 2 });
  assert.deepEqual(result.added.map(trait => trait.identifier), ['MiXeD']);
  assert.deepEqual(result.removed.map(trait => trait.identifier), ['old']);
  assert.deepEqual(result.changed.map(trait => [trait.identifier, trait.fields]), [
    ['deadly-d12', ['description']], ['deadly-d8', ['description', 'catalogs']],
  ]);
  assert.equal(result.before.catalogMemberships, 3);
  assert.equal(result.after.catalogMemberships, 4);
  assert.equal(result.after.traits, 3);
  assert.equal(result.changed[0].after.description, 'Changed');
});

test('tracks key and missing/empty text changes without variant inference or rarity/tag promotion', () => {
  const old = snapshot([group('traits', [entry('amp', null), entry('empty', '')]), group('rarity', [entry('rare')], 'rarity')]);
  const next = snapshot([
    group('traits', [{ ...entry('amp', ''), labelKey: 'PF2E.NewLabel', label: 'New' }, entry('empty', 'Now defined')]),
    group('tags', [entry('amp')], 'otherTag'),
  ]);
  const result = compareTraitSnapshots(old, next);
  assert.equal(result.before.missingDescriptions, 2);
  assert.equal(result.after.missingDescriptions, 1);
  assert.deepEqual(result.changed[0].fields, ['labelKey', 'descriptionKey', 'label', 'description']);
  assert.deepEqual(result.changed[1].fields, ['description']);
  assert.equal(result.after.traits, 2);
  assert.deepEqual(result.added, []);
  assert.deepEqual(result.removed, []);
});

test('ordering and source-line movement do not create semantic changes; conflicting mappings fail', () => {
  const old = snapshot([group('z', [entry('b'), entry('a')]), group('a', [entry('a')])]);
  const next = snapshot([group('a', [entry('a')]), group('z', [{ ...entry('a'), source: { ...source, line: 50 } }, entry('b')])]);
  assert.deepEqual(compareTraitSnapshots(old, next).counts, { added: 0, removed: 0, changed: 0 });
  const conflict = snapshot([group('z', [entry('a')]), group('a', [{ ...entry('a'), descriptionKey: 'Other' }])]);
  assert.throws(() => compareTraitSnapshots(old, conflict), /Conflicting trait descriptionKey/);
  conflict.catalog.catalogs[1].entries[0] = entry('a', 'Conflicting resolved text');
  assert.throws(() => compareTraitSnapshots(old, conflict), /Conflicting resolved text/);
});

test('readable review includes actual wording and keeps authored HTML/macros/fences literal', () => {
  const beforeText = '<script>authored()</script>\n```md\n# Authored heading\n@UUID[example]';
  const before = snapshot([group('traits', [entry('agile', beforeText)])]);
  const after = snapshot([group('traits', [entry('agile', 'Revised definition')])]);
  const markdown = renderTraitChangeReport(compareTraitSnapshots(before, after));
  assert(markdown.includes('#### Description before'));
  assert(markdown.includes(`\`\`\`\`text\n${beforeText}\n\`\`\`\``));
  assert(markdown.includes('#### Description after'));
  assert(markdown.includes('Revised definition'));
  assert(markdown.includes('Added: 0. Removed: 0. Changed: 1.'));
});

test('read snapshots bind exact artifact hashes and reject incomplete, duplicate or malformed inputs', async t => {
  const directory = await temporary(t);
  const value = snapshot([group('traits', [entry('agile')])]);
  await writeSnapshot(directory, value);
  const parsed = await readTraitSnapshot(directory);
  assert.equal(parsed.evidence.catalogSha256, createHash('sha256').update(await readFile(path.join(directory, 'trait-catalog.json'))).digest('hex'));
  for (const mutate of [
    (catalog: TraitCatalog) => { catalog.complete = false; },
    (catalog: TraitCatalog) => { catalog.catalogs[0].complete = false; },
    (catalog: TraitCatalog) => { catalog.catalogs[0].entries.push(entry('agile')); },
    (catalog: TraitCatalog) => { catalog.catalogs.push(group('traits', [])); },
    (catalog: TraitCatalog) => { catalog.diagnostics.push({ severity: 'error', code: 'incomplete', message: 'No', source }); },
    (catalog: TraitCatalog) => { delete (catalog.catalogs[0].entries[0] as Partial<Catalog['entries'][number]>).label; },
  ]) {
    const changed = structuredClone(value); mutate(changed.catalog);
    await writeSnapshot(directory, changed);
    await assert.rejects(readTraitSnapshot(directory));
  }
  await writeSnapshot(directory, value);
  const summary = JSON.parse(await readFile(path.join(directory, 'summary.json'), 'utf8'));
  summary.trait_catalog_complete = false;
  await writeFile(path.join(directory, 'summary.json'), JSON.stringify(summary));
  await assert.rejects(readTraitSnapshot(directory), /complete trait extraction summary/);
});

test('CLI resolves caller paths, returns success for reviewed changes, and preserves output on invalid input', async t => {
  const directory = await temporary(t);
  const before = path.join(directory, 'before'), after = path.join(directory, 'after'), out = path.join(directory, 'out');
  await writeSnapshot(before, snapshot([group('traits', [entry('agile')])]));
  await writeSnapshot(after, snapshot([group('traits', [entry('agile', 'Changed'), entry('new')])]));
  const cli = fileURLToPath(new URL('../src/cli/compare-traits.js', import.meta.url));
  const result = spawnSync(process.execPath, [cli, '--before', 'before', '--after', 'after', '--out', 'out'], {
    cwd: os.tmpdir(), env: { ...process.env, INIT_CWD: directory }, encoding: 'utf8',
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).changed, 1);
  assert.equal(JSON.parse(result.stdout).added, 1);
  const output = await readFile(path.join(out, 'trait-changes.json'), 'utf8');
  const review = await readFile(path.join(out, 'trait-changes.md'), 'utf8');
  assert(review.includes('Changed'));
  assert.equal(JSON.parse(output).changed[0].before.description, 'Definition');
  assert.equal(JSON.parse(output).changed[0].after.description, 'Changed');
  const invalid = snapshot([group('traits', [entry('agile')])]); invalid.catalog.complete = false;
  await writeSnapshot(after, invalid);
  await assert.rejects(compareTraits({ before, after, out }));
  assert.equal(await readFile(path.join(out, 'trait-changes.json'), 'utf8'), output);
  assert.equal(await readFile(path.join(out, 'trait-changes.md'), 'utf8'), review);
  const failure = spawnSync(process.execPath, [cli, '--before', before, '--after', after, '--out', out], { encoding: 'utf8' });
  assert.equal(failure.status, 1);
  assert.equal(await readFile(path.join(out, 'trait-changes.json'), 'utf8'), output);
  const missing = spawnSync(process.execPath, [cli, '--before', before], { encoding: 'utf8' });
  assert.equal(missing.status, 1);
  assert.match(missing.stderr, /--before, --after and --out are required/);
});
