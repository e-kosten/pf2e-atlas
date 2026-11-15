import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { publishOfficial } from '../src/official.js';
import { macosDmg } from '../src/release-data.js';

const sha = 'a'.repeat(40);
const image = 'checked DMG';
const digest = 'sha256:' + createHash('sha256').update(image).digest('hex');

function fixture(t: TestContext, tag: string, options = { draft: true, source: sha, digest }) {
  const dist = mkdtempSync(path.join(tmpdir(), 'atlas-official-test-'));
  t.after(() => rmSync(dist, { recursive: true, force: true }));
  writeFileSync(path.join(dist, 'atlas-release-manifest.json'), JSON.stringify({ version: tag }));
  writeFileSync(path.join(dist, macosDmg), image);
  const release = {
    tag_name: tag, draft: options.draft, prerelease: tag.includes('-'), body: 'Release notes',
    assets: [{ name: macosDmg, digest: options.digest }],
  };
  const mutations: string[][] = [];
  return { mutations, publish: () => publishOfficial(tag, sha, dist, (program, args) => {
    if (program === 'git') return `${'b'.repeat(40)}\trefs/tags/${tag}\n${options.source}\trefs/tags/${tag}^{}`;
    if (args[0] === 'api') return JSON.stringify([[release]]);
    mutations.push(args);
    return '';
  }) };
}

test('publishes an exact draft with correct stable and RC flags and preserves notes', async (t) => {
  for (const tag of ['v1.2.3', 'v1.2.3-rc.2']) {
    const { publish, mutations } = fixture(t, tag);
    await publish();
    const [upload, publishDraft] = mutations;
    assert.deepEqual(upload.slice(0, 3), ['release', 'upload', tag]);
    assert.ok(publishDraft.includes(`--prerelease=${tag.includes('-')}`));
    assert.ok(publishDraft.includes(`--latest=${!tag.includes('-')}`));
    assert.match(publishDraft.at(-1)!, /Release notes[\s\S]*\.dmg/);
  }
});

test('rejects an incorrect tag and keeps a draft private on uploaded digest mismatch', async (t) => {
  const wrongTag = fixture(t, 'v1.2.3', { draft: true, source: 'c'.repeat(40), digest });
  await assert.rejects(wrongTag.publish, /does not point to build commit/);
  assert.equal(wrongTag.mutations.length, 0);
  const badUpload = fixture(t, 'v1.2.3', { draft: true, source: sha, digest: 'wrong' });
  await assert.rejects(badUpload.publish, /Uploaded DMG digest mismatch/);
  assert.equal(badUpload.mutations.length, 1);
});

test('published retries verify the original DMG without mutating the release', async (t) => {
  const existing = fixture(t, 'v1.2.3', { draft: false, source: sha, digest });
  await existing.publish();
  assert.equal(existing.mutations.length, 0);
  const changed = fixture(t, 'v1.2.3', { draft: false, source: sha, digest: 'different' });
  await assert.rejects(changed.publish, /Uploaded DMG digest mismatch/);
  assert.equal(changed.mutations.length, 0);
});
