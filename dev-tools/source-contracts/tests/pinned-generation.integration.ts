import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { generateRustModules } from '../src/generation/source-generation.js';
import { validateInput, nodeReferences, type GenerationInput } from '../src/generation/generation-input.js';
import { formatRust, preparePortfolio } from '../src/generation/generate.js';
import { artifactFiles } from '../src/generation/generated-files.js';


const outputPath = fileURLToPath(new URL('../../../../crates/atlas-foundry-model/src/source_model/generated/', import.meta.url));
const { input } = await preparePortfolio();
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

test('fresh generation input stores nodes once and resolves every reference', () => {
  assert.equal(new Set(input.nodes.map(node => node.id)).size, input.nodes.length);
  const ids = new Set(input.nodes.map(node => node.id));
  for (const node of input.nodes) for (const ref of nodeReferences(node)) assert.ok(ids.has(ref), ref);
  const interleaved = structuredClone(input);
  interleaved.selection.push({...interleaved.selection[0],name:'InterleavedFields'});
  assert.throws(()=>validateInput(interleaved),/group each module/);
});

test('maintained portfolio covers every document, family and specific rule with reserved family owners', () => {
  const roots = input.selection.filter(root => root.documentKind || root.ruleKey);
  assert.equal(roots.length, 47);
  assert.equal(roots.filter(root => root.ruleKey).length, 42);
  assert.deepEqual(roots.flatMap(root => root.documentKind ? [root.documentKind] : []).sort(), ['Actor', 'Item', 'JournalEntry', 'Macro', 'RollTable']);
  assert.equal(new Set(input.selection.filter(root => root.module.startsWith('items/families/')).map(root => root.module)).size, 24);
  assert.equal(new Set(input.selection.filter(root => root.module.startsWith('actors/families/')).map(root => root.module)).size, 8);
  const files = generateRustModules(input);
  for (const root of input.selection.filter(root => root.module.includes('/families/'))) {
    assert.match(files[`${root.module}.rs`], new RegExp(`pub fn parse_${root.name.replace(/([a-z0-9])([A-Z])/g, '$1_$2').toLowerCase()}\\(`));
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
  const admission = JSON.parse(await readFile(new URL('../../fixtures/portfolio-admission-baseline.json', import.meta.url), 'utf8'));
  assert.equal(admission.sourceDigest,input.source.source_digest);
  assert.equal(admission.corpusDigest,baseline.corpusDigest);
  assert.equal(admission.counts.retained,admission.counts.occurrences);
  assert.equal(admission.counts.fidelityFailures,0);
  assert.equal(admission.counts.rejected,0);
  assert.equal(admission.counts.occurrences,admission.counts.fullyTyped+admission.counts.partial+admission.counts.rawOnly);
  assert.match(admission.outcomeDigest,/^[0-9a-f]{64}$/);
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
