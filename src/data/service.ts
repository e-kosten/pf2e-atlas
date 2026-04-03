import { mkdir, readdir, readFile, rm, stat } from "node:fs/promises";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

import {
  createEmbeddingProvider,
  EmbeddingProvider,
} from "../embeddings.js";
import { DERIVED_TAG_CATALOG, deriveRecordTags, normalizeDerivedTag } from "../tags/index.js";
import {
  categorySupportsSubcategory,
  classifyRecordCategory,
  extractSpellTraditions,
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import { NormalizedSearchFilters, NormalizedSearchScope, SqlValue } from "./service-types.js";
import { DEFAULT_RANKING_CONFIG, RankingConfig, RankingConfigStore } from "../search/ranking-config.js";
import { normalizeMetadataFilterNode } from "../search/metadata-filters.js";
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
  SearchExplainResult,
  SearchFilters,
  SearchRecordExplanation,
  SearchResult,
  SearchSubcategory,
  SourceCategory,
} from "../types.js";
import {
  buildCandidateQuery,
  buildFilterValueQuery,
  buildLexicalRetrievalQuery,
  buildSemanticRetrievalQuery,
  normalizeSearchScope,
  recordMatchesFilters,
  semanticQueryLimit,
} from "../search/sql.js";
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
} from "../utils.js";
import { buildLiteralQueryWeights, buildSearchQueryAnalysis } from "../search-query-analysis.js";
import {
  backlinkTypeRank,
  buildPlaceholders,
  CandidateRow,
  edgeRowToReferenceEdge,
  extractQuestionRuleNames,
  getLookupMatchType,
  ReferenceEdgeRow,
  rowToRecord,
  sourceCategoryRank,
  sqliteRowCount,
  ValueCountRow,
} from "./rows.js";
import {
  buildMissingIndexError,
  buildStaleIndexError,
  createSchema,
  defaultEmbeddingConfig,
  defaultIndexPath,
  getIndexInvalidReason,
  INDEX_SCHEMA_VERSION,
  loadAliasesByRecordKey,
  loadLegacyLinksByRecordKey,
  loadPacksFromIndex,
  openDatabase,
} from "./schema.js";
import {
  buildFusionConfigSummary,
  buildLexicalSignal,
  buildNormalizedRankScoreMap,
  buildRankMap,
  buildRerankAdjustments,
  compareOptionalRanks,
  computeWeightedRrfScore,
  hasStructuredFilterSignal,
  LexicalRetrievalRow,
  SemanticRetrievalRow,
  packQualityScore,
  rarityPreferenceScore,
  resolveHybridFusionProfile,
  resolveSearchMode,
  resolveSearchProfile,
  sourcePenaltyScore,
  sourceQualityScore,
  sumRerankAdjustments,
} from "../search/ranking.js";

const execFileAsync = promisify(execFile);
const VEC_TEXT_NONE = "";
const VEC_INT_NONE = -1n;
const LOOKUP_LEXICAL_TOP_K = 100;
const EMBEDDING_BATCH_SIZE = 64;
const MAX_ACTOR_SEMANTIC_ITEM_CHUNKS = 40;

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

type StageTiming = {
  label: string;
  durationMs: number;
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
  families: string[];
  sourcePath: string;
  isUnique: boolean;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  usage: string | null;
  hands: number | null;
  damageTypes: string[];
  weaponGroup: string | null;
  armorGroup: string | null;
  traditions: string[];
  spellKinds: string[];
  languages: string[];
  speedTypes: string[];
  immunities: string[];
  resistances: string[];
  weaknesses: string[];
  rangeValue: number | null;
  searchText: string;
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

type PendingCanonicalEmbedding = {
  record: NormalizedIndexRecord;
  encodedEmbeddingInput: string;
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

function formatDurationMs(durationMs: number): string {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds}s`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
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

function appendUniqueTextChunk(chunks: string[], seen: Set<string>, value: string | null | undefined): boolean {
  if (!value) {
    return false;
  }

  const normalized = normalizeText(value);
  if (!normalized || seen.has(normalized)) {
    return false;
  }

  seen.add(normalized);
  chunks.push(value.trim());
  return true;
}

function buildActorSemanticItemChunks(raw: Record<string, unknown>): string[] {
  const chunks: string[] = [];
  const seen = new Set<string>();
  const items = getNested(raw, ["items"]);
  if (!Array.isArray(items)) {
    return chunks;
  }

  for (const entry of items) {
    if (!entry || typeof entry !== "object") {
      continue;
    }

    const item = entry as Record<string, unknown>;
    appendUniqueTextChunk(chunks, seen, firstString(item.name));
    if (chunks.length >= MAX_ACTOR_SEMANTIC_ITEM_CHUNKS) {
      break;
    }

    for (const trait of getTraits(item)) {
      appendUniqueTextChunk(chunks, seen, trait);
      if (chunks.length >= MAX_ACTOR_SEMANTIC_ITEM_CHUNKS) {
        break;
      }
    }

    if (chunks.length >= MAX_ACTOR_SEMANTIC_ITEM_CHUNKS) {
      break;
    }
  }

  return chunks;
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

function buildSemanticEmbeddingText(record: NormalizedIndexRecord, raw: Record<string, unknown>, aliases: string[]): string {
  const chunks: string[] = [];
  const seen = new Set<string>();

  appendUniqueTextChunk(chunks, seen, record.name);
  for (const trait of record.traits) {
    appendUniqueTextChunk(chunks, seen, trait);
  }
  for (const family of record.families) {
    appendUniqueTextChunk(chunks, seen, family);
  }
  for (const tag of record.derivedTags) {
    appendUniqueTextChunk(chunks, seen, tag);
  }
  appendUniqueTextChunk(chunks, seen, record.descriptionSnippet);

  if (record.documentType === "Actor") {
    for (const itemChunk of buildActorSemanticItemChunks(raw)) {
      appendUniqueTextChunk(chunks, seen, itemChunk);
    }
  }

  for (const alias of aliases) {
    appendUniqueTextChunk(chunks, seen, alias);
  }

  return chunks.join("\n").trim();
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
    families: [],
    sourcePath,
    isUnique: normalizeText(rarity ?? "") === "unique",
    size: actorData?.size ?? null,
    itemCategory: itemData?.itemCategory ?? null,
    priceCp: itemData?.priceCp ?? null,
    bulkValue: itemData?.bulkValue ?? null,
    actionCost: spellData?.actionCost ?? itemData?.actionCost ?? null,
    usage: itemData?.usage ?? null,
    hands: itemData?.hands ?? null,
    damageTypes: spellData?.damageTypes ?? itemData?.damageTypes ?? [],
    weaponGroup: itemData?.weaponGroup ?? null,
    armorGroup: itemData?.armorGroup ?? null,
    traditions: spellData?.traditions ?? [],
    spellKinds: spellData?.spellKinds ?? [],
    languages: actorData?.languages ?? [],
    speedTypes: actorData?.speedTypes ?? [],
    immunities: actorData?.immunities ?? [],
    resistances: actorData?.resistances ?? [],
    weaknesses: actorData?.weaknesses ?? [],
    rangeValue: spellData?.rangeValue ?? null,
    searchText: buildSearchText(raw, { name, descriptionText, traits }),
  };
}

function getPackAliasValues(pack: PackInfo, value: string): boolean {
  const normalized = normalizeText(value);
  return normalized === normalizeText(pack.name) || normalized === normalizeText(pack.label);
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

function normalizeFamilyName(value: string): string | null {
  const normalized = normalizeText(value).replace(/\s+/g, "-");
  return normalized || null;
}

type FolderDefinition = {
  _id?: string;
  name?: string;
  folder?: string | null;
};

function buildPackAndFolderKey(packName: string, folderId: string): string {
  return `${normalizeText(packName)}:${normalizeText(folderId)}`;
}

function resolveFolderFamily(folderId: string, foldersById: Map<string, FolderDefinition>): string | null {
  const visited = new Set<string>();
  let currentId: string | null = folderId;
  let current: FolderDefinition | undefined;

  while (currentId) {
    if (visited.has(currentId)) {
      return null;
    }
    visited.add(currentId);
    current = foldersById.get(currentId);
    if (!current) {
      return null;
    }
    const parentId = firstString(current.folder);
    if (!parentId) {
      return normalizeFamilyName(firstString(current.name) ?? "");
    }
    currentId = parentId;
  }

  return current ? normalizeFamilyName(firstString(current.name) ?? "") : null;
}

async function loadFolderFamilyMap(pack: Pick<PackBuildInfo, "name" | "resolvedPath">): Promise<Map<string, string>> {
  if (!pack.resolvedPath) {
    return new Map();
  }

  const folderPath = path.join(pack.resolvedPath, "_folders.json");
  if (!await fileExists(folderPath)) {
    return new Map();
  }

  const raw = JSON.parse(await readFile(folderPath, "utf8")) as FolderDefinition[];
  if (!Array.isArray(raw)) {
    return new Map();
  }

  const foldersById = new Map<string, FolderDefinition>();
  for (const entry of raw) {
    const id = firstString(entry?._id);
    if (id) {
      foldersById.set(id, entry);
    }
  }

  const familyMap = new Map<string, string>();
  for (const folderId of foldersById.keys()) {
    const family = resolveFolderFamily(folderId, foldersById);
    if (family) {
      familyMap.set(buildPackAndFolderKey(pack.name, folderId), family);
    }
  }

  return familyMap;
}

function assignFamilies(
  entry: BuildSourceEntry & { record: NormalizedIndexRecord },
  recordsByPackAndId: Map<string, string>,
  glossaryFamilyByRecordKey: Map<string, string>,
  folderFamilyByPackAndFolderId: Map<string, string>,
): void {
  const families = new Set<string>();

  if (entry.record.folderId) {
    const folderFamily = folderFamilyByPackAndFolderId.get(buildPackAndFolderKey(entry.record.packName, entry.record.folderId));
    if (folderFamily) {
      families.add(folderFamily);
    }
  }

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

    families.add(family);
  }

  entry.record.families = uniqueSorted([...families]);
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

async function buildIndex(
  db: DatabaseSync,
  rootPath: string,
  manifestPath: string,
  embeddingProvider: EmbeddingProvider,
  sourceSignature: string,
  progressLogger?: (message: string) => void,
  progressStatusLogger?: (message: string) => void,
): Promise<{ packs: PackInfo[]; warnings: string[]; recordCount: number; stageTimings: StageTiming[] }> {
  const manifestRaw = JSON.parse(await readFile(manifestPath, "utf8")) as { packs?: PackManifestEntry[] };
  const manifestPacks = Array.isArray(manifestRaw.packs) ? manifestRaw.packs : [];
  const includedManifestPacks = manifestPacks.filter((manifestPack) => !isExcludedPackName(manifestPack.name));

  const warnings: string[] = [];
  const packs: PackInfo[] = [];
  let recordCount = 0;
  let processedPackCount = 0;
  const sourceEntries: BuildSourceEntry[] = [];
  const scanStartTime = Date.now();
  let scanNormalizationDurationMs = 0;
  let resolutionDurationMs = 0;
  let recordStorageDurationMs = 0;
  let embeddingGenerationDurationMs = 0;
  let vecInsertDurationMs = 0;

  const insertPack = db.prepare(`
    INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
    VALUES (?, ?, ?, ?, ?, ?)
  `);
  const insertRecord = db.prepare(`
    INSERT INTO records (
      record_key, id, name, normalized_name, category, subcategory, pack_name, pack_label, document_type, record_type,
      level, rarity, traits_json, derived_tags_json, publication_title, publication_remaster, description_text, has_description, description_snippet,
      source_category, folder_id, families_json, source_path, is_unique, is_search_canonical, search_text, raw_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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

    scanNormalizationDurationMs = Date.now() - scanStartTime;

    progressLogger?.("Finished scanning pack files. Resolving verified remaster aliases.");
    const resolutionStartTime = Date.now();

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
    const folderFamilyByPackAndFolderId = new Map<string, string>();
    for (const entry of indexedEntries) {
      if (entry.record.packName !== "bestiary-family-ability-glossary") {
        continue;
      }

      const family = deriveGlossaryFamilyFromPath(entry.filePath, entry.pack.resolvedPath);
      if (family) {
        glossaryFamilyByRecordKey.set(entry.record.recordKey, family);
      }
    }

    const folderFamilyMaps = await Promise.all(packs.map(async (pack) => loadFolderFamilyMap(pack)));
    for (const familyMap of folderFamilyMaps) {
      for (const [key, value] of familyMap) {
        folderFamilyByPackAndFolderId.set(key, value);
      }
    }

    for (const entry of indexedEntries) {
      assignFamilies(entry, recordsByPackAndId, glossaryFamilyByRecordKey, folderFamilyByPackAndFolderId);

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
        families: entry.record.families,
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

    resolutionDurationMs = Date.now() - resolutionStartTime;

    const canonicalEmbeddingCount = indexedEntries.filter((entry) => !suppressedRecordKeys.has(entry.record.recordKey)).length;
    const recordWriteProgressInterval = Math.max(100, Math.ceil(indexedEntries.length / 10));
    const embeddingProgressInterval = Math.max(25, Math.ceil(Math.max(canonicalEmbeddingCount, 1) / 10));
    const pendingCanonicalEmbeddings: PendingCanonicalEmbedding[] = [];
    let writtenRecordCount = 0;
    let embeddedRecordCount = 0;
    let lastRecordProgressLogTime = 0;
    let lastLoggedRecordCount = 0;
    let lastEmbeddingProgressLogTime = 0;
    let lastLoggedEmbeddedCount = 0;
    const recordStorageStartTime = Date.now();

    progressLogger?.("Writing indexed records and search metadata.");

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
        JSON.stringify(record.families),
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

      if (isSearchCanonical) {
        insertFts.run(record.recordKey, record.name, searchText);
        pendingCanonicalEmbeddings.push({
          record,
          encodedEmbeddingInput: buildSemanticEmbeddingText(record, entry.raw, aliasTexts),
        });
      }

      writtenRecordCount += 1;
      const now = Date.now();
      const shouldLogProgress = writtenRecordCount === indexedEntries.length ||
        (writtenRecordCount - lastLoggedRecordCount) >= recordWriteProgressInterval ||
        (now - lastRecordProgressLogTime) >= PACK_PROGRESS_LOG_INTERVAL_MS;

      if (shouldLogProgress) {
        progressStatusLogger?.(
          `[write] Stored records ${renderProgressBar(writtenRecordCount, indexedEntries.length)} ${formatPercentage(writtenRecordCount, indexedEntries.length)} (${formatInteger(writtenRecordCount)}/${formatInteger(indexedEntries.length)} records).`,
        );
        lastRecordProgressLogTime = now;
        lastLoggedRecordCount = writtenRecordCount;
      }
    }

    recordStorageDurationMs = Date.now() - recordStorageStartTime;

    progressLogger?.(`Generating canonical embeddings in batches of ${EMBEDDING_BATCH_SIZE}.`);

    for (let index = 0; index < pendingCanonicalEmbeddings.length; index += EMBEDDING_BATCH_SIZE) {
      const batch = pendingCanonicalEmbeddings.slice(index, index + EMBEDDING_BATCH_SIZE);
      const embeddingStartTime = Date.now();
      const embeddings = await embeddingProvider.embedMany(batch.map((entry) => entry.encodedEmbeddingInput));
      embeddingGenerationDurationMs += Date.now() - embeddingStartTime;

      const vecInsertStartTime = Date.now();
      for (const [batchIndex, entry] of batch.entries()) {
        const embedding = embeddings[batchIndex] ?? new Float32Array(embeddingProvider.identity.dimensions);
        const encodedEmbedding = encodeVector(embedding);
        const record = entry.record;

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
      }
      vecInsertDurationMs += Date.now() - vecInsertStartTime;

      embeddedRecordCount += batch.length;
      const now = Date.now();
      const shouldLogProgress = embeddedRecordCount === canonicalEmbeddingCount ||
        (embeddedRecordCount - lastLoggedEmbeddedCount) >= embeddingProgressInterval ||
        (now - lastEmbeddingProgressLogTime) >= PACK_PROGRESS_LOG_INTERVAL_MS;

      if (shouldLogProgress) {
        progressStatusLogger?.(
          `[embed] Canonical embeddings ${renderProgressBar(embeddedRecordCount, canonicalEmbeddingCount)} ${formatPercentage(embeddedRecordCount, canonicalEmbeddingCount)} (${formatInteger(embeddedRecordCount)}/${formatInteger(canonicalEmbeddingCount)} embeddings).`,
        );
        lastEmbeddingProgressLogTime = now;
        lastLoggedEmbeddedCount = embeddedRecordCount;
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
  return {
    packs,
    warnings,
    recordCount,
    stageTimings: [
      { label: "Scan and normalize records", durationMs: scanNormalizationDurationMs },
      { label: "Resolve families, references, tags, and aliases", durationMs: resolutionDurationMs },
      { label: "Write records and lexical search metadata", durationMs: recordStorageDurationMs },
      { label: "Generate canonical embeddings", durationMs: embeddingGenerationDurationMs },
      { label: "Insert vector rows", durationMs: vecInsertDurationMs },
    ],
  };
}

async function removeIndexFiles(indexPath: string): Promise<void> {
  await rm(indexPath, { force: true });
  await rm(`${indexPath}-wal`, { force: true });
  await rm(`${indexPath}-shm`, { force: true });
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
    const rebuildStartTime = Date.now();
    const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
    const embeddingConfig: EmbeddingConfig = options.embedding ?? defaultEmbeddingConfig(indexPath);
    const embeddingProviderFactory = options.embeddingProviderFactory ?? createEmbeddingProvider;
    options.progressLogger?.("Loading the configured embedding provider.");
    const embeddingProviderLoadStartTime = Date.now();
    const embeddingRuntime = await embeddingProviderFactory(embeddingConfig);
    const embeddingProviderLoadDurationMs = Date.now() - embeddingProviderLoadStartTime;
    const embeddingProvider = embeddingRuntime.provider;
    options.progressLogger?.(
      `Embedding provider ready: ${embeddingProvider.identity.model} (${embeddingProvider.identity.dimensions} dimensions).`,
    );
    options.progressLogger?.("Computing the PF2E source signature.");
    const sourceSignatureStartTime = Date.now();
    const sourceSignature = await computeSourceSignature(rootPath, manifestPath);
    const sourceSignatureDurationMs = Date.now() - sourceSignatureStartTime;
    options.progressLogger?.(`Preparing index output at ${indexPath}.`);
    const prepareOutputStartTime = Date.now();
    await mkdir(path.dirname(indexPath), { recursive: true });
    await removeIndexFiles(indexPath);
    const prepareOutputDurationMs = Date.now() - prepareOutputStartTime;

    const db = openDatabase(indexPath, {
      vectorExtensionLoader: options.vectorExtensionLoader,
    });
    options.progressLogger?.("Creating SQLite schema.");
    const schemaCreationStartTime = Date.now();
    createSchema(db, embeddingProvider.identity.dimensions);
    const schemaCreationDurationMs = Date.now() - schemaCreationStartTime;
    const { packs, warnings, recordCount, stageTimings } = await buildIndex(
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
    const rebuildDurationMs = Date.now() - rebuildStartTime;
    const summaryTimings: StageTiming[] = [
      { label: "Embedding provider load", durationMs: embeddingProviderLoadDurationMs },
      { label: "Source signature", durationMs: sourceSignatureDurationMs },
      { label: "Prepare index output", durationMs: prepareOutputDurationMs },
      { label: "Create SQLite schema", durationMs: schemaCreationDurationMs },
      ...stageTimings,
      { label: "Total rebuild time", durationMs: rebuildDurationMs },
    ];
    options.progressLogger?.("Index rebuild stage timings:");
    for (const timing of summaryTimings) {
      options.progressLogger?.(`- ${timing.label}: ${formatDurationMs(timing.durationMs)}`);
    }
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
      metadata: filters.metadata ? normalizeMetadataFilterNode(filters.metadata) : undefined,
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
            r.families_json AS familiesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
            a.languages_json AS languagesJson,
            a.speed_types_json AS speedTypesJson,
            a.immunities_json AS immunitiesJson,
            a.resistances_json AS resistancesJson,
            a.weaknesses_json AS weaknessesJson,
            i.item_category AS itemCategory,
            i.price_cp AS priceCp,
            i.bulk_value AS bulkValue,
            i.usage_text AS usage,
            i.hands AS hands,
            COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson,
            i.weapon_group AS weaponGroup,
            i.armor_group AS armorGroup,
            COALESCE(s.action_cost, i.action_cost) AS actionCost,
            s.traditions_json AS traditionsJson,
            s.spell_kinds_json AS spellKindsJson,
            s.range_value AS rangeValue
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
                r.families_json AS familiesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
                a.languages_json AS languagesJson,
                a.speed_types_json AS speedTypesJson,
                a.immunities_json AS immunitiesJson,
                a.resistances_json AS resistancesJson,
                a.weaknesses_json AS weaknessesJson,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                i.usage_text AS usage,
                i.hands AS hands,
                COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson,
                i.weapon_group AS weaponGroup,
                i.armor_group AS armorGroup,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson,
                s.spell_kinds_json AS spellKindsJson,
                s.range_value AS rangeValue,
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
                r.families_json AS familiesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
                a.languages_json AS languagesJson,
                a.speed_types_json AS speedTypesJson,
                a.immunities_json AS immunitiesJson,
                a.resistances_json AS resistancesJson,
                a.weaknesses_json AS weaknessesJson,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                i.usage_text AS usage,
                i.hands AS hands,
                COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson,
                i.weapon_group AS weaponGroup,
                i.armor_group AS armorGroup,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson,
                s.spell_kinds_json AS spellKindsJson,
                s.range_value AS rangeValue,
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
          metadata: { field: "sourceCategory", op: "eq", value: "core" },
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
