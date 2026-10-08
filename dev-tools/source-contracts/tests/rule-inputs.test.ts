import assert from 'node:assert/strict';
import test from 'node:test';
import type { GraphField, TypeGraph } from '../src/contracts.js';
import { authoredRuleInputs } from '../src/generation/rule-inputs.js';
import { rulePackets } from '../src/comparison/sample-rules.js';
import { itemPackets } from '../src/comparison/sample-items.js';
import { compareRuleResults } from '../src/comparison/compare-rules.js';
import { battleFormStrikeFixture } from '../fixtures/authored-rule-fixture.js';

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
  assert.deepEqual(graph.nodes.find(node => node.id === root.fields[0].ref), { id: 'strings#authored-string-array', kind: 'union', members: ['strings', 'string'] });
  const iwr = schema(); iwr.roots[0].ruleKey = 'Resistance'; iwr.roots[0].arrayInputs![0].field = 'type';
  const object = iwr.nodes[2]; assert.ok(object.kind === 'object'); object.fields[0].name = 'type';
  object.fields[0].declaredAt = iwr.roots[0].arrayInputs![0].declaredAt = [{file:'iwr/base.ts',line:1,column:1}];
  assert.equal(authoredRuleInputs(iwr).changes[0].field, 'type');
  const shared = schema(); shared.roots.push({ ...shared.roots[0], ruleKey: 'Note' });
  const result = authoredRuleInputs(shared);
  assert.equal(result.graph.roots[0].ref, result.graph.roots[1].ref);
  assert.equal(new Set(result.graph.nodes.map(node => node.id)).size, result.graph.nodes.length);
});
test('shared IWR declarations propagate through recursive ancestors without widening unrelated fields', () => {
  const original = schema(), declaration = [{file:'iwr/base.ts',line:1,column:1}];
  original.roots[0].ruleKey = 'Resistance'; original.roots[0].arrayInputs![0].field = 'type';
  original.roots[0].arrayInputs![0].declaredAt = declaration;
  const rule = original.nodes[2]; assert.ok(rule.kind === 'object');
  rule.fields[0].name = 'type'; rule.fields[0].declaredAt = declaration;
  original.nodes.push(
    {id:'nested',kind:'object',fields:[{...field('type'),declaredAt:declaration},field('exceptions')],indexSignatures:[]},
    {id:'nested-array',kind:'array',element:'nested',readonly:false,
      serialization:{basis:'array-subclass',rawRef:'nested',method:'toObject',declaredAt:declaration}},
    {id:'tuple',kind:'tuple',elements:[{ref:'nested',optional:false,rest:false}],readonly:false},
    {id:'map',kind:'object',fields:[],indexSignatures:[{key:'string',value:'nested',readonly:false}]},
    {id:'strict-iwr',kind:'object',fields:[{...field('type'),declaredAt:declaration}],indexSignatures:[]},
    {id:'form',kind:'object',fields:[{...field('immunities'),ref:'nested-array'},{...field('self'),ref:'form'},field('type'),
      {...field('tuple'),ref:'tuple'},{...field('map'),ref:'map'},{...field('strikes'),ref:'FixtureBattleStrikes'}],indexSignatures:[]});
  original.nodes.push(...battleFormStrikeFixture().filter(node=>!original.nodes.some(existing=>existing.id===node.id)));
  original.roots.push({file:'fixture',name:'Form',ruleKey:'BattleForm',ref:'form',arrayInputs:[]});
  original.roots.push({file:'fixture',name:'StrictIwr',ruleKey:'Weakness',ref:'strict-iwr',arrayInputs:[
    {field:'type',arrayRef:'strings',elementRef:'string',fieldClass:'StrictArrayField',declaredAt:declaration}]});
  const before = structuredClone(original), result = authoredRuleInputs(original);
  assert.deepEqual(original,before);
  for (const node of original.nodes) assert.deepEqual(result.graph.nodes.find(copy=>copy.id===node.id),node);
  assert.equal(result.sharedIwrChanges.length,1);
  assert.equal(result.graph.roots[2].ref,'strict-iwr');
  const nodes = new Map(result.graph.nodes.map(node=>[node.id,node]));
  const form = nodes.get(result.graph.roots[1].ref!)!; assert.ok(form.kind==='object');
  assert.equal(form.fields[1].ref,form.id); assert.equal(form.fields[2].ref,'strings');
  const array = nodes.get(form.fields[0].ref)!; assert.ok(array.kind==='array');
  assert.equal(array.serialization!.rawRef,'nested');
  const nested = nodes.get(array.element)!; assert.ok(nested.kind==='object');
  assert.equal(nested.fields[0].ref,result.changes[0].authoredRef);
  assert.equal(nested.fields[1].ref,'strings');
  const tuple = nodes.get(form.fields[3].ref)!; assert.ok(tuple.kind==='tuple');
  assert.equal(tuple.elements[0].ref,nested.id);
  const map = nodes.get(form.fields[4].ref)!; assert.ok(map.kind==='object');
  assert.equal(map.indexSignatures[0].value,nested.id);
  const drift = structuredClone(original), nestedDrift = drift.nodes.find(node=>node.id==='nested')!;
  assert.ok(nestedDrift.kind==='object'); nestedDrift.fields[0].ref='string';
  assert.throws(()=>authoredRuleInputs(drift),/Shared IWR type projection drift/);
  rule.fields[0].declaredAt=[];
  assert.throws(()=>authoredRuleInputs(original),/Missing shared IWR declaration provenance/);
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
