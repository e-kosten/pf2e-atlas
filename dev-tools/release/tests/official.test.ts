import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { publishOfficial } from '../src/official.js';
import { macosDmg, windowsInstaller } from '../src/release-data.js';

const sha = 'a'.repeat(40);
const image = 'checked DMG';
const digest = 'sha256:' + createHash('sha256').update(image).digest('hex');

const installer = 'checked Windows installer';
const installerDigest = 'sha256:' + createHash('sha256').update(installer).digest('hex');

function fixture(t: TestContext, tag: string, options: {
  draft?: boolean;
  source?: string;
  digest?: string;
  windowsDigest?: string;
  body?: string;
} = {}) {
  const dist = mkdtempSync(path.join(tmpdir(), 'atlas-official-test-'));
  t.after(() => rmSync(dist, { recursive: true, force: true }));
  writeFileSync(path.join(dist, 'atlas-release-manifest.json'), JSON.stringify({ version: tag.split('/')[1] }));
  writeFileSync(path.join(dist, macosDmg), image);
  writeFileSync(path.join(dist, windowsInstaller), installer);
  const release = {
    tag_name: tag, draft: options.draft ?? true, prerelease: tag.includes('-'), body: options.body ?? 'Release notes',
    assets: [
      { name: macosDmg, digest: options.digest ?? digest },
      { name: windowsInstaller, digest: options.windowsDigest ?? installerDigest },
    ],
  };
  const mutations: string[][] = [];
  return { mutations, publish: () => publishOfficial(tag, sha, dist, (program, args) => {
    if (program === 'git') return `${'b'.repeat(40)}\trefs/tags/${tag}\n${options.source ?? sha}\trefs/tags/${tag}^{}`;
    if (args[0] === 'api') return JSON.stringify([[release]]);
    mutations.push(args);
    return '';
  }) };
}

test('publishes an exact draft with correct stable and RC flags and preserves notes', async (t) => {
  for (const tag of ['release/v1.2.3', 'rc/v1.2.3-rc.2']) {
    const { publish, mutations } = fixture(t, tag);
    await publish();
    const [upload, publishDraft] = mutations;
    assert.deepEqual(upload.slice(0, 3), ['release', 'upload', tag]);
    assert.ok(upload.some((argument) => path.basename(argument) === windowsInstaller));
    assert.ok(publishDraft.includes(`--prerelease=${tag.includes('-')}`));
    assert.ok(publishDraft.includes(`--latest=${!tag.includes('-')}`));
    const notes = publishDraft.at(-1)!;
    assert.ok(notes.startsWith('Release notes\n'));
    assert.ok(notes.includes(`/releases/download/${tag}/${macosDmg}`));
    assert.ok(notes.includes(`/releases/download/${tag}/${windowsInstaller}`));

    const retry = fixture(t, tag, { body: notes });
    await retry.publish();
    assert.equal(retry.mutations[1].includes('--notes'), false);
  }
});

test('rejects an incorrect tag and keeps a draft private on uploaded digest mismatch', async (t) => {
  const wrongTag = fixture(t, 'release/v1.2.3', { source: 'c'.repeat(40) });
  await assert.rejects(wrongTag.publish, /does not point to build commit/);
  assert.equal(wrongTag.mutations.length, 0);
  const badUpload = fixture(t, 'release/v1.2.3', { digest: 'wrong' });
  await assert.rejects(badUpload.publish, /Uploaded DMG digest mismatch/);
  assert.equal(badUpload.mutations.length, 1);
  const badWindowsUpload = fixture(t, 'rc/v1.2.3-rc.2', { windowsDigest: 'wrong' });
  await assert.rejects(badWindowsUpload.publish, /Uploaded Windows installer digest mismatch/);
  assert.equal(badWindowsUpload.mutations.length, 1);
});

test('published retries verify the original desktop assets without mutating the release', async (t) => {
  const existing = fixture(t, 'release/v1.2.3', { draft: false });
  await existing.publish();
  assert.equal(existing.mutations.length, 0);
  const changed = fixture(t, 'release/v1.2.3', { draft: false, windowsDigest: 'different' });
  await assert.rejects(changed.publish, /Uploaded Windows installer digest mismatch/);
  assert.equal(changed.mutations.length, 0);
});
