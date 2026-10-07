import assert from 'node:assert/strict';
import test from 'node:test';
import type { GraphField, TypeGraph } from './contracts.js';
import { authoredRuleInputs } from './rule-inputs.js';
import { rulePackets } from './sample-rules.js';
import { itemPackets } from './sample-items.js';
import { compareRuleResults } from './compare-rules.js';

const field = (name: string): GraphField => ({ name, ref: 'strings', optional: false, nullable: false, undefinedAllowed: false, forbidden: false, declaredAt: [] });
function schema(): TypeGraph {
  return { format: 'atlas-source-type-graph/v1', typescript: 'fixture', complete: true, status: 'complete', diagnostics: [], projectDiagnostics: { selected: [], unrelated: [] },
    roots: [{ file: 'fixture', name: 'Rule', ruleKey: 'FlatModifier', ref: 'rule', arrayInputs: [
      { field: 'selector', arrayRef: 'strings', elementRef: 'string', fieldClass: 'ArrayField', declaredAt: [] },
      { field: 'selectors', arrayRef: 'strings', elementRef: 'string', fieldClass: 'StrictArrayField', declaredAt: [] }] }],
    nodes: [{ id: 'string', kind: 'primitive', value: 'string' }, { id: 'strings', kind: 'array', element: 'string', readonly: false },
      { id: 'rule', kind: 'object', fields: [field('selector'), field('selectors'), field('tags')], indexSignatures: [] }] };
}
test('authored projection preserves cleaned schema and widens only selected ordinary arrays', () => {
  const original = schema(), before = structuredClone(original);
  const { graph, changes } = authoredRuleInputs(original);
  assert.deepEqual(original, before);
  assert.deepEqual(changes.map(change => [change.rule, change.field]), [['FlatModifier', 'selector']]);
  const root = graph.nodes.find(node => node.id === graph.roots[0].ref)!;
  assert.ok(root.kind === 'object');
  assert.equal(root.fields[1].ref, 'strings'); assert.equal(root.fields[2].ref, 'strings');
  assert.deepEqual(graph.nodes.find(node => node.id === root.fields[0].ref), { id: 'rule#authored:selector', kind: 'union', members: ['strings', 'string'] });
  const iwr = schema(); iwr.roots[0].ruleKey = 'Resistance'; iwr.roots[0].arrayInputs![0].field = 'type';
  const object = iwr.nodes[2]; assert.ok(object.kind === 'object'); object.fields[0].name = 'type';
  assert.equal(authoredRuleInputs(iwr).changes[0].field, 'type');
  const shared = schema(); shared.roots.push({ ...shared.roots[0], ruleKey: 'Note' });
  const result = authoredRuleInputs(shared);
  assert.equal(result.graph.roots[0].ref, result.graph.roots[1].ref);
  assert.equal(new Set(result.graph.nodes.map(node => node.id)).size, result.graph.nodes.length);
});
test('missing schema provenance and changed selected shape fail rather than inventing authored types', () => {
  const missing = schema(); delete missing.roots[0].arrayInputs;
  assert.throws(() => authoredRuleInputs(missing), /Re-extract/);
  const drift = schema(); const string = drift.nodes[0]; assert.ok(string.kind === 'primitive'); string.value = 'number';
  assert.throws(() => authoredRuleInputs(drift), /projection drift/);
});
test('rule sampling walks root and embedded Items without losing numeric tokens or payload duplicates', () => {
  const text = '{"_id":"root","type":"equipment","system":{"rules":[{"key":"FlatModifier","value":9007199254740993,"future":1,"future":2}]},"child":{"_id":"child","type":"feat","system":{"rules":[{"key":"Resistance","type":"fire"}]}}}';
  const packets = itemPackets(text, 'pack.json', 'pack', 'Item').flatMap(rulePackets);
  assert.equal(packets.length, 2);
  assert.ok(packets[0].source.includes('9007199254740993')); assert.ok(packets[0].source.includes('"future":1,"future":2'));
  assert.equal(packets[1].context.json_path, '$.child.system.rules[0]');
  for (const rules of ['{}', '[null]', '[{"value":1}]', '[{"key":"a","key":"b"}]']) {
    const item = itemPackets(`{"_id":"x","type":"feat","system":{"rules":${rules}}}`, 'bad.json', 'pack', 'Item')[0];
    assert.throws(() => rulePackets(item));
  }
  const item = itemPackets('{"_id":"x","type":"feat","system":{"rules":[],"rules":[]}}', 'bad.json', 'pack', 'Item')[0];
  assert.throws(() => rulePackets(item), /Duplicate rules/);
});
test('comparison counts every affected occurrence and separates parser defects from unresolved rejections', () => {
  const packets = itemPackets('{"_id":"x","type":"feat","system":{"rules":[{"key":"a"},{"key":"a"},{"key":"b"}]}}', 'x.json', 'pack', 'Item').flatMap(rulePackets);
  const rejection = { ok: false as const, error: { json_path: '$.type', expected: 'array', actual: 'string' } };
  const report = compareRuleResults(packets, [rejection, rejection, { ok: true, fidelity: null }],
    [{ ok: true, fidelity: null }, rejection, { ok: true, fidelity: 'value lost' }]);
  assert.deepEqual(report.counts, { rules: 3, schemaAccepted: 1, authoredAccepted: 2, recovered: 1, regressed: 0, schemaFidelityFailures: 0, fidelityFailures: 1 });
  assert.deepEqual(report.failures.map(failure => failure.category), ['unresolved', 'parser-defect']);
  assert.equal(report.families.a.rules, 2);
  assert.throws(() => compareRuleResults(packets, [], []), /count mismatch/);
});
