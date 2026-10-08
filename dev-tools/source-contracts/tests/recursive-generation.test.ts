import assert from 'node:assert/strict';
import test from 'node:test';
import { generateRustModules } from '../src/generation/source-generation.js';
import {recursiveFixture} from '../fixtures/recursive-fixture.js';
import {readFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {formatRust} from '../src/generation/generate.js';
import {artifactFiles} from '../src/generation/generated-files.js';
import {nodeReferences} from '../src/generation/generation-input.js';
import type { GraphField, GraphNode } from '../src/contracts.js';
import type { GenerationInput } from '../src/generation/generation-input.js';

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
  assert.match(files['consumer.rs'],/Vec<Expression>/);assert.equal((Object.values(files).join('').match(/pub enum Expression \{/g)??[]).length,1);
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

test('fixed tuples preserve nullable position types; optional/rest tuples and nullable roots fail explicitly',()=>{
  const nodes:GraphNode[]=[primitive('string'),primitive('number'),{id:'Pair',kind:'tuple',elements:[{ref:'primitive:string',optional:false,rest:false},{ref:'primitive:number',optional:false,rest:false}],readonly:true}];
  const value=input(nodes,[{ref:'Pair',name:'Pair'}]);
  assert.match(generateRustModules(value)['fixture.rs'],/pub type Pair = \(String, Number\)/);
  for(const flag of ['optional','rest'] as const){const bad=structuredClone(value);const pair=bad.nodes.find(node=>node.kind==='tuple');assert.ok(pair?.kind==='tuple');pair.elements[1][flag]=true;
    assert.throws(()=>generateRustModules(bad),/Optional\/rest tuples/);}
  nodes.push(primitive('null'),{id:'Nullable',kind:'union',members:['primitive:string','primitive:null']});
  const pair=nodes.find(node=>node.kind==='tuple');assert.ok(pair?.kind==='tuple');pair.elements[0].ref='Nullable';
  assert.match(generateRustModules(value)['fixture.rs'],/pub type Pair = \(Option<String>, Number\)/);
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

test('disjoint declared tags identify an arm before defaults without requiring other payload fields',()=>{
  const rust=generateRustModules(recursiveFixture())['common.rs'];
  assert.match(rust,/union_object\(v, &\["type"\]\) && union_member\(v, "type", \|v\| matches!\(v, SourceValue::String\(value\) if value == "a"\)\)/);
  assert.match(rust,/union_required\(v, c, p, &\[\("type", false\)\]\)/);
  assert.doesNotMatch(rust,/union_required\(v, c, p, &\[\("type", false\), \("payload"/);
  // Overlapping literal domains do not justify this shortcut.
  const fixture=recursiveFixture();
  const second=fixture.nodes.find(node=>node.id==='TaggedTwo');assert.ok(second?.kind==='object');second.fields[0].ref='A';
  const overlap=generateRustModules(fixture)['common.rs'];
  assert.match(overlap,/union_object\(v, &\["type", "payload"\]\)/);
});

test('source names map to Rust fields with explicit serialization names and collision checks',()=>{
  const rust=generateRustModules(recursiveFixture())['common.rs'];
  assert.match(rust,/pub _id: SourcePresence<String>/);
  assert.match(rust,/#\[serde\(rename = "greater-darkvision"\)\]\s+pub greater_darkvision:/);
  assert.match(rust,/#\[serde\(rename = "self"\)\]\s+pub self_:/);
  assert.match(rust,/#\[serde\(rename = "1st"\)\]\s+pub _1st:/);
  for (const names of [['greater-darkvision','greater_darkvision'],['self','self_'],['additional-fields'],['_'],['']]) {
    const selected=input([primitive('string'),object('Collision',names.map(name=>field(name,'primitive:string')))],[{ref:'Collision',name:'Collision'}]);
    assert.throws(()=>generateRustModules(selected),/Rust field name/);
  }
});

test('compiler-resolved indexed intersections reuse named owners and pure maps',()=>{
  const rust=generateRustModules(recursiveFixture())['common.rs'];
  assert.match(rust,/pub type IntersectionBag = ConstrainedBag/);
  assert.match(rust,/parse_intersection_map[^]*?ParseResult<IntersectionMap>/);
  assert.match(rust,/pub type IntersectionMap = MaybeNumberMap/);
  assert.match(rust,/pub next: SourcePresence<Box<RecursiveIntersection>>/);
  assert.match(rust,/pub indexed_fields: crate::source_model::SourceMap<RecursiveIntersection>/);
  const legacy=recursiveFixture();
  const node=legacy.nodes.find(node=>node.kind==='intersection');assert.ok(node);
  delete (node as unknown as {indexSignatures?:unknown}).indexSignatures;
  assert.throws(()=>generateRustModules(legacy),/re-extract the graph/);
});

test('compiled generic fixture output is fresh and includes residual cycles and empty tuples',async()=>{
  const files=generateRustModules(recursiveFixture());
  const dir=fileURLToPath(new URL('../../../../crates/atlas-ingest/tests/fixtures/source_model/generated/', import.meta.url));
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

test('anonymous scalar unions use canonical member names and share across modules',()=>{
  for(const [members,name] of [
    [['primitive:string','primitive:number'],'StringOrNumber'],
    [['True','False','primitive:string'],'StringOrBoolean'],
    [['True','primitive:number','False'],'NumberOrBoolean'],
    [['True','primitive:number','primitive:string','False'],'StringOrNumberOrBoolean'],
  ] as [string[],string][]) {
    const nodes:GraphNode[]=[primitive('string'),primitive('number'),
      {id:'True',kind:'literal',value:true},{id:'False',kind:'literal',value:false},
      {id:'First',kind:'union',members},{id:'Second',kind:'union',members:[...members].reverse()},
      object('Base',[field('value','First')]),object('Consumer',[field('other','Second')])];
    const selected=input(nodes,[{ref:'Base',name:'Base'},{ref:'Consumer',name:'Consumer',module:'consumer'}]);
    const files=generateRustModules(selected),all=Object.values(files).join('');
    assert.equal((all.match(new RegExp(`pub enum ${name} \\{`,'g'))??[]).length,1);
    assert.match(files['fixture.rs'],new RegExp(`pub value: SourcePresence<${name}>`));
    assert.match(files['consumer.rs'],new RegExp(`generated::fixture::\\{[^}]*${name}`));
    assert.match(files['consumer.rs'],new RegExp(`pub other: SourcePresence<${name}>`));
    const reversed=structuredClone(selected);
    for(const node of reversed.nodes)if(node.kind==='union')node.members.reverse();
    assert.deepEqual(generateRustModules(reversed),files);
  }
});

test('declared union names and restricted literal alternatives retain their identity',()=>{
  const nodes:GraphNode[]=[primitive('string'),primitive('number'),
    {id:'Amount',name:'Amount',kind:'union',members:['primitive:number','primitive:string']},
    {id:'Yes',name:'Yes',kind:'literal',value:true},
    {id:'Restricted',name:'Restricted',kind:'union',members:['Yes','primitive:string']}];
  const files=generateRustModules(input(nodes,[{ref:'Amount',name:'Amount'},{ref:'Restricted',name:'Restricted'}]));
  assert.match(files['fixture.rs'],/pub enum Amount/);
  assert.match(files['fixture.rs'],/pub enum Restricted/);
  assert.doesNotMatch(files['fixture.rs'],/pub enum StringOrBoolean/);
  assert.match(files['fixture.rs'],/if value \{ Ok\(value\)/);
});

test('scalar union names fail explicitly when an unrelated declared type owns that name',()=>{
  const nodes:GraphNode[]=[primitive('string'),primitive('number'),
    {id:'Anonymous',kind:'union',members:['primitive:string','primitive:number']},
    object('StringOrNumber',[field('label','primitive:string')]),object('Container',[field('value','Anonymous')])];
  for(const roots of [[{ref:'StringOrNumber',name:'StringOrNumber'},{ref:'Container',name:'Container'}],
    [{ref:'Container',name:'Container'},{ref:'StringOrNumber',name:'StringOrNumber'}]])
    assert.throws(()=>generateRustModules(input(nodes,roots)),/Rust name collision.*StringOrNumber/);
});

test('anonymous tuples inline their types and import cross-module type dependencies',()=>{
  const nodes:GraphNode[]=[primitive('string'),primitive('number'),
    {id:'Scalar',kind:'union',members:['primitive:string','primitive:number']},
    {id:'Pair',kind:'tuple',elements:[{ref:'Scalar',optional:false,rest:false},{ref:'primitive:number',optional:false,rest:false}],readonly:false},
    object('Base',[field('pair','Pair')]),object('Consumer',[field('other','Pair')])];
  const files=generateRustModules(input(nodes,[{ref:'Base',name:'Base'},{ref:'Consumer',name:'Consumer',module:'consumer'}]));
  assert.match(files['fixture.rs'],/pub pair: SourcePresence<\(StringOrNumber, Number\)>/);
  assert.match(files['consumer.rs'],/pub other: SourcePresence<\(StringOrNumber, Number\)>/);
  assert.match(files['consumer.rs'],/use serde_json::Number/);
  assert.match(files['consumer.rs'],/generated::fixture::\{StringOrNumber, read_base_pair\}/);
  assert.doesNotMatch(Object.values(files).join(''),/pub type BasePair|use .*\(StringOrNumber/);
});
