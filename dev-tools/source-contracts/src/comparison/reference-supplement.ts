import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, readdir, realpath, rename, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { contains, packageRoot, repositoryRoot } from '../generation/pinned-source.js';

export const approvedReferenceData = path.join(repositoryRoot, 'crates/atlas-ingest/reference-data');

export interface SupplementMappings {
  schema: 'pc1-reference-mappings/v1'; languages: Record<string, string>;
  weapon_groups: Record<string, string>; wildsong_existing_record: string;
}
export interface SupplementEntry {
  namespace: 'language' | 'weapon_group'; identifier: string; source_field: 'typicalSpeakers' | 'specialization';
  publication: 'Pathfinder Player Core'; page: number; content_locale: 'en'; grants: string[];
  speakers: string[]; markup: string | null;
}
export interface SupplementPayload { schema: 1; entries: SupplementEntry[] }
const sha = (value: string | Uint8Array) => createHash('sha256').update(value).digest('hex');
const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0;
const object = (value: unknown): Record<string, unknown> => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Expected supplemental source object');
  return value as Record<string, unknown>;
};
const stringArray = (value: unknown): string[] => {
  if (!Array.isArray(value) || !value.length || !value.every(entry => typeof entry === 'string'))
    throw new Error('Expected a nonempty flat supplemental string array');
  return value;
};
const key = (entry: SupplementEntry) => `${entry.namespace}:${entry.identifier}`;
export function sourceTagText(text: string, allowed: ReadonlySet<string>): string {
  const result = text.replace(/\{@([A-Za-z][A-Za-z0-9]*)\s+([^{}]*)\}/g, (_match, tag: string, argument: string) => {
    if (!allowed.has(tag)) throw new Error('Unsupported selected supplemental tag: ' + tag);
    const parts = argument.split('|');
    if (!parts[0].trim() || parts.length > 3) throw new Error('Unsupported selected supplemental tag arguments');
    return parts[2] || parts[0];
  });
  if (result.includes('{@')) throw new Error('Nested or malformed selected supplemental tag');
  return result;
}
const escapeHtml = (value: string) => value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');

/** One finite PC1 field selection. No lore, whole-dataset export or source evaluation. */
export function extractReferenceSupplement(languages: unknown, groups: unknown, mapping: SupplementMappings): SupplementPayload {
  if (mapping.schema !== 'pc1-reference-mappings/v1') throw new Error('Unsupported supplemental mapping recipe');
  const languageRows = object(languages).language, groupRows = object(groups).group;
  if (!Array.isArray(languageRows) || !Array.isArray(groupRows)) throw new Error('Missing supplemental source arrays');
  const entries: SupplementEntry[] = [];
  const languagesSeen = new Set<string>();
  for (const input of languageRows) {
    const row = object(input);
    if (row.source !== 'PC1' || row.remaster !== true || !Object.hasOwn(row, 'typicalSpeakers')) continue;
    if (row.name === 'Wildsong') continue; // Its existing Foundry class feature is authoritative.
    if (typeof row.name !== 'string' || !Object.hasOwn(mapping.languages, row.name))
      throw new Error('Unreviewed PC1 language context mapping: ' + String(row.name));
    if (row.page !== 89 || languagesSeen.has(row.name)) throw new Error('Conflicting or unexpected PC1 language source identity');
    languagesSeen.add(row.name);
    const identifier = mapping.languages[row.name];
    entries.push({ namespace: 'language', identifier, source_field: 'typicalSpeakers', publication: 'Pathfinder Player Core',
      page: 89, content_locale: 'en', grants: row.name === 'Shadowtongue' ? ['ORC', 'CUP'] : ['ORC'],
      speakers: stringArray(row.typicalSpeakers).map(text => sourceTagText(text, new Set(['ancestry', 'class', 'trait', 'versatileHeritage']))), markup: null });
  }
  for (const [name, identifier] of Object.entries(mapping.weapon_groups)) {
    const selected = groupRows.map(object).filter(row => row.name === name && row.source === 'PC1' && row.remaster === true && row.type === 'Weapon');
    if (!selected.length) continue; // Explicit removal appears in the candidate diff, never a retained old body.
    if (selected.some(row => row.page !== 283)) throw new Error('Unexpected PC1 weapon group source identity');
    const bodies = selected.map(row => stringArray(row.specialization));
    if (selected.length > 1 && name !== 'Crossbow' || new Set(bodies.map(body => JSON.stringify(body))).size !== 1)
      throw new Error('Conflicting or unreviewed duplicate specialization: ' + name);
    const paragraphs = bodies[0].map(text => sourceTagText(text, new Set(['damage', 'condition'])));
    entries.push({ namespace: 'weapon_group', identifier, source_field: 'specialization', publication: 'Pathfinder Player Core',
      page: 283, content_locale: 'en', grants: ['ORC'], speakers: [], markup: paragraphs.map(text => '<p>' + escapeHtml(text) + '</p>').join('') });
  }
  entries.sort((a, b) => compare(a.namespace, b.namespace) || compare(a.identifier, b.identifier));
  if (new Set(entries.map(key)).size !== entries.length) throw new Error('Duplicate supplemental token mapping');
  return { schema: 1, entries };
}

export function referenceSupplementDiff(before: SupplementPayload, after: SupplementPayload,
  beforeManifest: Record<string, unknown>, afterManifest: Record<string, unknown>) {
  const prior = new Map(before.entries.map(entry => [key(entry), entry]));
  const next = new Map(after.entries.map(entry => [key(entry), entry]));
  const added = [...next.keys()].filter(id => !prior.has(id)).sort(compare);
  const removed = [...prior.keys()].filter(id => !next.has(id)).sort(compare);
  const changed = [...next.keys()].filter(id => prior.has(id) && JSON.stringify(prior.get(id)) !== JSON.stringify(next.get(id))).sort(compare);
  const beforeFiles = object(beforeManifest.files), afterFiles = object(afterManifest.files);
  const noticeFiles = [...new Set([...Object.keys(beforeFiles), ...Object.keys(afterFiles)])].filter(file => file !== 'payload.json').sort(compare);
  return {
    schema: 'reference-supplement-diff/v1', approval: 'candidate only; no content pin or product asset is updated',
    added: added.map(id => ({ key: id, after: next.get(id)! })),
    removed: removed.map(id => ({ key: id, before: prior.get(id)! })),
    text_changes: changed.map(id => ({ key: id, before: prior.get(id)!, after: next.get(id)! })),
    mapping: { before_recipe: beforeManifest.mapping_recipe, after_recipe: afterManifest.mapping_recipe,
      added_targets: added, removed_targets: removed,
      before_selection: beforeManifest.selection, after_selection: afterManifest.selection },
    notices: noticeFiles.filter(file => beforeFiles[file] !== afterFiles[file]).map(file => ({
      file, before_sha256: beforeFiles[file] ?? null, after_sha256: afterFiles[file] ?? null,
    })),
    permission_recipe: { before: beforeManifest.permission_recipe, after: afterManifest.permission_recipe },
    source: { before_commit: beforeManifest.source_commit, after_commit: afterManifest.source_commit,
      before_files: beforeManifest.source_files, after_files: afterManifest.source_files },
  };
}

export async function prepareReferenceSupplementCandidate(args: {
  languages: string; groups: string; sourceCommit: string; out: string;
}) {
  if (!/^[a-f0-9]{40}$/.test(args.sourceCommit)) throw new Error('An explicit immutable source commit is required');
  // Resolve existing ancestors so an output-directory symlink cannot bypass ownership checks.
  const resolveOutput = async (candidate: string): Promise<string> => {
    try { return await realpath(candidate); } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      const parent = path.dirname(candidate);
      if (parent === candidate) throw error;
      return path.join(await resolveOutput(parent), path.basename(candidate));
    }
  };
  const physicalOut = await resolveOutput(path.resolve(args.out));
  for (const input of [approvedReferenceData, args.languages, args.groups]) {
    const physicalInput = await realpath(input);
    if (contains(physicalOut, physicalInput) || contains(physicalInput, physicalOut))
      throw new Error('Candidate output must be separate from inputs');
  }
  for (const owned of ['crates', 'dev-tools', 'scripts', 'docs', '.git'])
    if (contains(path.join(repositoryRoot, owned), physicalOut)) throw new Error('Candidate output must be outside tracked product/tool directories');
  const existing = await readdir(args.out).catch((error: NodeJS.ErrnoException) => {
    if (error.code === 'ENOENT') return null; throw error;
  });
  if (existing !== null) throw new Error('Candidate output already exists; use a new review directory');
  const languageBytes = await readFile(args.languages), groupBytes = await readFile(args.groups);
  const mappings = JSON.parse(await readFile(path.join(packageRoot, 'fixtures/reference-supplement-mappings.json'), 'utf8')) as SupplementMappings;
  const payload = extractReferenceSupplement(JSON.parse(languageBytes.toString()), JSON.parse(groupBytes.toString()), mappings);
  const beforeManifest = object(JSON.parse(await readFile(path.join(approvedReferenceData, 'manifest.json'), 'utf8')));
  const beforePayload = JSON.parse(await readFile(path.join(approvedReferenceData, 'payload.json'), 'utf8')) as SupplementPayload;
  const manifest = { ...beforeManifest };
  if (manifest.schema !== 1 || manifest.recipe !== 'pc1-reference-subset/v1'
    || manifest.source_repository !== 'https://github.com/Pf2eToolsOrg/Pf2eTools') throw new Error('Unsupported PC1 asset manifest');
  const files = new Map<string, Uint8Array>();
  for (const file of ['notices.json', 'orc-license.pdf', 'community-use-policy.pdf']) files.set(file, await readFile(path.join(approvedReferenceData, file)));
  files.set('payload.json', Buffer.from(JSON.stringify(payload, null, 2) + '\n'));
  manifest.source_commit = args.sourceCommit;
  const sourceFiles: [string, Buffer][] = [['data/languages.json', languageBytes], ['data/groups.json', groupBytes]];
  manifest.source_files = Object.fromEntries(sourceFiles.map(([file, bytes]) => [file, {
    sha256: sha(bytes), url: `https://raw.githubusercontent.com/Pf2eToolsOrg/Pf2eTools/${args.sourceCommit}/${file}`,
  }]));
  manifest.files = Object.fromEntries([...files].sort(([a], [b]) => compare(a, b)).map(([name, bytes]) => [name, sha(bytes)]));
  manifest.selection = { language_contexts: payload.entries.filter(entry => entry.namespace === 'language').length,
    weapon_groups: payload.entries.filter(entry => entry.namespace === 'weapon_group').length, wildsong_existing_record: mappings.wildsong_existing_record };
  const diff = referenceSupplementDiff(beforePayload, payload, beforeManifest, manifest);
  files.set('manifest.json', Buffer.from(JSON.stringify(manifest, null, 2) + '\n'));
  const parent = path.dirname(args.out); await mkdir(parent, { recursive: true });
  const stage = await mkdtemp(path.join(parent, '.reference-candidate-'));
  try {
    for (const [name, bytes] of files) await writeFile(path.join(stage, name), bytes);
    await writeFile(path.join(stage, 'changes.json'), JSON.stringify(diff, null, 2) + '\n');
    await writeFile(path.join(stage, 'changes.md'), `# Reference supplement candidate\n\nCandidate only; review changes.json before approving or committing any source asset.\n\nAdded: ${diff.added.length}; removed: ${diff.removed.length}; text changes: ${diff.text_changes.length}; notice changes: ${diff.notices.length}.\n\nMapping recipes and exact targets, permission recipes, original/new text and source-file hashes are preserved in changes.json.\n`);
    await rename(stage, args.out);
  } finally { await rm(stage, { recursive: true, force: true }); }
  return { entries: payload.entries.length, added: diff.added.length, removed: diff.removed.length,
    textChanges: diff.text_changes.length, noticeChanges: diff.notices.length, out: args.out };
}
