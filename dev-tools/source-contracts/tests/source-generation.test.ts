import assert from 'node:assert/strict';
import { readFile, mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { generateRustModules } from '../src/generation/source-generation.js';
import { loadGenerationInput, snapshotFiles, nodeReferences, type GenerationInput } from '../src/generation/generation-input.js';
import { formatRust, generate } from '../src/generation/generate.js';
import { artifactFiles } from '../src/generation/generated-files.js';
import type { TypeGraph, ExtractionSummary } from '../src/contracts.js';

const manifestPath = fileURLToPath(new URL('../../snapshots/manifest.json', import.meta.url));
const outputPath = fileURLToPath(new URL('../../../../crates/atlas-ingest/src/source_model/generated/', import.meta.url));
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
  assert.equal(input.selection.find(root => root.name === 'PhysicalEquipmentFields')!.fields.find(field => field.name === 'usage')!.optional, true);
  assert.equal(input.selection.find(root => root.name === 'EquipmentFields')!.fields.find(field => field.name === 'usage')!.optional, false);
});

test('snapshots store nodes once and resolve references across input modules', async () => {
  const interleaved = structuredClone(input);
  interleaved.selection.push({...interleaved.selection[0],name:'InterleavedFields'});
  assert.throws(()=>snapshotFiles(interleaved),/group each module/);
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
    assert.equal(Object.entries(files).filter(([name]) => name !== 'manifest.json').reduce((sum,[,text]) => sum + JSON.parse(text).nodes.length,0),input.nodes.length);
    equipment.nodes.push(physical.nodes[0]);
    await writeFile(path.join(directory, 'items/equipment.json'), JSON.stringify({ format: 'atlas-source-generation-module/v1', roots: [], nodes: equipment.nodes }));
    await assert.rejects(loadGenerationInput(path.join(directory, 'manifest.json')), /Duplicate generation node/);
    await rm(path.join(directory, 'physical.json'));
    await assert.rejects(loadGenerationInput(path.join(directory, 'manifest.json')), /ENOENT/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('maintained portfolio covers every document, family and specific rule with reserved family owners', () => {
  const roots = input.selection.filter(root => root.documentKind || root.ruleKey);
  assert.equal(roots.length, 47);
  assert.equal(roots.filter(root => root.ruleKey).length, 42);
  assert.deepEqual(roots.flatMap(root => root.documentKind ? [root.documentKind] : []).sort(), ['Actor', 'Item', 'JournalEntry', 'Macro', 'RollTable']);
  assert.equal(new Set(input.selection.filter(root => root.module.startsWith('items/families/')).map(root => root.module)).size, 24);
  assert.equal(new Set(input.selection.filter(root => root.module.startsWith('actors/families/')).map(root => root.module)).size, 8);
  const files = generateRustModules(input), snapshots = snapshotFiles(input);
  for (const root of input.selection.filter(root => root.module.includes('/families/'))) {
    assert.match(files[`${root.module}.rs`], new RegExp(`pub fn parse_${root.name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase()}\\(`));
    const module = JSON.parse(snapshots[`${root.module}.json`]);
    assert.ok(module.nodes.some((node: {id: string}) => node.id === root.valueRef));
    assert.ok(module.nodes.some((node: {id: string}) => node.id === root.sourceRef));
  }
  assert.equal((Object.values(files).join('').match(/pub struct SpellSource \{/g) ?? []).length, 1);
  assert.match(files['rules/source.rs'], /FlatModifier\(Box<FlatModifierRule>\)/);
  assert.match(files['items/source.rs'], /SpellSource\(Box<SpellSource>\)/);
});

test('corpus baseline identifies the maintained source pin and retains zero measured value loss', async () => {
  const baseline = JSON.parse(await readFile(new URL('../../fixtures/portfolio-corpus-baseline.json', import.meta.url), 'utf8'));
  assert.equal(baseline.sourceDigest, input.source.source_digest);
  assert.equal(baseline.counts.occurrences, baseline.counts.accepted + baseline.counts.rejected);
  assert.equal(baseline.counts.fidelityFailures, 0);
  assert.match(baseline.corpusDigest, /^[0-9a-f]{64}$/);
  assert.match(baseline.rejectionDigest, /^[0-9a-f]{64}$/);
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

test('arrays, keyed objects and explicit trait policy retain distinct constraints', () => {
  const files = generateRustModules(input);
  assert.equal(input.selection.filter(root => root.family).length,24);
  assert.match(files['items/flags.rs'], /crate::source_model::SourceMap<ItemGranterSource>/);
  assert.match(files['items/traits.rs'], /no array elements/);
  assert.match(files['items/traits.rs'], /Explicit trait-array policies keep identifiers as strings/);
  assert.match(files['items/traits.rs'], /Deity\(DeityTraitsFields\)/);
  assert.equal((Object.values(files).join('').match(/pub struct ItemDescriptionSource \{/g)??[]).length,1);
  const bad = structuredClone(input);
  bad.openTraitArrays = ['primitive:number'];
  assert.throws(() => generateRustModules(bad),/string vocabulary array/);
  const changed = structuredClone(input);
  const ref = changed.openTraitArrays![0];
  const array = changed.nodes.find(node=>node.id===ref);
  assert.ok(array?.kind==='array');
  array.element='primitive:number';
  assert.throws(() => generateRustModules(changed),/string vocabulary array/);
  const nullable = structuredClone(input);
  nullable.nodes.push({id:'nullable-string',kind:'union',members:['primitive:string','primitive:null']},
    {id:'nullable-array',kind:'array',element:'nullable-string',readonly:false},
    {id:'primitive:null',kind:'primitive',value:'null'});
  nullable.nodes = [...new Map(nullable.nodes.map(node=>[node.id,node])).values()];
  nullable.selection.push({name:'NullableArrayFields',declaration:'fixture:nullable',module:'extra',deferred:[],
    fields:[{name:'values',ref:'nullable-array',optional:false,nullable:false,undefinedAllowed:false,forbidden:false,declaredAt:[]}]});
  assert.match(Object.values(generateRustModules(nullable)).join(''), /Vec<Option<String>>/);
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

test('unsupported shapes, unsupported indices, dangling refs and naming collisions reject', () => {
  for (const kind of ['unsupported', 'unresolved'] as const) {
    const changed = structuredClone(input);
    const hp = changed.nodes.findIndex(node => node.name === 'PhysicalItemHPSource');
    changed.nodes[hp] = { id: changed.nodes[hp].id, kind } as GenerationInput['nodes'][number];
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
  const indexSignatures = [{ key: 'primitive:boolean', value: 'primitive:number', readonly: false }];
  indexed.nodes.push({ id: 'indexed', kind: 'object', fields: [], indexSignatures });
  indexed.nodes[indexed.nodes.indexOf(hp)] = { ...hp, kind: 'intersection', members: ['indexed'], fields: hp.fields, indexSignatures };
  assert.throws(() => generateRustModules(indexed), /index signature/);
  const collision = structuredClone(input);
  collision.selection.find(root => root.name === 'EquipmentFields')!.name = 'Coins';
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
    await writeFile(path.join(outDir, 'obsolete.rs'), '// Generated by dev-tools/source-contracts/src/generation/generate.ts; do not edit.\n');
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
    const nodes = new Map(input.nodes.map(node => [node.id, node]));
    const visited = new Set<string>();
    const visit = (ref: string) => { if (visited.has(ref)) return; visited.add(ref); nodeReferences(nodes.get(ref)!).forEach(visit); };
    input.portfolio!.schemaRoots.forEach(root => visit(root.ref!));
    input.selection.forEach(root => { if (root.sourceRef && !root.sourceRef.includes('#authored')) visit(root.sourceRef); });
    const graph: TypeGraph = { format: "atlas-source-type-graph/v1", typescript: input.portfolio!.typescript, source: input.source,
      complete: true, status: "complete", roots: input.portfolio!.schemaRoots, diagnostics: [], projectDiagnostics: { selected: [], unrelated: [] },
      portfolio: { families: input.portfolio!.families, documentKinds: input.portfolio!.schemaRoots.flatMap(root => root.documentKind ? [root.documentKind] : []),
        ruleKeys: input.portfolio!.schemaRoots.flatMap(root => root.ruleKey ? [root.ruleKey] : []) },
      nodes: [...nodes.values()].filter(node => visited.has(node.id)) };
    const summary: ExtractionSummary = { format: 'atlas-source-extraction/v1', source: input.source,
      typescript_version: '5.9.3', dependency_lock_digest: 'test', complete: true,
      type_graph_complete: true, trait_catalog_complete: true, products: { type_graph: 'graph.json', trait_catalog: 'unused' } };
    const graphFile = path.join(directory, 'graph.json'), summaryFile = path.join(directory, 'summary.json');
    await writeFile(graphFile, JSON.stringify(graph));
    await writeFile(summaryFile, JSON.stringify(summary));
    const args = { graph: graphFile, summary: summaryFile, snapshotDir: path.join(directory, 'snapshots'), outDir: path.join(directory, 'rust') };
    await generate(args);
    await generate({ ...args, check: true });
    const equipment = graph.nodes.find(node => node.id === input.selection.find(root=>root.name==='EquipmentFields')!.declaration);
    assert.ok(equipment && equipment.kind === 'object');
    const usage = equipment.fields.find(field => field.name === 'usage');
    assert.ok(usage);
    usage.optional = !usage.optional;
    await writeFile(graphFile, JSON.stringify(graph));
    await assert.rejects(generate({ ...args, check: true }), /stale/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
