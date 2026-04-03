import type { SourceCategory } from "../types.js";
import { normalizeText, uniqueSorted } from "../utils.js";
import type {
  ActorIndexData,
  BuildSourceEntry,
  ItemIndexData,
  NormalizedIndexRecord,
  PackBuildInfo,
  SpellIndexData,
} from "./index-types.js";
import {
  AfflictionFamily,
  detectAfflictionFamily,
  extractLinkedNamesFromMarkup,
  getRecordDescriptionMarkup,
  getRecordDescriptionText,
  getRecordSlug,
  getRecordTraits,
  hasAfflictionShape,
} from "./nested-item-utils.js";

const DERIVED_AFFLICTIONS_PACK_NAME = "derived-afflictions";
const DERIVED_AFFLICTIONS_PACK_LABEL = "Derived Afflictions";
const DERIVED_AFFLICTION_INSTANCES_PACK_NAME = "derived-affliction-instances";
const DERIVED_AFFLICTION_INSTANCES_PACK_LABEL = "Derived Affliction Instances";
const COMPILED_SOURCE_PATTERN = /^Compendium\.pf2e\.([^.]+)\.[^.]+\.([^.]+)$/i;

type IndexedBuildEntry = BuildSourceEntry & { record: NormalizedIndexRecord };

type AfflictionOccurrence = {
  hostRecord: NormalizedIndexRecord;
  sourceRecord: NormalizedIndexRecord | null;
  sourceRaw: Record<string, unknown> | null;
  childRaw: Record<string, unknown>;
  family: AfflictionFamily;
  name: string;
  slug: string | null;
  traits: string[];
  linkedNames: string[];
  compendiumSource: string | null;
  sourcePath: string;
  occurrenceRef: string;
  identityKey: string;
};

export type DerivedBuildEntry = {
  record: NormalizedIndexRecord;
  raw: Record<string, unknown>;
  actorData: ActorIndexData | null;
  itemData: ItemIndexData | null;
  spellData: SpellIndexData | null;
  references: [];
  resolvedReferences: [];
  isSearchCanonical: boolean;
};

export type DerivedReferenceEdgeRow = {
  fromRecordKey: string;
  toRecordKey: string;
  displayText: string | null;
  referenceText: string;
  fromPackName: string;
  fromRecordType: string;
  fromDocumentType: string;
  fromSourceCategory: SourceCategory;
};

export type DerivedAfflictionBuild = {
  records: DerivedBuildEntry[];
  edges: DerivedReferenceEdgeRow[];
};

function hashText(value: string): string {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  return (hash >>> 0).toString(16);
}

function buildDescriptionSnippet(descriptionText: string | null): string | null {
  if (!descriptionText || descriptionText.trim().length === 0) {
    return null;
  }

  const normalized = descriptionText.replace(/\s+/g, " ").trim();
  const sentenceMatch = normalized.match(/^(.{1,240}?[.!?])(?:\s|$)/);
  if (sentenceMatch) {
    return sentenceMatch[1]!.trim();
  }

  if (normalized.length <= 240) {
    return normalized;
  }

  return `${normalized.slice(0, 237).trimEnd()}...`;
}

function parseCompendiumSource(compendiumSource: string | null): { packName: string; id: string } | null {
  if (!compendiumSource) {
    return null;
  }

  const parsed = compendiumSource.match(COMPILED_SOURCE_PATTERN);
  if (!parsed) {
    return null;
  }

  return {
    packName: parsed[1] ?? "",
    id: parsed[2] ?? "",
  };
}

function buildPackAndIdKey(packName: string, id: string): string {
  return `${normalizeText(packName)}:${normalizeText(id)}`;
}

function getCompendiumSource(raw: Record<string, unknown>): string | null {
  const stats = raw._stats;
  if (!stats || typeof stats !== "object") {
    return null;
  }

  const value = (stats as Record<string, unknown>).compendiumSource;
  return typeof value === "string" && value.trim().length > 0 ? value : null;
}

function buildOccurrenceIdentityKey(
  family: AfflictionFamily,
  name: string,
  slug: string | null,
  compendiumSource: string | null,
  sourceRecordKey: string | null,
): string {
  if (sourceRecordKey) {
    return `record:${sourceRecordKey}`;
  }

  if (compendiumSource) {
    return `compendium:${normalizeText(compendiumSource)}`;
  }

  if (slug) {
    return `slug:${family}:${normalizeText(slug)}`;
  }

  return `name:${family}:${normalizeText(name)}`;
}

function collectOccurrenceLinkedNames(raw: Record<string, unknown>): string[] {
  return extractLinkedNamesFromMarkup(getRecordDescriptionMarkup(raw));
}

function buildOccurrenceSearchText(name: string, family: AfflictionFamily, traits: string[], linkedNames: string[]): string {
  return uniqueSorted([
    name,
    family,
    ...traits,
    ...linkedNames,
  ].filter(Boolean)).join("\n");
}

function buildCanonicalSearchText(name: string, family: AfflictionFamily, traits: string[], slug: string | null, linkedNames: string[]): string {
  const slugAlias = slug ? slug.replace(/[-_]+/g, " ") : null;
  return uniqueSorted([
    name,
    family,
    slugAlias,
    ...traits,
    ...linkedNames,
  ].filter((value): value is string => Boolean(value))).join("\n");
}

function toDerivedPackBuildInfo(name: string, label: string): PackBuildInfo {
  return {
    name,
    label,
    documentType: "Item",
    declaredPath: "",
    resolvedPath: "",
  };
}

function buildCanonicalRaw(
  id: string,
  name: string,
  family: AfflictionFamily,
  traits: string[],
  descriptionMarkup: string | null,
  linkedNames: string[],
  representativeInstanceRecordKey: string | null,
  normalizationKey: string,
): Record<string, unknown> {
  return {
    _id: id,
    name,
    type: "affliction",
    system: {
      category: family,
      traits: {
        rarity: "common",
        value: traits,
      },
      description: {
        value: descriptionMarkup ?? "",
      },
    },
    _derived: {
      kind: "canonicalAffliction",
      normalizationKey,
      representativeInstanceRecordKey,
      linkedNames,
    },
  };
}

function buildInstanceRaw(
  id: string,
  occurrence: AfflictionOccurrence,
  canonicalRecordKey: string,
): Record<string, unknown> {
  return {
    ...occurrence.childRaw,
    _id: id,
    _derived: {
      kind: "afflictionInstance",
      hostRecordKey: occurrence.hostRecord.recordKey,
      sourceRecordKey: occurrence.sourceRecord?.recordKey ?? null,
      canonicalRecordKey,
      normalizationKey: occurrence.identityKey,
      occurrenceRef: occurrence.occurrenceRef,
    },
  };
}

function collectTopLevelOccurrence(entry: IndexedBuildEntry): AfflictionOccurrence | null {
  if (entry.record.category === "affliction") {
    return null;
  }

  const family = detectAfflictionFamily(entry.raw);
  if (!family || !hasAfflictionShape(entry.raw)) {
    return null;
  }

  return {
    hostRecord: entry.record,
    sourceRecord: entry.record,
    sourceRaw: entry.raw,
    childRaw: entry.raw,
    family,
    name: entry.record.name,
    slug: getRecordSlug(entry.raw),
    traits: uniqueSorted([family, ...getRecordTraits(entry.raw)]),
    linkedNames: collectOccurrenceLinkedNames(entry.raw),
    compendiumSource: getCompendiumSource(entry.raw),
    sourcePath: `${entry.record.sourcePath}#self`,
    occurrenceRef: "self",
    identityKey: buildOccurrenceIdentityKey(
      family,
      entry.record.name,
      getRecordSlug(entry.raw),
      getCompendiumSource(entry.raw),
      entry.record.recordKey,
    ),
  };
}

function collectEmbeddedOccurrences(
  entry: IndexedBuildEntry,
  recordsByPackAndId: Map<string, IndexedBuildEntry>,
): AfflictionOccurrence[] {
  const items = entry.raw.items;
  if (!Array.isArray(items)) {
    return [];
  }

  const occurrences: AfflictionOccurrence[] = [];
  for (const [itemIndex, item] of items.entries()) {
    if (!item || typeof item !== "object") {
      continue;
    }

    const childRaw = item as Record<string, unknown>;
    const family = detectAfflictionFamily(childRaw);
    const name = typeof childRaw.name === "string" ? childRaw.name.trim() : "";
    if (!family || !name || !hasAfflictionShape(childRaw)) {
      continue;
    }

    const slug = getRecordSlug(childRaw);
    const compendiumSource = getCompendiumSource(childRaw);
    const parsedCompendiumSource = parseCompendiumSource(compendiumSource);
    const sourceRecord = parsedCompendiumSource
      ? recordsByPackAndId.get(buildPackAndIdKey(parsedCompendiumSource.packName, parsedCompendiumSource.id))?.record ?? null
      : null;
    const sourceRaw = parsedCompendiumSource
      ? recordsByPackAndId.get(buildPackAndIdKey(parsedCompendiumSource.packName, parsedCompendiumSource.id))?.raw ?? null
      : null;
    const childId = typeof childRaw._id === "string" && childRaw._id.length > 0 ? childRaw._id : `item-${itemIndex}`;
    occurrences.push({
      hostRecord: entry.record,
      sourceRecord,
      sourceRaw,
      childRaw,
      family,
      name,
      slug,
      traits: uniqueSorted([family, ...getRecordTraits(childRaw)]),
      linkedNames: collectOccurrenceLinkedNames(childRaw),
      compendiumSource,
      sourcePath: `${entry.record.sourcePath}#item:${childId}`,
      occurrenceRef: childId,
      identityKey: buildOccurrenceIdentityKey(
        family,
        name,
        slug,
        compendiumSource,
        sourceRecord?.recordKey ?? null,
      ),
    });
  }

  return occurrences;
}

type CanonicalBuildCandidate = {
  occurrence: AfflictionOccurrence;
  authoritativeRecord: NormalizedIndexRecord | null;
  authoritativeRaw: Record<string, unknown> | null;
};

function chooseAuthoritativeCandidate(occurrences: AfflictionOccurrence[]): CanonicalBuildCandidate {
  const sorted = [...occurrences].sort((left, right) => {
    return (
      Number(Boolean(right.sourceRecord)) - Number(Boolean(left.sourceRecord)) ||
      left.hostRecord.recordKey.localeCompare(right.hostRecord.recordKey) ||
      left.occurrenceRef.localeCompare(right.occurrenceRef)
    );
  });
  const occurrence = sorted[0]!;
  const authoritativeRecord = occurrence.sourceRecord;
  const authoritativeRaw = occurrence.sourceRaw;

  return {
    occurrence,
    authoritativeRecord,
    authoritativeRaw,
  };
}

export function buildDerivedAfflictionArtifacts(indexedEntries: IndexedBuildEntry[]): DerivedAfflictionBuild {
  const recordsByPackAndId = new Map<string, IndexedBuildEntry>();
  for (const entry of indexedEntries) {
    recordsByPackAndId.set(buildPackAndIdKey(entry.record.packName, entry.record.id), entry);
  }

  const occurrences = indexedEntries.flatMap((entry) => {
    const topLevelOccurrence = collectTopLevelOccurrence(entry);
    return [
      ...collectEmbeddedOccurrences(entry, recordsByPackAndId),
      ...(topLevelOccurrence ? [topLevelOccurrence] : []),
    ];
  });
  if (occurrences.length === 0) {
    return { records: [], edges: [] };
  }

  const occurrencesByIdentity = new Map<string, AfflictionOccurrence[]>();
  for (const occurrence of occurrences) {
    const bucket = occurrencesByIdentity.get(occurrence.identityKey) ?? [];
    bucket.push(occurrence);
    occurrencesByIdentity.set(occurrence.identityKey, bucket);
  }

  const derivedRecords: DerivedBuildEntry[] = [];
  const derivedEdges: DerivedReferenceEdgeRow[] = [];
  const canonicalPack = toDerivedPackBuildInfo(DERIVED_AFFLICTIONS_PACK_NAME, DERIVED_AFFLICTIONS_PACK_LABEL);
  const instancePack = toDerivedPackBuildInfo(DERIVED_AFFLICTION_INSTANCES_PACK_NAME, DERIVED_AFFLICTION_INSTANCES_PACK_LABEL);

  for (const [identityKey, groupedOccurrences] of [...occurrencesByIdentity.entries()].sort((left, right) => left[0].localeCompare(right[0]))) {
    const candidate = chooseAuthoritativeCandidate(groupedOccurrences);
    const representativeOccurrence = candidate.occurrence;
    const representativeName = representativeOccurrence.name;
    const representativeSlug = representativeOccurrence.slug;
    const allTraits = uniqueSorted(groupedOccurrences.flatMap((occurrence) => occurrence.traits));
    const allLinkedNames = uniqueSorted(groupedOccurrences.flatMap((occurrence) => occurrence.linkedNames));
    const canonicalId = hashText(identityKey);
    const canonicalRecordKey = `${DERIVED_AFFLICTIONS_PACK_NAME}:${canonicalId}`;
    const canonicalDescriptionText = candidate.authoritativeRecord?.descriptionText ?? null;
    const canonicalDescriptionMarkup = candidate.authoritativeRaw
      ? getRecordDescriptionMarkup(candidate.authoritativeRaw)
      : null;

    let representativeInstanceRecordKey: string | null = null;
    const occurrenceInstanceKeys = groupedOccurrences.map((occurrence) => {
      const instanceId = hashText(`${identityKey}:${occurrence.hostRecord.recordKey}:${occurrence.occurrenceRef}`);
      return {
        occurrence,
        instanceId,
        recordKey: `${DERIVED_AFFLICTION_INSTANCES_PACK_NAME}:${instanceId}`,
      };
    });

    representativeInstanceRecordKey = occurrenceInstanceKeys[0]?.recordKey ?? null;

    const canonicalRaw = buildCanonicalRaw(
      canonicalId,
      representativeName,
      representativeOccurrence.family,
      allTraits,
      canonicalDescriptionMarkup,
      allLinkedNames,
      representativeInstanceRecordKey,
      identityKey,
    );
    const canonicalRecord: NormalizedIndexRecord = {
      recordKey: canonicalRecordKey,
      id: canonicalId,
      name: representativeName,
      normalizedName: normalizeText(representativeName),
      type: "affliction",
      category: "affliction",
      subcategory: representativeOccurrence.family,
      packName: canonicalPack.name,
      packLabel: canonicalPack.label,
      documentType: canonicalPack.documentType,
      level: candidate.authoritativeRecord?.level ?? null,
      rarity: candidate.authoritativeRecord?.rarity ?? null,
      traits: allTraits,
      derivedTags: [],
      publicationTitle: candidate.authoritativeRecord?.publicationTitle ?? null,
      publicationRemaster: candidate.authoritativeRecord?.publicationRemaster ?? false,
      descriptionText: canonicalDescriptionText,
      hasDescription: Boolean(canonicalDescriptionText),
      descriptionSnippet: buildDescriptionSnippet(canonicalDescriptionText),
      sourceCategory: candidate.authoritativeRecord?.sourceCategory ?? representativeOccurrence.hostRecord.sourceCategory,
      folderId: null,
      families: [],
      sourcePath: `derived://afflictions/${canonicalId}`,
      isUnique: false,
      size: null,
      itemCategory: null,
      priceCp: null,
      bulkValue: null,
      actionCost: null,
      usage: null,
      hands: null,
      damageTypes: [],
      weaponGroup: null,
      armorGroup: null,
      traditions: [],
      spellKinds: [],
      languages: [],
      speedTypes: [],
      immunities: [],
      resistances: [],
      weaknesses: [],
      rangeValue: null,
      searchText: buildCanonicalSearchText(
        representativeName,
        representativeOccurrence.family,
        allTraits,
        representativeSlug,
        allLinkedNames,
      ),
    };
    derivedRecords.push({
      record: canonicalRecord,
      raw: canonicalRaw,
      actorData: null,
      itemData: null,
      spellData: null,
      references: [],
      resolvedReferences: [],
      isSearchCanonical: true,
    });

    for (const { occurrence, instanceId, recordKey } of occurrenceInstanceKeys) {
      const instanceDescriptionText = getRecordDescriptionText(occurrence.childRaw);
      const instanceRecord: NormalizedIndexRecord = {
        recordKey,
        id: instanceId,
        name: occurrence.name,
        normalizedName: normalizeText(occurrence.name),
        type: "affliction-instance",
        category: "affliction",
        subcategory: occurrence.family,
        packName: instancePack.name,
        packLabel: instancePack.label,
        documentType: instancePack.documentType,
        level: occurrence.sourceRecord?.level ?? occurrence.hostRecord.level,
        rarity: occurrence.sourceRecord?.rarity ?? occurrence.hostRecord.rarity,
        traits: occurrence.traits,
        derivedTags: [],
        publicationTitle: occurrence.sourceRecord?.publicationTitle ?? occurrence.hostRecord.publicationTitle,
        publicationRemaster: occurrence.sourceRecord?.publicationRemaster ?? occurrence.hostRecord.publicationRemaster,
        descriptionText: instanceDescriptionText,
        hasDescription: Boolean(instanceDescriptionText),
        descriptionSnippet: buildDescriptionSnippet(instanceDescriptionText),
        sourceCategory: occurrence.hostRecord.sourceCategory,
        folderId: null,
        families: [],
        sourcePath: occurrence.sourcePath,
        isUnique: false,
        size: null,
        itemCategory: null,
        priceCp: null,
        bulkValue: null,
        actionCost: null,
        usage: null,
        hands: null,
        damageTypes: [],
        weaponGroup: null,
        armorGroup: null,
        traditions: [],
        spellKinds: [],
        languages: [],
        speedTypes: [],
        immunities: [],
        resistances: [],
        weaknesses: [],
        rangeValue: null,
        searchText: buildOccurrenceSearchText(occurrence.name, occurrence.family, occurrence.traits, occurrence.linkedNames),
      };
      derivedRecords.push({
        record: instanceRecord,
        raw: buildInstanceRaw(instanceId, occurrence, canonicalRecordKey),
        actorData: null,
        itemData: null,
        spellData: null,
        references: [],
        resolvedReferences: [],
        isSearchCanonical: false,
      });

      derivedEdges.push({
        fromRecordKey: occurrence.hostRecord.recordKey,
        toRecordKey: recordKey,
        displayText: occurrence.name,
        referenceText: `derived-affliction-instance:${recordKey}`,
        fromPackName: occurrence.hostRecord.packName,
        fromRecordType: occurrence.hostRecord.type,
        fromDocumentType: occurrence.hostRecord.documentType,
        fromSourceCategory: occurrence.hostRecord.sourceCategory,
      });
      derivedEdges.push({
        fromRecordKey: recordKey,
        toRecordKey: canonicalRecordKey,
        displayText: occurrence.name,
        referenceText: `derived-affliction-canonical:${canonicalRecordKey}`,
        fromPackName: instanceRecord.packName,
        fromRecordType: instanceRecord.type,
        fromDocumentType: instanceRecord.documentType,
        fromSourceCategory: instanceRecord.sourceCategory,
      });
      derivedEdges.push({
        fromRecordKey: canonicalRecordKey,
        toRecordKey: occurrence.hostRecord.recordKey,
        displayText: occurrence.hostRecord.name,
        referenceText: `derived-affliction-host:${occurrence.hostRecord.recordKey}:${recordKey}`,
        fromPackName: canonicalRecord.packName,
        fromRecordType: canonicalRecord.type,
        fromDocumentType: canonicalRecord.documentType,
        fromSourceCategory: canonicalRecord.sourceCategory,
      });
    }
  }

  return {
    records: derivedRecords,
    edges: derivedEdges,
  };
}
