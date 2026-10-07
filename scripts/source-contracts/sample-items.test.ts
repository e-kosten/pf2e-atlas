import assert from 'node:assert/strict';
import test from 'node:test';
import {itemPackets} from './sample-items.js';

test('recursive Item sampling keeps exact spans and paths without sampling Actor roots',()=>{
  const text = '{"_id":"actor","type":"npc","system":{},"items":[{"_id":"outer","type":"weapon","system":{"future":9007199254740993,"subitems":[{"_id":"nested","type":"equipment","system":{"x":1,"x":2}}]}}]}';
  const packets = itemPackets(text,'packs/actors/x.json','actors','Actor');
  assert.equal(packets.length,2);
  assert.equal(packets[0].context.json_path,'$.items[0]');
  assert.equal(packets[1].context.json_path,'$.items[0].system.subitems[0]');
  assert.match(packets[0].source,/9007199254740993/);
  assert.match(packets[1].source,/"x":1,"x":2/);
  const root = itemPackets('{"_id":"x","type":"futureFamily","system":{}}','x','items','Item');
  assert.equal(root[0].family,'futureFamily');
  assert.equal(root[0].context.json_path,'$');
});

test('unselected objects may repeat context-like names; actual Item context stays unique',()=>{
  const packets = itemPackets('{"_id":"x","type":"book","system":{"future":{"type":"a","type":"b","_id":"a","_id":"b"}}}','x','items','Item');
  assert.equal(packets.length,1);
  assert.match(packets[0].source,/"type":"a","type":"b"/);
  assert.throws(()=>itemPackets('{"_id":"x","type":"book","type":"equipment","system":{}}','x','items','Item'),/Duplicate Item context/);
  assert.throws(()=>itemPackets('{"_id":"x","type":"book"}','x','items','Item'),/Expected root Item/);
});
