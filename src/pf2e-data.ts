import { readFileSync } from "node:fs";
import { mkdir, readdir, readFile, rm, stat } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

import {
  LookupOptions,
  NormalizedRecord,
  PackInfo,
  PackManifestEntry,
  SearchFilters,
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

const execFileAsync = promisify(execFile);
const INDEX_SCHEMA_VERSION = 1;

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

function getLevel(raw: Record<string, unknown>): number | null {
  return asNumber(
    getNested(raw, ["system", "level", "value"]) ?? getNested(raw, ["system", "details", "level", "value"]),
  );
}

function getDescriptionText(raw: Record<string, unknown>): string | null {
  const html = firstString(
    getNested(raw, ["system", "description", "value"]),
    getNested(raw, ["system", "details", "description"]),
    getNested(raw, ["system", "details", "publicNotes"]),
    getNested(raw, ["system", "details", "blurb"]),
  );

  return stripHtml(html);
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
    CREATE INDEX record_traits_trait_idx ON record_traits(trait);
    CREATE INDEX actor_records_size_idx ON actor_records(size);
    CREATE INDEX item_records_category_idx ON item_records(item_category);
    CREATE INDEX item_records_price_idx ON item_records(price_cp);
    CREATE INDEX item_records_action_cost_idx ON item_records(action_cost);
    CREATE INDEX spell_records_action_cost_idx ON spell_records(action_cost);
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
      level, rarity, traits_json, publication_title, description_text, folder_id, source_path,
      is_unique, search_text
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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

        insertEmbedding.run(record.recordKey, embeddingProvider.dimensions, encodeVector(embedding));
        insertFts.run(record.recordKey, record.name, record.searchText);

        packRecordCount += 1;
        recordCount += 1;
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
  const mode: SearchMode = filters.mode ?? "structured";

  if (context === "list" && mode !== "structured") {
    throw new Error("List mode only supports structured retrieval.");
  }

  if (context === "list" && filters.themeQuery) {
    throw new Error("themeQuery is only supported for lexical or hybrid search.");
  }

  if (mode === "structured" && filters.themeQuery) {
    throw new Error("themeQuery requires mode lexical or hybrid.");
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

  private fetchFtsMatches(query: string): Set<string> {
    const ftsQuery = buildFtsQuery(query);
    if (!ftsQuery) {
      return new Set();
    }

    const rows = this.db
      .prepare("SELECT record_key as recordKey FROM records_fts WHERE records_fts MATCH ?")
      .all(ftsQuery) as Array<{ recordKey: string }>;
    return new Set(rows.map((row) => row.recordKey));
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

  listRecords(filters: SearchFilters): SearchResult {
    validateFilters(filters, "list");
    const limit = clampLimit(filters.limit);
    const offset = clampOffset(filters.offset);
    const records = this.fetchCandidates(filters).map((row) => rowToRecord(row));
    records.sort(sortRecords);
    return {
      total: records.length,
      offset,
      limit,
      records: records.slice(offset, offset + limit),
    };
  }

  search(filters: SearchFilters): SearchResult {
    validateFilters(filters, "search");
    const limit = clampLimit(filters.limit);
    const offset = clampOffset(filters.offset);
    const mode: SearchMode = filters.mode ?? "structured";
    const shouldIncludeSearchText = Boolean(filters.themeQuery || filters.nameQuery || mode !== "structured");
    const shouldIncludeEmbedding = Boolean(mode === "hybrid" && filters.themeQuery);
    let candidates = this.fetchCandidates(filters, shouldIncludeSearchText, shouldIncludeEmbedding);

    const lexicalQuery = filters.themeQuery?.trim() || filters.nameQuery?.trim() || "";
    const lexicalMatches = lexicalQuery ? this.fetchFtsMatches(lexicalQuery) : new Set<string>();

    if (mode === "lexical" && lexicalQuery && lexicalMatches.size > 0) {
      candidates = candidates.filter((candidate) => lexicalMatches.has(candidate.recordKey));
    }

    const semanticVector = mode === "hybrid" && filters.themeQuery
      ? this.embeddingProvider.embed(filters.themeQuery)
      : null;

    const scored = candidates
      .map((candidate) => {
        const record = rowToRecord(candidate);
        const lexicalScore =
          lexicalQuery.length > 0
            ? Math.max(
                lexicalMatches.has(candidate.recordKey) ? 1 : 0,
                queryTextScore(lexicalQuery, candidate.searchText ?? ""),
                filters.nameQuery ? nameScore(filters.nameQuery, record) : 0,
              )
            : 0;
        const semanticScore =
          semanticVector && candidate.embeddingBlob
            ? Math.max(0, cosineSimilarity(semanticVector, decodeVector(candidate.embeddingBlob)))
            : 0;

        let score = 0.5;
        if (mode === "structured") {
          score = (filters.nameQuery ? nameScore(filters.nameQuery, record) : 0.5) + packQualityScore(record);
        } else if (mode === "lexical") {
          score = lexicalScore + packQualityScore(record);
        } else {
          score = (semanticScore * 0.75) + (lexicalScore * 0.25) + packQualityScore(record);
        }

        return { record, score, lexicalScore, semanticScore };
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

    return {
      total: scored.length,
      offset,
      limit,
      records: scored.slice(offset, offset + limit).map(({ record }) => record),
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
