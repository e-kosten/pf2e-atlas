import type {
  ActorMetricMap,
  ItemMetricMap,
  LinkedRecordSummary,
  LookupResult,
  NormalizedRecord,
  RuleReferenceEdge,
  SearchCategory,
  SearchSubcategory,
  SourceCategory,
} from "../types.js";
import { normalizeText } from "../utils.js";

export type CandidateRow = {
  recordKey: string;
  canonicalRecordKey?: string;
  id: string;
  name: string;
  normalizedName: string;
  type: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  packName: string;
  packLabel: string;
  documentType: string;
  level: number | null;
  rarity: string | null;
  traitsJson: string;
  derivedTagsJson: string;
  publicationTitle: string | null;
  publicationRemaster: number;
  descriptionText: string | null;
  hasDescription: number;
  descriptionSnippet: string | null;
  sourceCategory: SourceCategory;
  folderId: string | null;
  familiesJson: string | null;
  variantFamilyKey: string | null;
  variantBaseName: string | null;
  variantLabel: string | null;
  variantAxesJson: string | null;
  variantConfidence: number | null;
  variantSource: string;
  sourcePath: string;
  isUnique: number;
  isSearchCanonical?: number;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  usage: string | null;
  hands: number | null;
  itemMetricsJson: string | null;
  damageTypesJson: string | null;
  weaponGroup: string | null;
  armorGroup: string | null;
  traditionsJson: string | null;
  spellKindsJson: string | null;
  rangeText: string | null;
  saveType: string | null;
  areaType: string | null;
  durationText: string | null;
  durationUnit: string | null;
  targetText: string | null;
  areaValue: number | null;
  sustained: number | null;
  basicSave: number | null;
  languagesJson: string | null;
  speedTypesJson: string | null;
  sensesJson: string | null;
  immunitiesJson: string | null;
  resistancesJson: string | null;
  weaknessesJson: string | null;
  disableText: string | null;
  disableSkillsJson: string | null;
  isComplex: number | null;
  actorMetricsJson: string | null;
  rangeValue: number | null;
  rawJson?: string | null;
  searchText?: string | null;
  embeddingBlob?: Uint8Array | null;
};

export type ReferenceEdgeRow = {
  fromRecordKey: string;
  toRecordKey: string;
  displayText: string | null;
  referenceText: string;
  fromPackName: string;
  fromRecordType: string;
  fromDocumentType: string;
  fromSourceCategory: SourceCategory;
};

export type ValueCountRow = {
  value: string;
  count: number;
};

type ActorMetricJsonRow = {
  metricKey: string;
  valueType: "number" | "text" | "boolean";
  numberValue: number | null;
  textValue: string | null;
  boolValue: number | null;
};

function parseActorMetricsJson(actorMetricsJson: string | null | undefined): ActorMetricMap {
  if (!actorMetricsJson) {
    return {};
  }

  const rows = JSON.parse(actorMetricsJson) as ActorMetricJsonRow[];
  const metrics: ActorMetricMap = {};
  for (const row of rows) {
    if (row.valueType === "number" && typeof row.numberValue === "number") {
      metrics[row.metricKey] = row.numberValue;
      continue;
    }

    if (row.valueType === "boolean") {
      metrics[row.metricKey] = Boolean(row.boolValue);
      continue;
    }

    if (row.valueType === "text" && typeof row.textValue === "string") {
      metrics[row.metricKey] = row.textValue;
    }
  }

  return metrics;
}

function parseItemMetricsJson(itemMetricsJson: string | null | undefined): ItemMetricMap {
  if (!itemMetricsJson) {
    return {};
  }

  const rows = JSON.parse(itemMetricsJson) as ActorMetricJsonRow[];
  const metrics: ItemMetricMap = {};
  for (const row of rows) {
    if (row.valueType === "number" && typeof row.numberValue === "number") {
      metrics[row.metricKey] = row.numberValue;
      continue;
    }

    if (row.valueType === "boolean") {
      metrics[row.metricKey] = Boolean(row.boolValue);
      continue;
    }

    if (row.valueType === "text" && typeof row.textValue === "string") {
      metrics[row.metricKey] = row.textValue;
    }
  }

  return metrics;
}

export function rowToRecord(row: CandidateRow, raw: Record<string, unknown> | null = null): NormalizedRecord {
  const resolvedRaw = raw ?? (row.rawJson ? JSON.parse(row.rawJson) as Record<string, unknown> : {});
  return {
    recordKey: row.recordKey,
    id: row.id,
    name: row.name,
    normalizedName: row.normalizedName,
    type: row.type,
    category: row.category,
    subcategory: row.subcategory,
    packName: row.packName,
    packLabel: row.packLabel,
    documentType: row.documentType,
    level: row.level,
    rarity: row.rarity,
    traits: JSON.parse(row.traitsJson) as string[],
    derivedTags: JSON.parse(row.derivedTagsJson) as string[],
    publicationTitle: row.publicationTitle,
    publicationRemaster: Boolean(row.publicationRemaster),
    descriptionText: row.descriptionText,
    hasDescription: Boolean(row.hasDescription),
    descriptionSnippet: row.descriptionSnippet,
    sourceCategory: row.sourceCategory,
    folderId: row.folderId,
    families: row.familiesJson ? (JSON.parse(row.familiesJson) as string[]) : [],
    variantFamilyKey: row.variantFamilyKey,
    variantBaseName: row.variantBaseName,
    variantLabel: row.variantLabel,
    variantAxes: row.variantAxesJson ? (JSON.parse(row.variantAxesJson) as string[]) : [],
    variantConfidence: row.variantConfidence,
    variantSource: (row.variantSource ?? "none") as NormalizedRecord["variantSource"],
    sourcePath: row.sourcePath,
    isUnique: Boolean(row.isUnique),
    size: row.size,
    itemCategory: row.itemCategory,
    priceCp: row.priceCp,
    bulkValue: row.bulkValue,
    actionCost: row.actionCost,
    usage: row.usage,
    hands: row.hands,
    itemMetrics: parseItemMetricsJson(row.itemMetricsJson),
    damageTypes: row.damageTypesJson ? (JSON.parse(row.damageTypesJson) as string[]) : [],
    weaponGroup: row.weaponGroup,
    armorGroup: row.armorGroup,
    traditions: row.traditionsJson ? (JSON.parse(row.traditionsJson) as string[]) : [],
    spellKinds: row.spellKindsJson ? (JSON.parse(row.spellKindsJson) as string[]) : [],
    saveType: row.saveType,
    areaType: row.areaType,
    rangeText: row.rangeText,
    durationText: row.durationText,
    durationUnit: row.durationUnit,
    targetText: row.targetText,
    areaValue: row.areaValue,
    sustained: Boolean(row.sustained),
    basicSave: Boolean(row.basicSave),
    languages: row.languagesJson ? (JSON.parse(row.languagesJson) as string[]) : [],
    speedTypes: row.speedTypesJson ? (JSON.parse(row.speedTypesJson) as string[]) : [],
    senses: row.sensesJson ? (JSON.parse(row.sensesJson) as string[]) : [],
    immunities: row.immunitiesJson ? (JSON.parse(row.immunitiesJson) as string[]) : [],
    resistances: row.resistancesJson ? (JSON.parse(row.resistancesJson) as string[]) : [],
    weaknesses: row.weaknessesJson ? (JSON.parse(row.weaknessesJson) as string[]) : [],
    disableText: row.disableText,
    disableSkills: row.disableSkillsJson ? (JSON.parse(row.disableSkillsJson) as string[]) : [],
    isComplex: Boolean(row.isComplex),
    actorMetrics: parseActorMetricsJson(row.actorMetricsJson),
    rangeValue: row.rangeValue,
    aliases: [],
    legacyRecordLinks: [],
    raw: resolvedRaw,
  };
}

export function buildPlaceholders(values: readonly unknown[]): string {
  return values.map(() => "?").join(", ");
}

export function getLookupMatchType(query: string, record: NormalizedRecord | null): LookupResult["matchType"] {
  if (!record) {
    return "none";
  }

  const normalizedQuery = normalizeText(query);
  if (normalizeText(record.name) === normalizedQuery) {
    return normalizeText(query) === normalizeText(record.name) && query.trim() === record.name ? "exact" : "normalized_exact";
  }

  return "fuzzy";
}

export function edgeRowToReferenceEdge(
  row: ReferenceEdgeRow,
  direction: RuleReferenceEdge["direction"],
): RuleReferenceEdge {
  return {
    fromRecordKey: row.fromRecordKey,
    toRecordKey: row.toRecordKey,
    displayText: row.displayText,
    referenceText: row.referenceText,
    direction,
    relationshipType: direction === "outgoing" ? "references" : "referenced_by",
    sourcePackName: row.fromPackName,
    sourceRecordType: row.fromRecordType,
    sourceDocumentType: row.fromDocumentType,
    sourceCategory: row.fromSourceCategory,
  };
}

export function sqliteRowCount(row: Record<string, unknown> | undefined, field = "total"): number {
  const value = row?.[field];
  return typeof value === "number" ? value : Number(value ?? 0);
}

export function backlinkTypeRank(recordType: string): number {
  if (recordType === "action") {
    return 0;
  }
  if (recordType === "feat") {
    return 1;
  }
  if (recordType === "classfeature") {
    return 2;
  }
  return 3;
}

export function sourceCategoryRank(sourceCategory: SourceCategory): number {
  if (sourceCategory === "core") {
    return 0;
  }
  if (sourceCategory === "rules") {
    return 1;
  }
  if (sourceCategory === "adventure") {
    return 2;
  }
  return 3;
}

export function extractQuestionRuleNames(question: string): string[] {
  const quoted = [...question.matchAll(/"([^"]+)"/g)]
    .map((match) => match[1]?.trim() ?? "")
    .filter((value) => value.length > 0);
  if (quoted.length > 0) {
    return quoted;
  }

  const cleaned = question
    .replace(/\?+$/g, "")
    .replace(/^(how|what|when|why|can)\s+(does|do|is|are)\s+/i, "")
    .replace(/^(how|what|when|why|can)\s+/i, "")
    .trim();

  const parts = cleaned
    .split(/\b(?:interplay with|interact with|works with|work with|with|vs\.?|versus|and)\b/i)
    .map((part) => part.trim().replace(/^[^A-Za-z0-9]+|[^A-Za-z0-9]+$/g, ""))
    .filter((part) => part.length > 0);

  return [...new Set(parts)].slice(0, 5);
}
