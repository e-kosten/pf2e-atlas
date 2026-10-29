import assert from 'node:assert/strict';
import test from 'node:test';
import { conditionValue, conditionPredicate } from '../src/generation/condition-expressions.js';
import { traitImmunities } from '../src/generation/trait-immunities.js';

const prefix = `function setImmunitiesFromTraits(actor: CreaturePF2e): void {
if (actor.isOfType("character")) return;
const traits = actor.system.traits.value;
const immunities = actor.attributes.immunities;
const existing = immunities.map(i => i.type);`;
const branch = (trait: string) => `if (traits.includes("${trait}") && !existing.includes("prone")) {
immunities.push(new Immunity({type:"prone",source:_loc("PF2E.TraitExample")})); }`;

test('new supported traits produce rules without a trait-specific branch', () => {
  const rules = traitImmunities(prefix + branch('swarm') + branch('new-trait') + '}', new Set(['swarm', 'new-trait']));
  assert.equal(rules.length, 2);
  assert.deepEqual(rules[1].predicate, {kind:'all',terms:[
    {kind:'not',term:{kind:'actorKind',value:'character'}},
    {kind:'all',terms:[{kind:'trait',value:'new-trait'},{kind:'not',term:{kind:'authored',value:'prone'}}]},
  ]});
});
test('trait exclusions and loop values remain explicit', () => {
  const rules = traitImmunities(prefix + `if (traits.includes("construct") && !traits.includes("eidolon")) {
const kinds = ["doomed","drained"];
for (const kind of kinds) { if (!existing.includes(kind)) immunities.push(new Immunity({type:kind,source:_loc("PF2E.TraitConstruct")})); }
}}`, new Set(['construct','eidolon']));
  assert.deepEqual(rules.map(rule => rule.type), ['doomed','drained']);
  assert.match(JSON.stringify(rules[0].predicate), /eidolon/);
});
test('unrecognized statements and unknown trait names stop extraction', () => {
  assert.throws(() => traitImmunities(prefix + branch('unknown') + '}', new Set()), /Unknown immunity trait/);
  assert.throws(() => traitImmunities(prefix + branch('swarm') + ' actor.update({}); }', new Set(['swarm'])), /Unsupported immunity syntax/);
  assert.throws(() => traitImmunities(prefix + 'for (const kind of computeKinds()) {} }', new Set()), /Unsupported immunity syntax/);
  assert.throws(() => traitImmunities(prefix + 'if (traits.includes("swarm")) { if (existing.includes("prone")) return; } }', new Set(['swarm'])), /Unsupported immunity syntax/);
});
test('condition expressions extract coefficients and level floors', () => {
  assert.deepEqual(conditionValue(-4), {kind:'fixed',value:-4});
  assert.deepEqual(conditionValue('-@item.badge.value'), {kind:'magnitude',coefficient:-1});
  assert.deepEqual(conditionValue('-2 * @item.badge.value'), {kind:'magnitude',coefficient:-2});
  assert.deepEqual(conditionValue('min(-1 * @actor.level,-1) * @item.badge.value'), {kind:'levelMagnitude',coefficient:-1,minimumLevel:1});
  assert.deepEqual(conditionValue('max(2,@actor.level) * @item.badge.value'), {kind:'levelMagnitude',coefficient:1,minimumLevel:2});
  for (const value of ['@actor.hp', '5 + @item.badge.value', 'max(1,@actor.level) / 2', 'min(@actor.level,1)*@item.badge.value'])
    assert.throws(() => conditionValue(value));
});
test('deafened predicates retain nested context requirements', () => {
  const compiled = conditionPredicate([{or:[{and:['check:type:skill','item:trait:auditory']},{and:['check:statistic:base:perception','check:statistic:initiative']}]}]);
  assert.deepEqual(compiled, {kind:'all',terms:[{kind:'any',terms:[
    {kind:'all',terms:[{kind:'atom',value:'check:type:skill'},{kind:'atom',value:'item:trait:auditory'}]},
    {kind:'all',terms:[{kind:'atom',value:'check:statistic:base:perception'},{kind:'atom',value:'check:statistic:initiative'}]},
  ]}]});
  assert.throws(() => conditionPredicate(['unknown:context']), /Unsupported predicate/);
});
