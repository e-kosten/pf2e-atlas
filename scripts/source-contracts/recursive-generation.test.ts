import assert from 'node:assert/strict';
import test from 'node:test';
import { generateRustModules } from './source-generation.js';
import {recursiveFixture} from './recursive-fixture.js';
import {readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {formatRust} from './generate.js';
import {artifactFiles} from './generated-files.js';
import {nodeReferences} from './generation-input.js';
import type { GraphField, GraphNode } from './contracts.js';
import type { GenerationInput } from './generation-input.js';

const field=(name:string,ref:string,optional=false):GraphField=>({name,ref,optional,nullable:false,undefinedAllowed:false,forbidden:false,declaredAt:[]});
const object=(id:string,fields:GraphField[]):GraphNode=>({id,name:id,kind:'object',fields,indexSignatures:[]});
const primitive=(value:string):GraphNode=>({id:'primitive:'+value,kind:'primitive',value});
function input(nodes:GraphNode[],roots:{ref:string;name:string;module?:string}[]):GenerationInput {
  return {source:{system_version:'fixture',source_digest:'fixture',input_file_count:1,git_commit:null,git_clean:null},nodes,
    selection:roots.map(root=>({name:root.name,declaration:root.ref,valueRef:root.ref,module:root.module??'fixture',fields:[],deferred:[]}))};
}

test('recursive union owners terminate and box inline arms while Vec cycles stay unboxed',()=>{
  const nodes:GraphNode[]=[primitive('string'),{id:'Expression',name:'Expression',kind:'union',members:['primitive:string','Not','And']},
    object('Not',[field('not','Expression')]),object('And',[field('and','List')]),{id:'List',kind:'array',element:'Expression',readonly:false}];
  const value=input(nodes,[{ref:'Expression',name:'Expression'},{ref:'List',name:'Expressions',module:'consumer'}]);
  const files=generateRustModules(value);
  assert.match(files['fixture.rs'],/Not\(Box<Not>\)/);assert.match(files['fixture.rs'],/And\(And\)/);
  assert.match(files['fixture.rs'],/pub not: SourcePresence<Expression>/);
  assert.match(files['fixture.rs'],/Vec<Expression>/);assert.equal((Object.values(files).join('').match(/pub enum Expression \{/g)??[]).length,1);
  assert.match(files['consumer.rs'],/generated::fixture/);
  assert.deepEqual(generateRustModules(value),files);
});

test('direct recursive object edges have explicit indirection and unanchored aliases reject',()=>{
  const rust=generateRustModules(input([object('Link',[field('next','Link',true)])],[{ref:'Link',name:'Link'}]))['fixture.rs'];
  assert.match(rust,/SourcePresence<Box<Link>>/);
  assert.match(rust,/map\(Box::new\)/);
  for(const nodes of [[{id:'Loop',kind:'array',element:'Loop',readonly:false}],
    [{id:'Loop',kind:'tuple',elements:[{ref:'Loop',optional:false,rest:false}],readonly:false}],
    [{id:'Loop',kind:'union',members:['Loop']}]] as GraphNode[][])
    assert.throws(()=>generateRustModules(input(nodes,[{ref:'Loop',name:'Loop'}])),/anchor/);
});

test('fixed tuples preserve position types; optional, rest and nullable entries fail explicitly',()=>{
  const nodes:GraphNode[]=[primitive('string'),primitive('number'),{id:'Pair',kind:'tuple',elements:[{ref:'primitive:string',optional:false,rest:false},{ref:'primitive:number',optional:false,rest:false}],readonly:true}];
  const value=input(nodes,[{ref:'Pair',name:'Pair'}]);
  assert.match(generateRustModules(value)['fixture.rs'],/pub type Pair = \(String, Number\)/);
  for(const flag of ['optional','rest'] as const){const bad=structuredClone(value);const pair=bad.nodes.find(node=>node.kind==='tuple');assert.ok(pair?.kind==='tuple');pair.elements[1][flag]=true;
    assert.throws(()=>generateRustModules(bad),/Optional\/rest tuples/);}
  nodes.push(primitive('null'),{id:'Nullable',kind:'union',members:['primitive:string','primitive:null']});
  const pair=nodes.find(node=>node.kind==='tuple');assert.ok(pair?.kind==='tuple');pair.elements[0].ref='Nullable';
  assert.throws(()=>generateRustModules(value),/Nullable collection entries/);
  assert.throws(()=>generateRustModules(input(nodes,[{ref:'Nullable',name:'Nullable'}])),/Nullable value roots/);
});

test('union selection metadata stays distinct when payload structs share pre-default fields',()=>{
  const nodes:GraphNode[]=[primitive('string'),object('Required',[field('x','primitive:string')]),object('Optional',[field('x','primitive:string',true)]),object('Other',[field('y','primitive:string')]),
    {id:'Strict',name:'Strict',kind:'union',members:['Required','Other']},{id:'Loose',name:'Loose',kind:'union',members:['Optional','Other']}];
  const files=generateRustModules(input(nodes,[{ref:'Strict',name:'Strict'},{ref:'Loose',name:'Loose',module:'consumer'}]));
  assert.match(files['fixture.rs'],/union_object\(v, &\["x"\]\)/);
  assert.match(files['consumer.rs'],/union_object\(v, &\[\]\)/);
  assert.match(files['consumer.rs'],/pub enum Loose/);
});

test('literal discriminants, boolean literals and Rust keyword fields remain checked',()=>{
  const nodes:GraphNode[]=[primitive('string'),{id:'A',kind:'literal',value:'a'},{id:'B',kind:'literal',value:'b'},
    {id:'Yes',kind:'literal',value:true},object('Left',[field('type','A'),field('if','Yes')]),object('Right',[field('type','B')]),
    {id:'Either',name:'Either',kind:'union',members:['Left','Right']}];
  const rust=generateRustModules(input(nodes,[{ref:'Either',name:'Either'}]))['fixture.rs'];
  assert.match(rust,/pub r#type:/);assert.match(rust,/pub r#if:/);
  assert.match(rust,/union_member\(v, "type"/);assert.match(rust,/if value \{ Ok\(value\)/);
  assert.match(rust,/union_required\(v, c, p/);
});

test('compiled generic fixture output is fresh and includes residual cycles and empty tuples',async()=>{
  const files=generateRustModules(recursiveFixture());
  const dir=fileURLToPath(new URL('../../../crates/atlas-ingest/tests/fixtures/source_model/generated/',import.meta.url));
  assert.deepEqual(Object.keys(files).sort(),await artifactFiles(dir));
  for(const [file,text] of Object.entries(files))assert.equal(formatRust(text),await readFile(path.join(dir,file),'utf8'));
  assert.match(files['common.rs'],/pub next: SourcePresence<Box<Node>>/);
  assert.match(files['common.rs'],/pub type Empty = \[\(\); 0\]/);
  assert.match(files['common.rs'],/pub type Yes = bool/);assert.match(files['common.rs'],/pub type One = Number/);
  assert.doesNotMatch(files['consumer.rs'],/use .*\bbool\b/);
  assert.match(files['common.rs'],/matches!\(v, SourceValue::Null\) \|\|/);
});

test('snapshot references include persisted value and retained serialization provenance',()=>{
  const array:GraphNode={id:'Array',kind:'array',element:'Statement',readonly:false,serialization:{basis:'array-subclass',rawRef:'RawArray',method:'toObject',declaredAt:[]}};
  assert.deepEqual(nodeReferences(array),['Statement','RawArray']);
  const object=field('predicate','Input');object.serialization={basis:'predicate-constructor-input',declaredRef:'Declared',declaredType:'Declared',declaredAt:[]};
  assert.deepEqual(nodeReferences({id:'Object',kind:'object',fields:[object],indexSignatures:[]}),['Input','Declared']);
});
