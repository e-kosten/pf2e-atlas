import assert from 'node:assert/strict';
import test from 'node:test';
import type { GraphField, GraphNode, TypeGraph } from '../src/contracts.js';
import { authoredDocumentInputs } from '../src/generation/document-inputs.js';
import { generateRustModules } from '../src/generation/source-generation.js';
import { sourceProbe } from '../src/comparison/source-probe.js';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

const weaponFile = 'src/module/item/weapon/data.ts', spellFile = 'src/module/item/spell/data.ts';
const location = (file: string) => [{ file, line: 1, column: 1 }];
const field = (name: string, ref: string, file: string, optional = false, nullable = false): GraphField => ({ name, ref, optional, nullable, undefinedAllowed: optional, forbidden: false, declaredAt: location(file) });
const object = (id: string, file: string, fields: GraphField[]): GraphNode => ({ id, name: id, kind: 'object', fields, indexSignatures: [], declaredAt: location(file) });
function fixture(): TypeGraph {
  return { format: 'atlas-source-type-graph/v1', typescript: 'fixture', complete: true, status: 'complete', roots: [{ file: 'root.ts', name: 'Item', documentKind: 'Item', ref: 'Item' }], diagnostics: [], projectDiagnostics: { selected: [], unrelated: [] },
    nodes: [{ id: 'token', kind: 'literal', value: '1' }, { id: 'primitive:null', kind: 'primitive', value: 'null' }, { id: 'primitive:string', kind: 'primitive', value: 'string' },
      { id: 'nullable', kind: 'union', members: ['token', 'primitive:null'] }, { id: 'primitive:number', kind: 'primitive', value: 'number' },
      { id: 'vocabulary', kind: 'union', members: ['token'] },
      object('Reload', weaponFile, [field('value', 'nullable', weaponFile, false, true)]),
      object('Bonus', weaponFile, [field('value','primitive:number',weaponFile)]), object('Splash', weaponFile, [field('value','primitive:number',weaponFile)]),
      object('WeaponSystemSource', weaponFile, [field('reload', 'Reload', weaponFile),field('bonusDamage','Bonus',weaponFile),field('splashDamage','Splash',weaponFile)]),
      object('WeaponDamage', weaponFile, [field('die', 'nullable', weaponFile, false, true)]),
      object('SpellDamageSource', spellFile, [field('category', 'nullable', spellFile, false, true)]),
      object('SpellArea', spellFile, [field('value', 'primitive:number', spellFile), field('type', 'vocabulary', spellFile)]),
      object('Passive', spellFile, [field('statistic', 'vocabulary', spellFile)]), object('Save', spellFile, [field('statistic', 'vocabulary', spellFile)]),
      {id:'PassiveNullable',kind:'union',members:['Passive','primitive:null']}, {id:'SaveNullable',kind:'union',members:['Save','primitive:null']},
      object('SpellDefenseSource', spellFile, [field('passive','PassiveNullable',spellFile,false,true),field('save','SaveNullable',spellFile,false,true)]),
      object('CreatureInitiativeSource', 'src/module/actor/creature/data.ts', [field('statistic', 'vocabulary', 'src/module/actor/creature/data.ts')]),
      object('SpellSystemSource', spellFile, [field('damage', 'SpellDamageSource', spellFile), field('next', 'SpellSystemSource', spellFile, true)]),
      object('DeepPartial', 'types/foundry/util.d.ts', [field('damage', 'SpellDamageSource', spellFile, true), field('next', 'SpellSystemSource', spellFile, true)]),
      object('Levels', spellFile, Array.from({length:10}, (_, index) => field(String(index + 1), 'DeepPartial', spellFile, true))),
      object('SpellHeighteningFixed', spellFile, [field('levels', 'Levels', spellFile)]),
      object('SpellOverlayOverride', spellFile, [field('system', 'DeepPartial', spellFile, true)]),
      object('Item', 'root.ts', [field('weapon', 'WeaponSystemSource', 'root.ts'), field('damage', 'WeaponDamage', 'root.ts'), field('spell', 'SpellSystemSource', 'root.ts'), field('overlay', 'SpellOverlayOverride', 'root.ts'), field('area', 'SpellArea', 'root.ts'), field('defense', 'SpellDefenseSource', 'root.ts'), field('initiative', 'CreatureInitiativeSource', 'root.ts'), field('fixed', 'SpellHeighteningFixed', 'root.ts'), field('unrelated', 'nullable', 'root.ts', false, true)])] };
}

test('document authored policies preserve declaration evidence and remain scoped to owners', () => {
  const schema = fixture(), before = structuredClone(schema);
  const projected = authoredDocumentInputs(schema);
  assert.deepEqual(schema, before);
  assert.deepEqual(authoredDocumentInputs(schema).graph, projected.graph);
  assert.equal(projected.changes.length, 12);
  assert.deepEqual(projected.graph.nodes.slice(0, schema.nodes.length), schema.nodes);
  const root = projected.graph.nodes.find(node => node.id === projected.graph.roots[0].ref)!;
  assert.ok(root.kind === 'object');
  assert.equal(root.fields.find(field => field.name === 'unrelated')!.ref, 'nullable');
  const spellPatch = projected.graph.nodes.find(node => node.id.endsWith('#object-patch') && node.name === 'PatchSpellDamageSource')!;
  assert.ok(spellPatch.kind === 'object');
  assert.match(spellPatch.fields[0].ref, /authored-empty-string/);
  const files = generateRustModules({ source: { system_version: null, source_digest: 'fixture', input_file_count: 1, git_commit: null, git_clean: null }, nodes: projected.graph.nodes,
    selection: [{ name: 'Item', declaration: projected.graph.roots[0].ref!, valueRef: projected.graph.roots[0].ref!, module: 'item', fields: [], deferred: [] }] });
  assert.match(files['item.rs'], /serde\(rename = ""\)/);
});

test('document projection preserves sentinels, area strings and custom statistic slugs without weakening unrelated fields', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-document-inputs-'));
  try {
    const schema = fixture(), graph = authoredDocumentInputs(schema).graph;
    const reference = graph.roots[0].ref!;
    const sources = ['{"weapon":{"reload":{"value":""}},"damage":{"die":""},"spell":{"damage":{"category":""}}}',
      '{"area":{"value":"20","type":""},"initiative":{"statistic":"warfare-lore"}}',
      '{"area":{"value":true}}', '{"area":{"type":"hexagon"}}', '{"damage":{"die":"d99"}}', '{"weapon":{"reload":{"value":"bad"}}}', '{"unrelated":""}',
      '{"overlay":{"system":{"damage":{"category":""},"next":{"damage":{"category":null}}}}}',
      '{"fixed":{"levels":{"1":{"damage":{"category":""}}}}}', '{"spell":{"damage":{"category":null}}}',
      '{"area":{"value":"unresolved authored text","type":"1"}}', '{"defense":{"save":{"statistic":""},"passive":{"statistic":""}}}',
      '{"weapon":{"bonusDamage":{"value":""},"splashDamage":{"value":""}}}', '{"weapon":{"splashDamage":{"value":"2"}}}'];
    const packets = path.join(directory, 'packets.ndjson');
    await writeFile(packets, sources.map(source => JSON.stringify({ key: 'Item', source, context: { record_key: 'fixture', source_path: 'fixture.json', json_path: '$' } })).join('\n') + '\n');
    const results = await sourceProbe({ graph, packets, roots: [{ key: 'Item', reference, name: 'Item', module: 'item' }], out: path.join(directory, 'probe'), target: path.join(directory, 'target'),
      input: { source: { system_version: null, source_digest: 'fixture', input_file_count: 1, git_commit: null, git_clean: null }, nodes: graph.nodes,
        selection: [{ name: 'Item', declaration: reference, valueRef: reference, module: 'item', fields: [], deferred: [] }] } });
    assert.deepEqual(results.map(result => result.ok), [true,true,false,false,false,false,false,true,true,true,true,true,true,false]);
    assert.ok(results.filter(result => result.ok).every(result => result.ok && result.fidelity === null));
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('document policy drift is a visible error, not a silently weakened field', () => {
  for (const alter of [(graph: TypeGraph) => { graph.complete = false; },
    (graph: TypeGraph) => { graph.nodes = graph.nodes.filter(node => node.name !== 'WeaponDamage'); },
    (graph: TypeGraph) => { const node = graph.nodes.find(node => node.id === 'Reload')!; if (node.kind === 'object') node.fields[0].ref = 'primitive:string'; },
    (graph: TypeGraph) => { const node = graph.nodes.find(node => node.id === 'Levels')!; if (node.kind === 'object') node.fields.forEach(field => {field.ref = 'primitive:string';}); },
    (graph: TypeGraph) => { const node = graph.nodes.find(node => node.id === 'SpellOverlayOverride')!; if (node.kind === 'object') node.fields[0].nullable = true; }]) {
    const graph = fixture(); alter(graph); assert.throws(() => authoredDocumentInputs(graph), /complete|drift/);
  }
});
