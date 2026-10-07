import assert from 'node:assert/strict';
import { readFile, mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { generateRustModules } from './source-generation.js';
import { loadGenerationInput, snapshotFiles, type GenerationInput } from './generation-input.js';
import { formatRust, generate } from './generate.js';
import { artifactFiles } from './generated-files.js';
import type { TypeGraph, ExtractionSummary } from './contracts.js';

const manifestPath = fileURLToPath(new URL('../snapshots/manifest.json', import.meta.url));
const outputPath = fileURLToPath(new URL('../../../crates/atlas-ingest/src/source_model/generated/', import.meta.url));
const input = await loadGenerationInput(manifestPath);
const rustFiles = (input: GenerationInput) => Object.fromEntries(Object.entries(generateRustModules(input)).map(([file, text]) => [file, formatRust(text)]));

test('pinned modular output is fresh and equipment imports the single physical owner', async () => {
  const files = rustFiles(input);
  assert.deepEqual(Object.keys(files).sort(), await artifactFiles(outputPath));
  for (const [file, text] of Object.entries(files)) assert.equal(text, await readFile(path.join(outputPath, file), 'utf8'));
  assert.equal((Object.values(files).join('').match(/pub struct EquippedData \{/g) ?? []).length, 1);
  assert.equal((Object.values(files).join('').match(/pub struct Coins \{/g) ?? []).length, 1);
  assert.match(files['items/equipment.rs'], /generated::physical/);
  assert.match(files['items/equipment.rs'], /pub type EquipmentFields = PhysicalEquipmentFields/);
  assert.doesNotMatch(files['items/equipment.rs'], /pub struct/);
  assert.match(files['physical.rs'], /SourcePresence<Number>/);
  assert.match(files['physical.rs'], /additional_fields: SourceObject/);
  assert.equal(input.selection[0].fields.find(field => field.name === 'usage')!.optional, true);
  assert.equal(input.selection[1].fields.find(field => field.name === 'usage')!.optional, false);
});

test('snapshots store nodes once and resolve references across input modules', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-generation-input-'));
  try {
    const files = snapshotFiles(input);
    for (const [file, text] of Object.entries(files)) {
      await mkdir(path.dirname(path.join(directory, file)), { recursive: true });
      await writeFile(path.join(directory, file), text);
    }
    assert.deepEqual(await loadGenerationInput(path.join(directory, 'manifest.json')), input);
    const physical = JSON.parse(files['physical.json']) as { nodes: GenerationInput['nodes'] };
    const equipment = JSON.parse(files['items/equipment.json']) as { nodes: GenerationInput['nodes'] };
    assert.equal(physical.nodes.length + equipment.nodes.length, 29);
    equipment.nodes.push(physical.nodes[0]);
    await writeFile(path.join(directory, 'items/equipment.json'), JSON.stringify({ format: 'atlas-source-generation-module/v1', roots: [], nodes: equipment.nodes }));
    await assert.rejects(loadGenerationInput(path.join(directory, 'manifest.json')), /Duplicate generation node/);
    await rm(path.join(directory, 'physical.json'));
    await assert.rejects(loadGenerationInput(path.join(directory, 'manifest.json')), /ENOENT/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('field and literal drift change generated types and preserve shared ownership', () => {
  const changed = structuredClone(input);
  const hp = changed.nodes.find(node => node.name === 'PhysicalItemHPSource');
  assert.ok(hp && hp.kind === 'object');
  hp.fields.push({ ...hp.fields[0], name: 'repairable', ref: 'primitive:string' });
  const files = generateRustModules(changed);
  assert.match(files['physical.rs'], /pub repairable: SourcePresence<String>/);
  assert.match(files['physical.rs'], /repairable: f.presence\("repairable", string\)/);
  assert.equal(files['items/equipment.rs'], generateRustModules(input)['items/equipment.rs']);
  const literal = changed.nodes.find(node => node.kind === 'literal' && node.value === 'held');
  assert.ok(literal && literal.kind === 'literal');
  literal.value = 'floating';
  assert.match(generateRustModules(changed)['physical.rs'], /Floating/);
});

test('shared numeric literal parsers import their function while Number stays locally imported', () => {
  const changed = structuredClone(input);
  const equipped = changed.nodes.find(node => node.name === 'EquippedData');
  assert.ok(equipped && equipped.kind === 'object');
  const hands = equipped.fields.find(field => field.name === 'handsHeld');
  assert.ok(hands);
  changed.selection.push({ name: 'CountFields', declaration: 'fixture#CountFields', module: 'items/counts',
    fields: [{ ...hands, name: 'count' }], deferred: [] });
  const rust = rustFiles(changed)['items/counts.rs'];
  assert.match(rust, /use serde_json::Number/);
  assert.match(rust, /generated::physical::read_/);
  assert.doesNotMatch(rust, /generated::physical::\{[^}]*\bNumber\b/);
});

test('unsupported shapes, indexed intersections, dangling refs and naming collisions reject', () => {
  for (const kind of ['unsupported', 'unresolved', 'open', 'template', 'tuple', 'array'] as const) {
    const changed = structuredClone(input);
    const hp = changed.nodes.findIndex(node => node.name === 'PhysicalItemHPSource');
    changed.nodes[hp] = { id: changed.nodes[hp].id, kind, ...(kind === 'array' ? { element: 'primitive:number', readonly: false } : {}) } as GenerationInput['nodes'][number];
    assert.throws(() => generateRustModules(changed), /Unsupported|outside this trial/);
  }
  const missing = structuredClone(input);
  missing.nodes = missing.nodes.filter(node => node.name !== 'PhysicalItemHPSource');
  assert.throws(() => generateRustModules(missing), /Missing generation node/);
  const duplicate = structuredClone(input);
  duplicate.nodes.push(duplicate.nodes[0]);
  assert.throws(() => generateRustModules(duplicate), /Duplicate/);
  const indexed = structuredClone(input);
  const hp = indexed.nodes.find(node => node.name === 'PhysicalItemHPSource');
  assert.ok(hp && hp.kind === 'object');
  indexed.nodes.push({ id: 'indexed', kind: 'object', fields: [], indexSignatures: [{ key: 'string', value: 'primitive:number', readonly: false }] });
  indexed.nodes[indexed.nodes.indexOf(hp)] = { ...hp, kind: 'intersection', members: ['indexed'], fields: hp.fields };
  assert.throws(() => generateRustModules(indexed), /index signature/);
  const collision = structuredClone(input);
  collision.selection[1].name = 'Coins';
  assert.throws(() => generateRustModules(collision), /collision/);
  collision.selection[1].module = '../escape';
  assert.throws(() => generateRustModules(collision), /Invalid Rust module/);
});

test('freshness covers the whole output set and removes only stale generated files', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-generation-output-'));
  try {
    const outDir = path.join(directory, 'output with spaces');
    const args = { manifest: manifestPath, outDir };
    await generate(args);
    await generate({ ...args, check: true });
    await writeFile(path.join(outDir, 'obsolete.rs'), '// Generated by scripts/source-contracts/generate.ts; do not edit.\n');
    await assert.rejects(generate({ ...args, check: true }), /stale/);
    await generate(args);
    assert.equal((await artifactFiles(outDir)).includes('obsolete.rs'), false);
    await writeFile(path.join(outDir, 'notes.txt'), 'human authored');
    const before = await readFile(path.join(outDir, 'physical.rs'), 'utf8');
    await assert.rejects(generate(args), /Unmanaged file/);
    assert.equal(await readFile(path.join(outDir, 'physical.rs'), 'utf8'), before);
    await assert.rejects(generate({ outDir }), /either/);
    await assert.rejects(generate({ manifest: manifestPath, outDir: path.dirname(manifestPath) }), /separate/);
    await rm(path.join(outDir, 'notes.txt'));
    await rm(path.join(outDir, 'mod.rs'));
    await mkdir(path.join(outDir, 'mod.rs'));
    await assert.rejects(generate(args), /not a regular file/);
    assert.equal(await readFile(path.join(outDir, 'physical.rs'), 'utf8'), before);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('graph refresh checks metadata changes that shared Rust value types collapse', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-generation-metadata-'));
  try {
    const graph: TypeGraph = { format: 'atlas-source-type-graph/v1', typescript: '5.9.3', complete: true,
      status: 'complete', roots: [], diagnostics: [], projectDiagnostics: { selected: [], unrelated: [] },
      nodes: [...input.nodes, ...input.selection.map(root => ({ id: root.declaration, kind: 'object' as const,
        fields: [...root.fields, ...root.deferred.map(name => ({ ...root.fields[0], name }))], indexSignatures: [] }))] };
    const summary: ExtractionSummary = { format: 'atlas-source-extraction/v1', source: input.source,
      typescript_version: '5.9.3', dependency_lock_digest: 'test', complete: true,
      type_graph_complete: true, trait_catalog_complete: true, products: { type_graph: 'graph.json', trait_catalog: 'unused' } };
    const graphFile = path.join(directory, 'graph.json'), summaryFile = path.join(directory, 'summary.json');
    await writeFile(graphFile, JSON.stringify(graph));
    await writeFile(summaryFile, JSON.stringify(summary));
    const args = { graph: graphFile, summary: summaryFile, snapshotDir: path.join(directory, 'snapshots'), outDir: path.join(directory, 'rust') };
    await generate(args);
    await generate({ ...args, check: true });
    const equipment = graph.nodes.find(node => node.id === input.selection[1].declaration);
    assert.ok(equipment && equipment.kind === 'object');
    const usage = equipment.fields.find(field => field.name === 'usage');
    assert.ok(usage);
    usage.optional = !usage.optional;
    await writeFile(graphFile, JSON.stringify(graph));
    await assert.rejects(generate({ ...args, check: true }), /stale/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
