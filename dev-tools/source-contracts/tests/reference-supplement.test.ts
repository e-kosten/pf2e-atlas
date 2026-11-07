import assert from 'node:assert/strict';
import { mkdtemp, readFile, readdir, rm, symlink, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { approvedReferenceData, extractReferenceSupplement, prepareReferenceSupplementCandidate, referenceSupplementDiff,
  sourceTagText, type SupplementMappings } from '../src/comparison/reference-supplement.js';

const mapping: SupplementMappings = { schema: 'pc1-reference-mappings/v1',
  languages: { Common: 'common', Shadowtongue: 'shadowtongue', Wildsong: 'wildsong' },
  weapon_groups: { Crossbow: 'crossbow', Pick: 'pick' }, wildsong_existing_record: 'classfeatures:RiAGlnnp4S21BAG3' };
const language = (name: string, speakers: unknown = ['{@ancestry Human|PC1|Humans}']) => ({ name, source: 'PC1', remaster: true, page: 89,
  typicalSpeakers: speakers, entries: ['Long unrelated regional lore must not be imported.'] });
const group = (name: string, specialization: unknown = ['The target takes {@damage 1d8} bleed damage.']) => ({
  name, source: 'PC1', remaster: true, type: 'Weapon', page: 283, specialization,
});

test('exact PC1 selection retains context and excludes legacy, Wildsong and unrelated lore', () => {
  const payload = extractReferenceSupplement({ language: [language('Common'), language('Shadowtongue', ['Nidalese', 'Netherworld creatures']),
    language('Wildsong', ['{@unsupported Not a selected payload}']), { ...language('Common'), source: 'CRB', remaster: false },
    { name: 'Regional', source: 'PC1', remaster: true, page: 89, entries: ['Unselected lore'] }] },
  { group: [group('Crossbow'), group('Crossbow'), group('Pick'), { ...group('Crossbow'), source: 'CRB', remaster: false }] }, mapping);
  assert.equal(payload.entries.length, 4);
  assert.deepEqual(payload.entries[0].speakers, ['Humans']);
  assert.deepEqual(payload.entries[1].grants, ['ORC', 'CUP']);
  assert.equal(payload.entries[2].markup, '<p>The target takes 1d8 bleed damage.</p>');
  assert.ok(!JSON.stringify(payload).includes('regional lore'));
  assert.ok(!payload.entries.some(entry => entry.identifier === 'wildsong'));
  assert.deepEqual(payload.entries.map(entry => entry.identifier), ['common', 'shadowtongue', 'crossbow', 'pick']);
});

test('selection rejects conflicting duplicates, unknown fields/tags and malformed reachable payloads', () => {
  assert.throws(() => extractReferenceSupplement({ language: [language('Common'), language('Common')] }, { group: [] }, mapping), /Conflicting/);
  assert.throws(() => extractReferenceSupplement({ language: [language('Unreviewed')] }, { group: [] }, mapping), /Unreviewed/);
  assert.throws(() => extractReferenceSupplement({ language: [language('Common', [['nested']])] }, { group: [] }, mapping), /flat/);
  assert.throws(() => extractReferenceSupplement({ language: [language('Common', ['{@spell unsupported}'])] }, { group: [] }, mapping), /Unsupported.*tag/);
  assert.throws(() => extractReferenceSupplement({ language: [] }, { group: [group('Crossbow'), group('Crossbow', ['Different'])] }, mapping), /Conflicting/);
  assert.throws(() => extractReferenceSupplement({ language: [] }, { group: [group('Pick'), group('Pick')] }, mapping), /unreviewed duplicate/);
  assert.throws(() => extractReferenceSupplement({ language: [] }, { group: [group('Pick', ['{@spell unapproved}'])] }, mapping), /Unsupported.*tag/);
  assert.throws(() => extractReferenceSupplement({ language: [] }, { group: [{ ...group('Pick'), page: 12 }] }, mapping), /source identity/);
  assert.throws(() => sourceTagText('{@damage {@condition nested}}', new Set(['damage', 'condition'])), /Nested/);
  assert.throws(() => sourceTagText('{@damage unterminated', new Set(['damage'])), /malformed/);
});

test('known tag text is literal and weapon prose escapes HTML while preserving quotes', () => {
  assert.equal(sourceTagText('{@class Druid|PC1|Druids}', new Set(['class'])), 'Druids');
  assert.equal(sourceTagText('{@trait Fey||}', new Set(['trait'])), 'Fey');
  const payload = extractReferenceSupplement({ language: [] }, { group: [group('Pick', ['{@damage 1d8} & < > "quoted"'])] }, mapping);
  assert.equal(payload.entries[0].markup, '<p>1d8 &amp; &lt; &gt; "quoted"</p>');
});

test('candidate diffs retain exact text, mapping/removal and notice evidence independently', () => {
  const before = extractReferenceSupplement({ language: [language('Common')] }, { group: [group('Crossbow')] }, mapping);
  const after = extractReferenceSupplement({ language: [language('Common', ['Changed speaker'])] }, { group: [] }, mapping);
  const diff = referenceSupplementDiff(before, after,
    { files: { 'notices.json': 'old-notice', 'payload.json': 'old-payload' }, mapping_recipe: 'mapping/v1', selection: { wildsong_existing_record: 'old' } },
    { files: { 'notices.json': 'new-notice', 'payload.json': 'new-payload' }, mapping_recipe: 'mapping/v2', selection: { wildsong_existing_record: 'new' } });
  assert.deepEqual(diff.removed.map(entry => entry.key), ['weapon_group:crossbow']);
  assert.deepEqual(diff.text_changes[0].before.speakers, ['Humans']);
  assert.deepEqual(diff.text_changes[0].after.speakers, ['Changed speaker']);
  assert.equal(diff.mapping.before_recipe, 'mapping/v1');
  assert.deepEqual(diff.mapping.removed_targets, ['weapon_group:crossbow']);
  assert.deepEqual(diff.mapping.after_selection, { wildsong_existing_record: 'new' });
  assert.deepEqual(diff.notices, [{ file: 'notices.json', before_sha256: 'old-notice', after_sha256: 'new-notice' }]);
});

test('maintainer candidate writes a new review artifact without changing approved inputs', async t => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'atlas-reference-candidate-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const payload = extractReferenceSupplement({ language: [language('Common')] }, { group: [group('Crossbow')] }, mapping);
  const languages = path.join(root, 'languages.json'), groups = path.join(root, 'groups.json');
  await writeFile(languages, JSON.stringify({ language: [language('Common')] }));
  await writeFile(groups, JSON.stringify({ group: [group('Crossbow'), group('Crossbow')] }));
  const original = await readFile(path.join(approvedReferenceData, 'manifest.json'));
  const args = { languages, groups, sourceCommit: 'b'.repeat(40), out: path.join(root, 'candidate') };
  const result = await prepareReferenceSupplementCandidate(args);
  assert.equal(result.entries, 2);
  assert.deepEqual(await readFile(path.join(approvedReferenceData, 'manifest.json')), original);
  assert.deepEqual(JSON.parse(await readFile(path.join(args.out, 'payload.json'), 'utf8')), payload);
  assert.deepEqual(await readFile(path.join(args.out, 'orc-license.pdf')), await readFile(path.join(approvedReferenceData, 'orc-license.pdf')));
  const selected = JSON.parse(await readFile(path.join(args.out, 'manifest.json'), 'utf8'));
  assert.equal(selected.source_commit, 'b'.repeat(40));
  assert.match(selected.source_files['data/languages.json'].sha256, /^[a-f0-9]{64}$/);
  assert.ok((await readdir(args.out)).includes('changes.json'));
  await assert.rejects(prepareReferenceSupplementCandidate(args), /already exists/);
  await assert.rejects(prepareReferenceSupplementCandidate({ ...args, out: path.join(approvedReferenceData, 'nested') }), /separate/);
  if (process.platform !== 'win32') {
    const link = path.join(root, 'linked-approved'); await symlink(approvedReferenceData, link);
    await assert.rejects(prepareReferenceSupplementCandidate({ ...args, out: path.join(link, 'escaped') }), /separate/);
  }
});
