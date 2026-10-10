import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { portfolioResults, portfolioBaselineMatches } from '../src/comparison/compare-portfolio.js';
import { sampleDocuments } from '../src/comparison/sample-documents.js';
import { admissionResults, admissionBaselineMatches } from '../src/comparison/admission-results.js';

test('admission distinguishes retained partial documents, excluded rules and value loss', () => {
  const contexts = ['Item','Item','ChoiceSet','Actor'].map((key,index)=>({key,context:{record_key:`pack:${index}`,source_path:`${index}.json`,json_path:'$'}}));
  const diagnostic = {json_path:'$.level',expected:'number',actual:'String("1")'};
  const report = admissionResults(contexts, [
    {ok:true,retained:true,modeled:true,fidelity:null,diagnostics:[]},
    {ok:true,retained:true,modeled:true,fidelity:null,diagnostics:[diagnostic]},
    {ok:true,retained:true,modeled:false,fidelity:null,diagnostics:[diagnostic]},
    {ok:true,retained:false,modeled:true,fidelity:'lost value',diagnostics:[]},
  ]);
  assert.deepEqual(report.counts, {occurrences:4,retained:3,modeled:3,fullyTyped:2,partial:1,rawOnly:1,rejected:0,fidelityFailures:1,diagnostics:2});
  assert.equal(report.byRoot.ChoiceSet.rawOnly,1);
  assert.equal(report.outcomes.length,3);
  assert.throws(()=>admissionResults(contexts,[]),/count mismatch/);
  const baseline = {sourceDigest:'source',corpusDigest:'corpus',outcomeDigest:report.outcomeDigest,counts:report.counts};
  assert.equal(admissionBaselineMatches(baseline,baseline),true);
  assert.equal(admissionBaselineMatches(baseline,{...baseline,counts:{...baseline.counts,rawOnly:0}}),false);
  assert.equal(admissionBaselineMatches(baseline,{...baseline,outcomeDigest:'changed'}),false);
});

test('portfolio baseline distinguishes changed rejections, acceptance and value loss', () => {
  const contexts = [
    { key: 'Item', context: { record_key: 'pack:a', source_path: 'a.json', json_path: '$' } },
    { key: 'FlatModifier', context: { record_key: 'pack:b', source_path: 'b.json', json_path: '$.system.rules[0]' } },
  ];
  const error = { json_path: '$.value', expected: 'number', actual: 'String("bad")' };
  const baseline = portfolioResults(contexts, [{ ok: true, fidelity: null }, { ok: false, error }]);
  assert.deepEqual(baseline.counts, { occurrences: 2, accepted: 1, rejected: 1, fidelityFailures: 0 });
  assert.equal(baseline.byRoot.FlatModifier.rejected, 1);
  assert.notEqual(portfolioResults(contexts, [{ ok: true, fidelity: null }, { ok: false, error: { ...error, actual: 'Boolean(true)' } }]).rejectionDigest, baseline.rejectionDigest);
  const changedContext = structuredClone(contexts); changedContext[1].context.json_path = '$.system.rules[1]';
  assert.notEqual(portfolioResults(changedContext, [{ ok: true, fidelity: null }, { ok: false, error }]).rejectionDigest, baseline.rejectionDigest);
  const loss = portfolioResults(contexts, [{ ok: true, fidelity: 'lost field' }, { ok: true, fidelity: null }]);
  assert.equal(loss.counts.fidelityFailures, 1);
  assert.equal(loss.failures.length, 1);
  assert.throws(() => portfolioResults(contexts, []), /count mismatch/);
  const pin = { sourceDigest: 'source', corpusDigest: 'corpus', rejectionDigest: baseline.rejectionDigest, counts: baseline.counts };
  assert.equal(portfolioBaselineMatches(pin, pin), true);
  for (const change of [{ sourceDigest: 'different' }, { corpusDigest: 'changed' }, { rejectionDigest: 'changed' },
    { counts: { ...pin.counts, accepted: 0 } }, { counts: { ...pin.counts, fidelityFailures: 1 } }])
    assert.equal(portfolioBaselineMatches(pin, { ...pin, ...change }), false);
});

test('full portfolio sampling includes non-Actor documents without changing raw payloads', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'atlas-portfolio-corpus-'));
  try {
    await mkdir(path.join(directory, 'static'));
    const packs = ['JournalEntry', 'Macro', 'RollTable'].map(type => ({ type, name: type.toLowerCase(), path: `packs/${type}` }));
    await writeFile(path.join(directory, 'static/system.json'), JSON.stringify({ packs }));
    const raw = '{"_id":"id","future":9007199254740993,"future":null}';
    for (const pack of packs) {
      await mkdir(path.join(directory, pack.path), { recursive: true });
      await writeFile(path.join(directory, pack.path, 'a.json'), raw);
      await writeFile(path.join(directory, pack.path, '_folders.json'), '[]');
    }
    const packets = [];
    for await (const packet of sampleDocuments(directory, true)) packets.push(packet);
    assert.deepEqual(packets.map(packet => packet.key), ['JournalEntry', 'Macro', 'RollTable']);
    assert.ok(packets.every(packet => packet.source === raw && packet.context.json_path === '$'));
    const limited = [];
    for await (const packet of sampleDocuments(directory)) limited.push(packet);
    assert.equal(limited.length, 0);
    await writeFile(path.join(directory, 'static/system.json'), JSON.stringify({ packs: [{ type: 'FutureDocument', path: 'packs/unknown', name: 'unknown' }] }));
    await assert.rejects(async () => { for await (const _ of sampleDocuments(directory, true)) { /* exhaust */ } }, /Unmodeled document kind/);
  } finally { await rm(directory, { recursive: true, force: true }); }
});
