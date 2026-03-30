import { readFileSync } from "node:fs";
import { mkdir, readdir, readFile, rm, stat } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

import {
  CollectRuleQuestionContextInput,
  CollectRuleQuestionContextResult,
  LookupOptions,
  LookupQuery,
  LookupResult,
  NormalizedRecord,
  PackInfo,
  PackManifestEntry,
  RuleGraphResult,
  RuleReferenceEdge,
  SourceCategory,
  SearchFilters,
  SearchExplainResult,
  SearchRecordExplanation,
  SearchMode,
  SearchResult,
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
import { buildCandidateQueryWeights, buildSearchQueryAnalysis } from "./search-expansion.js";

const execFileAsync = promisify(execFile);
const INDEX_SCHEMA_VERSION = 4;

type LoadOptions = {
  indexPath?: string;
};

type SqlValue = string | number | bigint | Uint8Array | Buffer | null;

type CandidateRow = {
  recordKey: string;
  id: string;
  name: string;
  normalizedName: string;
  type: string;
  packName: string;
  packLabel: string;
  documentType: string;
  level: number | null;
  rarity: string | null;
  traitsJson: string;
  publicationTitle: string | null;
  descriptionText: string | null;
  hasDescription: number;
  descriptionSnippet: string | null;
  sourceCategory: SourceCategory;
  folderId: string | null;
  sourcePath: string;
  isUnique: number;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  traditionsJson: string | null;
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
  packName: string;
  packLabel: string;
  documentType: string;
  level: number | null;
  rarity: string | null;
  traits: string[];
  publicationTitle: string | null;
  descriptionText: string | null;
  hasDescription: boolean;
  descriptionSnippet: string | null;
  sourceCategory: SourceCategory;
  folderId: string | null;
  sourcePath: string;
  isUnique: boolean;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  traditions: string[];
  searchText: string;
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
  rangeText: string | null;
  rangeValue: number | null;
  saveType: string | null;
  areaType: string | null;
  damageTypes: string[];
};

interface EmbeddingProvider {
  readonly dimensions: number;
  embed(text: string): Float32Array;
}

class HashEmbeddingProvider implements EmbeddingProvider {
  readonly dimensions: number;

  constructor(dimensions = 192) {
    this.dimensions = dimensions;
  }

  embed(text: string): Float32Array {
    const normalized = normalizeText(text);
    const vector = new Float32Array(this.dimensions);
    if (!normalized) {
      return vector;
    }

    for (const token of normalized.split(" ").filter(Boolean)) {
      const bucket = hashText(token) % this.dimensions;
      const sign = hashText(`${token}:sign`) % 2 === 0 ? 1 : -1;
      vector[bucket] = (vector[bucket] ?? 0) + sign;
    }

    return normalizeVector(vector);
  }
}

function hashText(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  return hash >>> 0;
}

function normalizeVector(vector: Float32Array): Float32Array {
  let magnitude = 0;
  for (const value of vector) {
    magnitude += value * value;
  }

  if (magnitude === 0) {
    return vector;
  }

  const scale = 1 / Math.sqrt(magnitude);
  for (let index = 0; index < vector.length; index += 1) {
    vector[index] = (vector[index] ?? 0) * scale;
  }

  return vector;
}

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
  const valueTraits = toStringArray(getNested(raw, ["system", "traits", "value"]));
  const traditionTraits = toStringArray(getNested(raw, ["system", "traits", "traditions"]));
  return uniqueSorted([...valueTraits, ...traditionTraits]);
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

function parseSpellIndexData(raw: Record<string, unknown>): SpellIndexData {
  return {
    actionCost: parseActionCost(raw),
    traditions: uniqueSorted(toStringArray(getNested(raw, ["system", "traits", "traditions"]))),
    rangeText: firstString(getNested(raw, ["system", "range", "value"])),
    rangeValue: parseRangeValue(raw),
    saveType: firstString(getNested(raw, ["system", "defense", "save", "statistic"])),
    areaType: firstString(getNested(raw, ["system", "area", "type"])),
    damageTypes: parseDamageTypes(raw),
  };
}

function buildSearchText(raw: Record<string, unknown>, base: { name: string; descriptionText: string | null; traits: string[]; publicationTitle: string | null }): string {
  const chunks: string[] = [base.name, ...base.traits];
  if (base.descriptionText) {
    chunks.push(base.descriptionText);
  }
  if (base.publicationTitle) {
    chunks.push(base.publicationTitle);
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
  return isExcludedPackName(pack.name) || isExcludedSocietyEffectPath(sourcePath) || isPfsBoonRecord(raw);
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
  const hasDescription = hasDescriptionText(descriptionText);
  const descriptionSnippet = buildDescriptionSnippet(descriptionText);
  const sourceCategory = getSourceCategory(pack.name, publicationTitle);
  const actorData = pack.documentType === "Actor" ? parseActorIndexData(raw) : null;
  const itemData = pack.documentType === "Item" ? parseItemIndexData(raw) : null;
  const spellData = recordType === "spell" ? parseSpellIndexData(raw) : null;

  return {
    recordKey: `${pack.name}:${id}`,
    id,
    name,
    normalizedName: normalizeText(name),
    type: recordType,
    packName: pack.name,
    packLabel: pack.label,
    documentType: pack.documentType,
    level: getLevel(raw),
    rarity,
    traits,
    publicationTitle,
    descriptionText,
    hasDescription,
    descriptionSnippet,
    sourceCategory,
    folderId: firstString(raw.folder),
    sourcePath,
    isUnique: normalizeText(rarity ?? "") === "unique",
    size: actorData?.size ?? null,
    itemCategory: itemData?.itemCategory ?? null,
    priceCp: itemData?.priceCp ?? null,
    bulkValue: itemData?.bulkValue ?? null,
    actionCost: spellData?.actionCost ?? itemData?.actionCost ?? null,
    traditions: spellData?.traditions ?? [],
    searchText: buildSearchText(raw, { name, descriptionText, traits, publicationTitle }),
  };
}

function getPackAliasValues(pack: PackInfo, value: string): boolean {
  const normalized = normalizeText(value);
  return normalized === normalizeText(pack.name) || normalized === normalizeText(pack.label);
}

function packQualityScore(record: NormalizedRecord): number {
  const normalizedPack = normalizeText(record.packName);
  let score = 0;

  if (normalizedPack.includes("macro")) {
    score -= 0.2;
  }

  if (normalizedPack.includes("glossary")) {
    score -= 0.1;
  }

  if (normalizedPack.includes("effect")) {
    score -= 0.05;
  }

  if (normalizedPack === "actions" || normalizedPack === "spells" || normalizedPack === "equipment" || normalizedPack === "feats") {
    score += 0.05;
  }

  return score;
}

function rankingProfileScore(record: NormalizedRecord, filters: SearchFilters): number {
  if (filters.rankingProfile !== "preferReusableReferenceContent") {
    return 0;
  }

  let score = 0;

  if (!record.hasDescription) {
    score -= 0.3;
  } else {
    score += 0.12;
  }

  if (record.sourceCategory === "core") {
    score += 0.15;
  } else if (record.sourceCategory === "rules") {
    score += 0.08;
  } else if (record.sourceCategory === "adventure") {
    score -= 0.08;
  }

  if (record.isUnique) {
    score -= 0.06;
  } else {
    score += 0.08;
  }

  return score;
}

function metadataOnlyBoostMultiplier(record: NormalizedRecord, filters: SearchFilters): number {
  if (record.hasDescription || !filters.themeQuery?.trim()) {
    return 1;
  }

  let multiplier = 1;
  if (isSocietyPublication(record.publicationTitle) || isSocietyPack(record.packName)) {
    multiplier *= 0.15;
  }
  if (hasScenarioScaleSuffix(record.name)) {
    multiplier *= 0.5;
  }

  return multiplier;
}

function sourcePenaltyScore(record: NormalizedRecord, filters: SearchFilters): number {
  if (record.hasDescription || !filters.themeQuery?.trim()) {
    return 0;
  }

  let penalty = 0;
  if (isSocietyPublication(record.publicationTitle) || isSocietyPack(record.packName)) {
    penalty -= 0.2;
  }
  if (hasScenarioScaleSuffix(record.name)) {
    penalty -= 0.1;
  }

  return penalty;
}

function nameScore(query: string, record: NormalizedRecord): number {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return 0.5;
  }

  if (record.normalizedName === normalizedQuery) {
    return 1;
  }

  if (record.normalizedName.startsWith(normalizedQuery)) {
    return 0.95;
  }

  if (record.normalizedName.includes(normalizedQuery)) {
    return 0.9;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const nameTokens = new Set(record.normalizedName.split(" ").filter(Boolean));
  const overlap = queryTokens.filter((token) => nameTokens.has(token)).length;
  const tokenScore = queryTokens.length > 0 ? overlap / queryTokens.length : 0;
  const dice = bigramDice(normalizedQuery, record.normalizedName);
  return Math.max(tokenScore * 0.8, dice * 0.75);
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

type SearchScoreComponents = SearchRecordExplanation["components"];

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

function buildMetadataText(record: NormalizedRecord): string {
  return [record.name, ...record.traits, record.publicationTitle ?? ""]
    .filter((value) => value.length > 0)
    .join(" ");
}

function resolveSearchMode(filters: SearchFilters, context: "list" | "search"): SearchMode {
  if (filters.mode) {
    return filters.mode;
  }

  if (context === "search" && filters.themeQuery?.trim()) {
    return "hybrid";
  }

  return "structured";
}

type ExtractedReference = {
  packName: string | null;
  recordLocator: string;
  displayText: string | null;
  referenceText: string;
};

type RulesContextEdge = {
  fromRecordKey: string;
  toRecordKey: string;
  displayText: string | null;
  referenceText: string;
  depth: number;
};

const UUID_REFERENCE_PATTERN = /@UUID\[([^\]]+)\](?:\{([^}]+)\})?/g;
const COMPILED_REFERENCE_PATTERN = /^Compendium\.pf2e\.([^.]+)\.[^.]+\.([^.\]]+)$/i;

function extractRulesReferences(raw: Record<string, unknown>): ExtractedReference[] {
  const markup = getDescriptionMarkup(raw);
  if (!markup) {
    return [];
  }

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

function rowToRecord(row: CandidateRow, raw: Record<string, unknown> = {}): NormalizedRecord {
  return {
    recordKey: row.recordKey,
    id: row.id,
    name: row.name,
    normalizedName: row.normalizedName,
    type: row.type,
    packName: row.packName,
    packLabel: row.packLabel,
    documentType: row.documentType,
    level: row.level,
    rarity: row.rarity,
    traits: JSON.parse(row.traitsJson) as string[],
    publicationTitle: row.publicationTitle,
    descriptionText: row.descriptionText,
    hasDescription: Boolean(row.hasDescription),
    descriptionSnippet: row.descriptionSnippet,
    sourceCategory: row.sourceCategory,
    folderId: row.folderId,
    sourcePath: row.sourcePath,
    isUnique: Boolean(row.isUnique),
    size: row.size,
    itemCategory: row.itemCategory,
    priceCp: row.priceCp,
    bulkValue: row.bulkValue,
    actionCost: row.actionCost,
    traditions: row.traditionsJson ? (JSON.parse(row.traditionsJson) as string[]) : [],
    raw,
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
      const [{ stdout: headStdout }, { stdout: statusStdout }] = await Promise.all([
        execFileAsync("git", ["-C", rootPath, "rev-parse", "HEAD"], { timeout: 10_000 }),
        execFileAsync("git", ["-C", rootPath, "status", "--porcelain", "--untracked-files=no"], { timeout: 10_000 }),
      ]);
      const head = headStdout.trim();
      const dirty = statusStdout.trim();
      return `git:${head}:${dirty}`;
    } catch {
      // Fall through to filesystem signature.
    }
  }

  const files = [manifestPath, ...(await walkJsonFiles(rootPath))].sort((left, right) => left.localeCompare(right));
  let hash = 2166136261;
  for (const filePath of files) {
    const details = await stat(filePath);
    const value = `${path.relative(rootPath, filePath)}:${details.size}:${Math.trunc(details.mtimeMs)}`;
    hash ^= hashText(value);
    hash = Math.imul(hash, 16777619);
  }

  return `fs:${hash >>> 0}`;
}

function createSchema(db: DatabaseSync): void {
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
      pack_name TEXT NOT NULL,
      pack_label TEXT NOT NULL,
      document_type TEXT NOT NULL,
      record_type TEXT NOT NULL,
      level INTEGER,
      rarity TEXT,
      traits_json TEXT NOT NULL,
      publication_title TEXT,
      description_text TEXT,
      has_description INTEGER NOT NULL,
      description_snippet TEXT,
      source_category TEXT NOT NULL,
      folder_id TEXT,
      source_path TEXT NOT NULL,
      is_unique INTEGER NOT NULL,
      search_text TEXT NOT NULL
    );

    CREATE TABLE record_traits (
      record_key TEXT NOT NULL,
      trait TEXT NOT NULL,
      PRIMARY KEY (record_key, trait),
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
    CREATE INDEX records_doc_type_idx ON records(document_type);
    CREATE INDEX records_record_type_idx ON records(record_type);
    CREATE INDEX records_level_idx ON records(level);
    CREATE INDEX records_rarity_idx ON records(rarity);
    CREATE INDEX records_unique_idx ON records(is_unique);
    CREATE INDEX records_has_description_idx ON records(has_description);
    CREATE INDEX records_source_category_idx ON records(source_category);
    CREATE INDEX record_traits_trait_idx ON record_traits(trait);
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
): Promise<{ packs: PackInfo[]; warnings: string[]; recordCount: number }> {
  const manifestRaw = JSON.parse(await readFile(manifestPath, "utf8")) as { packs?: PackManifestEntry[] };
  const manifestPacks = Array.isArray(manifestRaw.packs) ? manifestRaw.packs : [];

  const warnings: string[] = [];
  const packs: PackInfo[] = [];
  let recordCount = 0;

  const insertPack = db.prepare(`
    INSERT INTO packs (name, label, document_type, declared_path, resolved_path, record_count)
    VALUES (?, ?, ?, ?, ?, ?)
  `);
  const insertRecord = db.prepare(`
    INSERT INTO records (
      record_key, id, name, normalized_name, pack_name, pack_label, document_type, record_type,
      level, rarity, traits_json, publication_title, description_text, has_description, description_snippet,
      source_category, folder_id, source_path, is_unique, search_text
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertTrait = db.prepare(`
    INSERT INTO record_traits (record_key, trait) VALUES (?, ?)
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
      record_key, action_cost, traditions_json, range_text, range_value, save_type, area_type, damage_types_json
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);
  const insertEmbedding = db.prepare(`
    INSERT INTO embeddings (record_key, dimensions, vector_blob) VALUES (?, ?, ?)
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
    insertMetadata.run("schema_version", String(INDEX_SCHEMA_VERSION));
    insertMetadata.run("source_signature", sourceSignature);
    insertMetadata.run("embedding_dimensions", String(embeddingProvider.dimensions));

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

      let filePaths: string[];
      try {
        filePaths = await walkJsonFiles(pack.resolvedPath);
      } catch (error) {
        warnings.push(`Skipping pack ${pack.name}: ${(error as Error).message}`);
        continue;
      }

      let packRecordCount = 0;
      for (const filePath of filePaths) {
        const raw = JSON.parse(await readFile(filePath, "utf8")) as Record<string, unknown>;
        if (shouldExcludeRecordFromIndex(pack, filePath, raw)) {
          continue;
        }
        const record = normalizeIndexRecord(pack, filePath, raw);
        const actorData = pack.documentType === "Actor" ? parseActorIndexData(raw) : null;
        const itemData = pack.documentType === "Item" ? parseItemIndexData(raw) : null;
        const spellData = record.type === "spell" ? parseSpellIndexData(raw) : null;
        const embedding = embeddingProvider.embed(record.searchText);

        insertRecord.run(
          record.recordKey,
          record.id,
          record.name,
          record.normalizedName,
          record.packName,
          record.packLabel,
          record.documentType,
          record.type,
          record.level,
          record.rarity,
          JSON.stringify(record.traits),
          record.publicationTitle,
          record.descriptionText,
          record.hasDescription ? 1 : 0,
          record.descriptionSnippet,
          record.sourceCategory,
          record.folderId,
          record.sourcePath,
          record.isUnique ? 1 : 0,
          record.searchText,
        );

        for (const trait of record.traits) {
          insertTrait.run(record.recordKey, normalizeText(trait));
        }

        if (actorData) {
          insertActor.run(
            record.recordKey,
            actorData.size,
            JSON.stringify(actorData.languages),
            JSON.stringify(actorData.speedTypes),
            JSON.stringify(actorData.immunities),
            JSON.stringify(actorData.resistances),
            JSON.stringify(actorData.weaknesses),
          );
        }

        if (itemData) {
          insertItem.run(
            record.recordKey,
            itemData.itemCategory,
            itemData.priceCp,
            itemData.bulkValue,
            itemData.usage,
            itemData.hands,
            JSON.stringify(itemData.damageTypes),
            itemData.weaponGroup,
            itemData.armorGroup,
            itemData.actionCost,
          );
        }

        if (spellData) {
          insertSpell.run(
            record.recordKey,
            spellData.actionCost,
            JSON.stringify(spellData.traditions),
            spellData.rangeText,
            spellData.rangeValue,
            spellData.saveType,
            spellData.areaType,
            JSON.stringify(spellData.damageTypes),
          );
        }

        for (const reference of extractRulesReferences(raw)) {
          if (!reference.packName || !reference.recordLocator) {
            continue;
          }

          insertReferenceEdge.run(
            record.recordKey,
            `${reference.packName}:${reference.recordLocator}`,
            reference.displayText,
            reference.referenceText,
            record.packName,
            record.type,
            record.documentType,
            record.sourceCategory,
          );
        }

        insertEmbedding.run(record.recordKey, embeddingProvider.dimensions, encodeVector(embedding));
        insertFts.run(record.recordKey, record.name, record.searchText);

        packRecordCount += 1;
        recordCount += 1;
      }

      if (packRecordCount === 0) {
        continue;
      }

      packs.push({ ...pack, recordCount: packRecordCount });
      insertPack.run(pack.name, pack.label, pack.documentType, pack.declaredPath, pack.resolvedPath, packRecordCount);
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

function readMetadata(db: DatabaseSync): Map<string, string> {
  const rows = db.prepare("SELECT key, value FROM metadata").all() as Array<{ key: string; value: string }>;
  return new Map(rows.map((row) => [row.key, row.value]));
}

function canReuseIndex(db: DatabaseSync, sourceSignature: string, embeddingProvider: EmbeddingProvider): boolean {
  try {
    const metadata = readMetadata(db);
    return (
      metadata.get("schema_version") === String(INDEX_SCHEMA_VERSION) &&
      metadata.get("source_signature") === sourceSignature &&
      metadata.get("embedding_dimensions") === String(embeddingProvider.dimensions)
    );
  } catch {
    return false;
  }
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

function buildCandidateQuery(filters: SearchFilters, includeSearchText = false, includeEmbedding = false): { sql: string; params: SqlValue[] } {
  const fields = [
    "r.record_key AS recordKey",
    "r.id AS id",
    "r.name AS name",
    "r.normalized_name AS normalizedName",
    "r.record_type AS type",
    "r.pack_name AS packName",
    "r.pack_label AS packLabel",
    "r.document_type AS documentType",
    "r.level AS level",
    "r.rarity AS rarity",
    "r.traits_json AS traitsJson",
    "r.publication_title AS publicationTitle",
    "r.description_text AS descriptionText",
    "r.has_description AS hasDescription",
    "r.description_snippet AS descriptionSnippet",
    "r.source_category AS sourceCategory",
    "r.folder_id AS folderId",
    "r.source_path AS sourcePath",
    "r.is_unique AS isUnique",
    "a.size AS size",
    "i.item_category AS itemCategory",
    "i.price_cp AS priceCp",
    "i.bulk_value AS bulkValue",
    "COALESCE(s.action_cost, i.action_cost) AS actionCost",
    "s.traditions_json AS traditionsJson",
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

  if (filters.pack) {
    appendWhereClause(sql, params, "AND (LOWER(r.pack_name) = LOWER(?) OR LOWER(r.pack_label) = LOWER(?))", filters.pack, filters.pack);
  }

  if (filters.documentType) {
    appendWhereClause(sql, params, "AND LOWER(r.document_type) = LOWER(?)", filters.documentType);
  }

  if (filters.recordType) {
    appendWhereClause(sql, params, "AND LOWER(r.record_type) = LOWER(?)", filters.recordType);
  }

  if (filters.levelMin !== undefined) {
    appendWhereClause(sql, params, "AND r.level >= ?", filters.levelMin);
  }

  if (filters.levelMax !== undefined) {
    appendWhereClause(sql, params, "AND r.level <= ?", filters.levelMax);
  }

  if (filters.rarity) {
    appendWhereClause(sql, params, "AND LOWER(COALESCE(r.rarity, '')) = LOWER(?)", filters.rarity);
  }

  if (filters.publicationTitle) {
    appendWhereClause(sql, params, "AND LOWER(COALESCE(r.publication_title, '')) LIKE LOWER(?)", `%${filters.publicationTitle}%`);
  }

  if (filters.excludeUnique) {
    appendWhereClause(sql, params, "AND r.is_unique = 0");
  }

  if (filters.excludeMissingDescription) {
    appendWhereClause(sql, params, "AND r.has_description = 1");
  }

  if (filters.excludeAdventureContent) {
    appendWhereClause(sql, params, "AND r.source_category != 'adventure'");
  }

  if (filters.coreOnly) {
    appendWhereClause(sql, params, "AND r.source_category = 'core'");
  }

  if (filters.size) {
    appendWhereClause(sql, params, "AND LOWER(COALESCE(a.size, '')) = LOWER(?)", filters.size);
  }

  if (filters.itemCategory) {
    appendWhereClause(sql, params, "AND LOWER(COALESCE(i.item_category, '')) = LOWER(?)", filters.itemCategory);
  }

  if (filters.priceMin !== undefined) {
    appendWhereClause(sql, params, "AND i.price_cp >= ?", filters.priceMin);
  }

  if (filters.priceMax !== undefined) {
    appendWhereClause(sql, params, "AND i.price_cp <= ?", filters.priceMax);
  }

  if (filters.actionCost !== undefined) {
    appendWhereClause(sql, params, "AND COALESCE(s.action_cost, i.action_cost) = ?", filters.actionCost);
  }

  if (filters.tradition) {
    appendWhereClause(
      sql,
      params,
      "AND EXISTS (SELECT 1 FROM record_traits rt WHERE rt.record_key = r.record_key AND rt.trait = ?)",
      normalizeText(filters.tradition),
    );
  }

  for (const trait of filters.traitsAll ?? []) {
    appendWhereClause(
      sql,
      params,
      "AND EXISTS (SELECT 1 FROM record_traits rt WHERE rt.record_key = r.record_key AND rt.trait = ?)",
      normalizeText(trait),
    );
  }

  if (filters.traitsAny && filters.traitsAny.length > 0) {
    const placeholders = filters.traitsAny.map(() => "?").join(", ");
    appendWhereClause(
      sql,
      params,
      `AND EXISTS (SELECT 1 FROM record_traits rt WHERE rt.record_key = r.record_key AND rt.trait IN (${placeholders}))`,
      ...filters.traitsAny.map((trait) => normalizeText(trait)),
    );
  }

  return { sql: sql.join("\n"), params };
}

function validateFilters(filters: SearchFilters, context: "list" | "search"): void {
  const mode = resolveSearchMode(filters, context);

  if (context === "list" && mode !== "structured") {
    throw new Error("List mode only supports structured retrieval.");
  }

  if (context === "list" && filters.themeQuery) {
    throw new Error("themeQuery is only supported for lexical or hybrid search.");
  }

  if (mode === "structured" && filters.themeQuery) {
    throw new Error("themeQuery is not supported with mode=structured. Omit mode to default to hybrid, or set mode to lexical or hybrid.");
  }

  if (filters.coreOnly && filters.excludeAdventureContent) {
    throw new Error("coreOnly already excludes adventure content.");
  }
}

export class Pf2eDataService {
  readonly packs: PackInfo[];
  readonly warnings: string[];

  private readonly db: DatabaseSync;
  private readonly embeddingProvider: EmbeddingProvider;
  private readonly indexPath: string;
  private readonly recordCount: number;

  private constructor(
    db: DatabaseSync,
    packs: PackInfo[],
    warnings: string[],
    recordCount: number,
    indexPath: string,
    embeddingProvider: EmbeddingProvider,
  ) {
    this.db = db;
    this.packs = packs;
    this.warnings = warnings;
    this.recordCount = recordCount;
    this.indexPath = indexPath;
    this.embeddingProvider = embeddingProvider;
  }

  static async load(rootPath: string, manifestPath: string, options: LoadOptions = {}): Promise<Pf2eDataService> {
    const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
    const embeddingProvider = new HashEmbeddingProvider();
    const sourceSignature = await computeSourceSignature(rootPath, manifestPath);
    await mkdir(path.dirname(indexPath), { recursive: true });

    if (await fileExists(indexPath)) {
      const existingDb = new DatabaseSync(indexPath);
      if (canReuseIndex(existingDb, sourceSignature, embeddingProvider)) {
        const packs = loadPacksFromIndex(existingDb);
        const recordCount = sqliteRowCount(
          existingDb.prepare("SELECT COUNT(*) AS total FROM records").get() as Record<string, unknown> | undefined,
        );
        return new Pf2eDataService(existingDb, packs, [], recordCount, indexPath, embeddingProvider);
      }

      existingDb.close();
      await rm(indexPath, { force: true });
      await rm(`${indexPath}-wal`, { force: true });
      await rm(`${indexPath}-shm`, { force: true });
    }

    const db = new DatabaseSync(indexPath);
    createSchema(db);
    const { packs, warnings, recordCount } = await buildIndex(db, rootPath, manifestPath, embeddingProvider, sourceSignature);
    return new Pf2eDataService(db, packs, warnings, recordCount, indexPath, embeddingProvider);
  }

  getStats(): { packCount: number; recordCount: number } {
    return {
      packCount: this.packs.length,
      recordCount: this.recordCount,
    };
  }

  getSearchVocabulary(options: { traitLimitPerRecordType?: number } = {}): {
    documentTypes: Array<{ value: string; count: number }>;
    recordTypes: Array<{ value: string; count: number }>;
    itemCategories: Array<{ value: string; count: number }>;
    traditions: Array<{ value: string; count: number }>;
    sourceCategories: Array<{ value: SourceCategory; count: number }>;
    commonTraitsByRecordType: Array<{ recordType: string; traits: Array<{ value: string; count: number }> }>;
  } {
    const traitLimit = Math.max(3, Math.min(options.traitLimitPerRecordType ?? 12, 25));
    const documentTypes = this.db
      .prepare(
        `
          SELECT r.document_type AS value, COUNT(*) AS count
          FROM records r
          GROUP BY r.document_type
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const recordTypes = this.db
      .prepare(
        `
          SELECT r.record_type AS value, COUNT(*) AS count
          FROM records r
          GROUP BY r.record_type
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const itemCategories = this.db
      .prepare(
        `
          SELECT i.item_category AS value, COUNT(*) AS count
          FROM item_records i
          WHERE i.item_category IS NOT NULL AND i.item_category <> ''
          GROUP BY i.item_category
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const sourceCategories = this.db
      .prepare(
        `
          SELECT r.source_category AS value, COUNT(*) AS count
          FROM records r
          GROUP BY r.source_category
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: SourceCategory; count: number }>;
    const traitRows = this.db
      .prepare(
        `
          SELECT r.record_type AS recordType, rt.trait AS value, COUNT(*) AS count
          FROM record_traits rt
          JOIN records r ON r.record_key = rt.record_key
          GROUP BY r.record_type, rt.trait
          ORDER BY r.record_type ASC, count DESC, value ASC
        `,
      )
      .all() as Array<{ recordType: string; value: string; count: number }>;
    const commonTraitsByRecordType = (() => {
      const grouped = new Map<string, Array<{ value: string; count: number }>>();
      for (const row of traitRows) {
        const bucket = grouped.get(row.recordType) ?? [];
        if (bucket.length < traitLimit) {
          bucket.push({ value: row.value, count: row.count });
          grouped.set(row.recordType, bucket);
        }
      }

      return [...grouped.entries()]
        .sort((left, right) => left[0].localeCompare(right[0]))
        .map(([recordType, traits]) => ({ recordType, traits }));
    })();
    const traditionCounts = new Map<string, number>();
    const traditionRows = this.db
      .prepare("SELECT traditions_json AS traditionsJson FROM spell_records")
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

    return {
      documentTypes,
      recordTypes,
      itemCategories,
      traditions,
      sourceCategories,
      commonTraitsByRecordType,
    };
  }

  getPack(packValue: string): PackInfo | undefined {
    return this.packs.find((pack) => getPackAliasValues(pack, packValue));
  }

  listPacks(): PackInfo[] {
    return this.packs;
  }

  close(): void {
    this.db.close();
  }

  private fetchCandidates(filters: SearchFilters, includeSearchText = false, includeEmbedding = false): CandidateRow[] {
    const { sql, params } = buildCandidateQuery(filters, includeSearchText, includeEmbedding);
    return this.db.prepare(sql).all(...params) as CandidateRow[];
  }

  private fetchFtsMatches(query: string): Map<string, number> {
    const ftsQuery = buildFtsQuery(query);
    if (!ftsQuery) {
      return new Map();
    }

    const rows = this.db
      .prepare(
        `
          SELECT record_key AS recordKey, bm25(records_fts, 8.0, 1.5) AS rank
          FROM records_fts
          WHERE records_fts MATCH ?
          ORDER BY rank
        `,
      )
      .all(ftsQuery) as Array<{ recordKey: string; rank: number }>;

    const scores = new Map<string, number>();
    const total = rows.length;
    rows.forEach((row, index) => {
      scores.set(row.recordKey, total <= 1 ? 1 : 1 - (index / (total - 1)));
    });
    return scores;
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
            r.pack_name AS packName,
            r.pack_label AS packLabel,
            r.document_type AS documentType,
            r.level AS level,
            r.rarity AS rarity,
            r.traits_json AS traitsJson,
            r.publication_title AS publicationTitle,
            r.description_text AS descriptionText,
            r.has_description AS hasDescription,
            r.description_snippet AS descriptionSnippet,
            r.source_category AS sourceCategory,
            r.folder_id AS folderId,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            a.size AS size,
            i.item_category AS itemCategory,
            i.price_cp AS priceCp,
            i.bulk_value AS bulkValue,
            COALESCE(s.action_cost, i.action_cost) AS actionCost,
            s.traditions_json AS traditionsJson
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

  private resolveReference(reference: ExtractedReference): NormalizedRecord | null {
    if (reference.packName) {
      const direct = this.getRecord(reference.packName, reference.recordLocator);
      if (direct) {
        return direct;
      }

      const fallback = this.lookup(reference.displayText ?? reference.recordLocator, { pack: reference.packName }).match;
      if (fallback) {
        return this.getRecord(fallback.recordKey) ?? fallback;
      }
    }

    const match = this.lookup(reference.displayText ?? reference.recordLocator).match;
    return match ? (this.getRecord(match.recordKey) ?? match) : null;
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
                r.pack_name AS packName,
                r.pack_label AS packLabel,
                r.document_type AS documentType,
                r.level AS level,
                r.rarity AS rarity,
                r.traits_json AS traitsJson,
                r.publication_title AS publicationTitle,
                r.description_text AS descriptionText,
                r.has_description AS hasDescription,
                r.description_snippet AS descriptionSnippet,
                r.source_category AS sourceCategory,
                r.folder_id AS folderId,
                r.source_path AS sourcePath,
                r.is_unique AS isUnique,
                a.size AS size,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson
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
                r.pack_name AS packName,
                r.pack_label AS packLabel,
                r.document_type AS documentType,
                r.level AS level,
                r.rarity AS rarity,
                r.traits_json AS traitsJson,
                r.publication_title AS publicationTitle,
                r.description_text AS descriptionText,
                r.has_description AS hasDescription,
                r.description_snippet AS descriptionSnippet,
                r.source_category AS sourceCategory,
                r.folder_id AS folderId,
                r.source_path AS sourcePath,
                r.is_unique AS isUnique,
                a.size AS size,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson
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

    const raw = JSON.parse(readFileSyncUtf8(row.sourcePath)) as Record<string, unknown>;
    return rowToRecord(row, raw);
  }

  getRecordsByKeys(recordKeys: string[]): NormalizedRecord[] {
    const rows = this.fetchRecordRowsByKeys([...new Set(recordKeys)]);
    const byKey = new Map(rows.map((row) => [row.recordKey, rowToRecord(row)]));
    return [...new Set(recordKeys)].map((recordKey) => byKey.get(recordKey)).filter((record): record is NormalizedRecord => Boolean(record));
  }

  lookupMany(queries: LookupQuery[], options: { coreOnly?: boolean } = {}): LookupResult[] {
    return queries.map((query) => {
      const lookup = (() => {
        if (!options.coreOnly) {
          return this.lookup(query.name, query);
        }

        const results = this.search({
          mode: "structured",
          nameQuery: query.name,
          pack: query.pack,
          documentType: query.documentType,
          recordType: query.recordType,
          coreOnly: true,
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

  getLinkedRules(
    recordKeys: string[],
    options: { coreOnly?: boolean; maxPerPrimary?: number } = {},
  ): RuleGraphResult {
    return this.fetchReferenceEdgeRows("outgoing", [...new Set(recordKeys)], options);
  }

  getBacklinks(
    recordKeys: string[],
    options: { coreOnly?: boolean; maxPerPrimary?: number } = {},
  ): RuleGraphResult {
    return this.fetchReferenceEdgeRows("backlink", [...new Set(recordKeys)], options);
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
    const outgoing = this.getLinkedRules(primaryKeys, {
      coreOnly: input.coreOnly,
      maxPerPrimary: input.maxOutgoingPerPrimary ?? 4,
    });
    const backlinks = input.includeBacklinks
      ? this.getBacklinks(primaryKeys, {
          coreOnly: input.coreOnly,
          maxPerPrimary: input.maxBacklinksPerPrimary ?? 4,
        })
      : { records: [], edges: [] };

    return {
      primary,
      outgoing,
      backlinks,
      edges: [...outgoing.edges, ...backlinks.edges],
    };
  }

  listRecords(filters: SearchFilters): SearchResult {
    validateFilters(filters, "list");
    const limit = clampLimit(filters.limit);
    const offset = clampOffset(filters.offset);
    const records = this.fetchCandidates(filters).map((row) => rowToRecord(row));
    records.sort((left, right) => (
      rankingProfileScore(right, filters) - rankingProfileScore(left, filters) ||
      sortRecords(left, right)
    ));
    return {
      mode: "structured",
      total: records.length,
      offset,
      limit,
      records: records.slice(offset, offset + limit),
    };
  }

  getRulesContext(
    name: string,
    options: LookupOptions & { referenceDepth?: number; maxReferences?: number } = {},
  ): { record: NormalizedRecord; references: NormalizedRecord[]; edges: RulesContextEdge[] } | null {
    const baseMatch = this.lookup(name, options).match;
    if (!baseMatch) {
      return null;
    }
    const baseRecord = this.getRecord(baseMatch.recordKey) ?? baseMatch;

    const maxReferences = Math.max(1, Math.min(options.maxReferences ?? 8, 25));
    const maxDepth = options.referenceDepth === 2 ? 2 : 1;
    const references: NormalizedRecord[] = [];
    const edges: RulesContextEdge[] = [];
    const queued = new Set<string>([baseRecord.recordKey]);
    const addedReferences = new Set<string>();
    const queue: Array<{ record: NormalizedRecord; depth: number }> = [{ record: baseRecord, depth: 1 }];

    while (queue.length > 0 && references.length < maxReferences) {
      const current = queue.shift()!;
      const extracted = extractRulesReferences(current.record.raw);

      for (const reference of extracted) {
        const resolved = this.resolveReference(reference);
        if (!resolved || resolved.recordKey === current.record.recordKey) {
          continue;
        }

        edges.push({
          fromRecordKey: current.record.recordKey,
          toRecordKey: resolved.recordKey,
          displayText: reference.displayText,
          referenceText: reference.referenceText,
          depth: current.depth,
        });

        if (!addedReferences.has(resolved.recordKey)) {
          references.push(resolved);
          addedReferences.add(resolved.recordKey);
        }

        if (current.depth < maxDepth && !queued.has(resolved.recordKey) && references.length < maxReferences) {
          queue.push({ record: resolved, depth: current.depth + 1 });
          queued.add(resolved.recordKey);
        }

        if (references.length >= maxReferences) {
          break;
        }
      }
    }

    return { record: baseRecord, references, edges };
  }

  search(filters: SearchFilters): SearchResult {
    validateFilters(filters, "search");
    const limit = clampLimit(filters.limit);
    const offset = clampOffset(filters.offset);
    const mode = resolveSearchMode(filters, "search");
    const shouldIncludeSearchText = Boolean(filters.themeQuery || filters.nameQuery || mode !== "structured");
    const shouldIncludeEmbedding = Boolean(mode === "hybrid" && filters.themeQuery);
    let candidates = this.fetchCandidates(filters, shouldIncludeSearchText, shouldIncludeEmbedding);

    const rawLexicalQuery = filters.themeQuery?.trim() || filters.nameQuery?.trim() || "";
    const queryAnalysis = rawLexicalQuery
      ? buildSearchQueryAnalysis(rawLexicalQuery, filters, { expandQuery: filters.expandQuery ?? true })
      : null;
    const lexicalQuery = queryAnalysis?.expandedQuery ?? rawLexicalQuery;
    const lexicalMatches = lexicalQuery ? this.fetchFtsMatches(lexicalQuery) : new Map<string, number>();

    if (mode === "lexical" && lexicalQuery && lexicalMatches.size > 0) {
      candidates = candidates.filter((candidate) => lexicalMatches.has(candidate.recordKey));
    }

    const semanticVector = mode === "hybrid" && queryAnalysis
      ? this.embeddingProvider.embed(queryAnalysis.expandedQuery)
      : null;

    const scored = candidates
      .map((candidate) => {
        const record = rowToRecord(candidate);
        const ftsScore = lexicalQuery.length > 0 ? (lexicalMatches.get(candidate.recordKey) ?? 0) : 0;
        const metadataTextScore =
          lexicalQuery.length > 0
            ? queryTextScore(lexicalQuery, buildMetadataText(record))
            : 0;
        const descriptionTextScore =
          lexicalQuery.length > 0
            ? queryTextScore(lexicalQuery, record.descriptionText ?? "")
            : 0;
        const candidateQueryWeights = queryAnalysis
          ? buildCandidateQueryWeights(record, queryAnalysis)
          : null;
        const themeName = candidateQueryWeights
          ? scoreWeightedOverlap(candidateQueryWeights.nameWeights, tokenize(record.name), 1.5)
          : { score: 0, matchedTokens: [] };
        const themeTraits = candidateQueryWeights
          ? scoreWeightedOverlap(candidateQueryWeights.traitWeights, record.traits, 2)
          : { score: 0, matchedTokens: [] };
        const themeMetadata = candidateQueryWeights
          ? scoreWeightedOverlap(candidateQueryWeights.metadataWeights, tokenize(buildMetadataText(record)), 2.5)
          : { score: 0, matchedTokens: [] };
        const metadataOnlyBoostBase = !record.hasDescription
          ? Math.max(themeTraits.score * 0.35, themeName.score * 0.2, themeMetadata.score * 0.15, metadataTextScore * 0.15)
          : 0;
        const metadataOnlyBoost = metadataOnlyBoostBase * metadataOnlyBoostMultiplier(record, filters);
        const lexicalScore =
          (ftsScore * 0.1) +
          (metadataTextScore * 0.15) +
          (descriptionTextScore * 0.05) +
          (themeName.score * 0.25) +
          (themeTraits.score * 0.35) +
          (themeMetadata.score * 0.1) +
          metadataOnlyBoost;
        const semanticScore =
          semanticVector && candidate.embeddingBlob
            ? Math.max(0, cosineSimilarity(semanticVector, decodeVector(candidate.embeddingBlob)))
            : 0;
        const packQuality = packQualityScore(record);
        const sourcePenalty = sourcePenaltyScore(record, filters);
        const rankingProfile = rankingProfileScore(record, filters);
        const components: SearchScoreComponents = {
          fts: ftsScore,
          metadataText: metadataTextScore,
          descriptionText: descriptionTextScore,
          themeName: themeName.score,
          themeTraits: themeTraits.score,
          themeMetadata: themeMetadata.score,
          metadataOnlyBoost,
          sourcePenalty,
          packQuality,
          rankingProfile,
        };

        let score = 0.5;
        if (mode === "structured") {
          score = (filters.nameQuery ? nameScore(filters.nameQuery, record) : 0.5) + packQuality;
        } else if (mode === "lexical") {
          score = lexicalScore + packQuality;
        } else {
          score = (lexicalScore * 0.85) + (semanticScore * 0.15) + packQuality;
        }

        score += sourcePenalty + rankingProfile;

        const explanation: SearchRecordExplanation = {
          recordKey: record.recordKey,
          name: record.name,
          totalScore: score,
          lexicalScore,
          semanticScore,
          matchedTraits: themeTraits.matchedTokens,
          matchedNameTokens: themeName.matchedTokens,
          matchedMetadataTokens: themeMetadata.matchedTokens,
          matchedRuleIds: candidateQueryWeights?.matchedRuleIds ?? [],
          components,
        };

        return { record, score, lexicalScore, semanticScore, explanation };
      })
      .filter(({ score }) => {
        if (filters.nameQuery && mode === "structured") {
          return score >= 0.2;
        }

        if (mode === "lexical" && lexicalQuery) {
          return score > 0;
        }

        return true;
      })
      .sort((left, right) => {
        return (
          right.score - left.score ||
          right.semanticScore - left.semanticScore ||
          right.lexicalScore - left.lexicalScore ||
          sortRecords(left.record, right.record)
        );
      });

    const page = scored.slice(offset, offset + limit);
    const explain: SearchExplainResult | undefined = filters.explain
      ? {
          mode,
          lexicalQuery: rawLexicalQuery,
          semanticQuery: queryAnalysis?.expandedQuery ?? "",
          query: queryAnalysis
            ? {
                rawQuery: queryAnalysis.rawQuery,
                normalizedQuery: queryAnalysis.normalizedQuery,
                queryTokens: queryAnalysis.queryTokens,
                expandedQuery: queryAnalysis.expandedQuery,
                boostedTraits: queryAnalysis.boostedTraits,
                boostedNameTokens: queryAnalysis.boostedNameTokens,
                boostedMetadataTokens: queryAnalysis.boostedMetadataTokens,
                matchedRules: queryAnalysis.matchedRules,
                skippedRules: queryAnalysis.skippedRules,
              }
            : null,
          records: page.map(({ explanation }) => explanation),
        }
      : undefined;

    return {
      mode,
      total: scored.length,
      offset,
      limit,
      records: page.map(({ record }) => record),
      explain,
    };
  }

  lookup(name: string, options: LookupOptions = {}): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const results = this.search({
      mode: "structured",
      nameQuery: name,
      pack: options.pack,
      documentType: options.documentType,
      recordType: options.recordType,
      limit: 5,
    }).records;

    return {
      match: results[0] ?? null,
      alternatives: results.slice(1),
    };
  }
}

function readFileSyncUtf8(filePath: string): string {
  return readFileSync(filePath, "utf8");
}
