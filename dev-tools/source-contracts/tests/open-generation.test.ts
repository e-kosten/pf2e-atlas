import assert from 'node:assert/strict';
import test from 'node:test';
import type {GraphNode} from '../src/contracts.js';
import type {GenerationInput} from '../src/generation/generation-input.js';
import {generateRustModules} from '../src/generation/source-generation.js';
import {recursiveFixture} from '../fixtures/recursive-fixture.js';

function input(node:GraphNode):GenerationInput {
  return {source:recursiveFixture().source,nodes:[node,{id:'primitive:string',kind:'primitive',value:'string'}],
    selection:[{name:'Selected',declaration:node.id,valueRef:node.id,module:'fixture',fields:[],deferred:[]}]};
}

test('explicit open domains reuse SourceValue with distinct parsing constraints',()=>{
  const rust=generateRustModules(recursiveFixture());
  assert.match(rust['common.rs'],/pub type UnknownValue = SourceValue/);
  assert.match(rust['common.rs'],/non_primitive\(v, c, p\)/);
  assert.match(rust['common.rs'],/non_nullish\(v, c, p\)/);
  assert.match(rust['common.rs'],/SourceValue::Array\(_\) \| SourceValue::Object\(_\)/);
  assert.match(rust['consumer.rs'],/payload: f.presence\("payload", non_primitive\)/);
});

test('anonymous unions with open object domains retain descriptive canonical names',()=>{
  const fixture=recursiveFixture();
  fixture.nodes.push({id:'Atomic',kind:'union',members:['Open:object','primitive:number','primitive:string']});
  fixture.selection.push({name:'Atomic',declaration:'Atomic',valueRef:'Atomic',module:'consumer',fields:[],deferred:[]});
  const files=generateRustModules(fixture);
  assert.match(files['consumer.rs'],/pub enum StringOrNumberOrObject/);
  assert.match(files['consumer.rs'],/Object\(SourceValue\)/);
  const reversed=structuredClone(fixture);
  const union=reversed.nodes.find(node=>node.id==='Atomic');assert.ok(union?.kind==='union');union.members.reverse();
  assert.deepEqual(generateRustModules(reversed),files);
});

test('named indexed owners remain structs and share typed dynamic values across modules',()=>{
  const rust=generateRustModules(recursiveFixture());
  assert.match(rust['common.rs'],/pub struct OpenBag/);
  assert.match(rust['common.rs'],/pub indexed_fields: crate::source_model::SourceMap<SourceValue>/);
  assert.match(rust['common.rs'],/f.indexed\(&\["label"\], &\["retired"\], unknown\)/);
  assert.match(rust['common.rs'],/additional_fields: f.retained\(&\["retired"\]\)/);
  assert.match(rust['consumer.rs'],/generated::common::\{[^}]*OpenBag/);
  assert.match(rust['common.rs'],/pub child: SourcePresence<Box<RecursiveBag>>/);
});

test('tuple union kind guards include null as a unit Rust variant for open values',()=>{
  const rust=generateRustModules(recursiveFixture())['common.rs'];
  assert.match(rust,/SourceValue::Null \|/);
  assert.doesNotMatch(rust,/SourceValue::Null\(_\)/);
});

test('open support does not mask unresolved, structured open or unsupported index domains',()=>{
  for(const node of [
    {id:'gap',kind:'unsupported',reason:'fixture'}, {id:'gap',kind:'unresolved'},
    {id:'gap',kind:'open',domain:'unknown',fields:[{name:'field',ref:'primitive:string',optional:false,nullable:false,undefinedAllowed:false,forbidden:false,declaredAt:[]}]},
    {id:'gap',kind:'object',fields:[],indexSignatures:[{key:'primitive:string',value:'primitive:string',readonly:false},{key:'primitive:number',value:'primitive:string',readonly:false}]},
  ] as GraphNode[])assert.throws(()=>generateRustModules(input(node)),/Unsupported|unsupported/);
});

test('indexed owner identity includes its entry constraints and forbidden member retention',()=>{
  const fixture=recursiveFixture();
  const first=fixture.nodes.find(node=>node.id==='OpenBag');assert.ok(first?.kind==='object');
  fixture.nodes.push({...first,id:'OtherBag',name:'OtherBag',fields:first.fields.filter(field=>!field.forbidden)});
  fixture.selection.push({name:'OtherBag',declaration:'OtherBag',valueRef:'OtherBag',module:'consumer',fields:[],deferred:[]});
  assert.match(generateRustModules(fixture)['consumer.rs'],/pub struct OtherBag/);
});
