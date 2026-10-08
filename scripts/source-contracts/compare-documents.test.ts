import assert from 'node:assert/strict';
import test from 'node:test';
import { documentPackets } from './sample-documents.js';
import { compareDocumentResults } from './compare-documents.js';
import { compareDocuments } from './compare-documents.js';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';

test('document sampling includes Actor roots and nested Items with untouched numeric and duplicate payloads', () => {
  const source = '{"_id":"a","type":"npc","system":{"x":9007199254740993,"x":2},"items":[{"_id":"i","type":"equipment","system":{}}]}';
  const packets = documentPackets(source, 'packs/test.json', 'test', 'Actor');
  assert.deepEqual(packets.map(packet=>[packet.key,packet.family,packet.context.json_path]), [['Actor','npc','$'],['Item','equipment','$.items[0]']]);
  assert.equal(packets[0].source, source);
  assert.match(packets[0].source, /9007199254740993,"x":2/);
  assert.equal(documentPackets('{"_id":"i","type":"equipment","system":{}}','i','test','Item').length, 1);
  assert.throws(()=>documentPackets('{"type":"npc","system":{}}','a','test','Actor'), /Expected root Actor/);
});
test('document reports distinguish rejection from accepted-but-lossy values across families', () => {
  const packets = documentPackets('{"_id":"a","type":"npc","system":{},"items":[{"_id":"i","type":"equipment","system":{}},{"_id":"j","type":"equipment","system":{}}]}','a','test','Actor');
  const report = compareDocumentResults(packets, [{ok:true,fidelity:null},{ok:true,fidelity:'lost key'},{ok:false,error:{json_path:'$.system.x',expected:'number',actual:'string'}}]);
  assert.deepEqual(report.counts, {occurrences:3,accepted:2,rejected:1,fidelityFailures:1});
  assert.equal(report.families['Item/equipment'].fidelityFailures, 1);
  assert.equal(report.failures.length, 2);
  assert.equal(report.rejectionGroups[0].family, 'Item/equipment');
  assert.equal(report.rejectionGroups[0].pathPattern, '$.system.x');
  assert.throws(()=>compareDocumentResults(packets,[]), /count mismatch/);
});

test('rejection grouping combines collection positions while retaining exact contexts', () => {
  const packets = [0,1].map(index=>({key:'Item' as const, family:'spell',context:{record_key:`spell:${index}`,source_path:'spell.json',json_path:'$'}}));
  const report = compareDocumentResults(packets, packets.map((_,index)=>({ok:false,error:{json_path:`$.overlays["key${index}"].levels.7.damage[0].type`,expected:'damage type',actual:'String("")'}})));
  assert.equal(report.rejectionGroups.length, 1);
  assert.equal(report.rejectionGroups[0].occurrences, 2);
  assert.equal(report.rejectionGroups[0].pathPattern, '$.overlays[*].levels.*.damage[].type');
  assert.equal(report.rejectionGroups[0].examples[1].context.record_key, 'spell:1');
});

test('failed repeat invalidates an old report, and overlap checks do not write into source inputs', async () => {
  const directory=await mkdtemp(path.join(os.tmpdir(),'atlas-document-comparison-'));
  try {
    const out=path.join(directory,'out');await mkdir(out);
    await writeFile(path.join(out,'comparison.json'),JSON.stringify({status:'complete',counts:{accepted:99}}));
    const args={source:path.join(directory,'source'),graph:path.join(directory,'missing-graph.json'),summary:path.join(directory,'summary.json'),out};
    await assert.rejects(compareDocuments(args),/ENOENT/);
    assert.equal(JSON.parse(await readFile(path.join(out,'comparison.json'),'utf8')).status,'incomplete');
    await assert.rejects(compareDocuments({...args,out:directory}),/separate/);
  } finally { await rm(directory,{recursive:true,force:true}); }
});
