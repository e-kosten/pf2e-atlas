import assert from 'node:assert/strict';
import test from 'node:test';
import type {GraphField,GraphNode} from './contracts.js';
import type {GenerationInput} from './generation-input.js';
import {generateRustModules} from './source-generation.js';

const field=(name:string,ref:string):GraphField=>({name,ref,optional:false,nullable:false,undefinedAllowed:false,forbidden:false,declaredAt:[]});
const object=(id:string,fields:GraphField[],index?:string):GraphNode=>({id,name:id,kind:'object',fields,
  indexSignatures:index?[{key:'primitive:string',value:index,readonly:false}]:[]});
const array=(id:string,element:string):GraphNode=>({id,kind:'array',element,readonly:false});
const primitives:GraphNode[]=['string','null','undefined','never'].map(value=>({id:'primitive:'+value,kind:'primitive',value}));
const input=(nodes:GraphNode[],roots:string[]):GenerationInput=>({source:{system_version:'fixture',source_digest:'fixture',input_file_count:1,git_commit:null,git_clean:null},
  nodes:[...primitives,...nodes],selection:roots.map(name=>({name,declaration:name,valueRef:name,module:name==='Consumer'?'consumer':'common',fields:[],deferred:[]}))});

test('nullable and undefined array entries share persisted shapes without widening ordinary arrays or maps',()=>{
  const nodes:GraphNode[]=[{id:'Nullable',kind:'union',members:['primitive:string','primitive:null']},
    {id:'Optional',kind:'union',members:['primitive:string','primitive:undefined']},
    array('Nullables','Nullable'),array('Optionals','Optional'),array('Strings','primitive:string'),
    object('NullableMap',[],'Nullable'),object('OptionalMap',[],'Optional'),
    object('Base',[field('nullable','Nullables'),field('optional','Optionals'),field('plain','Strings'),field('map','NullableMap'),field('optional_map','OptionalMap')]),
    object('Consumer',[field('values','Optionals')])];
  const files=generateRustModules(input(nodes,['Base','Consumer']));
  assert.match(files['common.rs'],/Vec<Option<String>>/);
  assert.match(files['common.rs'],/Vec<String>/);
  assert.match(files['common.rs'],/SourceMap<Option<String>>/);
  assert.match(files['common.rs'],/SourceMap<String>/);
  assert.equal((files['common.rs'].match(/nullable\(v, c, p, string\)/g)??[]).length,1);
  assert.match(files['consumer.rs'],/generated::common::/);
  assert.deepEqual(generateRustModules(input(nodes,['Base','Consumer'])),files);
});

test('nullable scalar array union guards include null and keep literal discrimination',()=>{
  const nodes:GraphNode[]=[{id:'A',kind:'literal',value:'a'},{id:'B',kind:'literal',value:'b'},
    {id:'NullableA',kind:'union',members:['A','primitive:null']},
    array('As','NullableA'),array('Bs','B'),
    {id:'Either',name:'Either',kind:'union',members:['As','Bs']}];
  const rust=generateRustModules(input(nodes,['Either']))['common.rs'];
  assert.match(rust,/values\.iter\(\)\.all\(\|v\| \(matches!\(v, SourceValue::Null\) \|\| matches!\(v, SourceValue::String\(value\) if value == "a"\)\)\)/);
  assert.match(rust,/value == "b"/);
});

test('null-only collections preserve JSON slots and undefined-only object indices fail explicitly',()=>{
  const nodes:GraphNode[]=[array('Nulls','primitive:null'),array('Undefineds','primitive:undefined'),
    {id:'Nullish',kind:'union',members:['primitive:null','primitive:undefined']},array('Both','Nullish'),
    {id:'NullTuple',kind:'tuple',elements:[{ref:'primitive:undefined',optional:false,rest:false}],readonly:false},
    object('Base',[field('nulls','Nulls'),field('undefineds','Undefineds'),field('both','Both'),field('tuple','NullTuple')])];
  const rust=generateRustModules(input(nodes,['Base']))['common.rs'];
  assert.match(rust,/Vec<\(\)>/);
  assert.match(rust,/array\(v, c, p, null\)/);
  assert.match(rust,/null\(&values\[0\]/);
  assert.throws(()=>generateRustModules(input([object('NoValues',[],'primitive:undefined')],['NoValues'])),/no persisted value alternatives/);
});

test('JSON distinguishes omitted object properties from undefined and sparse array slots',()=>{
  assert.equal(JSON.stringify({missing:undefined,present:null}),'{"present":null}');
  assert.equal(JSON.stringify([undefined,null,,1]),'[null,null,null,1]');
});

test('shared shape signatures stay bounded across deeply repeated descendants',()=>{
  const nodes:GraphNode[]=[object('Leaf',[field('label','primitive:string')])];
  let ref='Leaf';
  for(let depth=0;depth<40;depth++) {
    const id='Layer'+depth;
    nodes.push(object(id,[field('left',ref),field('right',ref)]));
    ref=id;
  }
  const files=generateRustModules(input(nodes,[ref]));
  assert.equal((files['common.rs'].match(/pub struct /g)??[]).length,41);
  assert.deepEqual(generateRustModules(input(nodes,[ref])),files);
  nodes.push({id:'Unsupported',kind:'unsupported',reason:'fixture'});
  const leaf=nodes[0];assert.ok(leaf.kind==='object');leaf.fields.push(field('bad','Unsupported'));
  assert.throws(()=>generateRustModules(input(nodes,[ref])),/Unsupported selected construct/);
});

test('nominal recursive anchors still validate all reachable field and collection shapes',()=>{
  const nodes:GraphNode[]=[object('Link',[field('next','Link'),field('values','BadList')]),
    array('BadList','Unknown'),{id:'Unknown',kind:'unsupported',reason:'fixture'}];
  assert.throws(()=>generateRustModules(input(nodes,['Link'])),/Unsupported selected construct/);
});
