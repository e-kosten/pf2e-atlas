import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import type { GraphField, GraphNode, TypeGraph } from '../src/contracts.js';
import { objectPatch } from '../src/generation/object-patch.js';
import { sourceProbe } from '../src/comparison/source-probe.js';

const field = (name: string, ref: string): GraphField => ({ name, ref, optional: false, nullable: false, undefinedAllowed: false, forbidden: false, declaredAt: [] });
const object = (id: string, fields: GraphField[]): GraphNode => ({ id, name: id, kind: 'object', fields, indexSignatures: [] });
function fixture(): TypeGraph {
  return { format: 'atlas-source-type-graph/v1', typescript: 'fixture', complete: true, status: 'complete', roots: [], diagnostics: [], projectDiagnostics: { selected: [], unrelated: [] },
    nodes: [{ id: 'string', kind: 'primitive', value: 'string' }, { id: 'number', kind: 'primitive', value: 'number' },
      { id: 'fixed', kind: 'literal', value: 'fixed' }, { id: 'interval', kind: 'literal', value: 'interval' },
      object('Damage', [field('formula', 'string')]), { id: 'DamageMap', kind: 'object', fields: [], indexSignatures: [{ key: 'primitive:string', value: 'Damage', readonly: false }] },
      object('Fixed', [field('type', 'fixed'), field('levels', 'DamageMap')]), object('Interval', [field('type', 'interval'), field('damage', 'DamageMap'), field('interval', 'number')]),
      { id: 'Heightening', kind: 'union', members: ['Fixed', 'Interval'] },
      { id: 'Array', kind: 'array', element: 'Heightening', readonly: false },
      { id: 'Pair', kind: 'tuple', elements: [{ ref: 'string', optional: false, rest: false }, { ref: 'number', optional: false, rest: false }], readonly: false },
      object('Spell', [field('heightening', 'Heightening'), field('array', 'Array'), field('pair', 'Pair'), { ...field('next', 'Spell'), optional: true }])] };
}

test('object diffs keep declarations and replacement collections, merge partial union shapes and terminate cycles', () => {
  const graph = fixture(), original = structuredClone(graph.nodes);
  const root = objectPatch(graph, 'Spell');
  assert.deepEqual(graph.nodes.slice(0, original.length), original);
  const patch = graph.nodes.find(node => node.id === root);
  assert.ok(patch?.kind === 'object');
  assert.ok(patch.fields.every(field => field.optional));
  assert.equal(patch.fields.find(field => field.name === 'array')!.ref, 'Array');
  assert.equal(patch.fields.find(field => field.name === 'pair')!.ref, 'Pair');
  assert.equal(patch.fields.find(field => field.name === 'next')!.ref, root);
  const heightening = graph.nodes.find(node => node.id === 'Heightening#object-patch');
  assert.ok(heightening?.kind === 'object');
  assert.deepEqual(heightening.fields.map(field => field.name), ['damage', 'interval', 'levels', 'type']);
  assert.ok(heightening.fields.every(field => field.optional));
  assert.throws(() => objectPatch(graph, 'Spell'), /collision/);
  assert.throws(() => objectPatch(fixture(), 'missing'), /Missing/);
  const impossible = fixture();
  impossible.nodes.push({id:'Impossible',kind:'intersection',members:[],fields:[],indexSignatures:[],impossible:true}, {id:'Invalid',kind:'union',members:['Fixed','Impossible']});
  assert.throws(() => objectPatch(impossible, 'Invalid'), /Impossible/);
});

test('generated Rust parses union patches losslessly while enforcing present values and full replacement elements', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-object-patch-'));
  try {
    const graph = fixture(), reference = objectPatch(graph, 'Spell');
    const roots = [{ key: 'Item', reference, name: 'SpellPatch', module: 'patch' }];
    const sources = ['{}', '{"heightening":{"damage":{"x":{}}}}', '{"heightening":{"type":"fixed","levels":{"x":{"formula":"1d6"}}}}',
      '{"heightening":{},"next":{"next":{}},"extra":9007199254740993,"extra":2}',
      '{"heightening":{"type":"other"}}', '{"heightening":{"interval":"2"}}', '{"heightening":{"damage":{"x":{"formula":false}}}}',
      '{"array":[{"damage":{}}]}', '{"pair":["a"]}', '{"pair":["a",2]}'];
    const packets = path.join(directory, 'packets.ndjson');
    await writeFile(packets, sources.map(source => JSON.stringify({ key: 'Item', source, context: { record_key: 'fixture', source_path: 'fixture.json', json_path: '$' } })).join('\n') + '\n');
    const results = await sourceProbe({ graph, roots, packets, out: path.join(directory, 'probe'), target: path.join(directory, 'target'),
      input: { source: { system_version: null, source_digest: 'fixture', input_file_count: 1, git_commit: null, git_clean: null }, nodes: graph.nodes,
        selection: [{ name: 'SpellPatch', declaration: reference, valueRef: reference, module: 'patch', fields: [], deferred: [] }] } });
    assert.deepEqual(results.map(result => result.ok), [true, true, true, true, false, false, false, false, false, true]);
    assert.ok(results.filter(result => result.ok).every(result => result.ok && result.fidelity === null));
    const badFormula = results[6];
    assert.ok(!badFormula.ok); assert.equal(badFormula.error.json_path, '$.heightening.damage["x"].formula');
  } finally { await rm(directory, { recursive: true, force: true }); }
});
