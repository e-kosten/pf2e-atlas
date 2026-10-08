import assert from 'node:assert/strict';
import test from 'node:test';
import { authoredRuleFixture } from '../fixtures/authored-rule-fixture.js';
import { authoredRuleInputs } from '../src/generation/rule-inputs.js';

test('authored constructor projections retain schema nodes and declaration provenance', () => {
  const schema = authoredRuleFixture(), before = structuredClone(schema);
  const projected = authoredRuleInputs(schema);
  assert.deepEqual(schema, before);
  for (const node of schema.nodes) assert.deepEqual(projected.graph.nodes.find(copy => copy.id === node.id), node);
  assert.equal(projected.valueChanges.length, 7);
  const config = projected.graph.nodes.find(node => node.id === 'ChoiceSetConfig#authored-input:predicate');
  assert.ok(config?.kind === 'object');
  assert.equal(config.fields.find(field => field.name === 'predicate')!.optional, true);
  const damage = projected.graph.nodes.find(node => node.id === 'DamageDiceOverride#authored-input:damageType,diceNumber,dieSize');
  assert.ok(damage?.kind === 'object');
  assert.equal(damage.fields.find(field => field.name === 'upgrade')!.ref, 'primitive:boolean');
  assert.equal(damage.fields.find(field => field.name === 'damageType')!.ref, 'primitive:string');
  assert.deepEqual(damage.fields[0].declaredAt, (schema.nodes.find(node => node.id === damage.name) as typeof damage).fields[0].declaredAt);
});

test('BattleForm authored base types widen only the reachable nested declaration', () => {
  const schema = authoredRuleFixture(), projected = authoredRuleInputs(schema);
  const nested = projected.graph.nodes.find(node => node.id === 'BattleFormStrike#authored-input:baseType');
  assert.ok(nested?.kind === 'object');
  assert.equal(nested.fields[0].ref, 'primitive:string');
  assert.equal(nested.fields[0].nullable, true);
  assert.equal(nested.fields[1].ref, 'FixtureRange');
  const direct = projected.graph.nodes.find(node => node.id === 'StrikeRule#authored-input:traits');
  assert.ok(direct?.kind === 'object');
  assert.equal(direct.fields[1].ref, 'FixtureBaseType');
  for (const change of ['shape', 'provenance', 'nullable', 'owner'] as const) {
    const drift = authoredRuleFixture(), owner = drift.nodes.find(node => node.name === 'BattleFormStrike')!;
    assert.ok(owner.kind === 'object');
    if (change === 'shape') owner.fields[0].ref = 'primitive:number';
    if (change === 'provenance') owner.fields[0].declaredAt[0].file = 'unrelated.ts';
    if (change === 'nullable') owner.fields[0].nullable = false;
    if (change === 'owner') owner.name = 'Renamed';
    assert.throws(() => authoredRuleInputs(drift), /declaration drift|field drift/);
  }
});

test('authored projections stop on changed or missing constructor declarations', () => {
  for (const change of ['missing', 'provenance', 'predicate', 'damage'] as const) {
    const schema = authoredRuleFixture();
    const name = change === 'damage' ? 'DamageDiceOverride' : 'ChoiceSetConfig';
    const node = schema.nodes.find(node => node.name === name)!; assert.ok(node.kind === 'object');
    if (change === 'missing') node.name = 'Renamed';
    if (change === 'provenance') node.fields[1].declaredAt = [];
    if (change === 'predicate') node.fields[1].ref = 'primitive:string';
    if (change === 'damage') node.fields[0].ref = 'primitive:boolean';
    assert.throws(() => authoredRuleInputs(schema), /declaration drift|field drift/);
  }
});

test('Strike scalar traits use the same explicit open vocabulary policy as arrays', () => {
  const schema = authoredRuleFixture();
  const closed = authoredRuleInputs(schema), open = authoredRuleInputs(schema, ['FixtureTraits']);
  const scalar = (graph: typeof closed.graph) => graph.nodes.find(node => node.id === 'FixtureTraits#authored-string-array');
  assert.deepEqual(scalar(closed.graph), { id: 'FixtureTraits#authored-string-array', kind: 'union', members: ['FixtureTraits', 'FixtureTrait'] });
  assert.deepEqual(scalar(open.graph), { id: 'FixtureTraits#authored-string-array', kind: 'union', members: ['FixtureTraits', 'primitive:string'] });
});
