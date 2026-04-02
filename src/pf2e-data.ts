import { mkdir, readdir, readFile, rm, stat } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import * as sqliteVec from "sqlite-vec";

import {
  createEmbeddingProvider,
  DEFAULT_EMBEDDING_MODEL_ID,
  DEFAULT_EMBEDDING_REVISION,
  EmbeddingProvider,
} from "./embeddings.js";
import { DERIVED_TAG_CATALOG, deriveRecordTags, normalizeDerivedTag } from "./derived-tags.js";
import {
  categorySupportsSubcategory,
  classifyRecordCategory,
  extractSpellTraditions,
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "./categories.js";
import { DEFAULT_RANKING_CONFIG, RankingConfig, RankingConfigStore } from "./ranking-config.js";
import {
  CollectRuleQuestionContextInput,
  CollectRuleQuestionContextResult,
  DerivedTagCatalogEntry,
  EmbeddingConfig,
  FilterValueField,
  FilterValueResult,
  FilterValueQuery,
  LinkedRecordSummary,
  LookupOptions,
  LookupQuery,
  LookupResult,
  NormalizedRecord,
  PackInfo,
  PackManifestEntry,
  RuleGraphCollectionResult,
  RuleGraphResult,
  RuleReferenceEdge,
  SearchCategory,
  SearchFilters,
  SearchExplainResult,
  SearchRecordExplanation,
  SearchMode,
  SearchProfile,
  SearchResult,
  SearchScope,
  SearchSubcategory,
  SourceCategory,
} from "./types.js";
import {
  bigramDice,
  clampLimit,
  clampOffset,
  firstString,
  getNested,
  normalizeText,
  stripHtml,
  toStringArray,
  uniqueSorted,
} from "./utils.js";
import { buildLiteralQueryWeights, buildSearchQueryAnalysis } from "./search-query-analysis.js";

const execFileAsync = promisify(execFile);
const INDEX_SCHEMA_VERSION = 14;
const VEC_TEXT_NONE = "";
const VEC_INT_NONE = -1n;
const LOOKUP_LEXICAL_TOP_K = 100;

type LoadOptions = {
  indexPath?: string;
  embedding?: EmbeddingConfig;
  embeddingProviderFactory?: (
    config: EmbeddingConfig,
  ) => Promise<{ provider: EmbeddingProvider; warnings: string[] }>;
  rankingConfigStore?: RankingConfigStore;
  progressLogger?: (message: string) => void;
  progressStatusLogger?: (message: string) => void;
  vectorExtensionLoader?: (db: DatabaseSync) => void;
};

type SqlValue = string | number | bigint | Uint8Array | Buffer | null;

type CandidateRow = {
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
  glossaryFamily: string | null;
  additionalGlossaryFamiliesJson: string | null;
  sourcePath: string;
  isUnique: number;
  isSearchCanonical?: number;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  traditionsJson: string | null;
  spellKindsJson: string | null;
  rawJson?: string | null;
  searchText?: string | null;
  embeddingBlob?: Uint8Array | null;
};

type ReferenceEdgeRow = {
  fromRecordKey: string;
  toRecordKey: string;
  displayText: string | null;
  referenceText: string;
  fromPackName: string;
  fromRecordType: string;
  fromDocumentType: string;
  fromSourceCategory: SourceCategory;
};

type PackBuildInfo = Omit<PackInfo, "recordCount">;

type NormalizedIndexRecord = {
  recordKey: string;
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
  traits: string[];
  derivedTags: string[];
  publicationTitle: string | null;
  publicationRemaster: boolean;
  descriptionText: string | null;
  hasDescription: boolean;
  descriptionSnippet: string | null;
  sourceCategory: SourceCategory;
  folderId: string | null;
  glossaryFamily: string | null;
  additionalGlossaryFamilies: string[];
  sourcePath: string;
  isUnique: boolean;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  traditions: string[];
  spellKinds: string[];
  searchText: string;
};

type NormalizedSearchScope = {
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
};

type NormalizedSearchFilters = Omit<SearchFilters, "category" | "subcategory" | "scopes"> & {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  scopes?: NormalizedSearchScope[];
};

type ValueCountRow = {
  value: string;
  count: number;
};

type BuildSourceEntry = {
  pack: PackBuildInfo;
  filePath: string;
  raw: Record<string, unknown>;
  record: NormalizedIndexRecord | null;
  actorData: ActorIndexData | null;
  itemData: ItemIndexData | null;
  spellData: SpellIndexData | null;
  references: ExtractedReference[];
  resolvedReferences: ResolvedBuildReference[];
};

type ResolvedBuildReference = {
  targetRecordKey: string;
  targetRecord: NormalizedIndexRecord;
  displayText: string | null;
  referenceText: string;
};

type RecordAliasRow = {
  canonicalRecordKey: string;
  aliasText: string;
  normalizedAlias: string;
  sourceKind: string;
  sourceRef: string;
};

type RecordLegacyLinkRow = {
  canonicalRecordKey: string;
  legacyRecordKey: string;
  sourceKind: string;
  sourceRef: string;
};

type ActorIndexData = {
  size: string | null;
  languages: string[];
  speedTypes: string[];
  immunities: string[];
  resistances: string[];
  weaknesses: string[];
};

type ItemIndexData = {
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  usage: string | null;
  hands: number | null;
  damageTypes: string[];
  weaponGroup: string | null;
  armorGroup: string | null;
  actionCost: number | null;
};

type SpellIndexData = {
  actionCost: number | null;
  traditions: string[];
  spellKinds: string[];
  rangeText: string | null;
  rangeValue: number | null;
  saveType: string | null;
  areaType: string | null;
  damageTypes: string[];
};

const INTEGER_FORMATTER = new Intl.NumberFormat("en-US");
const PACK_PROGRESS_BAR_WIDTH = 24;
const PACK_PROGRESS_LOG_INTERVAL_MS = 5_000;

function encodeVector(vector: Float32Array): Buffer {
  return Buffer.from(vector.buffer.slice(vector.byteOffset, vector.byteOffset + vector.byteLength));
}

function decodeVector(blob: Uint8Array | null | undefined): Float32Array {
  if (!blob || blob.byteLength === 0) {
    return new Float32Array(0);
  }

  const copy = Uint8Array.from(blob);
  return new Float32Array(copy.buffer);
}

function cosineSimilarity(left: Float32Array, right: Float32Array): number {
  if (left.length === 0 || right.length === 0 || left.length !== right.length) {
    return 0;
  }

  let total = 0;
  for (let index = 0; index < left.length; index += 1) {
    total += left[index]! * right[index]!;
  }

  return total;
}

function normalizeVecText(value: string | null | undefined): string {
  return normalizeText(value ?? "") || VEC_TEXT_NONE;
}

function normalizeVecInteger(value: number | null | undefined): bigint {
  return value === null || value === undefined ? VEC_INT_NONE : BigInt(value);
}

function hashText(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  return hash >>> 0;
}

function isJsonRecord(filename: string): boolean {
  return filename.endsWith(".json") && filename !== "_folders.json";
}

async function walkJsonFiles(root: string): Promise<string[]> {
  const entries = await readdir(root, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry) => {
      const entryPath = path.join(root, entry.name);
      if (entry.isDirectory()) {
        return walkJsonFiles(entryPath);
      }

      if (entry.isFile() && isJsonRecord(entry.name)) {
        return [entryPath];
      }

      return [];
    }),
  );

  return files.flat();
}

async function directoryExists(targetPath: string): Promise<boolean> {
  try {
    const details = await stat(targetPath);
    return details.isDirectory();
  } catch {
    return false;
  }
}

function asNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function getPublicationTitle(raw: Record<string, unknown>): string | null {
  return firstString(
    getNested(raw, ["system", "publication", "title"]),
    getNested(raw, ["system", "details", "publication", "title"]),
  );
}

function getPublicationRemaster(raw: Record<string, unknown>): boolean {
  return getNested(raw, ["system", "publication", "remaster"]) === true ||
    getNested(raw, ["system", "details", "publication", "remaster"]) === true;
}

function getDescriptionMarkup(raw: Record<string, unknown>): string | null {
  return firstString(
    getNested(raw, ["system", "description", "value"]),
    getNested(raw, ["system", "details", "description"]),
    getNested(raw, ["system", "details", "publicNotes"]),
    getNested(raw, ["system", "details", "blurb"]),
  );
}

function getLevel(raw: Record<string, unknown>): number | null {
  return asNumber(
    getNested(raw, ["system", "level", "value"]) ?? getNested(raw, ["system", "details", "level", "value"]),
  );
}

function getDescriptionText(raw: Record<string, unknown>): string | null {
  return stripHtml(getDescriptionMarkup(raw));
}

function getTraits(raw: Record<string, unknown>): string[] {
  return uniqueSorted(toStringArray(getNested(raw, ["system", "traits", "value"])));
}

function getRarity(raw: Record<string, unknown>): string | null {
  const rarity = getNested(raw, ["system", "traits", "rarity"]);
  return typeof rarity === "string" && rarity.length > 0 ? rarity : null;
}

function parseSize(raw: Record<string, unknown>): string | null {
  return firstString(getNested(raw, ["system", "traits", "size", "value"]));
}

function parseLanguages(raw: Record<string, unknown>): string[] {
  return uniqueSorted(toStringArray(getNested(raw, ["system", "details", "languages", "value"])));
}

function parseSpeedTypes(raw: Record<string, unknown>): string[] {
  const values = ["land"];
  const otherSpeeds = getNested(raw, ["system", "attributes", "speed", "otherSpeeds"]);
  if (Array.isArray(otherSpeeds)) {
    for (const speed of otherSpeeds) {
      const speedType = firstString(getNested(speed, ["type"]));
      if (speedType) {
        values.push(speedType);
      }
    }
  }

  return uniqueSorted(values);
}

function parseTypedCollection(raw: Record<string, unknown>, pathSegments: string[]): string[] {
  const collection = getNested(raw, pathSegments);
  if (!Array.isArray(collection)) {
    return [];
  }

  return uniqueSorted(
    collection
      .map((entry) => firstString(getNested(entry, ["type"])))
      .filter((value): value is string => Boolean(value)),
  );
}

function parseActorIndexData(raw: Record<string, unknown>): ActorIndexData {
  return {
    size: parseSize(raw),
    languages: parseLanguages(raw),
    speedTypes: parseSpeedTypes(raw),
    immunities: parseTypedCollection(raw, ["system", "attributes", "immunities"]),
    resistances: parseTypedCollection(raw, ["system", "attributes", "resistances"]),
    weaknesses: parseTypedCollection(raw, ["system", "attributes", "weaknesses"]),
  };
}

function normalizePriceToCopper(rawValue: unknown): number | null {
  if (!rawValue || typeof rawValue !== "object") {
    return null;
  }

  const price = rawValue as Record<string, unknown>;
  const gp = asNumber(price.gp) ?? 0;
  const sp = asNumber(price.sp) ?? 0;
  const cp = asNumber(price.cp) ?? 0;
  const pp = asNumber(price.pp) ?? 0;
  const total = (pp * 1000) + (gp * 100) + (sp * 10) + cp;
  return total > 0 ? total : null;
}

function parseBulkValue(rawValue: unknown): number | null {
  if (typeof rawValue === "number") {
    return rawValue;
  }

  if (typeof rawValue !== "string") {
    return null;
  }

  if (rawValue === "L") {
    return 0.1;
  }

  const parsed = Number(rawValue);
  return Number.isFinite(parsed) ? parsed : null;
}

function parseHands(usage: string | null): number | null {
  if (!usage) {
    return null;
  }

  if (usage.includes("held-in-two-hands")) {
    return 2;
  }

  if (usage.includes("held-in-one-plus-hands")) {
    return 1;
  }

  if (usage.includes("held-in-one-hand")) {
    return 1;
  }

  return null;
}

function parseDamageTypes(raw: Record<string, unknown>): string[] {
  const damageTypes = new Set<string>();

  const directDamageType = firstString(getNested(raw, ["system", "damage", "damageType"]));
  if (directDamageType) {
    damageTypes.add(directDamageType);
  }

  const damageRolls = getNested(raw, ["system", "damageRolls"]);
  if (damageRolls && typeof damageRolls === "object") {
    for (const entry of Object.values(damageRolls as Record<string, unknown>)) {
      const damageType = firstString(getNested(entry, ["damageType"]));
      if (damageType) {
        damageTypes.add(damageType);
      }
    }
  }

  const spellDamage = getNested(raw, ["system", "damage"]);
  if (spellDamage && typeof spellDamage === "object") {
    for (const entry of Object.values(spellDamage as Record<string, unknown>)) {
      const damageType = firstString(getNested(entry, ["type"]));
      if (damageType) {
        damageTypes.add(damageType);
      }
    }
  }

  return [...damageTypes].sort((left, right) => left.localeCompare(right));
}

function parseItemCategory(raw: Record<string, unknown>): string | null {
  const recordType = firstString(raw.type);
  if (!recordType) {
    return null;
  }

  return recordType;
}

function parseActionCost(raw: Record<string, unknown>): number | null {
  return asNumber(getNested(raw, ["system", "actions", "value"]));
}

function parseItemIndexData(raw: Record<string, unknown>): ItemIndexData {
  const usage = firstString(getNested(raw, ["system", "usage", "value"]));
  return {
    itemCategory: parseItemCategory(raw),
    priceCp: normalizePriceToCopper(getNested(raw, ["system", "price", "value"])),
    bulkValue: parseBulkValue(getNested(raw, ["system", "bulk", "value"])),
    usage,
    hands: parseHands(usage),
    damageTypes: parseDamageTypes(raw),
    weaponGroup: firstString(getNested(raw, ["system", "group"])),
    armorGroup: firstString(getNested(raw, ["system", "group"])),
    actionCost: parseActionCost(raw),
  };
}

function parseRangeValue(raw: Record<string, unknown>): number | null {
  return asNumber(
    getNested(raw, ["system", "range", "value"]) ??
      getNested(raw, ["system", "range", "increment"]) ??
      getNested(raw, ["system", "area", "value"]),
  );
}

function extractSpellKinds(raw: Record<string, unknown>): string[] {
  const spellTraits = new Set(getTraits(raw).map((trait) => normalizeText(trait)).filter(Boolean));
  return ["focus", "ritual", "cantrip"].filter((kind) => spellTraits.has(kind));
}

function parseSpellIndexData(raw: Record<string, unknown>): SpellIndexData {
  return {
    actionCost: parseActionCost(raw),
    traditions: extractSpellTraditions(raw),
    spellKinds: extractSpellKinds(raw),
    rangeText: firstString(getNested(raw, ["system", "range", "value"])),
    rangeValue: parseRangeValue(raw),
    saveType: firstString(getNested(raw, ["system", "defense", "save", "statistic"])),
    areaType: firstString(getNested(raw, ["system", "area", "type"])),
    damageTypes: parseDamageTypes(raw),
  };
}

function buildSearchText(raw: Record<string, unknown>, base: { name: string; descriptionText: string | null; traits: string[] }): string {
  const chunks: string[] = [base.name, ...base.traits];
  if (base.descriptionText) {
    chunks.push(base.descriptionText);
  }

  const items = getNested(raw, ["items"]);
  if (Array.isArray(items)) {
    for (const entry of items) {
      if (!entry || typeof entry !== "object") {
        continue;
      }

      const item = entry as Record<string, unknown>;
      const itemName = firstString(item.name);
      const itemDescription = stripHtml(firstString(getNested(item, ["system", "description", "value"])));
      const itemTraits = getTraits(item);
      if (itemName) {
        chunks.push(itemName);
      }
      if (itemDescription) {
        chunks.push(itemDescription);
      }
      chunks.push(...itemTraits);
    }
  }

  return chunks
    .filter((value): value is string => Boolean(value))
    .join("\n")
    .trim();
}

function hasDescriptionText(descriptionText: string | null): boolean {
  return Boolean(descriptionText && descriptionText.trim().length > 0);
}

function buildDescriptionSnippet(descriptionText: string | null): string | null {
  if (!hasDescriptionText(descriptionText)) {
    return null;
  }

  const normalized = descriptionText!.replace(/\s+/g, " ").trim();
  const sentenceMatch = normalized.match(/^(.{1,240}?[.!?])(?:\s|$)/);
  if (sentenceMatch) {
    return sentenceMatch[1]!.trim();
  }

  if (normalized.length <= 240) {
    return normalized;
  }

  return `${normalized.slice(0, 237).trimEnd()}...`;
}

function isCorePublication(publicationTitle: string | null): boolean {
  const normalized = normalizeText(publicationTitle ?? "");
  return normalized === "pathfinder player core" ||
    normalized === "pathfinder player core 2" ||
    normalized === "pathfinder gm core" ||
    normalized === "pathfinder monster core" ||
    normalized === "pathfinder monster core 2" ||
    normalized === "pathfinder beginner box";
}

function isAdventurePublication(publicationTitle: string | null): boolean {
  const normalized = normalizeText(publicationTitle ?? "");
  if (!normalized) {
    return false;
  }

  return normalized.includes("adventure path") ||
    normalized.includes("pathfinder society") ||
    normalized.includes("quest") ||
    normalized.includes("one shot") ||
    normalized.includes("special") ||
    normalized.startsWith("pathfinder adventure ") ||
    /^pathfinder \d+ /.test(normalized);
}

function isAdventurePack(packName: string): boolean {
  const normalizedPack = normalizeText(packName);
  return normalizedPack.startsWith("pfs ") ||
    normalizedPack.includes("one shot") ||
    normalizedPack.includes("quest");
}

function isSocietyPublication(publicationTitle: string | null): boolean {
  const normalized = normalizeText(publicationTitle ?? "");
  return normalized.startsWith("pathfinder society scenario") ||
    normalized.startsWith("pathfinder society special") ||
    normalized.startsWith("pathfinder society intro");
}

function isSocietyPack(packName: string): boolean {
  return normalizeText(packName).startsWith("pfs ");
}

function hasScenarioScaleSuffix(name: string): boolean {
  return /\(\d+\s*-\s*\d+\)\s*$/.test(name.trim());
}

function isExcludedPackName(packName: string): boolean {
  const normalized = normalizeText(packName);
  return normalized.startsWith("pfs ") ||
    normalized === "pathfinder society boons" ||
    normalized === "macros" ||
    normalized === "action macros";
}

function isExcludedSocietyEffectPath(sourcePath: string): boolean {
  return sourcePath.replace(/\\/g, "/").includes("/campaign-effects/pathfinder-society/");
}

function isPfsBoonRecord(raw: Record<string, unknown>): boolean {
  return normalizeText(firstString(getNested(raw, ["system", "category"])) ?? "") === "pfsboon";
}

function shouldExcludeRecordFromIndex(pack: PackBuildInfo, sourcePath: string, raw: Record<string, unknown>): boolean {
  if (isExcludedPackName(pack.name) || isExcludedSocietyEffectPath(sourcePath) || isPfsBoonRecord(raw)) {
    return true;
  }

  const classification = classifyRecordCategory({
    documentType: pack.documentType,
    recordType: firstString(raw.type) ?? "unknown",
    packName: pack.name,
    sourcePath,
    traits: getTraits(raw),
    traditions: extractSpellTraditions(raw),
    raw,
  });
  return classification === null;
}

function getSourceCategory(packName: string, publicationTitle: string | null): SourceCategory {
  if (isCorePublication(publicationTitle)) {
    return "core";
  }

  if (isAdventurePublication(publicationTitle) || isAdventurePack(packName)) {
    return "adventure";
  }

  if (publicationTitle) {
    return "rules";
  }

  return "unknown";
}

function normalizeIndexRecord(pack: PackBuildInfo, sourcePath: string, raw: Record<string, unknown>): NormalizedIndexRecord {
  const id = firstString(raw._id);
  const name = firstString(raw.name);
  const recordType = firstString(raw.type) ?? "unknown";

  if (!id || !name) {
    throw new Error(`Invalid record in ${sourcePath}: expected both _id and name.`);
  }

  const rarity = getRarity(raw);
  const traits = getTraits(raw);
  const descriptionText = getDescriptionText(raw);
  const publicationTitle = getPublicationTitle(raw);
  const publicationRemaster = getPublicationRemaster(raw);
  const hasDescription = hasDescriptionText(descriptionText);
  const descriptionSnippet = buildDescriptionSnippet(descriptionText);
  const sourceCategory = getSourceCategory(pack.name, publicationTitle);
  const actorData = pack.documentType === "Actor" ? parseActorIndexData(raw) : null;
  const itemData = pack.documentType === "Item" ? parseItemIndexData(raw) : null;
  const spellData = recordType === "spell" ? parseSpellIndexData(raw) : null;
  const classification = classifyRecordCategory({
    documentType: pack.documentType,
    recordType,
    packName: pack.name,
    sourcePath,
    traits,
    traditions: spellData?.traditions ?? [],
    raw,
  });

  if (!classification) {
    throw new Error(`Record in ${sourcePath} did not map to a public search category.`);
  }

  return {
    recordKey: `${pack.name}:${id}`,
    id,
    name,
    normalizedName: normalizeText(name),
    type: recordType,
    category: classification.category,
    subcategory: classification.subcategory,
    packName: pack.name,
    packLabel: pack.label,
    documentType: pack.documentType,
    level: getLevel(raw),
    rarity,
    traits,
    derivedTags: [],
    publicationTitle,
    publicationRemaster,
    descriptionText,
    hasDescription,
    descriptionSnippet,
    sourceCategory,
    folderId: firstString(raw.folder),
    glossaryFamily: null,
    additionalGlossaryFamilies: [],
    sourcePath,
    isUnique: normalizeText(rarity ?? "") === "unique",
    size: actorData?.size ?? null,
    itemCategory: itemData?.itemCategory ?? null,
    priceCp: itemData?.priceCp ?? null,
    bulkValue: itemData?.bulkValue ?? null,
    actionCost: spellData?.actionCost ?? itemData?.actionCost ?? null,
    traditions: spellData?.traditions ?? [],
    spellKinds: spellData?.spellKinds ?? [],
    searchText: buildSearchText(raw, { name, descriptionText, traits }),
  };
}

function getPackAliasValues(pack: PackInfo, value: string): boolean {
  const normalized = normalizeText(value);
  return normalized === normalizeText(pack.name) || normalized === normalizeText(pack.label);
}

function packQualityScore(record: NormalizedRecord, rankingConfig: RankingConfig): number {
  const normalizedPack = normalizeText(record.packName);
  let score = 0;

  if (normalizedPack.includes("macro")) {
    score += rankingConfig.packQuality.macroPenalty;
  }

  if (normalizedPack.includes("glossary")) {
    score += rankingConfig.packQuality.glossaryPenalty;
  }

  if (normalizedPack.includes("effect")) {
    score += rankingConfig.packQuality.effectPenalty;
  }

  if (normalizedPack === "actions" || normalizedPack === "spells" || normalizedPack === "equipment" || normalizedPack === "feats") {
    score += rankingConfig.packQuality.utilityPackBoost;
  }

  return score;
}

function sourceQualityScore(record: NormalizedRecord, rankingConfig: RankingConfig): number {
  if (record.sourceCategory === "core") {
    return rankingConfig.sourceQuality.core;
  }
  if (record.sourceCategory === "rules") {
    return rankingConfig.sourceQuality.rules;
  }
  if (record.sourceCategory === "adventure") {
    return rankingConfig.sourceQuality.adventure;
  }

  return rankingConfig.sourceQuality.unknown;
}

function rarityPreferenceScore(record: NormalizedRecord, filters: SearchFilters, rankingConfig: RankingConfig): number {
  const normalizedRarity = normalizeText(record.rarity ?? "");
  let score = 0;

  if (normalizedRarity === "common" || normalizedRarity === "uncommon") {
    score += normalizedRarity === "common"
      ? rankingConfig.rarityPreference.common
      : rankingConfig.rarityPreference.uncommon;
  } else if (normalizedRarity === "rare") {
    score += rankingConfig.rarityPreference.rare;
  } else if (normalizedRarity === "unique") {
    score += rankingConfig.rarityPreference.unique;
  }

  if (record.isUnique && filters.query?.trim()) {
    score += rankingConfig.rarityPreference.themeQueryUniquePenalty;
  }

  return score;
}

function sourcePenaltyScore(record: NormalizedRecord, filters: SearchFilters, rankingConfig: RankingConfig): number {
  if (record.hasDescription || !filters.query?.trim()) {
    return 0;
  }

  let penalty = 0;
  if (isSocietyPublication(record.publicationTitle) || isSocietyPack(record.packName)) {
    penalty += rankingConfig.sourcePenalty.societyMetadataOnlyPenalty;
  }
  if (hasScenarioScaleSuffix(record.name)) {
    penalty += rankingConfig.sourcePenalty.scenarioScaleSuffixPenalty;
  }

  return penalty;
}

function scoreNameCandidate(query: string, normalizedName: string): number {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return 0.5;
  }

  if (normalizedName === normalizedQuery) {
    return 1;
  }

  if (normalizedName.startsWith(normalizedQuery)) {
    return 0.95;
  }

  if (normalizedName.includes(normalizedQuery)) {
    return 0.9;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const nameTokens = new Set(normalizedName.split(" ").filter(Boolean));
  const overlap = queryTokens.filter((token) => nameTokens.has(token)).length;
  const tokenScore = queryTokens.length > 0 ? overlap / queryTokens.length : 0;
  const dice = bigramDice(normalizedQuery, normalizedName);
  return Math.max(tokenScore * 0.8, dice * 0.75);
}

function nameScore(query: string, record: NormalizedRecord, aliases: string[] = []): number {
  let best = scoreNameCandidate(query, record.normalizedName);
  for (const alias of aliases) {
    best = Math.max(best, scoreNameCandidate(query, normalizeText(alias)));
  }
  return best;
}

function sortRecords(left: NormalizedRecord, right: NormalizedRecord): number {
  return left.name.localeCompare(right.name) || left.packLabel.localeCompare(right.packLabel) || left.id.localeCompare(right.id);
}

function queryTextScore(query: string, haystack: string): number {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return 0;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const targetTokens = new Set(normalizeText(haystack).split(" ").filter(Boolean));
  if (queryTokens.length === 0 || targetTokens.size === 0) {
    return 0;
  }

  const overlap = queryTokens.filter((token) => targetTokens.has(token)).length;
  return overlap / queryTokens.length;
}

function buildFtsQuery(query: string): string | null {
  const tokens = normalizeText(query).split(" ").filter(Boolean);
  if (tokens.length === 0) {
    return null;
  }

  return tokens.map((token) => `"${token}"*`).join(" OR ");
}

type RerankAdjustments = SearchRecordExplanation["rerankAdjustments"];
type HybridFusionProfileName = NonNullable<SearchExplainResult["fusionProfile"]>;
type HybridFusionProfile = RankingConfig["hybridFusion"]["balanced"];
type FusionConfigSummary = NonNullable<SearchExplainResult["fusionConfig"]>;
type LexicalSignal = {
  lexicalScore: number;
  matchedTraits: string[];
  matchedNameTokens: string[];
};
type LexicalRetrievalRow = {
  recordKey: string;
  rank: number;
};
type SemanticRetrievalRow = {
  recordKey: string;
  distance: number;
};

function tokenize(value: string): string[] {
  const normalized = normalizeText(value);
  return normalized ? normalized.split(" ").filter(Boolean) : [];
}

function scoreWeightedOverlap(
  weights: Map<string, number>,
  targetTokens: Iterable<string>,
  saturationWeight: number,
): { score: number; matchedTokens: string[] } {
  if (weights.size === 0) {
    return { score: 0, matchedTokens: [] };
  }

  let matchedWeight = 0;
  const matchedTokens: string[] = [];
  for (const token of new Set([...targetTokens].map((value) => normalizeText(value)).filter(Boolean))) {
    const weight = weights.get(token);
    if (!weight) {
      continue;
    }

    matchedWeight += weight;
    matchedTokens.push(token);
  }

  return {
    score: Math.min(1, matchedWeight / Math.max(1, saturationWeight)),
    matchedTokens: matchedTokens.sort((left, right) => left.localeCompare(right)),
  };
}

function buildTraitText(record: NormalizedRecord): string {
  return record.traits.join(" ");
}

function resolveSearchMode(filters: SearchFilters, context: "list" | "search"): SearchMode {
  if (context === "search") {
    if (filters.searchProfile === "lexical") {
      return filters.query?.trim() ? "lexical" : "structured";
    }

    if (filters.searchProfile === "balanced" || filters.searchProfile === "concept") {
      return filters.query?.trim() ? "hybrid" : "structured";
    }
  }

  if (context === "search" && filters.query?.trim()) {
    return "hybrid";
  }

  return "structured";
}

function hasStructuredFilterSignal(filters: SearchFilters): boolean {
  return Boolean(
    filters.pack ||
    filters.category ||
    filters.subcategory ||
    (filters.scopes && filters.scopes.length > 0) ||
    filters.levelMin !== undefined ||
    filters.levelMax !== undefined ||
    filters.rarity ||
    (filters.traitsAll && filters.traitsAll.length > 0) ||
    (filters.traitsAny && filters.traitsAny.length > 0) ||
    (filters.excludeTraits && filters.excludeTraits.length > 0) ||
    (filters.derivedTagsAll && filters.derivedTagsAll.length > 0) ||
    (filters.derivedTagsAny && filters.derivedTagsAny.length > 0) ||
    (filters.excludeDerivedTags && filters.excludeDerivedTags.length > 0) ||
    (filters.sources && filters.sources.length > 0) ||
    (filters.excludeSources && filters.excludeSources.length > 0) ||
    (filters.traditions && filters.traditions.length > 0) ||
    (filters.spellKinds && filters.spellKinds.length > 0) ||
    filters.publicationTitle ||
    filters.excludeUnique ||
    filters.excludeMissingDescription ||
    filters.size ||
    filters.priceMin !== undefined ||
    filters.priceMax !== undefined ||
    filters.actionCost !== undefined
  );
}

function resolveSearchProfile(
  filters: SearchFilters,
  context: "list" | "search",
  mode: SearchMode,
): SearchProfile | null {
  if (context === "list") {
    return null;
  }

  if (filters.query?.trim()) {
    if (filters.searchProfile) {
      return filters.searchProfile;
    }

    if (mode === "hybrid") {
      return "balanced";
    }

    return "lexical";
  }

  if (filters.nameQuery?.trim()) {
    return "lexical";
  }

  return null;
}

function resolveHybridFusionProfile(
  searchProfile: SearchProfile | null,
  mode: SearchMode,
  rankingConfig: RankingConfig,
): { profile: HybridFusionProfileName; config: HybridFusionProfile } | null {
  if (mode !== "hybrid") {
    return null;
  }

  if (searchProfile === "concept") {
    return {
      profile: "concept",
      config: rankingConfig.hybridFusion.concept,
    };
  }

  return {
    profile: "balanced",
    config: rankingConfig.hybridFusion.balanced,
  };
}

function buildRerankAdjustments(
  record: NormalizedRecord,
  filters: SearchFilters,
  rankingConfig: RankingConfig,
): RerankAdjustments {
  return {
    packQuality: packQualityScore(record, rankingConfig),
    sourceQuality: sourceQualityScore(record, rankingConfig),
    rarityPreference: rarityPreferenceScore(record, filters, rankingConfig),
    sourcePenalty: sourcePenaltyScore(record, filters, rankingConfig),
  };
}

function sumRerankAdjustments(adjustments: RerankAdjustments): number {
  return adjustments.packQuality +
    adjustments.sourceQuality +
    adjustments.rarityPreference +
    adjustments.sourcePenalty;
}

function buildLexicalSignal(
  record: NormalizedRecord,
  lexicalQuery: string,
  literalQueryWeights: ReturnType<typeof buildLiteralQueryWeights> | null,
  lexicalMatches: Map<string, number>,
  rankingConfig: RankingConfig,
): LexicalSignal {
  const ftsScore = lexicalQuery.length > 0 ? (lexicalMatches.get(record.recordKey) ?? 0) : 0;
  const descriptionTextScore =
    lexicalQuery.length > 0
      ? queryTextScore(lexicalQuery, record.descriptionText ?? "")
      : 0;
  const traitTextScore =
    lexicalQuery.length > 0
      ? queryTextScore(lexicalQuery, buildTraitText(record))
      : 0;
  const themeName = literalQueryWeights
    ? scoreWeightedOverlap(literalQueryWeights.nameWeights, tokenize(record.name), 1.5)
    : { score: 0, matchedTokens: [] };
  const themeTraits = literalQueryWeights
    ? scoreWeightedOverlap(literalQueryWeights.traitWeights, record.traits, 2)
    : { score: 0, matchedTokens: [] };
  const lexicalWeights = rankingConfig.lexicalChannels;
  const lexicalScoreBeforeNormalization =
    (ftsScore * lexicalWeights.fullTextSearch) +
    (descriptionTextScore * lexicalWeights.descriptionText) +
    (themeName.score * lexicalWeights.themeName) +
    (Math.max(traitTextScore, themeTraits.score) * lexicalWeights.themeTraits);
  const normalizationMultiplier = !record.hasDescription
    ? 1 / (1 - lexicalWeights.descriptionText)
    : 1;
  const lexicalScore = lexicalScoreBeforeNormalization * normalizationMultiplier;

  return {
    lexicalScore,
    matchedTraits: themeTraits.matchedTokens,
    matchedNameTokens: themeName.matchedTokens,
  };
}

function buildFusionConfigSummary(
  fusionProfile: HybridFusionProfileName | null,
  fusionConfig: HybridFusionProfile | null,
  rankingConfig: RankingConfig,
): FusionConfigSummary | null {
  if (!fusionProfile || !fusionConfig) {
    return null;
  }

  return {
    rrfK: rankingConfig.hybridFusion.rrfK,
    lexicalWeight: fusionConfig.lexicalWeight,
    semanticWeight: fusionConfig.semanticWeight,
    lexicalTopK: fusionConfig.lexicalTopK,
    semanticTopK: fusionConfig.semanticTopK,
  };
}

function compareOptionalRanks(left: number | null, right: number | null): number {
  if (left === right) {
    return 0;
  }
  if (left === null) {
    return 1;
  }
  if (right === null) {
    return -1;
  }

  return left - right;
}

function computeWeightedRrfScore(
  lexicalRank: number | null,
  semanticRank: number | null,
  fusionConfig: HybridFusionProfile,
  rrfK: number,
): number {
  const lexicalContribution = lexicalRank === null ? 0 : fusionConfig.lexicalWeight / (rrfK + lexicalRank);
  const semanticContribution = semanticRank === null ? 0 : fusionConfig.semanticWeight / (rrfK + semanticRank);

  // Scale rank-fusion output to roughly the same order of magnitude as the legacy rerank adjustments.
  return (lexicalContribution + semanticContribution) * (rrfK + 1);
}

function buildNormalizedRankScoreMap(recordKeysInRankOrder: string[]): Map<string, number> {
  const scores = new Map<string, number>();
  const total = recordKeysInRankOrder.length;
  recordKeysInRankOrder.forEach((recordKey, index) => {
    scores.set(recordKey, total <= 1 ? 1 : 1 - (index / (total - 1)));
  });
  return scores;
}

function buildRankMap(recordKeysInRankOrder: string[]): Map<string, number> {
  const ranks = new Map<string, number>();
  recordKeysInRankOrder.forEach((recordKey, index) => {
    ranks.set(recordKey, index + 1);
  });
  return ranks;
}

type ExtractedReference = {
  packName: string | null;
  recordLocator: string;
  displayText: string | null;
  referenceText: string;
};

const UUID_REFERENCE_PATTERN = /@UUID\[([^\]]+)\](?:\{([^}]+)\})?/g;
const COMPILED_REFERENCE_PATTERN = /^Compendium\.pf2e\.([^.]+)\.[^.]+\.([^.\]]+)$/i;
const TABLE_ROW_PATTERN = /<tr[^>]*>(.*?)<\/tr>/gis;
const TABLE_CELL_PATTERN = /<t[dh][^>]*>(.*?)<\/t[dh]>/gis;
const LIST_ITEM_PATTERN = /<li[^>]*>(.*?)<\/li>/gis;
const MIGRATION_RENAME_PATTERN = /Rename all uses and mentions of "([^"]+)" to "([^"]+)"/g;
const COMPILED_SOURCE_PATTERN = /^Compendium\.pf2e\.([^.]+)\.[^.]+\.([^.]+)$/i;

function extractRulesReferences(raw: Record<string, unknown>): ExtractedReference[] {
  const markup = getDescriptionMarkup(raw);
  if (!markup) {
    return [];
  }

  return extractUuidReferences(markup);
}

function extractUuidReferences(markup: string): ExtractedReference[] {
  const extracted: ExtractedReference[] = [];
  for (const match of markup.matchAll(UUID_REFERENCE_PATTERN)) {
    const referenceText = match[0]!;
    const target = match[1]!;
    const displayText = match[2] ?? null;
    const parsed = target.match(COMPILED_REFERENCE_PATTERN);
    if (!parsed) {
      continue;
    }

    extracted.push({
      packName: parsed[1] ?? null,
      recordLocator: parsed[2] ?? "",
      displayText,
      referenceText,
    });
  }

  return extracted;
}

function parseCompendiumLocator(locator: string): { packName: string; recordLocator: string } | null {
  const parsed = locator.match(COMPILED_SOURCE_PATTERN);
  if (!parsed) {
    return null;
  }

  return {
    packName: parsed[1] ?? "",
    recordLocator: parsed[2] ?? "",
  };
}

function extractItemCompendiumSources(raw: Record<string, unknown>): { packName: string; recordLocator: string }[] {
  const items = getNested(raw, ["items"]);
  if (!Array.isArray(items)) {
    return [];
  }

  const extracted: { packName: string; recordLocator: string }[] = [];
  for (const item of items) {
    if (!item || typeof item !== "object") {
      continue;
    }

    const compendiumSource = firstString(getNested(item as Record<string, unknown>, ["_stats", "compendiumSource"]));
    if (!compendiumSource) {
      continue;
    }

    const parsed = parseCompendiumLocator(compendiumSource);
    if (parsed) {
      extracted.push(parsed);
    }
  }

  return extracted;
}

function deriveGlossaryFamilyFromPath(filePath: string, packRoot: string): string | null {
  const relativePath = path.relative(packRoot, filePath);
  const segments = relativePath.split(path.sep).filter(Boolean);
  if (segments.length < 2) {
    return null;
  }

  const family = normalizeText(segments[0] ?? "").replace(/\s+/g, "-");
  return family || null;
}

function sortGlossaryFamilyCounts(counts: Map<string, number>): [string, number][] {
  return [...counts.entries()].sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
}

function assignGlossaryFamilies(
  entry: BuildSourceEntry & { record: NormalizedIndexRecord },
  recordsByPackAndId: Map<string, string>,
  glossaryFamilyByRecordKey: Map<string, string>,
): void {
  const familyCounts = new Map<string, number>();

  for (const reference of extractItemCompendiumSources(entry.raw)) {
    if (reference.packName !== "bestiary-family-ability-glossary") {
      continue;
    }

    const recordKey = recordsByPackAndId.get(buildPackAndIdKey(reference.packName, reference.recordLocator));
    if (!recordKey) {
      continue;
    }

    const family = glossaryFamilyByRecordKey.get(recordKey);
    if (!family) {
      continue;
    }

    familyCounts.set(family, (familyCounts.get(family) ?? 0) + 1);
  }

  const sortedFamilies = sortGlossaryFamilyCounts(familyCounts);
  entry.record.glossaryFamily = sortedFamilies[0]?.[0] ?? null;
  entry.record.additionalGlossaryFamilies = sortedFamilies.slice(1).map(([family]) => family);
}

function stripOuterHtml(value: string): string {
  return (stripHtml(value) ?? "").replace(/\s+/g, " ").trim();
}

function buildPackAndIdKey(packName: string, id: string): string {
  return `${normalizeText(packName)}:${normalizeText(id)}`;
}

function buildPackAndNameKey(packName: string, name: string): string {
  return `${normalizeText(packName)}:${normalizeText(name)}`;
}

function resolveTargetRecordKey(
  packName: string | null,
  locatorOrName: string,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
): string | null {
  const normalizedValue = normalizeText(locatorOrName);
  if (!normalizedValue) {
    return null;
  }

  if (packName) {
    const byId = recordsByPackAndId.get(buildPackAndIdKey(packName, locatorOrName));
    if (byId) {
      return byId;
    }

    const byPackName = recordsByPackAndName.get(buildPackAndNameKey(packName, locatorOrName)) ?? [];
    if (byPackName.length === 1) {
      return byPackName[0]!;
    }
  }

  const byName = recordsByName.get(normalizedValue) ?? [];
  return byName.length === 1 ? byName[0]! : null;
}

function resolveExtractedReference(
  reference: ExtractedReference,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): ResolvedBuildReference | null {
  if (!reference.packName || !reference.recordLocator) {
    return null;
  }

  const targetRecordKey = resolveTargetRecordKey(
    reference.packName,
    reference.recordLocator,
    recordsByPackAndId,
    recordsByPackAndName,
    recordsByName,
  );
  if (!targetRecordKey) {
    return null;
  }

  const targetRecord = recordsByKey.get(targetRecordKey);
  if (!targetRecord) {
    return null;
  }

  return {
    targetRecordKey,
    targetRecord,
    displayText: reference.displayText,
    referenceText: reference.referenceText,
  };
}

function extractFirstUuidReference(value: string): { packName: string; recordLocator: string } | null {
  const match = extractUuidReferences(value)[0];
  if (!match?.packName) {
    return null;
  }

  return {
    packName: match.packName,
    recordLocator: match.recordLocator,
  };
}

function resolveAliasSourceName(
  cellHtml: string,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): string | null {
  const directText = stripOuterHtml(cellHtml);
  if (directText && !directText.includes("@UUID[")) {
    return directText;
  }

  const target = extractFirstUuidReference(cellHtml);
  if (!target) {
    return directText || null;
  }

  const recordKey = resolveTargetRecordKey(
    target.packName,
    target.recordLocator,
    recordsByPackAndId,
    recordsByPackAndName,
    recordsByName,
  );
  return recordKey ? (recordsByKey.get(recordKey)?.name ?? null) : (directText || null);
}

function splitAliasListText(segment: string): string[] {
  return segment
    .replace(/\s+/g, " ")
    .replace(/\s*,?\s+and\s+/gi, ", ")
    .split(/\s*,\s*/)
    .map((value) => value.trim())
    .filter(Boolean);
}

function expandGroupedAliasText(aliasText: string, expectedCount: number): string[] | null {
  const match = aliasText.match(/^(.*?)\s*\((.*?)\)\s*$/);
  if (!match?.[1] || !match[2]) {
    return null;
  }

  const baseName = match[1].trim();
  const variants = splitAliasListText(match[2]);
  if (!baseName || variants.length !== expectedCount) {
    return null;
  }

  return variants.map((variant) => `${baseName} (${variant})`);
}

type ResolvedJournalTarget = {
  recordKey: string;
  record: NormalizedIndexRecord;
  displayText: string | null;
};

function extractResolvedJournalTargets(
  cellHtml: string,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): ResolvedJournalTarget[] {
  const resolvedTargets: ResolvedJournalTarget[] = [];
  const uuidReferences = extractUuidReferences(cellHtml);
  if (uuidReferences.length > 0) {
    for (const reference of uuidReferences) {
      if (!reference.packName) {
        continue;
      }

      const recordKey = resolveTargetRecordKey(
        reference.packName,
        reference.recordLocator,
        recordsByPackAndId,
        recordsByPackAndName,
        recordsByName,
      );
      const record = recordKey ? recordsByKey.get(recordKey) : null;
      if (!record || record.documentType === "JournalEntry") {
        continue;
      }

      resolvedTargets.push({
        recordKey: recordKey!,
        record,
        displayText: reference.displayText ? stripOuterHtml(reference.displayText) : null,
      });
    }

    return resolvedTargets;
  }

  const directText = stripOuterHtml(cellHtml);
  const recordKey = resolveTargetRecordKey(null, directText, recordsByPackAndId, recordsByPackAndName, recordsByName);
  const record = recordKey ? recordsByKey.get(recordKey) : null;
  if (!record || record.documentType === "JournalEntry") {
    return [];
  }

  return [{
    recordKey: recordKey!,
    record,
    displayText: null,
  }];
}

function extractIntroListAliases(
  listHtml: string,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): { aliasTexts: string[]; targetRecordKey: string } | null {
  const targets = extractResolvedJournalTargets(listHtml, recordsByPackAndId, recordsByPackAndName, recordsByName, recordsByKey);
  if (targets.length !== 1) {
    return null;
  }

  const plain = stripOuterHtml(listHtml);
  const oldSegment = plain.split(/\b(?:are merged into|is merged into|are now|is now)\b/i)[0]?.trim() ?? "";
  const aliasTexts = splitAliasListText(oldSegment);
  if (aliasTexts.length === 0) {
    return null;
  }

  return {
    aliasTexts,
    targetRecordKey: targets[0]!.recordKey,
  };
}

function hasConflictingJournalDisplayTarget(
  target: ResolvedJournalTarget,
  targetCount: number,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
): boolean {
  if (targetCount !== 1 || !target.displayText) {
    return false;
  }

  const displayRecordKey = resolveTargetRecordKey(
    null,
    target.displayText,
    recordsByPackAndId,
    recordsByPackAndName,
    recordsByName,
  );
  return Boolean(displayRecordKey && displayRecordKey !== target.recordKey);
}

function shouldIgnoreCompendiumAlias(aliasText: string, targetName: string): boolean {
  const normalizedAlias = normalizeText(aliasText);
  const normalizedTarget = normalizeText(targetName);
  if (!normalizedAlias || normalizedAlias === normalizedTarget) {
    return true;
  }

  if (/[\d]/.test(aliasText) || /\b(feet|foot|mile|miles|precise|imprecise|status|circumstance)\b/i.test(aliasText)) {
    return true;
  }

  if (/\([^)]*\)/.test(aliasText) || /^\([^)]*\)\s+/.test(targetName)) {
    return true;
  }

  return false;
}

function extractMigrationAliases(
  migrationSource: string,
  recordsByName: Map<string, string[]>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): RecordAliasRow[] {
  const aliases: RecordAliasRow[] = [];
  for (const match of migrationSource.matchAll(MIGRATION_RENAME_PATTERN)) {
    const aliasText = match[1]?.trim() ?? "";
    const targetName = match[2]?.trim() ?? "";
    const targetRecordKey = resolveTargetRecordKey(null, targetName, new Map(), new Map(), recordsByName);
    if (!aliasText || !targetRecordKey || !recordsByKey.has(targetRecordKey)) {
      continue;
    }

    aliases.push({
      canonicalRecordKey: targetRecordKey,
      aliasText,
      normalizedAlias: normalizeText(aliasText),
      sourceKind: "migration",
      sourceRef: "src/module/migration/migrations",
    });
  }

  return aliases;
}

function extractRemasterJournalAliases(
  journalRaw: Record<string, unknown>,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): { aliases: RecordAliasRow[]; legacyLinks: RecordLegacyLinkRow[] } {
  const aliases: RecordAliasRow[] = [];
  const legacyLinks: RecordLegacyLinkRow[] = [];
  const pages = getNested(journalRaw, ["pages"]);
  if (!Array.isArray(pages)) {
    return { aliases, legacyLinks };
  }

  const addAlias = (aliasText: string, canonicalRecordKey: string, sourceRef: string): void => {
    const normalizedAlias = normalizeText(aliasText);
    if (!normalizedAlias || normalizedAlias === recordsByKey.get(canonicalRecordKey)?.normalizedName) {
      return;
    }

    aliases.push({
      canonicalRecordKey,
      aliasText: aliasText.trim(),
      normalizedAlias,
      sourceKind: "remaster_journal",
      sourceRef,
    });

    const legacyCandidates = recordsByName.get(normalizedAlias) ?? [];
    for (const legacyRecordKey of legacyCandidates) {
      if (legacyRecordKey === canonicalRecordKey) {
        continue;
      }

      const legacyRecord = recordsByKey.get(legacyRecordKey);
      const canonicalRecord = recordsByKey.get(canonicalRecordKey);
      if (!legacyRecord || !canonicalRecord) {
        continue;
      }

      if (legacyRecord.publicationRemaster || !canonicalRecord.publicationRemaster) {
        continue;
      }

      legacyLinks.push({
        canonicalRecordKey,
        legacyRecordKey,
        sourceKind: "remaster_journal",
        sourceRef,
      });
    }
  };

  for (const page of pages) {
    if (!page || typeof page !== "object") {
      continue;
    }

    const pageName = firstString(getNested(page, ["name"])) ?? "journal-page";
    const content = firstString(getNested(page, ["text", "content"]));
    if (!content) {
      continue;
    }

    if (pageName === "Remaster Changes") {
      for (const listMatch of content.matchAll(LIST_ITEM_PATTERN)) {
        const listHtml = listMatch[1] ?? "";
        const parsedIntroList = extractIntroListAliases(
          listHtml,
          recordsByPackAndId,
          recordsByPackAndName,
          recordsByName,
          recordsByKey,
        );
        if (!parsedIntroList) {
          continue;
        }

        for (const aliasText of parsedIntroList.aliasTexts) {
          addAlias(aliasText, parsedIntroList.targetRecordKey, `journal:${pageName}`);
        }
      }
    }

    for (const rowMatch of content.matchAll(TABLE_ROW_PATTERN)) {
      const rowHtml = rowMatch[1] ?? "";
      const cells = [...rowHtml.matchAll(TABLE_CELL_PATTERN)].map((match) => match[1] ?? "");
      if (cells.length < 2) {
        continue;
      }

      const oldCell = cells[0] ?? "";
      const statusCell = cells.length >= 4 ? (cells[2] ?? "") : "Renamed";
      const newCell = cells[cells.length - 1] ?? "";
      const statusText = normalizeText(stripOuterHtml(statusCell));
      if (!(statusText === "renamed" || statusText === "merged" || statusText === "replaced")) {
        continue;
      }

      const targets = extractResolvedJournalTargets(newCell, recordsByPackAndId, recordsByPackAndName, recordsByName, recordsByKey);
      if (targets.length === 0) {
        continue;
      }

      if (targets.length === 1) {
        const oldName = resolveAliasSourceName(oldCell, recordsByPackAndId, recordsByPackAndName, recordsByName, recordsByKey);
        if (
          !oldName ||
          hasConflictingJournalDisplayTarget(
            targets[0]!,
            targets.length,
            recordsByPackAndId,
            recordsByPackAndName,
            recordsByName,
          )
        ) {
          continue;
        }

        addAlias(oldName, targets[0]!.recordKey, `journal:${pageName}`);
        continue;
      }

      const groupedAliases = expandGroupedAliasText(stripOuterHtml(oldCell), targets.length);
      if (!groupedAliases || targets.some((target) => !target.displayText)) {
        continue;
      }

      for (const [index, aliasText] of groupedAliases.entries()) {
        const target = targets[index];
        if (!target) {
          continue;
        }

        addAlias(aliasText, target.recordKey, `journal:${pageName}`);
      }
    }
  }

  return { aliases, legacyLinks };
}

function extractCompendiumSourceAliases(
  entries: BuildSourceEntry[],
  recordsByPackAndId: Map<string, string>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): RecordAliasRow[] {
  const aliases: RecordAliasRow[] = [];
  for (const entry of entries) {
    const items = getNested(entry.raw, ["items"]);
    if (!Array.isArray(items)) {
      continue;
    }

    for (const item of items) {
      if (!item || typeof item !== "object") {
        continue;
      }

      const embedded = item as Record<string, unknown>;
      const aliasText = firstString(embedded.name);
      const compendiumSource = firstString(getNested(embedded, ["_stats", "compendiumSource"]));
      if (!aliasText || !compendiumSource) {
        continue;
      }

      const target = parseCompendiumLocator(compendiumSource);
      if (!target) {
        continue;
      }

      const targetRecordKey = recordsByPackAndId.get(buildPackAndIdKey(target.packName, target.recordLocator));
      if (!targetRecordKey) {
        continue;
      }

      const targetRecord = recordsByKey.get(targetRecordKey);
      const embeddedRemaster = getPublicationRemaster(embedded);
      if (!targetRecord || embeddedRemaster || !targetRecord.publicationRemaster || shouldIgnoreCompendiumAlias(aliasText, targetRecord.name)) {
        continue;
      }

      aliases.push({
        canonicalRecordKey: targetRecordKey,
        aliasText,
        normalizedAlias: normalizeText(aliasText),
        sourceKind: "compendium_source",
        sourceRef: entry.record?.recordKey ?? entry.filePath,
      });
    }
  }

  return aliases;
}

function dedupeAliasRows(rows: RecordAliasRow[]): RecordAliasRow[] {
  const seen = new Set<string>();
  const deduped: RecordAliasRow[] = [];
  for (const row of rows) {
    const key = [
      row.canonicalRecordKey,
      row.normalizedAlias,
      row.sourceKind,
      row.sourceRef,
    ].join("\u0000");
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    deduped.push(row);
  }

  return deduped;
}

function dedupeLegacyLinkRows(rows: RecordLegacyLinkRow[]): RecordLegacyLinkRow[] {
  const seen = new Set<string>();
  const deduped: RecordLegacyLinkRow[] = [];
  for (const row of rows) {
    const key = [
      row.canonicalRecordKey,
      row.legacyRecordKey,
      row.sourceKind,
      row.sourceRef,
    ].join("\u0000");
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    deduped.push(row);
  }

  return deduped;
}

function rowToRecord(row: CandidateRow, raw: Record<string, unknown> | null = null): NormalizedRecord {
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
    glossaryFamily: row.glossaryFamily,
    additionalGlossaryFamilies: row.additionalGlossaryFamiliesJson
      ? (JSON.parse(row.additionalGlossaryFamiliesJson) as string[])
      : [],
    sourcePath: row.sourcePath,
    isUnique: Boolean(row.isUnique),
    size: row.size,
    itemCategory: row.itemCategory,
    priceCp: row.priceCp,
    bulkValue: row.bulkValue,
    actionCost: row.actionCost,
    traditions: row.traditionsJson ? (JSON.parse(row.traditionsJson) as string[]) : [],
    spellKinds: row.spellKindsJson ? (JSON.parse(row.spellKindsJson) as string[]) : [],
    aliases: [],
    legacyRecordLinks: [],
    raw: resolvedRaw,
  };
}

function buildPlaceholders(values: readonly unknown[]): string {
  return values.map(() => "?").join(", ");
}

function getLookupMatchType(query: string, record: NormalizedRecord | null): LookupResult["matchType"] {
  if (!record) {
    return "none";
  }

  const normalizedQuery = normalizeText(query);
  if (normalizeText(record.name) === normalizedQuery) {
    return normalizeText(query) === normalizeText(record.name) && query.trim() === record.name ? "exact" : "normalized_exact";
  }

  return "fuzzy";
}

function extractQuestionRuleNames(question: string): string[] {
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

function backlinkTypeRank(recordType: string): number {
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

function sourceCategoryRank(sourceCategory: SourceCategory): number {
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

function edgeRowToReferenceEdge(
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

function sqliteRowCount(row: Record<string, unknown> | undefined, field = "total"): number {
  const value = row?.[field];
  return typeof value === "number" ? value : Number(value ?? 0);
}

function formatInteger(value: number): string {
  return INTEGER_FORMATTER.format(value);
}

function renderProgressBar(completed: number, total: number, width = PACK_PROGRESS_BAR_WIDTH): string {
  if (total <= 0) {
    return `[${"-".repeat(width)}]`;
  }

  const boundedCompleted = Math.max(0, Math.min(completed, total));
  const filled = Math.max(0, Math.min(width, Math.round((boundedCompleted / total) * width)));
  return `[${"#".repeat(filled)}${"-".repeat(width - filled)}]`;
}

function formatPercentage(completed: number, total: number): string {
  if (total <= 0) {
    return "  0%";
  }

  return `${Math.round((Math.max(0, Math.min(completed, total)) / total) * 100)}`.padStart(3, " ") + "%";
}

function defaultIndexPath(manifestPath: string): string {
  return path.join(os.tmpdir(), `pf2e-mcp-${hashText(manifestPath).toString(16)}.sqlite`);
}

async function fileExists(targetPath: string): Promise<boolean> {
  try {
    await stat(targetPath);
    return true;
  } catch {
    return false;
  }
}

async function isGitCheckout(rootPath: string): Promise<boolean> {
  return fileExists(path.join(rootPath, ".git"));
}

async function computeSourceSignature(rootPath: string, manifestPath: string): Promise<string> {
  if (await isGitCheckout(rootPath)) {
    try {
      const [{ stdout: headStdout }, { stdout: statusStdout }, { stdout: untrackedStdout }] = await Promise.all([
        execFileAsync("git", ["-C", rootPath, "rev-parse", "HEAD"], { timeout: 10_000 }),
        execFileAsync("git", ["-C", rootPath, "status", "--porcelain", "--untracked-files=no"], { timeout: 10_000 }),
        execFileAsync(
          "git",
          ["-C", rootPath, "ls-files", "--others", "--exclude-standard", "--full-name", "--", "*.json", ":(glob)**/*.json"],
          { timeout: 10_000 },
        ),
      ]);
      const head = headStdout.trim();
      const dirty = statusStdout.trim();
      const untrackedJsonFiles = untrackedStdout
        .split(/\r?\n/)
        .map((filePath) => filePath.trim())
        .filter(Boolean)
        .map((filePath) => path.join(rootPath, filePath));
      const untrackedJsonSignature = await computeFileSignature(rootPath, untrackedJsonFiles);
      return `git:${head}:${dirty}:${untrackedJsonSignature}`;
    } catch {
      // Fall through to filesystem signature.
    }
  }

  const files = [manifestPath, ...(await walkJsonFiles(rootPath))];
  return `fs:${await computeFileSignature(rootPath, files)}`;
}

async function computeFileSignature(rootPath: string, filePaths: string[]): Promise<string> {
  const files = [...new Set(filePaths)].sort((left, right) => left.localeCompare(right));
  let hash = 2166136261;
  for (const filePath of files) {
    const details = await stat(filePath);
    const value = `${path.relative(rootPath, filePath)}:${details.size}:${Math.trunc(details.mtimeMs)}`;
    hash ^= hashText(value);
    hash = Math.imul(hash, 16777619);
  }

  return String(hash >>> 0);
}

function createSchema(db: DatabaseSync, embeddingDimensions: number): void {
  db.exec(`
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA temp_store = MEMORY;
    PRAGMA foreign_keys = ON;

    CREATE TABLE metadata (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );

    CREATE TABLE packs (
      name TEXT PRIMARY KEY,
      label TEXT NOT NULL,
      document_type TEXT NOT NULL,
      declared_path TEXT NOT NULL,
      resolved_path TEXT NOT NULL,
      record_count INTEGER NOT NULL
    );

    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      id TEXT NOT NULL,
      name TEXT NOT NULL,
      normalized_name TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      pack_name TEXT NOT NULL,
      pack_label TEXT NOT NULL,
      document_type TEXT NOT NULL,
      record_type TEXT NOT NULL,
      level INTEGER,
      rarity TEXT,
      traits_json TEXT NOT NULL,
      derived_tags_json TEXT NOT NULL,
      publication_title TEXT,
      publication_remaster INTEGER NOT NULL,
      description_text TEXT,
      has_description INTEGER NOT NULL,
      description_snippet TEXT,
      source_category TEXT NOT NULL,
      folder_id TEXT,
      glossary_family TEXT,
      additional_glossary_families_json TEXT NOT NULL,
      source_path TEXT NOT NULL,
      is_unique INTEGER NOT NULL,
      is_search_canonical INTEGER NOT NULL,
      search_text TEXT NOT NULL,
      raw_json TEXT NOT NULL
    );

    CREATE TABLE record_aliases (
      canonical_record_key TEXT NOT NULL,
      alias_text TEXT NOT NULL,
      normalized_alias TEXT NOT NULL,
      source_kind TEXT NOT NULL,
      source_ref TEXT NOT NULL,
      PRIMARY KEY (canonical_record_key, normalized_alias, source_kind, source_ref),
      FOREIGN KEY (canonical_record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE record_legacy_links (
      canonical_record_key TEXT NOT NULL,
      legacy_record_key TEXT NOT NULL,
      source_kind TEXT NOT NULL,
      source_ref TEXT NOT NULL,
      PRIMARY KEY (canonical_record_key, legacy_record_key, source_kind, source_ref),
      FOREIGN KEY (canonical_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
      FOREIGN KEY (legacy_record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE record_traits (
      record_key TEXT NOT NULL,
      trait TEXT NOT NULL,
      PRIMARY KEY (record_key, trait),
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL,
      PRIMARY KEY (record_key, tag),
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE actor_records (
      record_key TEXT PRIMARY KEY,
      size TEXT,
      languages_json TEXT NOT NULL,
      speed_types_json TEXT NOT NULL,
      immunities_json TEXT NOT NULL,
      resistances_json TEXT NOT NULL,
      weaknesses_json TEXT NOT NULL,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE item_records (
      record_key TEXT PRIMARY KEY,
      item_category TEXT,
      price_cp INTEGER,
      bulk_value REAL,
      usage_text TEXT,
      hands INTEGER,
      damage_types_json TEXT NOT NULL,
      weapon_group TEXT,
      armor_group TEXT,
      action_cost INTEGER,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE spell_records (
      record_key TEXT PRIMARY KEY,
      action_cost INTEGER,
      traditions_json TEXT NOT NULL,
      spell_kinds_json TEXT NOT NULL,
      range_text TEXT,
      range_value REAL,
      save_type TEXT,
      area_type TEXT,
      damage_types_json TEXT NOT NULL,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE TABLE embeddings (
      record_key TEXT PRIMARY KEY,
      dimensions INTEGER NOT NULL,
      vector_blob BLOB NOT NULL,
      FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE VIRTUAL TABLE record_embeddings USING vec0(
      record_key TEXT PRIMARY KEY,
      embedding FLOAT[${embeddingDimensions}],
      category TEXT partition key,
      subcategory TEXT,
      pack_name TEXT,
      pack_label TEXT,
      document_type TEXT,
      record_type TEXT,
      level INTEGER,
      rarity TEXT,
      source_category TEXT,
      publication_title TEXT,
      publication_remaster INTEGER,
      has_description INTEGER,
      is_unique INTEGER,
      size TEXT,
      item_category TEXT,
      price_cp INTEGER,
      action_cost INTEGER
    );

    CREATE TABLE reference_edges (
      from_record_key TEXT NOT NULL,
      to_record_key TEXT NOT NULL,
      display_text TEXT,
      reference_text TEXT NOT NULL,
      from_pack_name TEXT NOT NULL,
      from_record_type TEXT NOT NULL,
      from_document_type TEXT NOT NULL,
      from_source_category TEXT NOT NULL,
      PRIMARY KEY (from_record_key, to_record_key, reference_text),
      FOREIGN KEY (from_record_key) REFERENCES records(record_key) ON DELETE CASCADE
    );

    CREATE VIRTUAL TABLE records_fts USING fts5(
      record_key UNINDEXED,
      name,
      search_text
    );

    CREATE INDEX records_pack_idx ON records(pack_name);
    CREATE INDEX records_category_idx ON records(category);
    CREATE INDEX records_subcategory_idx ON records(subcategory);
    CREATE INDEX records_doc_type_idx ON records(document_type);
    CREATE INDEX records_record_type_idx ON records(record_type);
    CREATE INDEX records_level_idx ON records(level);
    CREATE INDEX records_rarity_idx ON records(rarity);
    CREATE INDEX records_unique_idx ON records(is_unique);
    CREATE INDEX records_publication_remaster_idx ON records(publication_remaster);
    CREATE INDEX records_search_canonical_idx ON records(is_search_canonical);
    CREATE INDEX records_has_description_idx ON records(has_description);
    CREATE INDEX records_source_category_idx ON records(source_category);
    CREATE INDEX records_glossary_family_idx ON records(glossary_family);
    CREATE INDEX record_aliases_normalized_alias_idx ON record_aliases(normalized_alias);
    CREATE INDEX record_legacy_links_canonical_idx ON record_legacy_links(canonical_record_key);
    CREATE INDEX record_traits_trait_idx ON record_traits(trait);
    CREATE INDEX record_derived_tags_tag_idx ON record_derived_tags(tag);
    CREATE INDEX actor_records_size_idx ON actor_records(size);
    CREATE INDEX item_records_category_idx ON item_records(item_category);
    CREATE INDEX item_records_price_idx ON item_records(price_cp);
    CREATE INDEX item_records_action_cost_idx ON item_records(action_cost);
    CREATE INDEX spell_records_action_cost_idx ON spell_records(action_cost);
    CREATE INDEX reference_edges_to_idx ON reference_edges(to_record_key);
    CREATE INDEX reference_edges_from_type_idx ON reference_edges(from_record_type);
    CREATE INDEX reference_edges_from_pack_idx ON reference_edges(from_pack_name);
    CREATE INDEX reference_edges_from_source_category_idx ON reference_edges(from_source_category);
  `);
}

async function buildIndex(
  db: DatabaseSync,
  rootPath: string,
  manifestPath: string,
  embeddingProvider: EmbeddingProvider,
  sourceSignature: string,
  progressLogger?: (message: string) => void,
  progressStatusLogger?: (message: string) => void,
): Promise<{ packs: PackInfo[]; warnings: string[]; recordCount: number }> {
  const manifestRaw = JSON.parse(await readFile(manifestPath, "utf8")) as { packs?: PackManifestEntry[] };
  const manifestPacks = Array.isArray(manifestRaw.packs) ? manifestRaw.packs : [];
  const includedManifestPacks = manifestPacks.filter((manifestPack) => !isExcludedPackName(manifestPack.name));

  const warnings: string[] = [];
  const packs: PackInfo[] = [];
  let recordCount = 0;
  let processedPackCount = 0;
  const sourceEntries: BuildSourceEntry[] = [];

  const insertPack = db.prepare(`
    INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
    VALUES (?, ?, ?, ?, ?, ?)
  `);
  const insertRecord = db.prepare(`
    INSERT INTO records (
      record_key, id, name, normalized_name, category, subcategory, pack_name, pack_label, document_type, record_type,
      level, rarity, traits_json, derived_tags_json, publication_title, publication_remaster, description_text, has_description, description_snippet,
      source_category, folder_id, glossary_family, additional_glossary_families_json, source_path, is_unique, is_search_canonical, search_text, raw_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertAlias = db.prepare(`
    INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
    VALUES (?, ?, ?, ?, ?)
  `);
  const insertLegacyLink = db.prepare(`
    INSERT INTO record_legacy_links (canonical_record_key, legacy_record_key, source_kind, source_ref)
    VALUES (?, ?, ?, ?)
  `);
  const insertTrait = db.prepare(`
    INSERT INTO record_traits (record_key, trait) VALUES (?, ?)
  `);
  const insertDerivedTag = db.prepare(`
    INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)
  `);
  const insertActor = db.prepare(`
    INSERT INTO actor_records (
      record_key, size, languages_json, speed_types_json, immunities_json, resistances_json, weaknesses_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
  `);
  const insertItem = db.prepare(`
    INSERT INTO item_records (
      record_key, item_category, price_cp, bulk_value, usage_text, hands, damage_types_json, weapon_group, armor_group, action_cost
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertSpell = db.prepare(`
    INSERT INTO spell_records (
      record_key, action_cost, traditions_json, spell_kinds_json, range_text, range_value, save_type, area_type, damage_types_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertEmbedding = db.prepare(`
    INSERT INTO embeddings (record_key, dimensions, vector_blob) VALUES (?, ?, ?)
  `);
  const insertVecEmbedding = db.prepare(`
    INSERT INTO record_embeddings (
      record_key,
      embedding,
      category,
      subcategory,
      pack_name,
      pack_label,
      document_type,
      record_type,
      level,
      rarity,
      source_category,
      publication_title,
      publication_remaster,
      has_description,
      is_unique,
      size,
      item_category,
      price_cp,
      action_cost
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertReferenceEdge = db.prepare(`
    INSERT OR IGNORE INTO reference_edges (
      from_record_key, to_record_key, display_text, reference_text, from_pack_name, from_record_type, from_document_type, from_source_category
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertFts = db.prepare(`
    INSERT INTO records_fts (record_key, name, search_text) VALUES (?, ?, ?)
  `);
  const insertMetadata = db.prepare(`
    INSERT INTO metadata (key, value) VALUES (?, ?)
  `);

  db.exec("BEGIN");
  try {
    progressLogger?.(
      `Building SQLite index from ${formatInteger(includedManifestPacks.length)} PF2E packs.`,
    );
    insertMetadata.run("schema_version", String(INDEX_SCHEMA_VERSION));
    insertMetadata.run("source_signature", sourceSignature);
    insertMetadata.run("embedding_provider", embeddingProvider.identity.provider);
    insertMetadata.run("embedding_model", embeddingProvider.identity.model);
    insertMetadata.run("embedding_revision", embeddingProvider.identity.revision ?? "");
    insertMetadata.run("embedding_dimensions", String(embeddingProvider.identity.dimensions));

    for (const manifestPack of manifestPacks) {
      const resolvedPath = await resolvePackPath(rootPath, manifestPack);
      if (!resolvedPath) {
        warnings.push(`Skipping pack ${manifestPack.name}: could not resolve a readable directory.`);
        continue;
      }

      const pack: PackBuildInfo = {
        name: manifestPack.name,
        label: manifestPack.label,
        documentType: manifestPack.type,
        declaredPath: manifestPack.path,
        resolvedPath,
      };

      if (isExcludedPackName(pack.name)) {
        continue;
      }

      processedPackCount += 1;

      let filePaths: string[];
      try {
        filePaths = await walkJsonFiles(pack.resolvedPath);
      } catch (error) {
        warnings.push(`Skipping pack ${pack.name}: ${(error as Error).message}`);
        continue;
      }

      let packRecordCount = 0;
      const progressInterval = Math.max(100, Math.ceil(filePaths.length / 10));
      let lastProgressLogTime = 0;
      let lastLoggedFileCount = 0;

      for (const [fileIndex, filePath] of filePaths.entries()) {
        const raw = JSON.parse(await readFile(filePath, "utf8")) as Record<string, unknown>;
        const shouldIndexRecord = !shouldExcludeRecordFromIndex(pack, filePath, raw);
        if (!shouldIndexRecord) {
          sourceEntries.push({
            pack,
            filePath,
            raw,
            record: null,
            actorData: null,
            itemData: null,
            spellData: null,
            references: [],
            resolvedReferences: [],
          });
          const processedFiles = fileIndex + 1;
          const now = Date.now();
          const shouldLogProgress = processedFiles === filePaths.length ||
            (processedFiles - lastLoggedFileCount) >= progressInterval ||
            (now - lastProgressLogTime) >= PACK_PROGRESS_LOG_INTERVAL_MS;

          if (shouldLogProgress) {
            progressStatusLogger?.(
              `[scan ${processedPackCount}/${includedManifestPacks.length}] ${pack.label} ${renderProgressBar(processedFiles, filePaths.length)} ${formatPercentage(processedFiles, filePaths.length)} (${formatInteger(processedFiles)}/${formatInteger(filePaths.length)} files, ${formatInteger(recordCount)} records discovered total).`,
            );
            lastProgressLogTime = now;
            lastLoggedFileCount = processedFiles;
          }
          continue;
        }
        const record = normalizeIndexRecord(pack, filePath, raw);
        sourceEntries.push({
          pack,
          filePath,
          raw,
          record,
          actorData: pack.documentType === "Actor" ? parseActorIndexData(raw) : null,
          itemData: pack.documentType === "Item" ? parseItemIndexData(raw) : null,
          spellData: record.type === "spell" ? parseSpellIndexData(raw) : null,
          references: extractRulesReferences(raw),
          resolvedReferences: [],
        });
        packRecordCount += 1;
        recordCount += 1;

        const processedFiles = fileIndex + 1;
        const now = Date.now();
        const shouldLogProgress = processedFiles === 1 ||
          processedFiles === filePaths.length ||
          (processedFiles - lastLoggedFileCount) >= progressInterval ||
          (now - lastProgressLogTime) >= PACK_PROGRESS_LOG_INTERVAL_MS;

        if (shouldLogProgress) {
          progressStatusLogger?.(
            `[scan ${processedPackCount}/${includedManifestPacks.length}] ${pack.label} ${renderProgressBar(processedFiles, filePaths.length)} ${formatPercentage(processedFiles, filePaths.length)} (${formatInteger(processedFiles)}/${formatInteger(filePaths.length)} files, ${formatInteger(recordCount)} records discovered total).`,
          );
          lastProgressLogTime = now;
          lastLoggedFileCount = processedFiles;
        }
      }

      if (packRecordCount === 0) {
        continue;
      }

      packs.push({ ...pack, recordCount: packRecordCount });
      insertPack.run(pack.name, pack.label, pack.documentType, pack.declaredPath, pack.resolvedPath, packRecordCount);
    }

    progressLogger?.("Finished scanning pack files. Resolving verified remaster aliases.");

    const indexedEntries = sourceEntries.filter((entry): entry is BuildSourceEntry & { record: NormalizedIndexRecord } => entry.record !== null);
    const recordsByKey = new Map(indexedEntries.map((entry) => [entry.record.recordKey, entry.record]));
    const recordsByPackAndId = new Map<string, string>();
    const recordsByPackAndName = new Map<string, string[]>();
    const recordsByName = new Map<string, string[]>();

    for (const entry of indexedEntries) {
      const record = entry.record;
      recordsByPackAndId.set(buildPackAndIdKey(record.packName, record.id), record.recordKey);

      const byPackAndNameKey = buildPackAndNameKey(record.packName, record.name);
      const samePackNames = recordsByPackAndName.get(byPackAndNameKey) ?? [];
      samePackNames.push(record.recordKey);
      recordsByPackAndName.set(byPackAndNameKey, samePackNames);

      const sameNames = recordsByName.get(record.normalizedName) ?? [];
      sameNames.push(record.recordKey);
      recordsByName.set(record.normalizedName, sameNames);
    }

    const glossaryFamilyByRecordKey = new Map<string, string>();
    for (const entry of indexedEntries) {
      if (entry.record.packName !== "bestiary-family-ability-glossary") {
        continue;
      }

      const family = deriveGlossaryFamilyFromPath(entry.filePath, entry.pack.resolvedPath);
      if (family) {
        glossaryFamilyByRecordKey.set(entry.record.recordKey, family);
      }
    }

    for (const entry of indexedEntries) {
      assignGlossaryFamilies(entry, recordsByPackAndId, glossaryFamilyByRecordKey);

      entry.resolvedReferences = entry.references
        .map((reference) => resolveExtractedReference(
          reference,
          recordsByPackAndId,
          recordsByPackAndName,
          recordsByName,
          recordsByKey,
        ))
        .filter((reference): reference is ResolvedBuildReference => Boolean(reference));

      entry.record.derivedTags = deriveRecordTags({
        name: entry.record.name,
        category: entry.record.category,
        subcategory: entry.record.subcategory,
        descriptionText: entry.record.descriptionText,
        traits: entry.record.traits,
        glossaryFamily: entry.record.glossaryFamily,
        additionalGlossaryFamilies: entry.record.additionalGlossaryFamilies,
        references: entry.resolvedReferences.map((reference) => ({
          recordKey: reference.targetRecordKey,
          packName: reference.targetRecord.packName,
          name: reference.targetRecord.name,
          category: reference.targetRecord.category,
          subcategory: reference.targetRecord.subcategory,
          traits: reference.targetRecord.traits,
        })),
      });
    }

    let aliasRows: RecordAliasRow[] = [];
    let legacyLinkRows: RecordLegacyLinkRow[] = [];

    const migrationDir = path.join(rootPath, "src", "module", "migration", "migrations");
    if (await directoryExists(migrationDir)) {
      const migrationFiles = (await readdir(migrationDir))
        .filter((fileName) => fileName.endsWith(".ts"))
        .sort((left, right) => left.localeCompare(right));
      for (const migrationFile of migrationFiles) {
        const migrationSource = await readFile(path.join(migrationDir, migrationFile), "utf8");
        aliasRows.push(...extractMigrationAliases(migrationSource, recordsByName, recordsByKey));
      }
    }

    for (const entry of sourceEntries) {
      if (normalizeText(firstString(entry.raw.name) ?? "") !== "remaster changes") {
        continue;
      }

      const extracted = extractRemasterJournalAliases(
        entry.raw,
        recordsByPackAndId,
        recordsByPackAndName,
        recordsByName,
        recordsByKey,
      );
      aliasRows.push(...extracted.aliases);
      legacyLinkRows.push(...extracted.legacyLinks);
    }

    aliasRows.push(...extractCompendiumSourceAliases(sourceEntries, recordsByPackAndId, recordsByKey));

    aliasRows = dedupeAliasRows(aliasRows);
    for (const alias of aliasRows) {
      if (alias.sourceKind !== "migration") {
        continue;
      }

      const targetRecord = recordsByKey.get(alias.canonicalRecordKey);
      if (!targetRecord?.publicationRemaster) {
        continue;
      }

      for (const legacyRecordKey of recordsByName.get(alias.normalizedAlias) ?? []) {
        if (legacyRecordKey === alias.canonicalRecordKey) {
          continue;
        }

        const legacyRecord = recordsByKey.get(legacyRecordKey);
        if (!legacyRecord || legacyRecord.publicationRemaster) {
          continue;
        }

        legacyLinkRows.push({
          canonicalRecordKey: alias.canonicalRecordKey,
          legacyRecordKey,
          sourceKind: "migration",
          sourceRef: alias.sourceRef,
        });
      }
    }
    legacyLinkRows = dedupeLegacyLinkRows(legacyLinkRows);

    progressLogger?.(
      `Resolved ${formatInteger(aliasRows.length)} verified aliases and ${formatInteger(legacyLinkRows.length)} legacy-to-remaster links.`,
    );

    const suppressedRecordKeys = new Set(legacyLinkRows.map((row) => row.legacyRecordKey));
    const aliasesByCanonicalRecordKey = new Map<string, string[]>();
    for (const alias of aliasRows) {
      const bucket = aliasesByCanonicalRecordKey.get(alias.canonicalRecordKey) ?? [];
      bucket.push(alias.aliasText);
      aliasesByCanonicalRecordKey.set(alias.canonicalRecordKey, uniqueSorted(bucket));
    }

    const canonicalEmbeddingCount = indexedEntries.filter((entry) => !suppressedRecordKeys.has(entry.record.recordKey)).length;
    const writeProgressInterval = Math.max(100, Math.ceil(indexedEntries.length / 10));
    let writtenEntryCount = 0;
    let embeddedRecordCount = 0;
    let lastWriteProgressLogTime = 0;
    let lastLoggedWriteCount = 0;

    progressLogger?.("Writing indexed records and canonical embeddings.");

    for (const entry of indexedEntries) {
      const record = entry.record;
      const aliasTexts = aliasesByCanonicalRecordKey.get(record.recordKey) ?? [];
      const isSearchCanonical = !suppressedRecordKeys.has(record.recordKey);
      const searchText = uniqueSorted([record.searchText, ...aliasTexts].filter(Boolean)).join("\n");

      insertRecord.run(
        record.recordKey,
        record.id,
        record.name,
        record.normalizedName,
        record.category,
        record.subcategory,
        record.packName,
        record.packLabel,
        record.documentType,
        record.type,
        record.level,
        record.rarity,
        JSON.stringify(record.traits),
        JSON.stringify(record.derivedTags),
        record.publicationTitle,
        record.publicationRemaster ? 1 : 0,
        record.descriptionText,
        record.hasDescription ? 1 : 0,
        record.descriptionSnippet,
        record.sourceCategory,
        record.folderId,
        record.glossaryFamily,
        JSON.stringify(record.additionalGlossaryFamilies),
        record.sourcePath,
        record.isUnique ? 1 : 0,
        isSearchCanonical ? 1 : 0,
        searchText,
        JSON.stringify(entry.raw),
      );

      for (const trait of record.traits) {
        insertTrait.run(record.recordKey, normalizeText(trait));
      }

      for (const tag of record.derivedTags) {
        insertDerivedTag.run(record.recordKey, normalizeDerivedTag(tag));
      }

      if (entry.actorData) {
        insertActor.run(
          record.recordKey,
          entry.actorData.size,
          JSON.stringify(entry.actorData.languages),
          JSON.stringify(entry.actorData.speedTypes),
          JSON.stringify(entry.actorData.immunities),
          JSON.stringify(entry.actorData.resistances),
          JSON.stringify(entry.actorData.weaknesses),
        );
      }

      if (entry.itemData) {
        insertItem.run(
          record.recordKey,
          entry.itemData.itemCategory,
          entry.itemData.priceCp,
          entry.itemData.bulkValue,
          entry.itemData.usage,
          entry.itemData.hands,
          JSON.stringify(entry.itemData.damageTypes),
          entry.itemData.weaponGroup,
          entry.itemData.armorGroup,
          entry.itemData.actionCost,
        );
      }

      if (entry.spellData) {
        insertSpell.run(
          record.recordKey,
          entry.spellData.actionCost,
          JSON.stringify(entry.spellData.traditions),
          JSON.stringify(entry.spellData.spellKinds),
          entry.spellData.rangeText,
          entry.spellData.rangeValue,
          entry.spellData.saveType,
          entry.spellData.areaType,
          JSON.stringify(entry.spellData.damageTypes),
        );
      }

      for (const reference of entry.resolvedReferences) {
        insertReferenceEdge.run(
          record.recordKey,
          reference.targetRecordKey,
          reference.displayText,
          reference.referenceText,
          record.packName,
          record.type,
          record.documentType,
          record.sourceCategory,
        );
      }

      if (!isSearchCanonical) {
        writtenEntryCount += 1;
        const now = Date.now();
        const shouldLogProgress = writtenEntryCount === indexedEntries.length ||
          (writtenEntryCount - lastLoggedWriteCount) >= writeProgressInterval ||
          (now - lastWriteProgressLogTime) >= PACK_PROGRESS_LOG_INTERVAL_MS;

        if (shouldLogProgress) {
          progressStatusLogger?.(
            `[write] Indexed records ${renderProgressBar(writtenEntryCount, indexedEntries.length)} ${formatPercentage(writtenEntryCount, indexedEntries.length)} (${formatInteger(writtenEntryCount)}/${formatInteger(indexedEntries.length)} records, ${formatInteger(embeddedRecordCount)}/${formatInteger(canonicalEmbeddingCount)} canonical embeddings).`,
          );
          lastWriteProgressLogTime = now;
          lastLoggedWriteCount = writtenEntryCount;
        }
        continue;
      }

      const embedding = await embeddingProvider.embed(searchText);
      const encodedEmbedding = encodeVector(embedding);
      insertEmbedding.run(record.recordKey, embeddingProvider.identity.dimensions, encodedEmbedding);
      insertVecEmbedding.run(
        record.recordKey,
        encodedEmbedding,
        normalizeVecText(record.category),
        normalizeVecText(record.subcategory),
        normalizeVecText(record.packName),
        normalizeVecText(record.packLabel),
        normalizeVecText(record.documentType),
        normalizeVecText(record.type),
        normalizeVecInteger(record.level),
        normalizeVecText(record.rarity),
        normalizeVecText(record.sourceCategory),
        normalizeVecText(record.publicationTitle),
        BigInt(record.publicationRemaster ? 1 : 0),
        BigInt(record.hasDescription ? 1 : 0),
        BigInt(record.isUnique ? 1 : 0),
        normalizeVecText(record.size),
        normalizeVecText(record.itemCategory),
        normalizeVecInteger(record.priceCp),
        normalizeVecInteger(record.actionCost),
      );
      insertFts.run(record.recordKey, record.name, searchText);
      writtenEntryCount += 1;
      embeddedRecordCount += 1;
      const now = Date.now();
      const shouldLogProgress = writtenEntryCount === indexedEntries.length ||
        (writtenEntryCount - lastLoggedWriteCount) >= writeProgressInterval ||
        (now - lastWriteProgressLogTime) >= PACK_PROGRESS_LOG_INTERVAL_MS;

      if (shouldLogProgress) {
        progressStatusLogger?.(
          `[write] Indexed records ${renderProgressBar(writtenEntryCount, indexedEntries.length)} ${formatPercentage(writtenEntryCount, indexedEntries.length)} (${formatInteger(writtenEntryCount)}/${formatInteger(indexedEntries.length)} records, ${formatInteger(embeddedRecordCount)}/${formatInteger(canonicalEmbeddingCount)} canonical embeddings).`,
        );
        lastWriteProgressLogTime = now;
        lastLoggedWriteCount = writtenEntryCount;
      }
    }

    for (const alias of aliasRows) {
      insertAlias.run(alias.canonicalRecordKey, alias.aliasText, alias.normalizedAlias, alias.sourceKind, alias.sourceRef);
    }

    for (const legacyLink of legacyLinkRows) {
      insertLegacyLink.run(legacyLink.canonicalRecordKey, legacyLink.legacyRecordKey, legacyLink.sourceKind, legacyLink.sourceRef);
    }

    db.exec("COMMIT");
  } catch (error) {
    db.exec("ROLLBACK");
    throw error;
  }

  packs.sort((left, right) => left.label.localeCompare(right.label));
  return { packs, warnings, recordCount };
}

function loadPacksFromIndex(db: DatabaseSync): PackInfo[] {
  const rows = db
    .prepare(`
      SELECT
        name,
        label,
        document_type AS documentType,
        declared_path AS declaredPath,
        resolved_path AS resolvedPath,
        record_count AS recordCount
      FROM packs
      ORDER BY label ASC
    `)
    .all() as Array<Record<string, unknown>>;

  return rows.map((row) => ({
    name: String(row.name),
    label: String(row.label),
    documentType: String(row.documentType),
    declaredPath: String(row.declaredPath),
    resolvedPath: String(row.resolvedPath),
    recordCount: Number(row.recordCount ?? 0),
  }));
}

function loadAliasesByRecordKey(db: DatabaseSync): Map<string, string[]> {
  const rows = db.prepare(`
    SELECT canonical_record_key AS canonicalRecordKey, alias_text AS aliasText
    FROM record_aliases
    ORDER BY canonical_record_key ASC, alias_text ASC
  `).all() as Array<{ canonicalRecordKey: string; aliasText: string }>;

  const aliasesByRecordKey = new Map<string, string[]>();
  for (const row of rows) {
    const bucket = aliasesByRecordKey.get(row.canonicalRecordKey) ?? [];
    bucket.push(row.aliasText);
    aliasesByRecordKey.set(row.canonicalRecordKey, uniqueSorted(bucket));
  }

  return aliasesByRecordKey;
}

function loadLegacyLinksByRecordKey(db: DatabaseSync): Map<string, LinkedRecordSummary[]> {
  const rows = db.prepare(`
    SELECT
      rll.canonical_record_key AS canonicalRecordKey,
      rll.legacy_record_key AS legacyRecordKey,
      records.name AS legacyName
    FROM record_legacy_links rll
    JOIN records ON records.record_key = rll.legacy_record_key
    ORDER BY rll.canonical_record_key ASC, records.name ASC, rll.legacy_record_key ASC
  `).all() as Array<{ canonicalRecordKey: string; legacyRecordKey: string; legacyName: string }>;

  const linksByRecordKey = new Map<string, LinkedRecordSummary[]>();
  for (const row of rows) {
    const bucket = linksByRecordKey.get(row.canonicalRecordKey) ?? [];
    bucket.push({
      recordKey: row.legacyRecordKey,
      name: row.legacyName,
    });
    linksByRecordKey.set(row.canonicalRecordKey, bucket);
  }

  return linksByRecordKey;
}

function readMetadata(db: DatabaseSync): Map<string, string> {
  const rows = db.prepare("SELECT key, value FROM metadata").all() as Array<{ key: string; value: string }>;
  return new Map(rows.map((row) => [row.key, row.value]));
}

function canReuseIndex(db: DatabaseSync, sourceSignature: string, embeddingProvider: EmbeddingProvider): boolean {
  return getIndexInvalidReason(db, sourceSignature, embeddingProvider) === null;
}

function getIndexInvalidReason(
  db: DatabaseSync,
  sourceSignature: string,
  embeddingProvider: EmbeddingProvider,
): string | null {
  try {
    const metadata = readMetadata(db);
    if (metadata.get("schema_version") !== String(INDEX_SCHEMA_VERSION)) {
      return "index schema version does not match the current code";
    }
    if (metadata.get("source_signature") !== sourceSignature) {
      return "PF2E source data changed since the index was built";
    }
    if (metadata.get("embedding_provider") !== embeddingProvider.identity.provider) {
      return "embedding provider changed since the index was built";
    }
    if (metadata.get("embedding_model") !== embeddingProvider.identity.model) {
      return "embedding model changed since the index was built";
    }
    if (metadata.get("embedding_revision") !== (embeddingProvider.identity.revision ?? "")) {
      return "embedding model revision changed since the index was built";
    }
    if (metadata.get("embedding_dimensions") !== String(embeddingProvider.identity.dimensions)) {
      return "embedding dimensions changed since the index was built";
    }
    return null;
  } catch {
    return "index metadata could not be read";
  }
}

async function removeIndexFiles(indexPath: string): Promise<void> {
  await rm(indexPath, { force: true });
  await rm(`${indexPath}-wal`, { force: true });
  await rm(`${indexPath}-shm`, { force: true });
}

function loadRequiredVectorExtension(
  db: DatabaseSync,
  loader: NonNullable<LoadOptions["vectorExtensionLoader"]> = sqliteVec.load,
): void {
  try {
    loader(db);
    db.enableLoadExtension(false);
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to load required sqlite-vec extension. Fix the installation and retry startup. Underlying error: ${reason}`);
  }
}

function openDatabase(indexPath: string, options: { vectorExtensionLoader?: NonNullable<LoadOptions["vectorExtensionLoader"]> } = {}): DatabaseSync {
  const db = new DatabaseSync(indexPath, { allowExtension: true });
  loadRequiredVectorExtension(db, options.vectorExtensionLoader);
  return db;
}

function defaultEmbeddingConfig(indexPath: string): EmbeddingConfig {
  return {
    provider: "hf-local",
    modelId: DEFAULT_EMBEDDING_MODEL_ID,
    modelRevision: DEFAULT_EMBEDDING_REVISION,
    cachePath: path.join(path.dirname(indexPath), "hf-models"),
    localModelPath: null,
  };
}

function buildMissingIndexError(indexPath: string): Error {
  return new Error(
    `PF2E index not found at ${indexPath}. Run 'npm run refresh-index' or 'npm run refresh-external' before starting the MCP server.`,
  );
}

function buildStaleIndexError(indexPath: string, reason: string): Error {
  return new Error(
    `PF2E index at ${indexPath} is stale: ${reason}. Run 'npm run refresh-index' or 'npm run refresh-external' before starting the MCP server.`,
  );
}

async function resolvePackPath(rootPath: string, pack: PackManifestEntry): Promise<string | null> {
  const candidates = [
    path.join(rootPath, pack.path),
    path.join(rootPath, "packs", "pf2e", pack.name),
    path.join(rootPath, pack.path.replace(/^packs\//, "packs/pf2e/")),
  ];

  for (const candidate of candidates) {
    const normalized = path.normalize(candidate);
    if (await directoryExists(normalized)) {
      return normalized;
    }
  }

  return null;
}

function appendWhereClause(sql: string[], params: SqlValue[], clause: string, ...values: SqlValue[]): void {
  sql.push(clause);
  params.push(...values);
}

function normalizeSearchScope(scope: SearchScope): NormalizedSearchScope {
  const category = normalizeSearchCategory(scope.category);
  if (!category) {
    throw new Error(getSearchCategoryErrorMessage(String(scope.category)));
  }

  const subcategories = scope.subcategories?.map((subcategory) => {
    const canonicalSubcategory = normalizeSearchSubcategory(subcategory);
    if (!canonicalSubcategory) {
      throw new Error(getSearchSubcategoryErrorMessage(String(subcategory)));
    }
    return canonicalSubcategory;
  });

  const uniqueSubcategories = subcategories
    ? uniqueSorted(subcategories) as SearchSubcategory[]
    : undefined;

  return {
    category,
    subcategories: uniqueSubcategories && uniqueSubcategories.length > 0 ? uniqueSubcategories : undefined,
  };
}

function resolveEffectiveCategory(filters: Pick<NormalizedSearchFilters, "category" | "subcategory" | "scopes" | "traditions" | "spellKinds">): SearchCategory | null {
  if (filters.scopes && filters.scopes.length > 0) {
    return null;
  }

  const inferredCategoryFromSubcategory = !filters.category && filters.subcategory
    ? getCategoryForSubcategory(filters.subcategory)
    : null;
  const hasSpellFacetFilter = (filters.traditions?.length ?? 0) > 0 || (filters.spellKinds?.length ?? 0) > 0;
  return filters.category ?? inferredCategoryFromSubcategory ?? (hasSpellFacetFilter ? "spell" : null);
}

function appendScopedCategoryClauses(
  sql: string[],
  params: SqlValue[],
  scopes: NormalizedSearchScope[],
  renderTerm: (category: SearchCategory, subcategories: SearchSubcategory[] | undefined) => { clause: string; values: SqlValue[] },
): void {
  const renderedScopes = scopes.map((scope) => renderTerm(scope.category, scope.subcategories));
  appendWhereClause(
    sql,
    params,
    `AND (${renderedScopes.map((entry) => entry.clause).join(" OR ")})`,
    ...renderedScopes.flatMap((entry) => entry.values),
  );
}

function applySearchFilterClauses(
  sql: string[],
  params: SqlValue[],
  filters: NormalizedSearchFilters,
  aliases: {
    records: string;
    actor: string;
    item: string;
    spell: string;
  },
  options: {
    recordKeys?: string[];
  } = {},
): void {
  const recordAlias = aliases.records;
  const actorAlias = aliases.actor;
  const itemAlias = aliases.item;
  const spellAlias = aliases.spell;

  appendWhereClause(sql, params, `AND ${recordAlias}.is_search_canonical = 1`);

  if (options.recordKeys && options.recordKeys.length > 0) {
    const placeholders = buildPlaceholders(options.recordKeys);
    appendWhereClause(sql, params, `AND ${recordAlias}.record_key IN (${placeholders})`, ...options.recordKeys);
  }

  if (filters.pack) {
    appendWhereClause(
      sql,
      params,
      `AND (LOWER(${recordAlias}.pack_name) = LOWER(?) OR LOWER(${recordAlias}.pack_label) = LOWER(?))`,
      filters.pack,
      filters.pack,
    );
  }

  if (filters.scopes && filters.scopes.length > 0) {
    appendScopedCategoryClauses(sql, params, filters.scopes, (category, subcategories) => {
      if (!subcategories || subcategories.length === 0) {
        return {
          clause: `LOWER(${recordAlias}.category) = LOWER(?)`,
          values: [category],
        };
      }

      const placeholders = subcategories.map(() => "?").join(", ");
      return {
        clause: `(LOWER(${recordAlias}.category) = LOWER(?) AND LOWER(COALESCE(${recordAlias}.subcategory, '')) IN (${placeholders}))`,
        values: [category, ...subcategories.map((subcategory) => normalizeText(subcategory))],
      };
    });
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory) {
      appendWhereClause(sql, params, `AND LOWER(${recordAlias}.category) = LOWER(?)`, effectiveCategory);
    }

    if (filters.subcategory) {
      appendWhereClause(sql, params, `AND LOWER(COALESCE(${recordAlias}.subcategory, '')) = LOWER(?)`, filters.subcategory);
    }
  }

  if (filters.levelMin !== undefined) {
    appendWhereClause(sql, params, `AND ${recordAlias}.level >= ?`, filters.levelMin);
  }

  if (filters.levelMax !== undefined) {
    appendWhereClause(sql, params, `AND ${recordAlias}.level <= ?`, filters.levelMax);
  }

  if (filters.rarity) {
    appendWhereClause(sql, params, `AND LOWER(COALESCE(${recordAlias}.rarity, '')) = LOWER(?)`, filters.rarity);
  }

  if (filters.publicationTitle) {
    appendWhereClause(sql, params, `AND LOWER(COALESCE(${recordAlias}.publication_title, '')) LIKE LOWER(?)`, `%${filters.publicationTitle}%`);
  }

  if (filters.excludeUnique) {
    appendWhereClause(sql, params, `AND ${recordAlias}.is_unique = 0`);
  }

  if (filters.excludeMissingDescription) {
    appendWhereClause(sql, params, `AND ${recordAlias}.has_description = 1`);
  }

  if (filters.sources && filters.sources.length > 0) {
    const placeholders = filters.sources.map(() => "?").join(", ");
    appendWhereClause(sql, params, `AND ${recordAlias}.source_category IN (${placeholders})`, ...filters.sources);
  }

  if (filters.excludeSources && filters.excludeSources.length > 0) {
    const placeholders = filters.excludeSources.map(() => "?").join(", ");
    appendWhereClause(sql, params, `AND ${recordAlias}.source_category NOT IN (${placeholders})`, ...filters.excludeSources);
  }

  if (filters.size) {
    appendWhereClause(sql, params, `AND LOWER(COALESCE(${actorAlias}.size, '')) = LOWER(?)`, filters.size);
  }

  if (filters.priceMin !== undefined) {
    appendWhereClause(sql, params, `AND ${itemAlias}.price_cp >= ?`, filters.priceMin);
  }

  if (filters.priceMax !== undefined) {
    appendWhereClause(sql, params, `AND ${itemAlias}.price_cp <= ?`, filters.priceMax);
  }

  if (filters.actionCost !== undefined) {
    appendWhereClause(sql, params, `AND COALESCE(${spellAlias}.action_cost, ${itemAlias}.action_cost) = ?`, filters.actionCost);
  }

  const normalizedTraditions = (filters.traditions ?? [])
    .map((tradition) => normalizeText(tradition))
    .filter(Boolean);
  if (normalizedTraditions.length > 0) {
    const placeholders = normalizedTraditions.map(() => "?").join(", ");
    appendWhereClause(
      sql,
      params,
      `AND EXISTS (
        SELECT 1
        FROM json_each(COALESCE(${spellAlias}.traditions_json, '[]')) AS tradition
        WHERE LOWER(tradition.value) IN (${placeholders})
      )`,
      ...normalizedTraditions,
    );
  }

  const normalizedSpellKinds = (filters.spellKinds ?? [])
    .map((spellKind) => normalizeText(spellKind))
    .filter(Boolean);
  if (normalizedSpellKinds.length > 0) {
    const placeholders = normalizedSpellKinds.map(() => "?").join(", ");
    appendWhereClause(
      sql,
      params,
      `AND EXISTS (
        SELECT 1
        FROM json_each(COALESCE(${spellAlias}.spell_kinds_json, '[]')) AS spell_kind
        WHERE LOWER(spell_kind.value) IN (${placeholders})
      )`,
      ...normalizedSpellKinds,
    );
  }

  const includedTraitsAll = (filters.traitsAll ?? [])
    .map((trait) => normalizeText(trait))
    .filter(Boolean);
  for (const trait of includedTraitsAll) {
    appendWhereClause(
      sql,
      params,
      `AND EXISTS (SELECT 1 FROM record_traits rt WHERE rt.record_key = ${recordAlias}.record_key AND rt.trait = ?)`,
      trait,
    );
  }

  const includedTraitsAny = (filters.traitsAny ?? [])
    .map((trait) => normalizeText(trait))
    .filter(Boolean);
  if (includedTraitsAny.length > 0) {
    const placeholders = includedTraitsAny.map(() => "?").join(", ");
    appendWhereClause(
      sql,
      params,
      `AND EXISTS (SELECT 1 FROM record_traits rt WHERE rt.record_key = ${recordAlias}.record_key AND rt.trait IN (${placeholders}))`,
      ...includedTraitsAny,
    );
  }

  const excludedTraits = (filters.excludeTraits ?? [])
    .map((trait) => normalizeText(trait))
    .filter(Boolean);
  if (excludedTraits.length > 0) {
    const placeholders = excludedTraits.map(() => "?").join(", ");
    appendWhereClause(
      sql,
      params,
      `AND NOT EXISTS (SELECT 1 FROM record_traits rt WHERE rt.record_key = ${recordAlias}.record_key AND rt.trait IN (${placeholders}))`,
      ...excludedTraits,
    );
  }

  const includedDerivedTagsAll = (filters.derivedTagsAll ?? [])
    .map((tag) => normalizeDerivedTag(tag))
    .filter(Boolean);
  for (const tag of includedDerivedTagsAll) {
    appendWhereClause(
      sql,
      params,
      `AND EXISTS (SELECT 1 FROM record_derived_tags rdt WHERE rdt.record_key = ${recordAlias}.record_key AND rdt.tag = ?)`,
      tag,
    );
  }

  const includedDerivedTagsAny = (filters.derivedTagsAny ?? [])
    .map((tag) => normalizeDerivedTag(tag))
    .filter(Boolean);
  if (includedDerivedTagsAny.length > 0) {
    const placeholders = includedDerivedTagsAny.map(() => "?").join(", ");
    appendWhereClause(
      sql,
      params,
      `AND EXISTS (SELECT 1 FROM record_derived_tags rdt WHERE rdt.record_key = ${recordAlias}.record_key AND rdt.tag IN (${placeholders}))`,
      ...includedDerivedTagsAny,
    );
  }

  const excludedDerivedTags = (filters.excludeDerivedTags ?? [])
    .map((tag) => normalizeDerivedTag(tag))
    .filter(Boolean);
  if (excludedDerivedTags.length > 0) {
    const placeholders = excludedDerivedTags.map(() => "?").join(", ");
    appendWhereClause(
      sql,
      params,
      `AND NOT EXISTS (SELECT 1 FROM record_derived_tags rdt WHERE rdt.record_key = ${recordAlias}.record_key AND rdt.tag IN (${placeholders}))`,
      ...excludedDerivedTags,
    );
  }
}

function buildCandidateQuery(
  filters: NormalizedSearchFilters,
  includeSearchText = false,
  includeEmbedding = false,
  options: { recordKeys?: string[] } = {},
): { sql: string; params: SqlValue[] } {
  const fields = [
    "r.record_key AS recordKey",
    "r.id AS id",
    "r.name AS name",
    "r.normalized_name AS normalizedName",
    "r.record_type AS type",
    "r.category AS category",
    "r.subcategory AS subcategory",
    "r.pack_name AS packName",
    "r.pack_label AS packLabel",
    "r.document_type AS documentType",
    "r.level AS level",
    "r.rarity AS rarity",
    "r.traits_json AS traitsJson",
    "r.derived_tags_json AS derivedTagsJson",
    "r.publication_title AS publicationTitle",
    "r.publication_remaster AS publicationRemaster",
    "r.description_text AS descriptionText",
    "r.has_description AS hasDescription",
    "r.description_snippet AS descriptionSnippet",
    "r.source_category AS sourceCategory",
    "r.folder_id AS folderId",
    "r.glossary_family AS glossaryFamily",
    "r.additional_glossary_families_json AS additionalGlossaryFamiliesJson",
    "r.source_path AS sourcePath",
    "r.is_unique AS isUnique",
    "r.is_search_canonical AS isSearchCanonical",
    "a.size AS size",
    "i.item_category AS itemCategory",
    "i.price_cp AS priceCp",
    "i.bulk_value AS bulkValue",
    "COALESCE(s.action_cost, i.action_cost) AS actionCost",
    "s.traditions_json AS traditionsJson",
    "s.spell_kinds_json AS spellKindsJson",
  ];

  if (includeSearchText) {
    fields.push("r.search_text AS searchText");
  }

  if (includeEmbedding) {
    fields.push("e.vector_blob AS embeddingBlob");
  }

  const sql = [
    `SELECT ${fields.join(", ")}`,
    "FROM records r",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
  ];

  if (includeEmbedding) {
    sql.push("LEFT JOIN embeddings e ON e.record_key = r.record_key");
  }

  sql.push("WHERE 1 = 1");
  const params: SqlValue[] = [];
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  }, options);

  return { sql: sql.join("\n"), params };
}

function buildFilterValueQuery(field: FilterValueField, filters: NormalizedSearchFilters): { sql: string; params: SqlValue[] } {
  const joins = [
    "FROM records r",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
  ];
  const sql: string[] = [];
  const params: SqlValue[] = [];
  const postFilterClauses: string[] = [];
  let valueExpression = "";

  switch (field) {
    case "traits":
      joins.push("JOIN record_traits rt ON rt.record_key = r.record_key");
      valueExpression = "rt.trait";
      break;
    case "derivedTags":
      joins.push("JOIN record_derived_tags rdt ON rdt.record_key = r.record_key");
      valueExpression = "rdt.tag";
      break;
    case "rarity":
      valueExpression = "r.rarity";
      postFilterClauses.push("AND r.rarity IS NOT NULL AND r.rarity <> ''");
      break;
    case "size":
      valueExpression = "a.size";
      postFilterClauses.push("AND a.size IS NOT NULL AND a.size <> ''");
      break;
    case "publicationTitle":
      valueExpression = "r.publication_title";
      postFilterClauses.push("AND r.publication_title IS NOT NULL AND r.publication_title <> ''");
      break;
    case "traditions":
      joins.push("JOIN json_each(COALESCE(s.traditions_json, '[]')) AS tradition");
      valueExpression = "tradition.value";
      break;
    case "spellKinds":
      joins.push("JOIN json_each(COALESCE(s.spell_kinds_json, '[]')) AS spell_kind");
      valueExpression = "spell_kind.value";
      break;
    case "sources":
      valueExpression = "r.source_category";
      break;
    case "categories":
      valueExpression = "r.category";
      break;
    case "subcategories":
      valueExpression = "r.subcategory";
      postFilterClauses.push("AND r.subcategory IS NOT NULL AND r.subcategory <> ''");
      break;
    case "packs":
      valueExpression = "r.pack_label";
      postFilterClauses.push("AND r.pack_label IS NOT NULL AND r.pack_label <> ''");
      break;
  }

  sql.push(`SELECT ${valueExpression} AS value, COUNT(*) AS count`);
  sql.push(...joins);
  sql.push("WHERE 1 = 1");
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  });
  sql.push(...postFilterClauses);
  sql.push("GROUP BY value");
  sql.push("ORDER BY count DESC, value ASC");
  return { sql: sql.join("\n"), params };
}

function buildLexicalRetrievalQuery(filters: NormalizedSearchFilters, query: string, limit: number): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT r.record_key AS recordKey, bm25(records_fts, 8.0, 1.5) AS rank",
    "FROM records_fts",
    "JOIN records r ON r.record_key = records_fts.record_key",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
    "WHERE records_fts MATCH ?",
  ];
  const params: SqlValue[] = [query];
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  });
  sql.push("ORDER BY rank");
  sql.push("LIMIT ?");
  params.push(limit);
  return { sql: sql.join("\n"), params };
}

function semanticQueryLimit(baseLimit: number, filters: NormalizedSearchFilters): number {
  const hasPostFilterOnlyConstraints = Boolean(
    filters.publicationTitle ||
    (filters.traditions?.length ?? 0) > 0 ||
    (filters.spellKinds?.length ?? 0) > 0 ||
    (filters.traitsAll?.length ?? 0) > 0 ||
    (filters.traitsAny?.length ?? 0) > 0 ||
    (filters.excludeTraits?.length ?? 0) > 0 ||
    (filters.derivedTagsAll?.length ?? 0) > 0 ||
    (filters.derivedTagsAny?.length ?? 0) > 0 ||
    (filters.excludeDerivedTags?.length ?? 0) > 0,
  );
  return hasPostFilterOnlyConstraints ? Math.min(1000, Math.max(baseLimit * 4, baseLimit + 100)) : baseLimit;
}

function buildSemanticRetrievalQuery(filters: NormalizedSearchFilters, limit: number): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT record_key AS recordKey, distance",
    "FROM record_embeddings",
    "WHERE embedding MATCH ?",
    `AND k = ${limit}`,
  ];
  const params: SqlValue[] = [];

  if (filters.scopes && filters.scopes.length > 0) {
    appendScopedCategoryClauses(sql, params, filters.scopes, (category, subcategories) => {
      if (!subcategories || subcategories.length === 0) {
        return {
          clause: "category = ?",
          values: [normalizeVecText(category)],
        };
      }

      const placeholders = subcategories.map(() => "?").join(", ");
      return {
        clause: `(category = ? AND subcategory IN (${placeholders}))`,
        values: [normalizeVecText(category), ...subcategories.map((subcategory) => normalizeVecText(subcategory))],
      };
    });
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory) {
      appendWhereClause(sql, params, "AND category = ?", normalizeVecText(effectiveCategory));
    }
    if (filters.subcategory) {
      appendWhereClause(sql, params, "AND subcategory = ?", normalizeVecText(filters.subcategory));
    }
  }
  if (filters.pack) {
    appendWhereClause(sql, params, "AND pack_name = ?", normalizeVecText(filters.pack));
  }
  if (filters.levelMin !== undefined) {
    appendWhereClause(sql, params, "AND level >= ?", BigInt(filters.levelMin));
  }
  if (filters.levelMax !== undefined) {
    appendWhereClause(sql, params, "AND level <= ?", BigInt(filters.levelMax));
  }
  if (filters.rarity) {
    appendWhereClause(sql, params, "AND rarity = ?", normalizeVecText(filters.rarity));
  }
  if (filters.excludeUnique) {
    appendWhereClause(sql, params, "AND is_unique = 0");
  }
  if (filters.excludeMissingDescription) {
    appendWhereClause(sql, params, "AND has_description = 1");
  }
  if (filters.sources && filters.sources.length > 0) {
    const normalizedSources = filters.sources.map((source) => normalizeVecText(source));
    const placeholders = normalizedSources.map(() => "?").join(", ");
    appendWhereClause(sql, params, `AND source_category IN (${placeholders})`, ...normalizedSources);
  }
  if (filters.excludeSources && filters.excludeSources.length > 0) {
    const normalizedSources = filters.excludeSources.map((source) => normalizeVecText(source));
    const placeholders = normalizedSources.map(() => "?").join(", ");
    appendWhereClause(sql, params, `AND source_category NOT IN (${placeholders})`, ...normalizedSources);
  }
  if (filters.size) {
    appendWhereClause(sql, params, "AND size = ?", normalizeVecText(filters.size));
  }
  if (filters.priceMin !== undefined) {
    appendWhereClause(sql, params, "AND price_cp >= ?", BigInt(filters.priceMin));
  }
  if (filters.priceMax !== undefined) {
    appendWhereClause(sql, params, "AND price_cp <= ?", BigInt(filters.priceMax));
  }
  if (filters.actionCost !== undefined) {
    appendWhereClause(sql, params, "AND action_cost = ?", BigInt(filters.actionCost));
  }

  return { sql: sql.join("\n"), params };
}

function recordMatchesScope(record: NormalizedRecord, scope: NormalizedSearchScope): boolean {
  if (record.category !== scope.category) {
    return false;
  }

  if (!scope.subcategories || scope.subcategories.length === 0) {
    return true;
  }

  return record.subcategory !== null && scope.subcategories.includes(record.subcategory);
}

function recordMatchesFilters(record: NormalizedRecord, filters: NormalizedSearchFilters): boolean {
  if (filters.pack) {
    const normalizedPack = normalizeText(filters.pack);
    if (normalizeText(record.packName) !== normalizedPack && normalizeText(record.packLabel) !== normalizedPack) {
      return false;
    }
  }

  if (filters.scopes && filters.scopes.length > 0) {
    if (!filters.scopes.some((scope) => recordMatchesScope(record, scope))) {
      return false;
    }
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory && record.category !== effectiveCategory) {
      return false;
    }
    if (filters.subcategory && record.subcategory !== filters.subcategory) {
      return false;
    }
  }
  if (filters.levelMin !== undefined && (record.level === null || record.level < filters.levelMin)) {
    return false;
  }
  if (filters.levelMax !== undefined && (record.level === null || record.level > filters.levelMax)) {
    return false;
  }
  if (filters.rarity && normalizeText(record.rarity ?? "") !== normalizeText(filters.rarity)) {
    return false;
  }
  if (filters.publicationTitle && !normalizeText(record.publicationTitle ?? "").includes(normalizeText(filters.publicationTitle))) {
    return false;
  }
  if (filters.excludeUnique && record.isUnique) {
    return false;
  }
  if (filters.excludeMissingDescription && !record.hasDescription) {
    return false;
  }
  if (filters.sources && filters.sources.length > 0) {
    const allowedSources = new Set(filters.sources.map((source) => normalizeText(source)));
    if (!allowedSources.has(normalizeText(record.sourceCategory))) {
      return false;
    }
  }
  if (filters.excludeSources && filters.excludeSources.length > 0) {
    const excludedSources = new Set(filters.excludeSources.map((source) => normalizeText(source)));
    if (excludedSources.has(normalizeText(record.sourceCategory))) {
      return false;
    }
  }
  if (filters.size && normalizeText(record.size ?? "") !== normalizeText(filters.size)) {
    return false;
  }
  if (filters.priceMin !== undefined && (record.priceCp === null || record.priceCp < filters.priceMin)) {
    return false;
  }
  if (filters.priceMax !== undefined && (record.priceCp === null || record.priceCp > filters.priceMax)) {
    return false;
  }
  if (filters.actionCost !== undefined && record.actionCost !== filters.actionCost) {
    return false;
  }
  if (filters.traditions && filters.traditions.length > 0) {
    const normalizedTraditions = new Set(record.traditions.map((tradition) => normalizeText(tradition)));
    if (!filters.traditions.some((tradition) => normalizedTraditions.has(normalizeText(tradition)))) {
      return false;
    }
  }
  if (filters.spellKinds && filters.spellKinds.length > 0) {
    const normalizedSpellKinds = new Set(record.spellKinds.map((spellKind) => normalizeText(spellKind)));
    if (!filters.spellKinds.some((spellKind) => normalizedSpellKinds.has(normalizeText(spellKind)))) {
      return false;
    }
  }
  if (filters.traitsAll && filters.traitsAll.length > 0) {
    const normalizedTraits = new Set(record.traits.map((trait) => normalizeText(trait)));
    if (!filters.traitsAll.every((trait) => normalizedTraits.has(normalizeText(trait)))) {
      return false;
    }
  }
  if (filters.traitsAny && filters.traitsAny.length > 0) {
    const normalizedTraits = new Set(record.traits.map((trait) => normalizeText(trait)));
    if (!filters.traitsAny.some((trait) => normalizedTraits.has(normalizeText(trait)))) {
      return false;
    }
  }
  if (filters.excludeTraits && filters.excludeTraits.length > 0) {
    const normalizedTraits = new Set(record.traits.map((trait) => normalizeText(trait)));
    if (filters.excludeTraits.some((trait) => normalizedTraits.has(normalizeText(trait)))) {
      return false;
    }
  }
  if (filters.derivedTagsAll && filters.derivedTagsAll.length > 0) {
    const normalizedDerivedTags = new Set(record.derivedTags.map((tag) => normalizeDerivedTag(tag)));
    if (!filters.derivedTagsAll.every((tag) => normalizedDerivedTags.has(normalizeDerivedTag(tag)))) {
      return false;
    }
  }
  if (filters.derivedTagsAny && filters.derivedTagsAny.length > 0) {
    const normalizedDerivedTags = new Set(record.derivedTags.map((tag) => normalizeDerivedTag(tag)));
    if (!filters.derivedTagsAny.some((tag) => normalizedDerivedTags.has(normalizeDerivedTag(tag)))) {
      return false;
    }
  }
  if (filters.excludeDerivedTags && filters.excludeDerivedTags.length > 0) {
    const normalizedDerivedTags = new Set(record.derivedTags.map((tag) => normalizeDerivedTag(tag)));
    if (filters.excludeDerivedTags.some((tag) => normalizedDerivedTags.has(normalizeDerivedTag(tag)))) {
      return false;
    }
  }

  return true;
}

function validateFilters(filters: NormalizedSearchFilters, context: "list" | "search"): void {
  const mode = resolveSearchMode(filters, context);

  if (context === "list" && filters.searchProfile) {
    throw new Error("searchProfile is only supported for pf2e_search.");
  }

  if (context === "list" && mode !== "structured") {
    throw new Error("List mode only supports structured retrieval.");
  }

  if (context === "list" && filters.query) {
    throw new Error("query is only supported for pf2e_search.");
  }

  if (mode === "structured" && filters.query) {
    throw new Error("query requires a themed search profile such as balanced or concept.");
  }

  if (context === "search" && !filters.query?.trim() && !filters.nameQuery?.trim() && !hasStructuredFilterSignal(filters)) {
    throw new Error("pf2e_search requires search text and/or at least one structured filter.");
  }

  if (filters.scopes && filters.scopes.length > 0 && (filters.category || filters.subcategory)) {
    throw new Error("scopes can't be combined with top-level category or subcategory filters.");
  }

  if (filters.category && filters.subcategory && !categorySupportsSubcategory(filters.category, filters.subcategory)) {
    throw new Error(`Subcategory "${filters.subcategory}" does not belong to category "${filters.category}".`);
  }

  if (filters.scopes) {
    for (const scope of filters.scopes) {
      for (const subcategory of scope.subcategories ?? []) {
        if (!categorySupportsSubcategory(scope.category, subcategory)) {
          throw new Error(`Subcategory "${subcategory}" does not belong to category "${scope.category}".`);
        }
      }
    }
  }

  if (filters.sources && filters.excludeSources) {
    const overlappingSources = filters.sources.filter((source) => filters.excludeSources?.includes(source));
    if (overlappingSources.length > 0) {
      throw new Error(`sources and excludeSources overlap: ${overlappingSources.join(", ")}`);
    }
  }
}

export class Pf2eDataService {
  readonly packs: PackInfo[];
  readonly warnings: string[];

  private readonly db: DatabaseSync;
  private readonly embeddingProvider: EmbeddingProvider;
  private readonly indexPath: string;
  private readonly recordCount: number;
  private readonly rankingConfigStore: RankingConfigStore | null;
  private readonly aliasesByRecordKey: Map<string, string[]>;
  private readonly legacyLinksByRecordKey: Map<string, LinkedRecordSummary[]>;

  private constructor(
    db: DatabaseSync,
    packs: PackInfo[],
    warnings: string[],
    recordCount: number,
    indexPath: string,
    embeddingProvider: EmbeddingProvider,
    rankingConfigStore: RankingConfigStore | null,
  ) {
    this.db = db;
    this.packs = packs;
    this.warnings = warnings;
    this.recordCount = recordCount;
    this.indexPath = indexPath;
    this.embeddingProvider = embeddingProvider;
    this.rankingConfigStore = rankingConfigStore;
    this.aliasesByRecordKey = loadAliasesByRecordKey(db);
    this.legacyLinksByRecordKey = loadLegacyLinksByRecordKey(db);
  }

  static async load(rootPath: string, manifestPath: string, options: LoadOptions = {}): Promise<Pf2eDataService> {
    const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
    const embeddingConfig: EmbeddingConfig = options.embedding ?? defaultEmbeddingConfig(indexPath);
    const embeddingProviderFactory = options.embeddingProviderFactory ?? createEmbeddingProvider;
    const embeddingRuntime = await embeddingProviderFactory(embeddingConfig);
    const embeddingProvider = embeddingRuntime.provider;
    const sourceSignature = await computeSourceSignature(rootPath, manifestPath);
    if (!(await fileExists(indexPath))) {
      throw buildMissingIndexError(indexPath);
    }

    const existingDb = openDatabase(indexPath, {
      vectorExtensionLoader: options.vectorExtensionLoader,
    });
    const invalidReason = getIndexInvalidReason(existingDb, sourceSignature, embeddingProvider);
    if (invalidReason) {
      existingDb.close();
      throw buildStaleIndexError(indexPath, invalidReason);
    }

    const packs = loadPacksFromIndex(existingDb);
    const recordCount = sqliteRowCount(
      existingDb.prepare("SELECT COUNT(*) AS total FROM records").get() as Record<string, unknown> | undefined,
    );
    return new Pf2eDataService(
      existingDb,
      packs,
      [...embeddingRuntime.warnings, ...(options.rankingConfigStore?.warnings ?? [])],
      recordCount,
      indexPath,
      embeddingProvider,
      options.rankingConfigStore ?? null,
    );
  }

  static async rebuildIndex(rootPath: string, manifestPath: string, options: LoadOptions = {}): Promise<Pf2eDataService> {
    const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
    const embeddingConfig: EmbeddingConfig = options.embedding ?? defaultEmbeddingConfig(indexPath);
    const embeddingProviderFactory = options.embeddingProviderFactory ?? createEmbeddingProvider;
    options.progressLogger?.("Loading the configured embedding provider.");
    const embeddingRuntime = await embeddingProviderFactory(embeddingConfig);
    const embeddingProvider = embeddingRuntime.provider;
    options.progressLogger?.(
      `Embedding provider ready: ${embeddingProvider.identity.model} (${embeddingProvider.identity.dimensions} dimensions).`,
    );
    options.progressLogger?.("Computing the PF2E source signature.");
    const sourceSignature = await computeSourceSignature(rootPath, manifestPath);
    options.progressLogger?.(`Preparing index output at ${indexPath}.`);
    await mkdir(path.dirname(indexPath), { recursive: true });
    await removeIndexFiles(indexPath);

    const db = openDatabase(indexPath, {
      vectorExtensionLoader: options.vectorExtensionLoader,
    });
    options.progressLogger?.("Creating SQLite schema.");
    createSchema(db, embeddingProvider.identity.dimensions);
    const { packs, warnings, recordCount } = await buildIndex(
      db,
      rootPath,
      manifestPath,
      embeddingProvider,
      sourceSignature,
      options.progressLogger,
      options.progressStatusLogger,
    );
    options.progressLogger?.(
      `Finished writing ${formatInteger(recordCount)} records across ${formatInteger(packs.length)} packs.`,
    );
    return new Pf2eDataService(
      db,
      packs,
      [...embeddingRuntime.warnings, ...warnings, ...(options.rankingConfigStore?.warnings ?? [])],
      recordCount,
      indexPath,
      embeddingProvider,
      options.rankingConfigStore ?? null,
    );
  }

  getStats(): { packCount: number; recordCount: number } {
    return {
      packCount: this.packs.length,
      recordCount: this.recordCount,
    };
  }

  getSearchVocabulary(options: { traitLimitPerCategory?: number } = {}): {
    categories: Array<{ value: SearchCategory; count: number }>;
    subcategories: Array<{ value: string; count: number }>;
    rarities: Array<{ value: string; count: number }>;
    sizes: Array<{ value: string; count: number }>;
    traditions: Array<{ value: string; count: number }>;
    spellKinds: Array<{ value: string; count: number }>;
    sourceCategories: Array<{ value: SourceCategory; count: number }>;
    commonTraitsByCategory: Array<{ category: SearchCategory; traits: Array<{ value: string; count: number }> }>;
    commonDerivedTagsByCategory: Array<{ category: SearchCategory; tags: Array<{ value: string; count: number }> }>;
    derivedTagCatalog: DerivedTagCatalogEntry[];
  } {
    const traitLimit = Math.max(3, Math.min(options.traitLimitPerCategory ?? 12, 25));
    const categories = this.db
      .prepare(
        `
          SELECT r.category AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1
          GROUP BY r.category
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: SearchCategory; count: number }>;
    const subcategories = this.db
      .prepare(
        `
          SELECT r.subcategory AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1 AND r.subcategory IS NOT NULL AND r.subcategory <> ''
          GROUP BY r.subcategory
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const sourceCategories = this.db
      .prepare(
        `
          SELECT r.source_category AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1
          GROUP BY r.source_category
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: SourceCategory; count: number }>;
    const rarities = this.db
      .prepare(
        `
          SELECT r.rarity AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1 AND r.rarity IS NOT NULL AND r.rarity <> ''
          GROUP BY r.rarity
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const sizes = this.db
      .prepare(
        `
          SELECT a.size AS value, COUNT(*) AS count
          FROM actor_records a
          JOIN records r ON r.record_key = a.record_key
          WHERE r.is_search_canonical = 1 AND a.size IS NOT NULL AND a.size <> ''
          GROUP BY a.size
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const categoryTraitRows = this.db
      .prepare(
        `
          SELECT r.category AS category, rt.trait AS value, COUNT(*) AS count
          FROM record_traits rt
          JOIN records r ON r.record_key = rt.record_key
          WHERE r.is_search_canonical = 1
          GROUP BY r.category, rt.trait
          ORDER BY r.category ASC, count DESC, value ASC
        `,
      )
      .all() as Array<{ category: SearchCategory; value: string; count: number }>;
    const commonTraitsByCategory = (() => {
      const grouped = new Map<SearchCategory, Array<{ value: string; count: number }>>();
      for (const row of categoryTraitRows) {
        const bucket = grouped.get(row.category) ?? [];
        if (bucket.length < traitLimit) {
          bucket.push({ value: row.value, count: row.count });
          grouped.set(row.category, bucket);
        }
      }
      return [...grouped.entries()]
        .sort((left, right) => left[0].localeCompare(right[0]))
        .map(([category, traits]) => ({ category, traits }));
    })();
    const categoryDerivedTagRows = this.db
      .prepare(
        `
          SELECT r.category AS category, rdt.tag AS value, COUNT(*) AS count
          FROM record_derived_tags rdt
          JOIN records r ON r.record_key = rdt.record_key
          WHERE r.is_search_canonical = 1
          GROUP BY r.category, rdt.tag
          ORDER BY r.category ASC, count DESC, value ASC
        `,
      )
      .all() as Array<{ category: SearchCategory; value: string; count: number }>;
    const commonDerivedTagsByCategory = (() => {
      const grouped = new Map<SearchCategory, Array<{ value: string; count: number }>>();
      for (const row of categoryDerivedTagRows) {
        const bucket = grouped.get(row.category) ?? [];
        if (bucket.length < traitLimit) {
          bucket.push({ value: row.value, count: row.count });
          grouped.set(row.category, bucket);
        }
      }
      return [...grouped.entries()]
        .sort((left, right) => left[0].localeCompare(right[0]))
        .map(([category, tags]) => ({ category, tags }));
    })();
    const traditionCounts = new Map<string, number>();
    const traditionRows = this.db
      .prepare(`
        SELECT s.traditions_json AS traditionsJson
        FROM spell_records s
        JOIN records r ON r.record_key = s.record_key
        WHERE r.is_search_canonical = 1
      `)
      .all() as Array<{ traditionsJson: string }>;
    for (const row of traditionRows) {
      const traditions = JSON.parse(row.traditionsJson) as string[];
      for (const tradition of traditions) {
        const normalized = normalizeText(tradition);
        if (!normalized) {
          continue;
        }

        traditionCounts.set(normalized, (traditionCounts.get(normalized) ?? 0) + 1);
      }
    }
    const traditions = [...traditionCounts.entries()]
      .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
      .map(([value, count]) => ({ value, count }));
    const spellKindCounts = new Map<string, number>();
    const spellKindRows = this.db
      .prepare(`
        SELECT s.spell_kinds_json AS spellKindsJson
        FROM spell_records s
        JOIN records r ON r.record_key = s.record_key
        WHERE r.is_search_canonical = 1
      `)
      .all() as Array<{ spellKindsJson: string }>;
    for (const row of spellKindRows) {
      const spellKinds = JSON.parse(row.spellKindsJson) as string[];
      for (const spellKind of spellKinds) {
        const normalized = normalizeText(spellKind);
        if (!normalized) {
          continue;
        }

        spellKindCounts.set(normalized, (spellKindCounts.get(normalized) ?? 0) + 1);
      }
    }
    const spellKinds = [...spellKindCounts.entries()]
      .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
      .map(([value, count]) => ({ value, count }));

    return {
      categories,
      subcategories,
      rarities,
      sizes,
      traditions,
      spellKinds,
      sourceCategories,
      commonTraitsByCategory,
      commonDerivedTagsByCategory,
      derivedTagCatalog: DERIVED_TAG_CATALOG,
    };
  }

  getRankingConfigStatus() {
    return this.rankingConfigStore?.getStatus() ?? {
      path: "<defaults>",
      source: "default" as const,
      revision: 1,
      loadedAt: new Date(0).toISOString(),
      lastError: null,
    };
  }

  getPack(packValue: string): PackInfo | undefined {
    return this.packs.find((pack) => getPackAliasValues(pack, packValue));
  }

  listPacks(): PackInfo[] {
    return this.packs;
  }

  listFilterValues(query: FilterValueQuery): FilterValueResult {
    const normalizedFilters = this.normalizeSearchFilters(query);
    validateFilters(normalizedFilters, "list");
    const { sql, params } = buildFilterValueQuery(query.field, normalizedFilters);
    const values = this.db.prepare(sql).all(...params) as ValueCountRow[];
    return {
      field: query.field,
      values,
    };
  }

  close(): void {
    this.rankingConfigStore?.close();
    this.db.close();
  }

  private decorateRecord(record: NormalizedRecord): NormalizedRecord {
    return {
      ...record,
      aliases: this.aliasesByRecordKey.get(record.recordKey) ?? [],
      legacyRecordLinks: this.legacyLinksByRecordKey.get(record.recordKey) ?? [],
    };
  }

  private normalizeSearchFilters(filters: SearchFilters): NormalizedSearchFilters {
    const normalizedCategory = filters.category !== undefined
      ? normalizeSearchCategory(filters.category)
      : null;
    if (filters.category !== undefined && !normalizedCategory) {
      throw new Error(getSearchCategoryErrorMessage(String(filters.category)));
    }

    const normalizedSubcategory = filters.subcategory !== undefined
      ? normalizeSearchSubcategory(filters.subcategory)
      : null;
    if (filters.subcategory !== undefined && !normalizedSubcategory) {
      throw new Error(getSearchSubcategoryErrorMessage(String(filters.subcategory)));
    }

    const normalizedScopes = filters.scopes?.map((scope) => normalizeSearchScope(scope));
    const pack = filters.pack ? this.getPack(filters.pack) : undefined;

    return {
      ...filters,
      pack: pack?.name ?? filters.pack,
      category: normalizedCategory ?? undefined,
      subcategory: normalizedSubcategory ?? undefined,
      scopes: normalizedScopes,
    };
  }

  private fetchCandidates(
    filters: NormalizedSearchFilters,
    includeSearchText = false,
    includeEmbedding = false,
    options: { recordKeys?: string[] } = {},
  ): CandidateRow[] {
    const { sql, params } = buildCandidateQuery(filters, includeSearchText, includeEmbedding, options);
    return this.db.prepare(sql).all(...params) as CandidateRow[];
  }

  private fetchLexicalRetrievalRows(filters: NormalizedSearchFilters, query: string, limit: number): LexicalRetrievalRow[] {
    const ftsQuery = buildFtsQuery(query);
    if (!ftsQuery) {
      return [];
    }

    const { sql, params } = buildLexicalRetrievalQuery(filters, ftsQuery, limit);
    return this.db.prepare(sql).all(...params) as LexicalRetrievalRow[];
  }

  private fetchSemanticRetrievalRows(filters: NormalizedSearchFilters, queryVector: Float32Array, limit: number): SemanticRetrievalRow[] {
    if (queryVector.length === 0) {
      return [];
    }

    const encodedQuery = encodeVector(queryVector);
    const { sql, params } = buildSemanticRetrievalQuery(filters, limit);
    return this.db.prepare(sql).all(encodedQuery, ...params) as SemanticRetrievalRow[];
  }

  private fetchRecordRowsByKeys(recordKeys: string[]): CandidateRow[] {
    if (recordKeys.length === 0) {
      return [];
    }

    const placeholders = buildPlaceholders(recordKeys);
    return this.db
      .prepare(
        `
          SELECT
            r.record_key AS recordKey,
            r.id AS id,
            r.name AS name,
            r.normalized_name AS normalizedName,
            r.record_type AS type,
            r.category AS category,
            r.subcategory AS subcategory,
            r.pack_name AS packName,
            r.pack_label AS packLabel,
            r.document_type AS documentType,
            r.level AS level,
            r.rarity AS rarity,
            r.traits_json AS traitsJson,
            r.derived_tags_json AS derivedTagsJson,
            r.publication_title AS publicationTitle,
            r.publication_remaster AS publicationRemaster,
            r.description_text AS descriptionText,
            r.has_description AS hasDescription,
            r.description_snippet AS descriptionSnippet,
            r.source_category AS sourceCategory,
            r.folder_id AS folderId,
            r.glossary_family AS glossaryFamily,
            r.additional_glossary_families_json AS additionalGlossaryFamiliesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
            i.item_category AS itemCategory,
            i.price_cp AS priceCp,
            i.bulk_value AS bulkValue,
            COALESCE(s.action_cost, i.action_cost) AS actionCost,
            s.traditions_json AS traditionsJson,
            s.spell_kinds_json AS spellKindsJson
          FROM records r
          LEFT JOIN actor_records a ON a.record_key = r.record_key
          LEFT JOIN item_records i ON i.record_key = r.record_key
          LEFT JOIN spell_records s ON s.record_key = r.record_key
          WHERE r.record_key IN (${placeholders})
        `,
      )
      .all(...recordKeys) as CandidateRow[];
  }

  private fetchReferenceEdgeRows(
    direction: RuleReferenceEdge["direction"],
    recordKeys: string[],
    {
      coreOnly = false,
      maxPerPrimary = 4,
    }: { coreOnly?: boolean; maxPerPrimary?: number } = {},
  ): RuleGraphResult {
    if (recordKeys.length === 0) {
      return { records: [], edges: [] };
    }

    const placeholders = buildPlaceholders(recordKeys);
    const targetFilter = direction === "outgoing"
      ? (coreOnly ? "AND target.source_category = 'core'" : "")
      : (coreOnly ? "AND re.from_source_category = 'core'" : "AND re.from_source_category IN ('core', 'rules')");
    const backlinkFilter = direction === "backlink"
      ? "AND (re.from_record_type = 'action' OR re.from_record_type = 'feat' OR LOWER(re.from_pack_name) = 'classfeatures')"
      : "";
    const keyColumn = direction === "outgoing" ? "re.from_record_key" : "re.to_record_key";

    const rows = this.db
      .prepare(
        `
          SELECT
            re.from_record_key AS fromRecordKey,
            re.to_record_key AS toRecordKey,
            re.display_text AS displayText,
            re.reference_text AS referenceText,
            re.from_pack_name AS fromPackName,
            re.from_record_type AS fromRecordType,
            re.from_document_type AS fromDocumentType,
            re.from_source_category AS fromSourceCategory
          FROM reference_edges re
          JOIN records target ON target.record_key = re.to_record_key
          WHERE ${keyColumn} IN (${placeholders})
          ${targetFilter}
          ${backlinkFilter}
        `,
      )
      .all(...recordKeys) as ReferenceEdgeRow[];

    const grouped = new Map<string, ReferenceEdgeRow[]>();
    for (const row of rows) {
      const groupKey = direction === "outgoing" ? row.fromRecordKey : row.toRecordKey;
      const bucket = grouped.get(groupKey) ?? [];
      bucket.push(row);
      grouped.set(groupKey, bucket);
    }

    const keptRows: ReferenceEdgeRow[] = [];
    for (const primaryKey of recordKeys) {
      const bucket = grouped.get(primaryKey) ?? [];
      bucket.sort((left, right) => {
        const leftTypeRank =
          left.fromPackName === "classfeatures" ? 2 : backlinkTypeRank(left.fromRecordType);
        const rightTypeRank =
          right.fromPackName === "classfeatures" ? 2 : backlinkTypeRank(right.fromRecordType);
        const leftLabel = left.displayText ?? (direction === "outgoing" ? left.toRecordKey : left.fromRecordKey);
        const rightLabel = right.displayText ?? (direction === "outgoing" ? right.toRecordKey : right.fromRecordKey);
        return (
          sourceCategoryRank(left.fromSourceCategory) - sourceCategoryRank(right.fromSourceCategory) ||
          leftTypeRank - rightTypeRank ||
          leftLabel.localeCompare(rightLabel) ||
          left.referenceText.localeCompare(right.referenceText)
        );
      });
      keptRows.push(...bucket.slice(0, Math.max(1, maxPerPrimary)));
    }

    const relatedRecordKeys = [
      ...new Set(keptRows.map((row) => direction === "outgoing" ? row.toRecordKey : row.fromRecordKey)),
    ];
    return {
      records: this.getRecordsByKeys(relatedRecordKeys),
      edges: keptRows.map((row) => edgeRowToReferenceEdge(row, direction)),
    };
  }

  getRecord(recordKeyOrPack: string, maybeId?: string): NormalizedRecord | undefined {
    const row = maybeId
      ? (this.db
          .prepare(
            `
              SELECT
                r.record_key AS recordKey,
                r.id AS id,
                r.name AS name,
                r.normalized_name AS normalizedName,
                r.record_type AS type,
                r.category AS category,
                r.subcategory AS subcategory,
                r.pack_name AS packName,
                r.pack_label AS packLabel,
                r.document_type AS documentType,
            r.level AS level,
            r.rarity AS rarity,
            r.traits_json AS traitsJson,
            r.derived_tags_json AS derivedTagsJson,
            r.publication_title AS publicationTitle,
            r.publication_remaster AS publicationRemaster,
            r.description_text AS descriptionText,
                r.has_description AS hasDescription,
                r.description_snippet AS descriptionSnippet,
                r.source_category AS sourceCategory,
                r.folder_id AS folderId,
                r.glossary_family AS glossaryFamily,
                r.additional_glossary_families_json AS additionalGlossaryFamiliesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson,
                s.spell_kinds_json AS spellKindsJson,
                r.raw_json AS rawJson
              FROM records r
              LEFT JOIN actor_records a ON a.record_key = r.record_key
              LEFT JOIN item_records i ON i.record_key = r.record_key
              LEFT JOIN spell_records s ON s.record_key = r.record_key
              WHERE r.pack_name = ? AND r.id = ?
            `,
          )
          .get(recordKeyOrPack, maybeId) as CandidateRow | undefined)
      : (this.db
          .prepare(
            `
              SELECT
                r.record_key AS recordKey,
                r.id AS id,
                r.name AS name,
                r.normalized_name AS normalizedName,
                r.record_type AS type,
                r.category AS category,
                r.subcategory AS subcategory,
                r.pack_name AS packName,
                r.pack_label AS packLabel,
                r.document_type AS documentType,
            r.level AS level,
            r.rarity AS rarity,
            r.traits_json AS traitsJson,
            r.derived_tags_json AS derivedTagsJson,
            r.publication_title AS publicationTitle,
            r.publication_remaster AS publicationRemaster,
            r.description_text AS descriptionText,
                r.has_description AS hasDescription,
                r.description_snippet AS descriptionSnippet,
                r.source_category AS sourceCategory,
                r.folder_id AS folderId,
                r.glossary_family AS glossaryFamily,
                r.additional_glossary_families_json AS additionalGlossaryFamiliesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson,
                s.spell_kinds_json AS spellKindsJson,
                r.raw_json AS rawJson
              FROM records r
              LEFT JOIN actor_records a ON a.record_key = r.record_key
              LEFT JOIN item_records i ON i.record_key = r.record_key
              LEFT JOIN spell_records s ON s.record_key = r.record_key
              WHERE r.record_key = ?
            `,
          )
          .get(recordKeyOrPack) as CandidateRow | undefined);

    if (!row) {
      return undefined;
    }

    return this.decorateRecord(rowToRecord(row));
  }

  getRecordsByKeys(recordKeys: string[]): NormalizedRecord[] {
    const rows = this.fetchRecordRowsByKeys([...new Set(recordKeys)]);
    const byKey = new Map(rows.map((row) => [row.recordKey, this.decorateRecord(rowToRecord(row))]));
    return [...new Set(recordKeys)].map((recordKey) => byKey.get(recordKey)).filter((record): record is NormalizedRecord => Boolean(record));
  }

  lookupMany(queries: LookupQuery[], options: { coreOnly?: boolean } = {}): LookupResult[] {
    return queries.map((query) => {
      const lookup = (() => {
        if (!options.coreOnly) {
          return this.lookup(query.name, query);
        }

        const results = this.searchStructured({
          nameQuery: query.name,
          pack: query.pack,
          category: query.category,
          subcategory: query.subcategory,
          sources: ["core"],
          limit: 5,
        }).records;
        return {
          match: results[0] ?? null,
          alternatives: results.slice(1),
        };
      })();
      return {
        query,
        match: lookup.match,
        alternatives: lookup.alternatives,
        matchType: getLookupMatchType(query.name, lookup.match),
      };
    });
  }

  getRuleGraph(
    recordKeys: string[],
    {
      coreOnly,
      includeOutgoing,
      includeBacklinks,
      maxOutgoingPerPrimary,
      maxBacklinksPerPrimary,
    }: {
      coreOnly?: boolean;
      includeOutgoing?: boolean;
      includeBacklinks?: boolean;
      maxOutgoingPerPrimary?: number;
      maxBacklinksPerPrimary?: number;
    } = {},
  ): RuleGraphCollectionResult {
    const uniqueRecordKeys = [...new Set(recordKeys)];
    const directionsSpecified = includeOutgoing !== undefined || includeBacklinks !== undefined;
    const shouldIncludeOutgoing = directionsSpecified ? includeOutgoing === true : true;
    const shouldIncludeBacklinks = directionsSpecified ? includeBacklinks === true : false;
    const emptyGraph: RuleGraphResult = { records: [], edges: [] };
    const outgoing = shouldIncludeOutgoing
      ? this.fetchReferenceEdgeRows("outgoing", uniqueRecordKeys, {
          coreOnly,
          maxPerPrimary: maxOutgoingPerPrimary,
        })
      : emptyGraph;
    const backlinks = shouldIncludeBacklinks
      ? this.fetchReferenceEdgeRows("backlink", uniqueRecordKeys, {
          coreOnly,
          maxPerPrimary: maxBacklinksPerPrimary,
        })
      : emptyGraph;

    return {
      outgoing,
      backlinks,
      edges: [...outgoing.edges, ...backlinks.edges],
    };
  }

  collectRuleQuestionContext(input: CollectRuleQuestionContextInput): CollectRuleQuestionContextResult {
    const explicitRules = (input.rules ?? []).map((rule) => rule.trim()).filter((rule) => rule.length > 0);
    const derivedRules = explicitRules.length > 0
      ? explicitRules
      : input.question
        ? extractQuestionRuleNames(input.question)
        : [];
    const primary = this.lookupMany(derivedRules.map((name) => ({ name })), { coreOnly: input.coreOnly });
    const primaryKeys = primary
      .map((result) => result.match?.recordKey ?? null)
      .filter((recordKey): recordKey is string => Boolean(recordKey));
    const graph = this.getRuleGraph(primaryKeys, {
      coreOnly: input.coreOnly,
      includeOutgoing: true,
      includeBacklinks: input.includeBacklinks,
      maxOutgoingPerPrimary: input.maxOutgoingPerPrimary ?? 4,
      maxBacklinksPerPrimary: input.maxBacklinksPerPrimary ?? 4,
    });

    return {
      primary,
      ...graph,
    };
  }

  private searchStructured(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "search");
    const limit = clampLimit(normalizedFilters.limit);
    const offset = clampOffset(normalizedFilters.offset);
    const mode = resolveSearchMode(normalizedFilters, "search");
    const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
    const rankingConfig = this.rankingConfigStore?.getConfig() ?? DEFAULT_RANKING_CONFIG;
    const candidates = this.fetchCandidates(normalizedFilters);
    const scored = candidates
      .map((candidate) => {
        const record = this.decorateRecord(rowToRecord(candidate));
        const packQuality = packQualityScore(record, rankingConfig);
        const sourceQuality = sourceQualityScore(record, rankingConfig);
        const rarityPreference = rarityPreferenceScore(record, normalizedFilters, rankingConfig);
        const sourcePenalty = sourcePenaltyScore(record, normalizedFilters, rankingConfig);
        const score =
          (normalizedFilters.nameQuery ? nameScore(normalizedFilters.nameQuery, record, this.aliasesByRecordKey.get(record.recordKey) ?? []) : 0.5) +
          packQuality +
          sourceQuality +
          rarityPreference +
          sourcePenalty;

        return { record, score };
      })
      .filter(({ score }) => {
        if (normalizedFilters.nameQuery) {
          return score >= 0.2;
        }

        return true;
      })
      .sort((left, right) => right.score - left.score || sortRecords(left.record, right.record));

    return {
      searchProfile,
      mode: "structured",
      total: scored.length,
      offset,
      limit,
      records: scored.slice(offset, offset + limit).map(({ record }) => record),
    };
  }

  listRecords(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "list");
    const limit = clampLimit(normalizedFilters.limit);
    const offset = clampOffset(normalizedFilters.offset);
    const records = this.fetchCandidates(normalizedFilters).map((row) => this.decorateRecord(rowToRecord(row)));
    records.sort((left, right) => sortRecords(left, right));
    return {
      searchProfile: null,
      mode: "structured",
      total: records.length,
      offset,
      limit,
      records: records.slice(offset, offset + limit),
    };
  }

  async search(filters: SearchFilters): Promise<SearchResult> {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "search");
    const limit = clampLimit(normalizedFilters.limit);
    const offset = clampOffset(normalizedFilters.offset);
    const mode = resolveSearchMode(normalizedFilters, "search");
    const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
    const rawSemanticQuery = normalizedFilters.query?.trim() || "";
    const rawLexicalQuery = normalizedFilters.query?.trim() || normalizedFilters.nameQuery?.trim() || "";
    const rankingConfig = this.rankingConfigStore?.getConfig() ?? DEFAULT_RANKING_CONFIG;
    const hybridFusion = resolveHybridFusionProfile(searchProfile, mode, rankingConfig);
    const queryAnalysis = rawLexicalQuery
      ? buildSearchQueryAnalysis(rawLexicalQuery)
      : null;
    const literalQueryWeights = queryAnalysis
      ? buildLiteralQueryWeights(queryAnalysis)
      : null;
    const lexicalQuery = queryAnalysis?.normalizedQuery ?? rawLexicalQuery;
    const semanticVector = hybridFusion && rawSemanticQuery
      ? await this.embeddingProvider.embed(rawSemanticQuery)
      : null;
    const lexicalRetrievalRows = lexicalQuery
      ? this.fetchLexicalRetrievalRows(
          normalizedFilters,
          lexicalQuery,
          Math.max(mode === "lexical" ? LOOKUP_LEXICAL_TOP_K : (hybridFusion?.config.lexicalTopK ?? 0), (offset + limit) * 5),
        )
      : [];
    const lexicalRetrievedKeys = lexicalRetrievalRows.map((row) => row.recordKey);
    const lexicalRetrievalRanks = buildRankMap(lexicalRetrievedKeys);
    const lexicalMatches = buildNormalizedRankScoreMap(lexicalRetrievedKeys);

    const semanticRetrievalRows = semanticVector && hybridFusion
      ? this.fetchSemanticRetrievalRows(
          normalizedFilters,
          semanticVector,
          semanticQueryLimit(Math.max(hybridFusion.config.semanticTopK, (offset + limit) * 5), normalizedFilters),
        )
      : [];
    const semanticRetrievedKeys = semanticRetrievalRows.map((row) => row.recordKey);
    const semanticRetrievalRanks = buildRankMap(semanticRetrievedKeys);

    const candidateKeys = mode === "structured"
      ? []
      : [...new Set([...lexicalRetrievedKeys, ...semanticRetrievedKeys])];
    const candidateRows = mode === "structured"
      ? []
      : this.fetchCandidates(normalizedFilters, false, false, { recordKeys: candidateKeys });
    const candidateRecords = candidateRows
      .map((row) => this.decorateRecord(rowToRecord(row)))
      .filter((record) => recordMatchesFilters(record, normalizedFilters));
    const candidatesByKey = new Map(candidateRecords.map((record) => [record.recordKey, record]));

    const scored = (() => {
      if (mode === "structured") {
        return this.fetchCandidates(normalizedFilters)
          .map((candidate) => {
            const record = this.decorateRecord(rowToRecord(candidate));
            const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, rankingConfig);
            const totalScore =
              (normalizedFilters.nameQuery
                ? nameScore(normalizedFilters.nameQuery, record, this.aliasesByRecordKey.get(record.recordKey) ?? [])
                : 0.5) +
              sumRerankAdjustments(rerankAdjustments);
            const explanation: SearchRecordExplanation = {
              recordKey: record.recordKey,
              name: record.name,
              totalScore,
              fusionScore: null,
              lexicalRank: null,
              semanticRank: null,
              lexicalRerankScore: null,
              matchedTraits: [],
              matchedNameTokens: [],
              rerankAdjustments,
            };

            return { record, totalScore, explanation };
          })
          .filter(({ totalScore }) => {
            if (normalizedFilters.nameQuery) {
              return totalScore >= 0.2;
            }

            return true;
          })
          .sort((left, right) => right.totalScore - left.totalScore || sortRecords(left.record, right.record));
      }

      if (mode === "lexical") {
        return lexicalRetrievedKeys
          .map((recordKey) => candidatesByKey.get(recordKey))
          .filter((record): record is NormalizedRecord => Boolean(record))
          .map((record) => {
            const lexicalSignal = buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, rankingConfig);
            const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, rankingConfig);
            const totalScore = lexicalSignal.lexicalScore + sumRerankAdjustments(rerankAdjustments);
            const explanation: SearchRecordExplanation = {
              recordKey: record.recordKey,
              name: record.name,
              totalScore,
              fusionScore: null,
              lexicalRank: lexicalRetrievalRanks.get(record.recordKey) ?? null,
              semanticRank: null,
              lexicalRerankScore: lexicalSignal.lexicalScore,
              matchedTraits: lexicalSignal.matchedTraits,
              matchedNameTokens: lexicalSignal.matchedNameTokens,
              rerankAdjustments,
            };

            return {
              record,
              totalScore,
              lexicalRank: lexicalRetrievalRanks.get(record.recordKey) ?? null,
              lexicalRerankScore: lexicalSignal.lexicalScore,
              explanation,
            };
          })
          .filter(({ totalScore }) => {
            if (lexicalQuery) {
              return totalScore > 0;
            }

            return true;
          })
          .sort((left, right) => {
            return (
              right.totalScore - left.totalScore ||
              right.lexicalRerankScore - left.lexicalRerankScore ||
              compareOptionalRanks(left.lexicalRank, right.lexicalRank) ||
              sortRecords(left.record, right.record)
            );
          });
      }

      const fusionConfig = hybridFusion!.config;
      const rerankedLexical = lexicalRetrievedKeys
        .map((recordKey) => candidatesByKey.get(recordKey))
        .filter((record): record is NormalizedRecord => Boolean(record))
        .map((record) => ({
          record,
          lexicalSignal: buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, rankingConfig),
        }))
        .filter(({ lexicalSignal }) => lexicalSignal.lexicalScore > 0)
        .sort((left, right) => {
          return (
            right.lexicalSignal.lexicalScore - left.lexicalSignal.lexicalScore ||
            compareOptionalRanks(
              semanticRetrievalRanks.get(left.record.recordKey) ?? null,
              semanticRetrievalRanks.get(right.record.recordKey) ?? null,
            ) ||
            sortRecords(left.record, right.record)
          );
        })
        .slice(0, fusionConfig.lexicalTopK);
      const rerankedLexicalRanks = buildRankMap(rerankedLexical.map(({ record }) => record.recordKey));
      const semanticRanks = buildRankMap(
        semanticRetrievedKeys
          .filter((recordKey) => candidatesByKey.has(recordKey))
          .slice(0, fusionConfig.semanticTopK),
      );

      return candidateRecords
        .filter((record) => rerankedLexicalRanks.has(record.recordKey) || semanticRanks.has(record.recordKey))
        .map((record) => {
          const lexicalSignal = buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, rankingConfig);
          const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, rankingConfig);
          const lexicalRank = rerankedLexicalRanks.get(record.recordKey) ?? null;
          const semanticRank = semanticRanks.get(record.recordKey) ?? null;
          const fusionScore = computeWeightedRrfScore(
            lexicalRank,
            semanticRank,
            fusionConfig,
            rankingConfig.hybridFusion.rrfK,
          );
          const totalScore = fusionScore + sumRerankAdjustments(rerankAdjustments);
          const explanation: SearchRecordExplanation = {
            recordKey: record.recordKey,
            name: record.name,
            totalScore,
            fusionScore,
            lexicalRank,
            semanticRank,
            lexicalRerankScore: lexicalSignal.lexicalScore,
            matchedTraits: lexicalSignal.matchedTraits,
            matchedNameTokens: lexicalSignal.matchedNameTokens,
            rerankAdjustments,
          };

          return {
            record,
            totalScore,
            fusionScore,
            lexicalRank,
            semanticRank,
            lexicalRerankScore: lexicalSignal.lexicalScore,
            explanation,
          };
        })
        .sort((left, right) => {
          return (
            right.totalScore - left.totalScore ||
            right.fusionScore - left.fusionScore ||
            compareOptionalRanks(left.semanticRank, right.semanticRank) ||
            compareOptionalRanks(left.lexicalRank, right.lexicalRank) ||
            right.lexicalRerankScore - left.lexicalRerankScore ||
            sortRecords(left.record, right.record)
          );
        });
    })();

    const page = scored.slice(offset, offset + limit);
    const explain: SearchExplainResult | undefined = filters.explain
      ? {
          searchProfile,
          mode,
          fusionMethod: hybridFusion ? "weightedRrf" : null,
          fusionProfile: hybridFusion?.profile ?? null,
          fusionConfig: buildFusionConfigSummary(hybridFusion?.profile ?? null, hybridFusion?.config ?? null, rankingConfig),
          lexicalQuery,
          semanticQuery: rawSemanticQuery,
          query: queryAnalysis
            ? {
                rawQuery: queryAnalysis.rawQuery,
                normalizedQuery: queryAnalysis.normalizedQuery,
                queryTokens: queryAnalysis.queryTokens,
              }
            : null,
          rankingConfig: this.getRankingConfigStatus(),
          records: page.map(({ explanation }) => explanation),
        }
      : undefined;

    return {
      searchProfile,
      mode,
      total: scored.length,
      offset,
      limit,
      records: page.map(({ record }) => record),
      explain,
    };
  }

  lookup(name: string, options: LookupOptions = {}): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const results = this.searchStructured({
      nameQuery: name,
      pack: options.pack,
      category: options.category,
      subcategory: options.subcategory,
      limit: 5,
    }).records;

    return {
      match: results[0] ?? null,
      alternatives: results.slice(1),
    };
  }
}
