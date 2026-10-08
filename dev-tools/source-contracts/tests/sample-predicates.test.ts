import assert from 'node:assert/strict';
import test from 'node:test';
import {predicatePackets} from '../src/comparison/sample-predicates.js';

test('predicate discovery preserves raw spans and distinguishes constructor inputs from arrays',()=>{
  const text='{"_id":"x","system":{"rules":[{"key":"ChoiceSet","predicate":{"not":"invalid-array"},"choices":[{"predicate":{"not":"cold"}}]},{"key":"FlatModifier","removeAfterRoll":[{"eq":["level",18446744073709551615]}]},{"key":"ItemAlteration","definition":["a"]}]}}';
  const packets=predicatePackets(text,'packs/test/x.json','test');
  assert.equal(packets.length,4);assert.equal(packets[0].shape,'array');assert.equal(packets[1].shape,'input');
  assert.equal(packets[1].context.json_path,'$.system.rules[0].choices[0].predicate');
  assert.match(packets[2].source,/18446744073709551615/);assert.equal(packets[3].declaration,'legacy-definition');
});
test('declaration contexts cover additional predicate fields and retain malformed values',()=>{
  const packets=predicatePackets(JSON.stringify({_id:'x',system:{rules:[{key:'ChoiceSet',choices:{filter:42}},
    {key:'CraftingAbility',craftableItems:['a']},{key:'RollOption',disabledIf:null,removeAfterRoll:true},
    {key:'FlatModifier',removeAfterRoll:false},{key:'SubstituteRoll',removeAfterRoll:'if-enabled'},
    {key:'RollTwice',removeAfterRoll:true},{key:'Other',filter:['not-a-predicate'],definition:null,exceptions:[{definition:['nested']}]}]}}),'x.json','x');
  assert.deepEqual(packets.map(p=>[p.field,p.source]),[['choices.filter','42'],['craftableItems','["a"]'],['disabledIf','null'],['definition','null'],['definition','["nested"]']]);
});
test('recursive document traversal samples repeated authored members separately',()=>{
  const packets=predicatePackets('{"_id":"x","items":[{"system":{"rules":[{"key":"ChoiceSet","choices":[{"predicate":["a"]}]}]}}],"predicate":["b"],"predicate":["c"]}','x.json','x');
  assert.equal(packets.length,3);assert.equal(packets[0].shape,'input');
  assert.deepEqual(packets.slice(1).map(p=>p.source),['["b"]','["c"]']);
});
