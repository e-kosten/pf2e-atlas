import { DatabaseSync } from "node:sqlite";

import type {
  DerivedTagAssignmentMode,
  SearchCategory,
  SearchSubcategory,
  SourceCategory,
} from "../../types.js";
import {
  getDerivedTagExemplars,
  listDerivedTagLegacySeedMigrations,
  normalizeDerivedTag,
} from "../index.js";
import { getCurrentDerivedTagMigrationAuthoredState } from "./authored-state.js";
import type { OntologyExplorerEntityRecord } from "./entity-page.js";
import { getPublishedDerivedTagMigrationOntology } from "./runtime-state.js";

type ExplorerCountRow = {
  category: SearchCategory;
  tag: string;
  recordKey: string;
};

type ExplorerRecordRow = {
  recordKey: string;
  packName: string | null;
  name: string;
  type: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  documentType: string;
  level: number | bigint | null;
  rarity: string | null;
  traitsJson: string;
  derivedTagsJson: string;
  familiesJson: string | null;
  descriptionText: string | null;
  blurbText: string | null;
  sourceCategory: SourceCategory;
  publicationTitle: string | null;
  publicationRemaster: number;
  isUnique: number;
  size: string | null;
  languagesJson: string | null;
  speedTypesJson: string | null;
  sensesJson: string | null;
  immunitiesJson: string | null;
  resistancesJson: string | null;
  weaknessesJson: string | null;
  itemCategory: string | null;
  baseItem: string | null;
  priceCp: number | null;
  usage: string | null;
  hands: number | null;
  damageTypesJson: string | null;
  weaponGroup: string | null;
  armorGroup: string | null;
  traditionsJson: string | null;
  spellKindsJson: string | null;
  saveType: string | null;
  areaType: string | null;
  rangeText: string | null;
  durationText: string | null;
  targetText: string | null;
  areaValue: number | null;
  sustained: number | null;
  basicSave: number | null;
  disableText: string | null;
  disableSkillsJson: string | null;
  isComplex: number | null;
};

export type DerivedTagOntologyExplorerRecordNode = {
  kind: "record";
  key: string;
  category: SearchCategory;
  tag: string;
  record: OntologyExplorerEntityRecord;
  filterText: string;
};

export type DerivedTagOntologyExplorerTagNode = {
  kind: "tag";
  key: string;
  category: SearchCategory;
  family: string;
  subcategories?: SearchSubcategory[];
  tag: string;
  description: string;
  assignmentMode: DerivedTagAssignmentMode;
  nativeOntologyPolicy?: "distinct_required" | "aggregates_native_signals";
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  positiveSignals?: string[];
  negativeSignals?: string[];
  adjacentTags?: string[];
  compositeOfAnyTags?: string[];
  variantInheritance?: boolean;
  liveRecordCount: number;
  authoredRuleCount: number;
  exemplarPositiveCount: number;
  exemplarNegativeCount: number;
  legacyMigrationDefinitionCount: number;
  legacyMigrationRecordCount: number;
  records: DerivedTagOntologyExplorerRecordNode[];
  filterText: string;
};

export type DerivedTagOntologyExplorerFamilyNode = {
  kind: "family";
  key: string;
  category: SearchCategory;
  family: string;
  axis: string;
  description: string;
  subcategories?: SearchSubcategory[];
  variantInheritance?: boolean;
  tagCount: number;
  liveRecordCount: number;
  tags: DerivedTagOntologyExplorerTagNode[];
  filterText: string;
};

export type DerivedTagOntologyExplorerCategoryNode = {
  kind: "category";
  key: SearchCategory;
  category: SearchCategory;
  familyCount: number;
  tagCount: number;
  taggedRecordCount: number;
  families: DerivedTagOntologyExplorerFamilyNode[];
  filterText: string;
};

export type DerivedTagOntologyExplorerModel = {
  categories: DerivedTagOntologyExplorerCategoryNode[];
};

function parseStringArray(json: string | null | undefined): string[] {
  return json ? JSON.parse(json) as string[] : [];
}

function buildTagFilterText(tag: {
  category: SearchCategory;
  family: string;
  tag: string;
  description: string;
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  positiveSignals?: string[];
  negativeSignals?: string[];
  adjacentTags?: string[];
  compositeOfAnyTags?: string[];
}): string {
  return [
    tag.category,
    tag.family,
    tag.tag,
    tag.description,
    ...(tag.appliesWhen ?? []),
    ...(tag.doesNotApplyWhen ?? []),
    ...(tag.positiveSignals ?? []),
    ...(tag.negativeSignals ?? []),
    ...(tag.adjacentTags ?? []),
    ...(tag.compositeOfAnyTags ?? []),
  ].join(" ").toLowerCase();
}

function buildFamilyFilterText(family: {
  category: SearchCategory;
  family: string;
  axis: string;
  description: string;
  subcategories?: SearchSubcategory[];
}): string {
  return [
    family.category,
    family.family,
    family.axis,
    family.description,
    ...(family.subcategories ?? []),
  ].join(" ").toLowerCase();
}

function buildRecordFilterText(tag: string, record: OntologyExplorerEntityRecord): string {
  return [
    tag,
    record.name,
    record.recordKey,
    record.category,
    record.subcategory ?? "",
    record.type,
    record.documentType,
    record.rarity ?? "",
    ...(record.traits ?? []),
    ...(record.derivedTags ?? []),
    ...(record.families ?? []),
    record.blurbText ?? "",
    record.descriptionText ?? "",
  ].join(" ").toLowerCase();
}

function queryCanonicalTagRows(db: DatabaseSync): ExplorerCountRow[] {
  return db.prepare(`
    SELECT r.category AS category, d.tag AS tag, d.record_key AS recordKey
    FROM record_derived_tags d
    JOIN records r ON r.record_key = d.record_key
    WHERE r.is_search_canonical = 1
  `).all() as ExplorerCountRow[];
}

function queryExplorerRecordRows(
  db: DatabaseSync,
  recordKeys: string[],
): ExplorerRecordRow[] {
  if (recordKeys.length === 0) {
    return [];
  }

  const placeholders = recordKeys.map(() => "?").join(", ");
  return db.prepare(`
    SELECT
      r.record_key AS recordKey,
      r.pack_name AS packName,
      r.name AS name,
      r.type AS type,
      r.category AS category,
      r.subcategory AS subcategory,
      r.document_type AS documentType,
      r.level AS level,
      r.rarity AS rarity,
      r.traits_json AS traitsJson,
      r.derived_tags_json AS derivedTagsJson,
      r.families_json AS familiesJson,
      r.description_text AS descriptionText,
      r.blurb_text AS blurbText,
      r.source_category AS sourceCategory,
      r.publication_title AS publicationTitle,
      r.publication_remaster AS publicationRemaster,
      r.is_unique AS isUnique,
      a.size AS size,
      a.languages_json AS languagesJson,
      a.speed_types_json AS speedTypesJson,
      a.senses_json AS sensesJson,
      a.immunities_json AS immunitiesJson,
      a.resistances_json AS resistancesJson,
      a.weaknesses_json AS weaknessesJson,
      a.disable_text AS disableText,
      a.disable_skills_json AS disableSkillsJson,
      a.is_complex AS isComplex,
      i.item_category AS itemCategory,
      i.base_item AS baseItem,
      i.price_cp AS priceCp,
      i.usage AS usage,
      i.hands AS hands,
      i.damage_types_json AS damageTypesJson,
      i.weapon_group AS weaponGroup,
      i.armor_group AS armorGroup,
      s.traditions_json AS traditionsJson,
      s.spell_kinds_json AS spellKindsJson,
      s.save_type AS saveType,
      s.area_type AS areaType,
      s.range_text AS rangeText,
      s.duration_text AS durationText,
      s.target_text AS targetText,
      s.area_value AS areaValue,
      s.sustained AS sustained,
      s.basic_save AS basicSave
    FROM records r
    LEFT JOIN actor_records a ON a.record_key = r.record_key
    LEFT JOIN item_records i ON i.record_key = r.record_key
    LEFT JOIN spell_records s ON s.record_key = r.record_key
    WHERE r.record_key IN (${placeholders})
  `).all(...recordKeys) as ExplorerRecordRow[];
}

function rowToEntityRecord(row: ExplorerRecordRow): OntologyExplorerEntityRecord {
  return {
    recordKey: row.recordKey,
    packName: row.packName ?? row.recordKey.split(":")[0] ?? "",
    name: row.name,
    type: row.type,
    category: row.category,
    subcategory: row.subcategory,
    documentType: row.documentType,
    level: typeof row.level === "bigint" ? Number(row.level) : row.level,
    rarity: row.rarity,
    traits: parseStringArray(row.traitsJson),
    derivedTags: parseStringArray(row.derivedTagsJson),
    families: parseStringArray(row.familiesJson),
    descriptionText: row.descriptionText,
    blurbText: row.blurbText,
    sourceCategory: row.sourceCategory,
    publicationTitle: row.publicationTitle,
    publicationRemaster: Boolean(row.publicationRemaster),
    isUnique: Boolean(row.isUnique),
    size: row.size,
    languages: parseStringArray(row.languagesJson),
    speedTypes: parseStringArray(row.speedTypesJson),
    senses: parseStringArray(row.sensesJson),
    immunities: parseStringArray(row.immunitiesJson),
    resistances: parseStringArray(row.resistancesJson),
    weaknesses: parseStringArray(row.weaknessesJson),
    itemCategory: row.itemCategory,
    baseItem: row.baseItem,
    priceCp: row.priceCp,
    usage: row.usage,
    hands: row.hands,
    damageTypes: parseStringArray(row.damageTypesJson),
    weaponGroup: row.weaponGroup,
    armorGroup: row.armorGroup,
    traditions: parseStringArray(row.traditionsJson),
    spellKinds: parseStringArray(row.spellKindsJson),
    saveType: row.saveType,
    areaType: row.areaType,
    rangeText: row.rangeText,
    durationText: row.durationText,
    targetText: row.targetText,
    areaValue: row.areaValue,
    sustained: Boolean(row.sustained),
    basicSave: Boolean(row.basicSave),
    disableText: row.disableText,
    disableSkills: parseStringArray(row.disableSkillsJson),
    isComplex: Boolean(row.isComplex),
  };
}

function buildAuthoredRuleCounts(): Map<string, number> {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const counts = new Map<string, number>();

  for (const [category, rules] of Object.entries(state.authoredRules) as Array<[SearchCategory, typeof state.authoredRules[keyof typeof state.authoredRules]]>) {
    for (const rule of rules) {
      const key = `${category}:${normalizeDerivedTag(rule.tag)}`;
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }

  return counts;
}

function buildExemplarCounts(): Map<string, { positive: number; negative: number }> {
  const counts = new Map<string, { positive: number; negative: number }>();
  const state = getCurrentDerivedTagMigrationAuthoredState();

  for (const [category, exemplarCategory] of Object.entries(state.exemplars) as Array<[SearchCategory, typeof state.exemplars[keyof typeof state.exemplars]]>) {
    for (const exemplar of exemplarCategory.exemplars) {
      const key = `${category}:${normalizeDerivedTag(exemplar.tag)}`;
      counts.set(key, {
        positive: exemplar.positives?.length ?? 0,
        negative: exemplar.negatives?.length ?? 0,
      });
    }
  }

  return counts;
}

function buildLegacyMigrationCounts(): Map<string, { definitions: number; records: number }> {
  const counts = new Map<string, { definitions: number; records: number }>();

  for (const definition of listDerivedTagLegacySeedMigrations()) {
    const key = `${definition.category}:${normalizeDerivedTag(definition.tag)}`;
    const current = counts.get(key) ?? { definitions: 0, records: 0 };
    current.definitions += 1;
    current.records += definition.recordKeys.length;
    counts.set(key, current);
  }

  return counts;
}

function buildLiveCountMaps(rows: ExplorerCountRow[]): {
  tagCounts: Map<string, number>;
  familyCounts: Map<string, number>;
  categoryCounts: Map<SearchCategory, number>;
  recordKeysByTagKey: Map<string, string[]>;
} {
  const ontology = getPublishedDerivedTagMigrationOntology();
  const tagCounts = new Map<string, number>();
  const familyRecordKeys = new Map<string, Set<string>>();
  const categoryRecordKeys = new Map<SearchCategory, Set<string>>();
  const tagRecordKeys = new Map<string, Set<string>>();

  for (const row of rows) {
    const normalizedTag = normalizeDerivedTag(row.tag);
    const tagKey = `${row.category}:${normalizedTag}`;
    tagCounts.set(tagKey, (tagCounts.get(tagKey) ?? 0) + 1);

    const tagBucket = tagRecordKeys.get(tagKey) ?? new Set<string>();
    tagBucket.add(row.recordKey);
    tagRecordKeys.set(tagKey, tagBucket);

    const ontologyTag = ontology.tagByKey.get(tagKey as `${SearchCategory}:${string}`);
    if (!ontologyTag) {
      continue;
    }

    const familyKey = `${row.category}:${normalizeDerivedTag(ontologyTag.family)}`;
    const familyBucket = familyRecordKeys.get(familyKey) ?? new Set<string>();
    familyBucket.add(row.recordKey);
    familyRecordKeys.set(familyKey, familyBucket);

    const categoryBucket = categoryRecordKeys.get(row.category) ?? new Set<string>();
    categoryBucket.add(row.recordKey);
    categoryRecordKeys.set(row.category, categoryBucket);
  }

  const familyCounts = new Map<string, number>();
  for (const [key, recordKeys] of familyRecordKeys.entries()) {
    familyCounts.set(key, recordKeys.size);
  }

  const categoryCounts = new Map<SearchCategory, number>();
  for (const [category, recordKeys] of categoryRecordKeys.entries()) {
    categoryCounts.set(category, recordKeys.size);
  }

  const recordKeysByTagKey = new Map<string, string[]>();
  for (const [key, recordKeys] of tagRecordKeys.entries()) {
    recordKeysByTagKey.set(key, [...recordKeys]);
  }

  return { tagCounts, familyCounts, categoryCounts, recordKeysByTagKey };
}

function buildRecordNodesByTagKey(
  db: DatabaseSync,
  rowsByTagKey: Map<string, string[]>,
): Map<string, DerivedTagOntologyExplorerRecordNode[]> {
  const uniqueRecordKeys = [...new Set([...rowsByTagKey.values()].flat())];
  const recordsByKey = new Map(
    queryExplorerRecordRows(db, uniqueRecordKeys)
      .map((row) => rowToEntityRecord(row))
      .map((record) => [record.recordKey, record] as const),
  );
  const nodesByTagKey = new Map<string, DerivedTagOntologyExplorerRecordNode[]>();

  for (const [tagKey, recordKeys] of rowsByTagKey.entries()) {
    const [, normalizedTag = ""] = tagKey.split(":", 2);
    const records = recordKeys
      .map((recordKey) => recordsByKey.get(recordKey))
      .filter((record): record is OntologyExplorerEntityRecord => Boolean(record))
      .sort((left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey))
      .map((record) => ({
        kind: "record" as const,
        key: record.recordKey,
        category: record.category,
        tag: normalizedTag,
        record,
        filterText: buildRecordFilterText(normalizedTag, record),
      }));
    nodesByTagKey.set(tagKey, records);
  }

  return nodesByTagKey;
}

export function buildDerivedTagOntologyExplorerModel(db: DatabaseSync): DerivedTagOntologyExplorerModel {
  const ontology = getPublishedDerivedTagMigrationOntology();
  const liveCounts = buildLiveCountMaps(queryCanonicalTagRows(db));
  const authoredRuleCounts = buildAuthoredRuleCounts();
  const exemplarCounts = buildExemplarCounts();
  const legacyMigrationCounts = buildLegacyMigrationCounts();
  const recordNodesByTagKey = buildRecordNodesByTagKey(db, liveCounts.recordKeysByTagKey);

  const categories = ontology.families
    .reduce<Map<SearchCategory, DerivedTagOntologyExplorerCategoryNode>>((bucket, family) => {
      const categoryNode = bucket.get(family.category) ?? {
        kind: "category" as const,
        key: family.category,
        category: family.category,
        familyCount: 0,
        tagCount: 0,
        taggedRecordCount: liveCounts.categoryCounts.get(family.category) ?? 0,
        families: [],
        filterText: family.category.toLowerCase(),
      };

      const familyKey = `${family.category}:${normalizeDerivedTag(family.family)}`;
      const tags = (ontology.tagsByFamilyKey.get(familyKey as `${SearchCategory}:${string}`) ?? [])
        .map((tag) => {
          const tagKey = `${tag.category}:${normalizeDerivedTag(tag.tag)}`;
          const exemplars = exemplarCounts.get(tagKey) ?? { positive: 0, negative: 0 };
          const migrations = legacyMigrationCounts.get(tagKey) ?? { definitions: 0, records: 0 };
          return {
            kind: "tag" as const,
            key: tagKey,
            category: tag.category,
            family: tag.family,
            subcategories: family.subcategories,
            tag: tag.tag,
            description: tag.description,
            assignmentMode: tag.assignmentMode,
            nativeOntologyPolicy: tag.nativeOntologyPolicy,
            appliesWhen: tag.appliesWhen,
            doesNotApplyWhen: tag.doesNotApplyWhen,
            positiveSignals: tag.positiveSignals,
            negativeSignals: tag.negativeSignals,
            adjacentTags: tag.adjacentTags,
            compositeOfAnyTags: tag.compositeOfAnyTags,
            variantInheritance: tag.variantInheritance,
            liveRecordCount: liveCounts.tagCounts.get(tagKey) ?? 0,
            authoredRuleCount: authoredRuleCounts.get(tagKey) ?? 0,
            exemplarPositiveCount: exemplars.positive,
            exemplarNegativeCount: exemplars.negative,
            legacyMigrationDefinitionCount: migrations.definitions,
            legacyMigrationRecordCount: migrations.records,
            records: recordNodesByTagKey.get(tagKey) ?? [],
            filterText: buildTagFilterText(tag),
          };
        });

      const familyNode: DerivedTagOntologyExplorerFamilyNode = {
        kind: "family",
        key: familyKey,
        category: family.category,
        family: family.family,
        axis: family.axis,
        description: family.description,
        subcategories: family.subcategories,
        variantInheritance: family.variantInheritance,
        tagCount: tags.length,
        liveRecordCount: liveCounts.familyCounts.get(familyKey) ?? 0,
        tags,
        filterText: buildFamilyFilterText(family),
      };

      categoryNode.familyCount += 1;
      categoryNode.tagCount += tags.length;
      categoryNode.families.push(familyNode);
      bucket.set(family.category, categoryNode);
      return bucket;
    }, new Map())
    .values();

  return {
    categories: [...categories].map((category) => ({
      ...category,
      families: [...category.families].sort((left, right) =>
        left.axis.localeCompare(right.axis)
        || left.family.localeCompare(right.family)),
      filterText: [
        category.category,
        ...category.families.map((family) => `${family.family} ${family.description}`),
      ].join(" ").toLowerCase(),
    })),
  };
}

export function filterOntologyExplorerNodes<T extends { filterText: string }>(
  nodes: T[],
  filter: string,
): T[] {
  const normalized = filter.trim().toLowerCase();
  if (!normalized) {
    return nodes;
  }
  return nodes.filter((node) => node.filterText.includes(normalized));
}

export function getPublishedExemplarPresence(
  category: SearchCategory,
  tag: string,
): { positive: number; negative: number } {
  const exemplars = getDerivedTagExemplars(tag, { category });
  return exemplars.reduce((totals, exemplarSet) => ({
    positive: totals.positive + (exemplarSet.positives?.length ?? 0),
    negative: totals.negative + (exemplarSet.negatives?.length ?? 0),
  }), { positive: 0, negative: 0 });
}
