import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

import type { Catalog, SourceIdentity, TraitCatalog } from '../contracts.js';
import { traitDefinitions } from '../generation/trait-definitions.js';
import { renderTraitChangeReport } from './trait-change-report.js';

export interface TraitSnapshot {
  source: SourceIdentity;
  /** Hashes bind this report to the actual extraction artifacts read. */
  evidence: { summarySha256: string; catalogSha256: string };
  catalog: TraitCatalog;
}
export interface ComparedTrait {
  identifier: string;
  labelKey: string | null;
  descriptionKey: string | null;
  label: string | null;
  description: string | null;
  hasDescription: boolean;
  catalogs: string[];
}
type ComparedField = Exclude<keyof ComparedTrait, 'identifier' | 'hasDescription'>;
const fields: ComparedField[] = [
  'labelKey', 'descriptionKey', 'label', 'description', 'catalogs',
];
export interface TraitChangeReport {
  format: 'atlas-trait-changes/v1';
  before: SnapshotSummary;
  after: SnapshotSummary;
  counts: { added: number; removed: number; changed: number };
  added: ComparedTrait[];
  removed: ComparedTrait[];
  changed: { identifier: string; fields: ComparedField[]; before: ComparedTrait; after: ComparedTrait }[];
}
interface SnapshotSummary {
  source: SourceIdentity;
  evidence: TraitSnapshot['evidence'];
  traits: number;
  descriptions: number;
  missingDescriptions: number;
  catalogMemberships: number;
  catalogs: { name: string; traits: number }[];
}
const hash = (text: string) => createHash('sha256').update(text).digest('hex');
const compare = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0;
const isObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);
const nullableString = (value: unknown) => value === null || typeof value === 'string';
const nonemptyString = (value: unknown) => typeof value === 'string' && value.length > 0;
const sha256 = (value: unknown) => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
function requireInput(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function location(value: unknown) {
  return isObject(value) && nonemptyString(value.file)
    && Number.isInteger(value.line) && Number(value.line) > 0
    && Number.isInteger(value.column) && Number(value.column) > 0;
}

/** Read only the standard extraction products; never follow artifact-provided paths. */
export async function readTraitSnapshot(directory: string): Promise<TraitSnapshot> {
  const [summaryBytes, catalogBytes] = await Promise.all([
    readFile(path.join(directory, 'summary.json'), 'utf8'),
    readFile(path.join(directory, 'trait-catalog.json'), 'utf8'),
  ]);
  const summary: unknown = JSON.parse(summaryBytes);
  const catalog: unknown = JSON.parse(catalogBytes);
  requireInput(isObject(summary) && summary.format === 'atlas-source-extraction/v1'
    && summary.trait_catalog_complete === true, `${directory}: complete trait extraction summary required`);
  const source = summary.source;
  requireInput(isObject(source) && nullableString(source.system_version) && sha256(source.source_digest)
    && Number.isInteger(source.input_file_count) && Number(source.input_file_count) > 0
    && (source.git_commit === null || (typeof source.git_commit === 'string' && /^[a-f0-9]{40}$/.test(source.git_commit)))
    && (source.git_clean === null || typeof source.git_clean === 'boolean'), `${directory}: invalid source identity`);
  requireInput(isObject(catalog) && catalog.complete === true && Array.isArray(catalog.catalogs)
    && Array.isArray(catalog.descriptions) && Array.isArray(catalog.diagnostics), `${directory}: complete trait catalog required`);
  const names = new Set<string>();
  for (const group of catalog.catalogs) {
    requireInput(isObject(group) && nonemptyString(group.name) && !names.has(String(group.name))
      && ['trait', 'otherTag', 'rarity'].includes(String(group.namespace)) && group.complete === true
      && location(group.source) && Array.isArray(group.entries), `${directory}: invalid, duplicate or incomplete catalog`);
    names.add(String(group.name));
    const identifiers = new Set<string>();
    for (const entry of group.entries) {
      requireInput(isObject(entry) && nonemptyString(entry.identifier) && !identifiers.has(String(entry.identifier))
        && nullableString(entry.labelKey) && nullableString(entry.descriptionKey)
        && nullableString(entry.label) && nullableString(entry.description) && location(entry.source),
      `${directory}: invalid or duplicate entry in ${group.name}`);
      identifiers.add(String(entry.identifier));
    }
  }
  requireInput(catalog.catalogs.some(group => group.namespace === 'trait'), `${directory}: trait namespace is absent`);
  requireInput(catalog.diagnostics.every(d => isObject(d) && d.severity === 'warning'), `${directory}: catalog has extraction errors`);
  return {
    source: {
      system_version: source.system_version as string | null, source_digest: source.source_digest as string,
      input_file_count: source.input_file_count as number, git_commit: source.git_commit as string | null,
      git_clean: source.git_clean as boolean | null,
    },
    evidence: { summarySha256: hash(summaryBytes), catalogSha256: hash(catalogBytes) },
    catalog: catalog as unknown as TraitCatalog,
  };
}

function selectedTraits(snapshot: TraitSnapshot): Map<string, ComparedTrait> {
  // Share generation's exact identity, namespace and conflicting-mapping policy.
  const definitions = traitDefinitions(snapshot.catalog);
  const entries = new Map<string, Catalog['entries'][number]>();
  for (const group of snapshot.catalog.catalogs.filter(group => group.namespace === 'trait')) {
    for (const entry of group.entries) {
      const previous = entries.get(entry.identifier);
      requireInput(!previous || (previous.label === entry.label && previous.description === entry.description),
        `Conflicting resolved text for trait ${entry.identifier}`);
      entries.set(entry.identifier, entry);
    }
  }
  return new Map(definitions.map(definition => {
    const entry = entries.get(definition.identifier)!;
    return [definition.identifier, {
      identifier: definition.identifier, labelKey: definition.labelKey, descriptionKey: definition.descriptionKey,
      label: entry.label, description: entry.description,
      hasDescription: Boolean(entry.description?.trim()), catalogs: definition.catalogs,
    }];
  }));
}
function summarize(snapshot: TraitSnapshot, traits: Map<string, ComparedTrait>): SnapshotSummary {
  const descriptions = [...traits.values()].filter(trait => trait.hasDescription).length;
  return {
    source: snapshot.source, evidence: snapshot.evidence, traits: traits.size,
    descriptions, missingDescriptions: traits.size - descriptions,
    catalogMemberships: [...traits.values()].reduce((count, trait) => count + trait.catalogs.length, 0),
    catalogs: snapshot.catalog.catalogs.filter(group => group.namespace === 'trait')
      .map(group => ({ name: group.name, traits: group.entries.length }))
      .sort((a, b) => compare(a.name, b.name)),
  };
}
export function compareTraitSnapshots(before: TraitSnapshot, after: TraitSnapshot): TraitChangeReport {
  const oldTraits = selectedTraits(before), newTraits = selectedTraits(after);
  const added = [...newTraits.values()].filter(trait => !oldTraits.has(trait.identifier));
  const removed = [...oldTraits.values()].filter(trait => !newTraits.has(trait.identifier));
  const changed: TraitChangeReport['changed'] = [];
  for (const [identifier, current] of newTraits) {
    const previous = oldTraits.get(identifier);
    if (!previous) continue;
    const changedFields = fields.filter(field => JSON.stringify(previous[field]) !== JSON.stringify(current[field]));
    if (changedFields.length) changed.push({ identifier, fields: changedFields, before: previous, after: current });
  }
  return {
    format: 'atlas-trait-changes/v1', before: summarize(before, oldTraits), after: summarize(after, newTraits),
    counts: { added: added.length, removed: removed.length, changed: changed.length }, added, removed, changed,
  };
}
export async function compareTraits(options: { before: string; after: string; out: string }): Promise<TraitChangeReport> {
  const [before, after] = await Promise.all([readTraitSnapshot(options.before), readTraitSnapshot(options.after)]);
  const report = compareTraitSnapshots(before, after);
  const markdown = renderTraitChangeReport(report);
  // Validate both complete inputs before touching an existing output report.
  await mkdir(options.out, { recursive: true });
  await writeFile(path.join(options.out, 'trait-changes.json'), `${JSON.stringify(report, null, 2)}\n`);
  await writeFile(path.join(options.out, 'trait-changes.md'), markdown);
  return report;
}
