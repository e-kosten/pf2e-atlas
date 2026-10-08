import assert from 'node:assert/strict';
import test from 'node:test';
import type { TypeGraph, GraphField } from '../src/contracts.js';
import { documentTraitArrays } from '../src/generation/document-traits.js';
import { generateRustModules } from '../src/generation/source-generation.js';

test('open trait policy follows document unions, preserves small vocabularies and rejects nonstring trait drift', () => {
  const field=(name:string,ref:string):GraphField=>({name,ref,optional:false,nullable:false,undefinedAllowed:false,forbidden:false,declaredAt:[]});
  const graph:TypeGraph={format:'atlas-source-type-graph/v1',typescript:'fixture',complete:true,status:'complete',diagnostics:[],projectDiagnostics:{selected:[],unrelated:[]},
    roots:[{file:'fixture',name:'ItemSource',documentKind:'Item',ref:'documents'}],nodes:[
      {id:'primitive:string',kind:'primitive',value:'string'},{id:'primitive:null',kind:'primitive',value:'null'},
      {id:'primitive:number',kind:'primitive',value:'number'},
      {id:'trait',kind:'literal',value:'fire'}, {id:'rarity',kind:'literal',value:'common'},
      {id:'traits',kind:'array',element:'trait',readonly:false},
      {id:'traitObject',kind:'object',fields:[field('value','traits'),field('rarity','rarity')],indexSignatures:[]},
      {id:'system',kind:'object',fields:[field('traits','traitObject')],indexSignatures:[]},
      {id:'document',kind:'object',fields:[field('system','system')],indexSignatures:[]},
      {id:'documents',kind:'union',members:['document','primitive:null']}]};
  const arrays=documentTraitArrays(graph);assert.deepEqual(arrays,['traits']);
  const input={source:{system_version:'fixture',source_digest:'fixture',input_file_count:1,git_commit:null,git_clean:null},nodes:graph.nodes,openTraitArrays:arrays,
    selection:[{name:'Document',declaration:'document',valueRef:'document',module:'fixture',fields:[],deferred:[]}]};
  const rust=generateRustModules(input)['fixture.rs'];
  assert.match(rust,/Vec<String>/);assert.match(rust,/"common" => Ok/);
  const traits=graph.nodes.find(node=>node.id==='traits');assert.ok(traits?.kind==='array');traits.element='primitive:number';
  assert.throws(()=>generateRustModules(input),/requires a string vocabulary array/);
  graph.nodes.push({id:'primitive:never',kind:'primitive',value:'never'});traits.element='primitive:never';
  assert.deepEqual(documentTraitArrays(graph),[]);
});
