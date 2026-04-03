import {
  classifyRecordCategory,
  extractSpellTraditions,
} from "../domain/categories.js";
import type { SourceCategory } from "../types.js";
import {
  firstString,
  getNested,
  normalizeText,
  stripHtml,
  toStringArray,
  uniqueSorted,
} from "../utils.js";
import type {
  ActorIndexData,
  ItemIndexData,
  NormalizedIndexRecord,
  PackBuildInfo,
  SpellIndexData,
} from "./index-types.js";

const MAX_ACTOR_SEMANTIC_ITEM_CHUNKS = 40;

function asNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function getPublicationTitle(raw: Record<string, unknown>): string | null {
  return firstString(
    getNested(raw, ["system", "publication", "title"]),
    getNested(raw, ["system", "details", "publication", "title"]),
  );
}

export function getPublicationRemaster(raw: Record<string, unknown>): boolean {
  return getNested(raw, ["system", "publication", "remaster"]) === true ||
    getNested(raw, ["system", "details", "publication", "remaster"]) === true;
}

export function getDescriptionMarkup(raw: Record<string, unknown>): string | null {
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

export function parseActorIndexData(raw: Record<string, unknown>): ActorIndexData {
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

export function parseItemIndexData(raw: Record<string, unknown>): ItemIndexData {
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

export function parseSpellIndexData(raw: Record<string, unknown>): SpellIndexData {
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

export function buildSemanticEmbeddingText(record: NormalizedIndexRecord, raw: Record<string, unknown>, aliases: string[]): string {
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

  const normalized = (descriptionText ?? "").replace(/\s+/g, " ").trim();
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

export function isExcludedPackName(packName: string): boolean {
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

export function shouldExcludeRecordFromIndex(pack: PackBuildInfo, sourcePath: string, raw: Record<string, unknown>): boolean {
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

export function normalizeIndexRecord(pack: PackBuildInfo, sourcePath: string, raw: Record<string, unknown>): NormalizedIndexRecord {
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
