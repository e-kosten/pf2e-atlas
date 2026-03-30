import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

import {
  LookupOptions,
  NormalizedRecord,
  PackInfo,
  PackManifestEntry,
  SearchFilters,
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

function normalizeRecord(pack: Omit<PackInfo, "recordCount">, sourcePath: string, raw: Record<string, unknown>): NormalizedRecord {
  const id = firstString(raw._id);
  const name = firstString(raw.name);
  const recordType = firstString(raw.type) ?? "unknown";

  if (!id || !name) {
    throw new Error(`Invalid record in ${sourcePath}: expected both _id and name.`);
  }

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
    rarity: getRarity(raw),
    traits: getTraits(raw),
    publicationTitle: getPublicationTitle(raw),
    descriptionText: getDescriptionText(raw),
    folderId: firstString(raw.folder),
    sourcePath,
    raw,
  };
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

function getPackAliasValues(pack: PackInfo, value: string): boolean {
  const normalized = normalizeText(value);
  return normalized === normalizeText(pack.name) || normalized === normalizeText(pack.label);
}

function matchesFilters(record: NormalizedRecord, filters: SearchFilters): boolean {
  if (filters.pack && ![record.packName, record.packLabel].some((value) => normalizeText(value) === normalizeText(filters.pack!))) {
    return false;
  }

  if (filters.documentType && normalizeText(record.documentType) !== normalizeText(filters.documentType)) {
    return false;
  }

  if (filters.recordType && normalizeText(record.type) !== normalizeText(filters.recordType)) {
    return false;
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

  if (filters.publicationTitle) {
    const target = normalizeText(record.publicationTitle ?? "");
    const expected = normalizeText(filters.publicationTitle);
    if (!target.includes(expected)) {
      return false;
    }
  }

  const normalizedTraits = new Set(record.traits.map((trait) => normalizeText(trait)));
  if (filters.traitsAll?.some((trait) => !normalizedTraits.has(normalizeText(trait)))) {
    return false;
  }

  if (filters.traitsAny && filters.traitsAny.length > 0) {
    const matchesAny = filters.traitsAny.some((trait) => normalizedTraits.has(normalizeText(trait)));
    if (!matchesAny) {
      return false;
    }
  }

  return true;
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

function sortRecords(left: NormalizedRecord, right: NormalizedRecord): number {
  return left.name.localeCompare(right.name) || left.packLabel.localeCompare(right.packLabel) || left.id.localeCompare(right.id);
}

export class Pf2eDataService {
  readonly packs: PackInfo[];
  readonly records: NormalizedRecord[];
  readonly warnings: string[];

  private readonly recordsByKey: Map<string, NormalizedRecord>;
  private readonly recordsByPack: Map<string, NormalizedRecord[]>;

  private constructor(packs: PackInfo[], records: NormalizedRecord[], warnings: string[]) {
    this.packs = packs;
    this.records = records;
    this.warnings = warnings;
    this.recordsByKey = new Map(records.map((record) => [record.recordKey, record]));
    this.recordsByPack = new Map(
      packs.map((pack) => [pack.name, records.filter((record) => record.packName === pack.name).sort(sortRecords)]),
    );
  }

  static async load(rootPath: string, manifestPath: string): Promise<Pf2eDataService> {
    const manifestRaw = JSON.parse(await readFile(manifestPath, "utf8")) as { packs?: PackManifestEntry[] };
    const manifestPacks = Array.isArray(manifestRaw.packs) ? manifestRaw.packs : [];

    const warnings: string[] = [];
    const records: NormalizedRecord[] = [];
    const packs: PackInfo[] = [];

    for (const manifestPack of manifestPacks) {
      const resolvedPath = await resolvePackPath(rootPath, manifestPack);
      if (!resolvedPath) {
        warnings.push(`Skipping pack ${manifestPack.name}: could not resolve a readable directory.`);
        continue;
      }

      const pack = {
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

      const packRecords: NormalizedRecord[] = [];
      for (const filePath of filePaths) {
        try {
          const parsed = JSON.parse(await readFile(filePath, "utf8")) as Record<string, unknown>;
          packRecords.push(normalizeRecord(pack, filePath, parsed));
        } catch (error) {
          throw new Error(`Failed to load ${filePath}: ${(error as Error).message}`);
        }
      }

      packs.push({ ...pack, recordCount: packRecords.length });
      records.push(...packRecords);
    }

    packs.sort((left, right) => left.label.localeCompare(right.label));
    records.sort(sortRecords);
    return new Pf2eDataService(packs, records, warnings);
  }

  getStats(): { packCount: number; recordCount: number } {
    return {
      packCount: this.packs.length,
      recordCount: this.records.length,
    };
  }

  getPack(packValue: string): PackInfo | undefined {
    return this.packs.find((pack) => getPackAliasValues(pack, packValue));
  }

  listPacks(): PackInfo[] {
    return this.packs;
  }

  getRecord(recordKeyOrPack: string, maybeId?: string): NormalizedRecord | undefined {
    if (maybeId) {
      return this.recordsByKey.get(`${recordKeyOrPack}:${maybeId}`);
    }

    return this.recordsByKey.get(recordKeyOrPack);
  }

  listRecords(filters: SearchFilters): SearchResult {
    const limit = clampLimit(filters.limit);
    const offset = clampOffset(filters.offset);

    const scopedRecords = filters.pack
      ? this.recordsByPack.get(this.getPack(filters.pack)?.name ?? "") ?? []
      : this.records;

    const filtered = scopedRecords.filter((record) => matchesFilters(record, filters)).sort(sortRecords);
    return {
      total: filtered.length,
      offset,
      limit,
      records: filtered.slice(offset, offset + limit),
    };
  }

  search(filters: SearchFilters): SearchResult {
    const limit = clampLimit(filters.limit);
    const offset = clampOffset(filters.offset);
    const query = filters.nameQuery?.trim();

    const filtered = this.records
      .filter((record) => matchesFilters(record, filters))
      .map((record) => ({
        record,
        score: (query ? nameScore(query, record) : 0.5) + packQualityScore(record),
      }))
      .filter(({ score }) => (query ? score >= 0.2 : true))
      .sort((left, right) => right.score - left.score || sortRecords(left.record, right.record));

    const records = filtered.map(({ record }) => record);
    return {
      total: records.length,
      offset,
      limit,
      records: records.slice(offset, offset + limit),
    };
  }

  lookup(name: string, options: LookupOptions = {}): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const results = this.search({
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
