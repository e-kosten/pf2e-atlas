import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { conditionDefinitions, conditionSourceInputs, type ConditionCoverage } from '../src/generation/condition-definitions.js';
const helper = `function setImmunitiesFromTraits(actor: CreaturePF2e): void {
if(actor.isOfType("character")) return;
const traits=actor.system.traits.value;
const immunities=actor.attributes.immunities;
const existing=immunities.map(i=>i.type);
if(traits.includes("swarm") && !existing.includes("prone")) immunities.push(new Immunity({type:"prone",source:_loc("PF2E.TraitSwarm")}));
}`;
async function fixture(t: {after: (callback: () => Promise<void>) => void}) {
  const root = await mkdtemp(path.join(os.tmpdir(),'atlas-condition-catalog-'));
  t.after(()=>rm(root,{recursive:true,force:true}));
  for (const directory of ['packs/pf2e/conditions','src/module/actor/creature','src/module/rules/rule-element'])
    await mkdir(path.join(root,directory),{recursive:true});
  await writeFile(path.join(root,'system.pf2e.json'),JSON.stringify({packs:[{name:'conditionitems',path:'packs/conditions',type:'Item'}]}));
  await writeFile(path.join(root,'src/module/actor/creature/helpers.ts'),helper);
  await writeFile(path.join(root,'src/module/rules/rule-element/flat-modifier.ts'),
    'class FlatModifierRuleElement { static defineSchema() { return {type:new fields.StringField({initial:"untyped"})}; } }');
  const condition={_id:'nativeCondition1',name:'Enfeebled',type:'condition',system:{
    value:{isValued:true,value:1},overrides:[] as string[],rules:[{key:'FlatModifier',slug:'enfeebled',selector:['str-based','str-damage'],type:'status',value:'-@item.badge.value'}] as Record<string,unknown>[]}};
  const coverage: ConditionCoverage={adapters:[],overrides:[],persistentGrants:[]};
  const save=()=>writeFile(path.join(root,'packs/pf2e/conditions/enfeebled.json'),JSON.stringify(condition));
  await save();
  const read=()=>conditionDefinitions(root,new Set(['swarm']),coverage);
  return {root,condition,coverage,save,read};
}
test('numerical changes and source fields flow into a distinct catalog identity',async t=>{
  const f=await fixture(t);const before=await f.read();
  assert.equal(before.conditions[0].id,'nativeCondition1');
  assert.deepEqual(before.conditions[0].modifiers[0].value,{kind:'magnitude',coefficient:-1});
  f.condition.system.rules[0].value='-2 * @item.badge.value';await f.save();
  const after=await f.read();assert.notEqual(after.digest,before.digest);
  assert.deepEqual(after.conditions[0].modifiers[0].value,{kind:'magnitude',coefficient:-2});
  f.condition.system.rules[0].newBehavior=true;await f.save();
  await assert.rejects(f.read(),/Unreviewed condition fields/);
});
test('conditional modifier predicates retain authored context instead of becoming unconditional',async t=>{
  const f=await fixture(t);f.condition.system.rules[0].predicate=['item:trait:auditory'];await f.save();
  assert.deepEqual((await f.read()).conditions[0].modifiers[0].predicate,{kind:'all',terms:[{kind:'atom',value:'item:trait:auditory'}]});
  f.condition.system.rules[0].predicate=['unmodeled:context'];await f.save();
  await assert.rejects(f.read(),/Unsupported predicate/);
});
test('opaque Atlas adapters require an exact reviewed declaration',async t=>{
  const f=await fixture(t);const rule={key:'Immunity',type:'visual'};
  f.condition.system.rules=[rule];f.coverage.adapters.push({slug:'enfeebled',ordinal:0,key:'Immunity',
    digest:createHash('sha256').update(JSON.stringify(rule)).digest('hex'),owner:'rules/senses'});
  await f.save();await f.read();
  rule.type='auditory';await f.save();await assert.rejects(f.read(),/Unreviewed condition adapter/);
});
test('unknown grant lifetimes, missing destinations, cycles and unreviewed overrides fail',async t=>{
  const f=await fixture(t);f.condition.system.rules=[{key:'GrantItem',uuid:'Compendium.pf2e.conditionitems.Item.Missing',inMemoryOnly:true}];
  await f.save();await assert.rejects(f.read(),/destination missing/);
  f.condition.system.rules[0].uuid='Compendium.pf2e.conditionitems.Item.Enfeebled';
  await f.save();await assert.rejects(f.read(),/Cyclic/);
  f.condition.system.rules[0].inMemoryOnly=false;
  await f.save();await assert.rejects(f.read(),/Unreviewed persistent grant lifetime/);
  f.condition.system.rules=[];f.condition.system.overrides=['enfeebled'];
  await f.save();await assert.rejects(f.read(),/Unreviewed condition override/);
});
test('pack bytes participate in their own identity without a declaration change',async t=>{
  const f=await fixture(t);const before=await conditionSourceInputs(f.root);
  f.condition.name='A changed native name';await f.save();
  assert.notEqual((await conditionSourceInputs(f.root)).digest,before.digest);
});
test('HP loss on a condition without an Atlas event owner cannot silently generate',async t=>{
  const f=await fixture(t);
  f.condition.system.rules=[{key:'LoseHitPoints',reevaluateOnUpdate:true,value:'@item.badge.value'}];
  await f.save();
  await assert.rejects(f.read(),/Unsupported condition HP loss/);
});
