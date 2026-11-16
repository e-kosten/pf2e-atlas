import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { publishOfficial } from '../src/official.js';
import { macosDmg, windowsInstaller } from '../src/release-data.js';
import { releaseEndpoint } from '../src/release-upload.js';

const sha = 'a'.repeat(40);
const archive = 'atlas-cli-aarch64-apple-darwin.tar.xz';
const script = 'atlas-installer.sh';
const manifest = 'atlas-release-manifest.json';
const sums = 'SHA256SUMS';
const names = [archive, script, manifest, sums, macosDmg, windowsInstaller].sort();
const id = 123;
const endpoint = `${releaseEndpoint}/${id}`;
const upload = `https://uploads.github.com/repos/e-kosten/pf2e-atlas/releases/${id}/assets`;

function fixture(t: TestContext, tag: string, options: {
  draft?: boolean;
  source?: string;
  wrongId?: boolean;
  link?: string;
  body?: string;
  omit?: string;
  corrupt?: string;
  duplicate?: string;
} = {}) {
  const dist = mkdtempSync(path.join(tmpdir(), 'atlas-official-test-'));
  t.after(() => rmSync(dist, { recursive: true, force: true }));
  const assets = names.map((name, index) => {
    const contents = name === manifest ? JSON.stringify({ version: tag.split('/')[1] }) : `assembled ${name}`;
    writeFileSync(path.join(dist, name), contents);
    return { id: index + 1, name, digest: 'sha256:' + createHash('sha256').update(contents).digest('hex') };
  });
  if (options.link) {
    const target = path.join(dist, 'target');
    writeFileSync(target, readFileSync(path.join(dist, options.link)));
    rmSync(path.join(dist, options.link));
    symlinkSync(target, path.join(dist, options.link));
  }
  const draft = options.draft ?? true;
  const release = {
    id, tag_name: tag, draft, prerelease: tag.startsWith('rc/'), body: options.body ?? 'Release notes',
    upload_url: upload + '{?name,label}',
  };
  const remote = draft ? [assets[0]] : [...assets];
  const mutations: string[][] = [];
  return { mutations, publish: () => publishOfficial(tag, sha, dist, (program, args) => {
    if (program === 'git') return `${'b'.repeat(40)}\trefs/tags/${tag}\n${options.source ?? sha}\trefs/tags/${tag}^{}`;
    if (args[0] === 'api' && args[1] === '--paginate') {
      return JSON.stringify([[mutations.length ? { ...release, id: id + 1 } : release]]);
    }
    if (args[0] === 'api' && args[1] === endpoint) {
      const surfaced = remote.filter((asset) => asset.name !== options.omit)
        .map((asset) => asset.name === options.corrupt ? { ...asset, digest: 'wrong' } : asset);
      const duplicate = surfaced.find((asset) => asset.name === options.duplicate);
      if (duplicate) surfaced.push(duplicate);
      return JSON.stringify({ ...release, id: options.wrongId ? id + 1 : id, assets: surfaced });
    }
    mutations.push(args);
    const [, , method, target] = args;
    if (method === 'DELETE') {
      const index = remote.findIndex((asset) => target === `${releaseEndpoint}/assets/${asset.id}`);
      assert.notEqual(index, -1);
      remote.splice(index, 1);
    } else if (method === 'POST') {
      const url = new URL(target);
      assert.equal(url.origin + url.pathname, upload);
      const asset = assets.find((item) => item.name === url.searchParams.get('name'));
      assert.ok(asset);
      assert.ok(!remote.some((item) => item.name === asset.name));
      const input = readFileSync(args.at(-1)!);
      assert.equal('sha256:' + createHash('sha256').update(input).digest('hex'), asset.digest);
      remote.push(asset);
      return JSON.stringify(asset);
    } else {
      assert.equal(method, 'PATCH');
      assert.equal(target, endpoint);
    }
    return '';
  }) };
}

test('repairs and publishes the exact draft with correct stable and RC flags and notes', async (t) => {
  for (const tag of ['release/v1.2.3', 'rc/v1.2.3-rc.2']) {
    const { publish, mutations } = fixture(t, tag);
    await publish();
    assert.equal(mutations.filter((args) => args[2] === 'DELETE').length, 1);
    assert.equal(mutations.filter((args) => args[2] === 'POST').length, names.length);
    const published = mutations.at(-1)!;
    assert.equal(published[2], 'PATCH');
    assert.ok(published.includes('draft=false'));
    assert.ok(published.includes(`prerelease=${tag.startsWith('rc/')}`));
    assert.ok(published.includes(`make_latest=${tag.startsWith('release/')}`));
    const notes = published.at(-1)!.slice('body='.length);
    assert.ok(notes.startsWith('Release notes\n'));
    assert.ok(notes.includes(`/releases/download/${tag}/${macosDmg}`));
    assert.ok(notes.includes(`/releases/download/${tag}/${windowsInstaller}`));

    const retry = fixture(t, tag, { body: notes });
    await retry.publish();
    assert.ok(!retry.mutations.at(-1)!.some((argument) => argument.startsWith('body=')));
  }
});

test('rejects the wrong release ID or source before any mutation', async (t) => {
  const wrongTag = fixture(t, 'release/v1.2.3', { source: 'c'.repeat(40) });
  await assert.rejects(wrongTag.publish, /does not point to build commit/);
  assert.equal(wrongTag.mutations.length, 0);
  const wrongId = fixture(t, 'release/v1.2.3', { wrongId: true });
  await assert.rejects(wrongId.publish, /different ID/);
  assert.equal(wrongId.mutations.length, 0);
});

test('refuses a symbolic link instead of silently omitting the required asset', async (t) => {
  const release = fixture(t, 'release/v1.2.3', { link: archive });
  await assert.rejects(release.publish, { message: `Release asset must be a regular file: ${archive}` });
  assert.equal(release.mutations.length, 0);
});

test('leaves the draft private when any expected asset is missing, changed or duplicated', async (t) => {
  for (const [options, name] of [
    [{ omit: archive }, archive],
    [{ corrupt: script }, script],
    [{ corrupt: sums }, sums],
    [{ duplicate: manifest }, manifest],
  ] as const) {
    const release = fixture(t, 'release/v1.2.3', options);
    await assert.rejects(release.publish, { message: `Uploaded asset missing, duplicated or has a digest mismatch for release/v1.2.3: ${name}` });
    assert.ok(release.mutations.length > 0);
    assert.ok(release.mutations.every((args) => args[2] !== 'PATCH'));
  }
});

test('published retries verify the original assets without mutating the release', async (t) => {
  const existing = fixture(t, 'release/v1.2.3', { draft: false });
  await existing.publish();
  assert.equal(existing.mutations.length, 0);
  const changed = fixture(t, 'release/v1.2.3', { draft: false, corrupt: archive });
  await assert.rejects(changed.publish, { message: `Uploaded asset missing, duplicated or has a digest mismatch for release/v1.2.3: ${archive}` });
  assert.equal(changed.mutations.length, 0);
});
