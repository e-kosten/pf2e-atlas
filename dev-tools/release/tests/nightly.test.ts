import { createHash } from 'node:crypto';
import { coverageArchive } from '../src/embedding-coverage.js';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import { nightlyVersion, planNightly, prepareNightly, publishNightly, type Run } from '../src/nightly.js';
import { macosDmg, publicAssetNames, windowsInstaller } from '../src/release-data.js';

const sha = 'abcdef123456' + '0'.repeat(28);
const tag = 'nightly/20261018092300.gabcdef123456';
const packageTag = 'v0.1.0-nightly.20261018092300.gabcdef123456';
const channel = 'nightly/latest';
const datedReleaseId = 123;
const releaseEndpoint = 'repos/{owner}/{repo}/releases';
const uploadUrl = `https://uploads.github.com/repos/e-kosten/pf2e-atlas/releases/${datedReleaseId}/assets`;
const archiveBytes = 'checked CLI archive';
const archiveDigest = createHash('sha256').update(archiveBytes).digest('hex');
const manifestBytes = JSON.stringify({
  version: packageTag, assets: [{ name: coverageArchive, target: 'x86_64-unknown-linux-gnu', variant: 'cli-only', sha256: archiveDigest }],
});
function fixtureBytes(name: string): string {
  if (name === coverageArchive) return archiveBytes;
  if (name === 'atlas-release-manifest.json') return manifestBytes;
  return `assembled ${name}\n`;
}
const publishedAssets = publicAssetNames(packageTag).map((name, index) => ({
  id: index + 1, name, digest: 'sha256:' + createHash('sha256').update(fixtureBytes(name)).digest('hex'),
}));
const dmgAsset = publishedAssets.find((asset) => asset.name === macosDmg)!;
const windowsAsset = publishedAssets.find((asset) => asset.name === windowsInstaller)!;
const pins = { version: 2, profiles: {
  'bge-base': {
    base: { tag: 'embedding-seed/bge-base/v1', asset: 'bge-base.gz', file: { sha256: 'b'.repeat(64), bytes: 1 } },
    delta: { base_sha256: 'b'.repeat(64), asset: {
      tag: 'embedding-seed/bge-base/delta/v1', asset: 'bge-base-delta.gz', file: { sha256: 'd'.repeat(64), bytes: 1 },
    } },
  },
  embeddinggemma: {
    base: { tag: 'embedding-seed/embeddinggemma/v1', asset: 'embeddinggemma.gz', file: { sha256: 'e'.repeat(64), bytes: 1 } },
  },
} };
function fixture(t: TestContext) {
  const root = mkdtempSync(path.join(tmpdir(), 'atlas-nightly-test-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const manifest = path.join(root, 'crates/atlas-cli/Cargo.toml'), lock = path.join(root, 'Cargo.lock');
  mkdirSync(path.dirname(manifest), { recursive: true });
  writeFileSync(manifest, '[package]\nname = "atlas-cli"\nversion = "0.1.0-rc.6"\n\n[dependencies.other]\nversion = "0.1.0-rc.6"\n');
  writeFileSync(lock, 'version = 4\n\n[[package]]\nname = "atlas-cli"\nversion = "0.1.0-rc.6"\n\n[[package]]\nname = "other"\nversion = "0.1.0-rc.6"\n');
  const dist = path.join(root, 'dist'); mkdirSync(dist);
  for (const { name } of publishedAssets) writeFileSync(path.join(dist, name), fixtureBytes(name));
  writeFileSync(path.join(dist, 'dist-manifest.json'), 'private build output');
  const pinPath = path.join(root, 'crates/atlas-runtime/embedding-seeds.json');
  mkdirSync(path.dirname(pinPath), { recursive: true });
  writeFileSync(pinPath, JSON.stringify(pins));
  return { root, manifest, lock, dist };
}

test('nightly preparation changes only CLI versions and is idempotent', (t) => {
  const { root, manifest, lock } = fixture(t);
  prepareNightly(root, packageTag); prepareNightly(root, packageTag);
  assert.equal(readFileSync(manifest, 'utf8'), `[package]\nname = "atlas-cli"\nversion = "${packageTag.slice(1)}"\n\n[dependencies.other]\nversion = "0.1.0-rc.6"\n`);
  assert.equal(readFileSync(lock, 'utf8'), `version = 4\n\n[[package]]\nname = "atlas-cli"\nversion = "${packageTag.slice(1)}"\n\n[[package]]\nname = "other"\nversion = "0.1.0-rc.6"\n`);
});

test('invalid versions and inconsistent locks fail before changing either file', (t) => {
  const { root, manifest, lock } = fixture(t);
  for (const invalid of ['nightly', 'v0.1.0', packageTag + '\n', packageTag.replace('0.1.0', '9.0.0')]) {
    const before = [readFileSync(manifest, 'utf8'), readFileSync(lock, 'utf8')];
    assert.throws(() => prepareNightly(root, invalid));
    assert.deepEqual([readFileSync(manifest, 'utf8'), readFileSync(lock, 'utf8')], before);
  }
  writeFileSync(lock, readFileSync(lock, 'utf8').replace('name = "atlas-cli"', 'name = "missing"'));
  assert.throws(() => prepareNightly(root, packageTag), /one CLI lockfile entry/);
  assert.match(readFileSync(manifest, 'utf8'), /0.1.0-rc.6/);
});

test('ambiguous CLI packages and mismatched versions preserve both input files', (t) => {
  const { root, manifest, lock } = fixture(t);
  const original = { manifest: readFileSync(manifest, 'utf8'), lock: readFileSync(lock, 'utf8') };
  for (const [manifestText, lockText] of [
    [original.manifest, original.lock.replace('0.1.0-rc.6', '0.0.9')],
    [original.manifest, original.lock + '\n[[package]]\nname = "atlas-cli"\nversion = "0.0.9"\n'],
    [original.manifest + '\n[package]\nname = "atlas-cli"\nversion = "0.0.9"\n', original.lock],
  ]) {
    writeFileSync(manifest, manifestText);
    writeFileSync(lock, lockText);
    assert.throws(() => prepareNightly(root, packageTag));
    assert.equal(readFileSync(manifest, 'utf8'), manifestText);
    assert.equal(readFileSync(lock, 'utf8'), lockText);
  }
});

test('scheduled skip requires the published channel to point to this exact source', (t) => {
  const { root } = fixture(t);
  for (const scenario of ['same', 'changed', 'draft', 'missing-channel', 'missing-pointer', 'missing-dmg', 'missing-windows', 'manual']) {
    const dated = { tag_name: tag, target_commitish: scenario === 'changed' ? '1'.repeat(40) : sha, draft: scenario === 'draft', prerelease: true,
      assets: [dmgAsset, windowsAsset].filter((asset) => !(scenario === 'missing-dmg' && asset === dmgAsset)
        && !(scenario === 'missing-windows' && asset === windowsAsset)) };
    const channelRelease = { tag_name: channel, draft: false, assets: scenario === 'missing-pointer' ? [] : [{ name: 'nightly-version.txt' }] };
    const run: Run = (program, args) => {
      if (program === 'git') return sha;
      if (args[0] === 'api') return JSON.stringify([[dated, ...(scenario === 'missing-channel' ? [] : [channelRelease])]]);
      if (args[1] === 'download') writeFileSync(path.join(args[args.indexOf('--dir') + 1], 'nightly-version.txt'), tag + '\n');
      return '';
    };
    const result = planNightly(root, scenario !== 'manual', run, new Date('2026-10-18T09:23:00Z'));
    assert.equal(result, scenario === 'same' ? 'build=false\n' : `build=true\nsha=${sha}\ntag=${tag}\npackage-tag=${packageTag}\n`);
  }
});

function publisher(t: TestContext, options: { existing?: object[]; failInstall?: boolean; wrongVersion?: boolean; failValidation?: boolean; gap?: boolean; badCoverage?: boolean; remoteFault?: 'extra' | 'duplicate' | 'missing' | 'corrupt' | 'after-publication'; channelTag?: string; staleList?: boolean; listPublishedAsDraft?: boolean } = {}) {
  const { root, dist } = fixture(t);
  const coverage = path.join(root, 'embedding-coverage.json');
  const covered = { status: 'covered', inferred_inputs: 0, reused_inputs: 3 };
  writeFileSync(coverage, JSON.stringify({
    version: 2, source_sha: sha, tag: packageTag, archive_sha256: options.badCoverage ? '0'.repeat(64) : archiveDigest,
    pins, profiles: { 'bge-base': covered, embeddinggemma: options.gap
      ? { status: 'gap', reason: 'missing_input', first_hash: 'a'.repeat(64), reused_inputs: 100, missing_inputs: 1 } : covered },
  }));
  const calls: string[][] = [];
  const releases = [...options.existing ?? []] as { tag_name: string; target_commitish?: string; draft?: boolean; prerelease?: boolean; body?: string; assets?: { id?: number; name?: string; digest?: string }[] }[];
  const listedReleases = [...releases];
  const draftMetadata = { id: datedReleaseId, upload_url: `${uploadUrl}{?name,label}` };
  const run: Run = (program, args) => {
    calls.push([program, ...args]);
    if (program === 'git') return releases.some((release) => release.tag_name === tag && !release.draft) ? `${sha}\trefs/tags/${tag}` : '';
    if (program === 'gh' && args[0] === 'api') {
      if (args.at(-1) === `${releaseEndpoint}/${datedReleaseId}/assets?per_page=100`) {
        const dated = releases.find((item) => item.tag_name === tag)!;
        const assets = [...dated.assets ?? []];
        const sum = assets.find((asset) => asset.name === 'SHA256SUMS');
        if (sum && options.remoteFault) {
          switch (options.remoteFault) {
            case 'extra': assets.push({ id: 1000, name: 'unexpected.txt' }); break;
            case 'duplicate': assets.push(sum); break;
            case 'missing': assets.splice(assets.indexOf(sum), 1); break;
            case 'corrupt': assets[assets.indexOf(sum)] = { ...sum, digest: 'wrong' }; break;
            case 'after-publication': if (!dated.draft) assets.splice(assets.indexOf(sum), 1); break;
          }
        }
        return JSON.stringify([assets.slice(0, 1), assets.slice(1)]);
      }
      if (args[1] === '--paginate') {
        const listed = options.staleList ? listedReleases : releases;
        return JSON.stringify([listed.map((item) => options.listPublishedAsDraft && item.tag_name === tag
          ? { ...item, draft: true } : item)]);
      }
      if (args[1] === `${releaseEndpoint}/${datedReleaseId}`) {
        return JSON.stringify({ ...draftMetadata, ...releases.find((item) => item.tag_name === tag) });
      }
      const [, , method, endpoint] = args;
      if (method === 'POST' && endpoint === releaseEndpoint) {
        assert.ok(args.includes(`tag_name=${tag}`));
        assert.ok(args.includes(`target_commitish=${sha}`));
        assert.ok(args.includes('draft=true'));
        assert.ok(args.includes('prerelease=true'));
        assert.ok(args.includes('make_latest=false'));
        assert.ok(!releases.some((item) => item.tag_name === tag), 'duplicate dated release');
        const release = { ...draftMetadata, tag_name: tag, target_commitish: sha, draft: true, prerelease: true, assets: [] };
        releases.push(release);
        return JSON.stringify(release);
      }
      const draft = releases.find((item) => item.tag_name === tag);
      assert.ok(draft?.draft, 'dated release must be in draft before mutation');
      if (method === 'PATCH' && endpoint === `${releaseEndpoint}/${datedReleaseId}`) {
        if (args.includes('draft=false')) {
          assert.ok(args.includes('prerelease=true'));
          assert.ok(args.includes('make_latest=false'));
          draft.draft = false;
          draft.body = args.find((arg) => arg.startsWith('body='))?.slice(5);
        }
        return JSON.stringify({ ...draftMetadata, ...draft });
      }
      if (method === 'DELETE' && endpoint === `${releaseEndpoint}/assets/${dmgAsset.id}`) {
        draft.assets = draft.assets?.filter((asset) => asset.id !== dmgAsset.id);
        return '';
      }
      assert.equal(method, 'POST');
      const input = args[args.indexOf('--input') + 1];
      const name = path.basename(input);
      assert.equal(endpoint, `${uploadUrl}?name=${name}`);
      assert.equal(path.dirname(input), dist);
      assert.ok(!draft.assets?.some((asset) => asset.name === name), 'duplicate draft asset');
      const asset = publishedAssets.find((item) => item.name === name);
      assert.ok(asset, `unexpected upload: ${name}`);
      assert.equal('sha256:' + createHash('sha256').update(readFileSync(input)).digest('hex'), asset.digest);
      (draft.assets ??= []).push(asset);
      return JSON.stringify(asset);
    }
    if (args[0] === 'release' && args[1] === 'download' && args[2] === channel && options.channelTag) {
      writeFileSync(path.join(args[args.indexOf('--dir') + 1], 'nightly-version.txt'), options.channelTag);
    }
    if (program === 'sh' && options.failInstall) throw new Error('install failed');
    if (path.basename(program) === 'atlas' && args[0] === 'build-info') return JSON.stringify({ status: 'ok', data: { version: nightlyVersion(packageTag), variant: 'cli-only' } });
    if (path.basename(program) === 'atlas') return options.wrongVersion ? 'atlas wrong' : `atlas ${nightlyVersion(packageTag)}`;
    return '';
  };
  const validate = async () => { if (options.failValidation) throw new Error('invalid assets'); };
  return { calls, execute: () => publishNightly(root, tag, sha, dist, coverage, run, validate), root, dist, coverage, run, validate };
}

test('channel advances only after dated publication and successful downloaded install', async (t) => {
  // The list never sees the new draft or published release during this run.
  const { execute, calls } = publisher(t, { staleList: true }); await execute();
  const locate = (...prefix: string[]) => calls.findIndex((call) => prefix.every((value, index) => call[index] === value));
  const datedPublication = locate('gh', 'api', '--method', 'PATCH', `${releaseEndpoint}/${datedReleaseId}`);
  const installation = locate('sh');
  const channelPublication = locate('gh', 'release', 'create', channel);
  assert.notEqual(datedPublication, -1, 'dated release must be published');
  assert.notEqual(installation, -1, 'downloaded installer must run');
  assert.notEqual(channelPublication, -1, 'nightly channel must be published');
  assert.ok(datedPublication < installation);
  assert.ok(installation < channelPublication);
  const cliInstallation = calls.findIndex((call) => call[0] === 'sh' && call.includes('--without-web'));
  assert.ok(cliInstallation > installation && cliInstallation < channelPublication);
  const uploads = calls.filter((call) => call.slice(0, 4).join(' ') === `gh release upload ${channel}`);
  assert.deepEqual(uploads.flatMap((args) => args.slice(4).filter((item) => item !== '--clobber').map((item) => path.basename(item))),
    ['atlas-installer.sh', 'atlas-installer.ps1', 'nightly-version.txt']);
  const datedNames = calls.filter((call) => call[3] === 'POST' && call[4].startsWith(uploadUrl))
    .map((call) => new URL(call[4]).searchParams.get('name'));
  assert.equal(datedNames.length, 21);
  assert.deepEqual(datedNames.sort(), publishedAssets.map(({ name }) => name).sort());
  assert.ok(calls.some((call) => call[4] === `${uploadUrl}?name=${windowsInstaller}`));
  assert.ok(calls.some((call) => call.some((arg) => arg.includes(`/releases/download/${tag}/${windowsInstaller}`))));
  for (const call of calls.filter((call) => call[1] === 'release' && ['create', 'edit'].includes(call[2]))) {
    assert.ok(call.includes('--latest=false')); assert.ok(call.includes('--prerelease'));
  }
});

test('retry resumes the existing draft by ID and replaces its partial uploads', async (t) => {
  const { execute, calls } = publisher(t, { existing: [{
    id: datedReleaseId, tag_name: tag, target_commitish: sha, draft: true, prerelease: true, assets: [dmgAsset],
  }] });
  await execute();
  assert.ok(!calls.some((call) => call[3] === 'POST' && call[4] === releaseEndpoint));
  assert.ok(calls.some((call) => call[3] === 'DELETE' && call[4] === `${releaseEndpoint}/assets/${dmgAsset.id}`));
  assert.ok(calls.some((call) => call[4] === `${releaseEndpoint}/${datedReleaseId}` && call.includes('draft=false')));
});

test('validation, installation and binary-version failures preserve the channel', async (t) => {
  for (const options of [{ failValidation: true }, { failInstall: true }, { wrongVersion: true }]) {
    const { execute, calls } = publisher(t, options);
    await assert.rejects(execute);
    assert.ok(!calls.some((call) => call[3] === channel));
    if ('failValidation' in options) assert.deepEqual(calls, []);
  }
});

test('retry preserves published dated assets while repairing the channel', async (t) => {
  const { execute, calls } = publisher(t, { listPublishedAsDraft: true, existing: [{
    id: datedReleaseId, tag_name: tag, target_commitish: sha, draft: false, prerelease: true,
    assets: publishedAssets,
  }] });
  await execute();
  assert.ok(!calls.some((call) => call[1] === 'api' && call[2] === '--method'));
  assert.ok(calls.some((call) => call[2] === 'upload' && call[3] === channel));
});

test('remote asset verification gates dated publication and pointer advancement', async (t) => {
  for (const remoteFault of ['extra', 'missing', 'duplicate', 'corrupt', 'after-publication'] as const) {
    const bad = publisher(t, { remoteFault });
    await assert.rejects(bad.execute, remoteFault === 'extra' ? /Unexpected assets.*unexpected.txt/ : /digest mismatch.*SHA256SUMS/);
    assert.equal(bad.calls.some((call) => call.includes('draft=false')), remoteFault === 'after-publication');
    assert.ok(!bad.calls.some((call) => call[3] === channel || call[0] === 'sh'));
  }
});

test('unexpected pre-existing nightly attachments block all draft and channel mutations', async (t) => {
  const bad = publisher(t, { existing: [{
    id: datedReleaseId, tag_name: tag, target_commitish: sha, draft: true, prerelease: true,
    assets: [dmgAsset, { id: 1000, name: 'obsolete.sig' }],
  }] });
  await assert.rejects(bad.execute, /Unexpected assets.*obsolete.sig/);
  assert.ok(!bad.calls.some((call) => call.includes('--method') || call[3] === channel));
});

test('older retries preserve the channel and current retries repair its download page', async (t) => {
  const old = publisher(t, { existing: [{ tag_name: channel, assets: [{ name: 'nightly-version.txt' }] }],
    channelTag: tag.replace('20261018092300', '20261019092300') });
  await old.execute();
  assert.ok(!old.calls.some((call) => ['upload', 'edit', 'create', '--method'].includes(call[2])));
  const current = publisher(t, { existing: [
    { tag_name: channel, draft: false, assets: [{ name: 'nightly-version.txt' }] },
    { id: datedReleaseId, tag_name: tag, target_commitish: sha, draft: false, prerelease: true,
      assets: publishedAssets },
  ], channelTag: tag });
  await current.execute();
  assert.ok(!current.calls.some((call) => ['upload', 'create', '--method'].includes(call[2])));
  const edit = current.calls.find((call) => call[2] === 'edit' && call[3] === channel);
  assert.ok(edit);
  const notes = edit[edit.indexOf('--notes') + 1];
  assert.ok(notes.includes(`/download/${tag}/atlas-desktop-aarch64-apple-darwin.dmg`));
  assert.ok(notes.includes(`/download/${tag}/atlas-desktop-x86_64-pc-windows-msvc-setup.exe`));
});

test('an existing dated release without the verified Windows installer cannot advance the channel', async (t) => {
  const retry = publisher(t, { existing: [{
    id: datedReleaseId, tag_name: tag, target_commitish: sha, draft: false, prerelease: true,
    assets: publishedAssets.filter((asset) => asset !== windowsAsset),
  }] });
  await assert.rejects(retry.execute, /digest mismatch.*windows-msvc-setup\.exe/);
  assert.ok(!retry.calls.some((call) => ['upload', 'edit', 'create', '--method'].includes(call[2])));
});

test('source or manifest identity mismatches cannot publish', async (t) => {
  const { root, dist, coverage, run, validate, calls } = publisher(t);
  await assert.rejects(() => publishNightly(root, tag, '1'.repeat(40), dist, coverage, run, validate), /source SHA/);
  writeFileSync(path.join(dist, 'atlas-release-manifest.json'), JSON.stringify({ version: 'v9.9.9' }));
  await assert.rejects(() => publishNightly(root, tag, sha, dist, coverage, run, validate), /manifest version/);
  assert.deepEqual(calls, []);
});


test('nightly discloses the affected profile before install instructions', async (t) => {
  const { execute, calls } = publisher(t, { gap: true });
  await execute();
  const create = calls.find((call) => call[3] === 'POST' && call[4] === releaseEndpoint);
  assert.ok(create);
  const notes = create.find((argument) => argument.startsWith('body='))!;
  assert.match(notes, /\*\*EmbeddingGemma\*\*/);
  assert.match(notes, /--build-embeddings-locally/);
  assert.ok(notes.indexOf('Embedding coverage gap') < notes.indexOf('CLI installation instructions'));
});

test('bad report and missing notice on a published nightly block the channel', async (t) => {
  const invalid = publisher(t, { badCoverage: true });
  await assert.rejects(invalid.execute, /archive differs/);
  assert.deepEqual(invalid.calls, []);
  const retry = publisher(t, { gap: true, existing: [{
    id: datedReleaseId, tag_name: tag, target_commitish: sha, draft: false, prerelease: true, body: 'Not a stable release',
    assets: publishedAssets,
  }] });
  await assert.rejects(retry.execute, /notes differ/);
  assert.ok(!retry.calls.some((call) => call[3] === channel));
});
