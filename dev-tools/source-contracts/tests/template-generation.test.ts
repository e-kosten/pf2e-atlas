import assert from 'node:assert/strict';
import test from 'node:test';
import type {GraphNode} from '../src/contracts.js';
import {recursiveFixture} from '../fixtures/recursive-fixture.js';
import {generateRustModules} from '../src/generation/source-generation.js';

test('template literals retain constraints in parsers and union/tuple guards',()=>{
  const rust=generateRustModules(recursiveFixture());
  assert.match(rust['common.rs'],/pub type Color = String/);
  assert.match(rust['common.rs'],/matches_template\(value, &\["#", ""\]\)/);
  assert.match(rust['common.rs'],/pub type ImagePath = String/);
  assert.match(rust['common.rs'],/matches_template\(value, &\["", "\.png"\]\)/);
  assert.match(rust['common.rs'],/matches_template\(value, &\["Actor\.", "\.Item\.", ""\]\)/);
  assert.match(rust['consumer.rs'],/generated::common::\{[^}]*Color/);
});

test('template interpolation support is explicit and rejects unknown domains and malformed text',()=>{
  for(const node of [
    {id:'Template',kind:'template',text:['',''],parameters:['primitive:number']},
    {id:'Template',kind:'template',text:[''],parameters:['primitive:string']},
    {id:'Template',kind:'template',text:['',''],parameters:['Missing']},
  ] as GraphNode[]) {
    const input=recursiveFixture();input.nodes.push(node);
    input.selection.push({name:'Template',declaration:'Template',valueRef:'Template',module:'consumer',fields:[],deferred:[]});
    assert.throws(()=>generateRustModules(input),/Unsupported template interpolation|Missing generation node/);
  }
});

test('generic instantiations use argument or field context names and share equal shapes',()=>{
  const input=recursiveFixture(),files=generateRustModules(input);
  assert.match(files['common.rs'],/pub struct SourceFromSchemaFirstSchema/);
  assert.match(files['common.rs'],/pub struct SourceFromSchemaSecondSchema/);
  assert.match(files['consumer.rs'],/pub anonymous: SourcePresence<GenericConsumerAnonymousSourceFromSchema>/);
  assert.match(files['consumer.rs'],/pub repeated: SourcePresence<SourceFromSchemaFirstSchema>/);
  const reversed=structuredClone(input);reversed.nodes.reverse();
  assert.deepEqual(generateRustModules(reversed),files);
  const unrelated=input.nodes.find(node=>node.id==='Generic:second');assert.ok(unrelated);
  unrelated.typeArguments=[{expression:'FirstSchema',declaredAt:[]}];
  assert.throws(()=>generateRustModules(input),/Rust name collision/);
});

test('generic union alternatives keep distinct resolved names',()=>{
  const rust=generateRustModules(recursiveFixture())['common.rs'];
  assert.match(rust,/SourceFromSchemaFirstSchema\(SourceFromSchemaFirstSchema\)/);
  assert.match(rust,/SourceFromSchemaSecondSchema\(SourceFromSchemaSecondSchema\)/);
});
