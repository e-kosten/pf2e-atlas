import assert from 'node:assert/strict';
import { readFile, mkdtemp, writeFile, rm } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import test from 'node:test';
import { generateEquipmentRust, type EquipmentGenerationInput } from './equipment-generation.js';
import { formatRust, generateEquipment } from './generate-equipment.js';
import type { TypeGraph, ExtractionSummary } from './contracts.js';

const inputPath = new URL('../../../crates/atlas-ingest/src/source_model/generated.input.json', import.meta.url);
const outputPath = new URL('../../../crates/atlas-ingest/src/source_model/generated.rs', import.meta.url);
const input = JSON.parse(await readFile(inputPath, 'utf8')) as EquipmentGenerationInput;

test('pinned generated model is fresh and uses shared physical owners', async () => {
  const rust = formatRust(generateEquipmentRust(input));
  assert.equal(rust, await readFile(outputPath, 'utf8'));
  assert.equal((rust.match(/pub struct EquippedData \{/g) ?? []).length, 1);
  assert.equal((rust.match(/pub struct Coins \{/g) ?? []).length, 1);
  assert.match(rust, /pub type EquipmentFields = PhysicalEquipmentFields/);
  assert.match(rust, /SourcePresence<Number>/);
  assert.match(rust, /additional_fields: SourceObject/);
  const physicalUsage = input.selection[0].fields.find(f => f.name === 'usage')!;
  const equipmentUsage = input.selection[1].fields.find(f => f.name === 'usage')!;
  assert.equal(physicalUsage.optional, true);
  assert.equal(equipmentUsage.optional, false);
});

test('field and literal drift alter meaningful generated types without manual edits', () => {
  const changed = structuredClone(input);
  const hp = changed.nodes.find(n => n.name === 'PhysicalItemHPSource');
  assert.ok(hp && hp.kind === 'object');
  hp.fields.push({ ...hp.fields[0], name: 'repairable', ref: 'primitive:string' });
  changed.nodes.push({ id: 'primitive:string', kind: 'primitive', value: 'string' });
  // Existing string primitive can have an alias ID; reuse it rather than duplicate.
  changed.nodes = [...new Map(changed.nodes.map(n => [n.id, n])).values()];
  const rust = generateEquipmentRust(changed);
  assert.match(rust, /pub repairable: SourcePresence<String>/);
  assert.match(rust, /repairable: f.presence\("repairable", string\)/);
  const literal = changed.nodes.find(n => n.kind === 'literal' && n.value === 'held');
  assert.ok(literal && literal.kind === 'literal');
  literal.value = 'floating';
  assert.match(generateEquipmentRust(changed), /Floating/);
  assert.notEqual(rust, generateEquipmentRust(input));
});

test('unsupported constructs, dangling references and duplicate identities stop generation', () => {
  for (const kind of ['unsupported', 'unresolved', 'open', 'template', 'tuple', 'array'] as const) {
    const changed = structuredClone(input);
    const hp = changed.nodes.findIndex(n => n.name === 'PhysicalItemHPSource');
    const previous = changed.nodes[hp];
    // Simulate an upstream construct not supported by this deliberately bounded emitter.
    changed.nodes[hp] = { id: previous.id, kind, ...(kind === 'array' ? { element: 'primitive:number', readonly: false } : {}) } as EquipmentGenerationInput['nodes'][number];
    assert.throws(() => generateEquipmentRust(changed), /Unsupported|outside this trial/);
  }
  const missing = structuredClone(input);
  missing.nodes = missing.nodes.filter(n => n.name !== 'PhysicalItemHPSource');
  assert.throws(() => generateEquipmentRust(missing), /Missing generation node/);
  const duplicate = structuredClone(input);
  duplicate.nodes.push(duplicate.nodes[0]);
  assert.throws(() => generateEquipmentRust(duplicate), /Duplicate/);
  const indexed = structuredClone(input);
  const hp = indexed.nodes.find(n => n.name === 'PhysicalItemHPSource');
  assert.ok(hp && hp.kind === 'object');
  indexed.nodes.push({ id: 'indexed', kind: 'object', fields: [], indexSignatures: [{ key: 'string', value: 'primitive:number', readonly: false }] });
  indexed.nodes[indexed.nodes.indexOf(hp)] = { ...hp, kind: 'intersection', members: ['indexed'], fields: hp.fields };
  assert.throws(() => generateEquipmentRust(indexed), /index signature/);
});

test('stale checks fail without rewriting either input or generated output', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-equipment-generation-'));
  try {
    const inputFile = path.join(directory, 'input with spaces.json');
    const outputFile = path.join(directory, 'output with spaces.rs');
    await writeFile(inputFile, JSON.stringify(input));
    await writeFile(outputFile, 'stale');
    await assert.rejects(generateEquipment({ input: inputFile, out: outputFile, check: true }), /stale/);
    assert.equal(await readFile(outputFile, 'utf8'), 'stale');
    assert.deepEqual(JSON.parse(await readFile(inputFile, 'utf8')), input);
    await generateEquipment({ input: inputFile, out: outputFile });
    await generateEquipment({ input: inputFile, out: outputFile, check: true });
    await assert.rejects(generateEquipment({ out: outputFile }), /either/);
    await assert.rejects(generateEquipment({ input: inputFile, out: inputFile }), /separate/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('graph freshness also checks upstream presence metadata that shared Rust values collapse', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-equipment-metadata-'));
  try {
    const graph: TypeGraph = { format: 'atlas-source-type-graph/v1', typescript: '5.9.3', complete: true,
      status: 'complete', roots: [], diagnostics: [], projectDiagnostics: { selected: [], unrelated: [] },
      nodes: [...input.nodes, ...input.selection.map(selection => ({ id: selection.declaration, kind: 'object' as const,
        fields: [...selection.fields, ...selection.deferred.map(name => ({ ...selection.fields[0], name }))], indexSignatures: [] }))] };
    const summary: ExtractionSummary = { format: 'atlas-source-extraction/v1', source: input.source,
      typescript_version: '5.9.3', dependency_lock_digest: 'test', complete: true,
      type_graph_complete: true, trait_catalog_complete: true, products: { type_graph: 'graph.json', trait_catalog: 'unused' } };
    const graphFile = path.join(directory, 'graph.json'), summaryFile = path.join(directory, 'summary.json');
    const out = path.join(directory, 'generated.rs');
    await writeFile(graphFile, JSON.stringify(graph));
    await writeFile(summaryFile, JSON.stringify(summary));
    const args = { graph: graphFile, summary: summaryFile, out };
    await generateEquipment(args);
    await generateEquipment({ ...args, check: true });
    const equipment = graph.nodes.find(node => node.id === input.selection[1].declaration);
    assert.ok(equipment && equipment.kind === 'object');
    const usage = equipment.fields.find(field => field.name === 'usage');
    assert.ok(usage);
    usage.optional = !usage.optional;
    await writeFile(graphFile, JSON.stringify(graph));
    await assert.rejects(generateEquipment({ ...args, check: true }), /Generation input is stale/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
