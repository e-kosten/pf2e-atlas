import { readFile, readdir } from "node:fs/promises";
import path from "node:path";

import { pathExists } from "../shared/fs.js";
import { normalizeText, uniqueSorted } from "../shared/utils.js";
import type {
  BuildSourceEntry,
  ExtractedReference,
  NormalizedIndexRecord,
  PackBuildInfo,
  RecordAliasRow,
  RecordLegacyLinkRow,
  ResolvedBuildReference,
} from "./index-types.js";
import { firstString, getNested, stripHtml } from "./raw-utils.js";
import { getDescriptionMarkup, getPublicationRemaster } from "./record-normalization.js";

const UUID_REFERENCE_PATTERN = /@UUID\[([^\]]+)\](?:\{([^}]+)\})?/g;
const COMPILED_REFERENCE_PATTERN = /^Compendium\.pf2e\.([^.]+)\.[^.]+\.([^.\]]+)$/i;
const TABLE_ROW_PATTERN = /<tr[^>]*>(.*?)<\/tr>/gis;
const TABLE_CELL_PATTERN = /<t[dh][^>]*>(.*?)<\/t[dh]>/gis;
const LIST_ITEM_PATTERN = /<li[^>]*>(.*?)<\/li>/gis;
const MIGRATION_RENAME_PATTERN = /Rename all uses and mentions of "([^"]+)" to "([^"]+)"/g;
const COMPILED_SOURCE_PATTERN = /^Compendium\.pf2e\.([^.]+)\.[^.]+\.([^.]+)$/i;
const NPC_CORE_FAMILY_ALLOWLIST = new Set([
  "ancestry-npcs",
  "artisan",
  "courtier",
  "criminal",
  "devotee",
  "downtrodden",
  "engineer",
  "explorer",
  "healer",
  "laborer",
  "martial-artist",
  "maverick",
  "mercenary",
  "military",
  "mystic",
  "official",
  "performer",
  "primalist",
  "scholar",
  "seafarer",
  "villain",
]);

type FolderDefinition = {
  _id?: string;
  name?: string;
  folder?: string | null;
};

type ResolvedJournalTarget = {
  recordKey: string;
  record: NormalizedIndexRecord;
  displayText: string | null;
};

type IndexedRecordMaps = {
  recordsByKey: Map<string, NormalizedIndexRecord>;
  recordsByPackAndId: Map<string, string>;
  recordsByPackAndName: Map<string, string[]>;
  recordsByName: Map<string, string[]>;
};

export function extractRulesReferences(raw: Record<string, unknown>): ExtractedReference[] {
  const markup = getDescriptionMarkup(raw);
  if (!markup) {
    return [];
  }

  return extractUuidReferences(markup);
}

function extractUuidReferences(markup: string): ExtractedReference[] {
  const extracted: ExtractedReference[] = [];
  for (const match of markup.matchAll(UUID_REFERENCE_PATTERN)) {
    const referenceText = match[0];
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

function buildPackAndFolderKey(packName: string, folderId: string): string {
  return `${normalizeText(packName)}:${normalizeText(folderId)}`;
}

function shouldKeepFolderFamily(packName: string, family: string): boolean {
  return packName === "pathfinder-npc-core" && NPC_CORE_FAMILY_ALLOWLIST.has(family);
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
  if (!(await pathExists(folderPath))) {
    return new Map();
  }

  const raw = JSON.parse(await readFile(folderPath, "utf8")) as FolderDefinition[];
  if (!Array.isArray(raw)) {
    return new Map();
  }

  const foldersById = new Map<string, FolderDefinition>();
  for (const entry of raw) {
    const id = firstString(entry._id);
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
    const folderFamily = folderFamilyByPackAndFolderId.get(
      buildPackAndFolderKey(entry.record.packName, entry.record.folderId),
    );
    if (folderFamily && shouldKeepFolderFamily(entry.record.packName, folderFamily)) {
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
  return recordKey ? (recordsByKey.get(recordKey)?.name ?? null) : directText || null;
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

  return [
    {
      recordKey: recordKey!,
      record,
      displayText: null,
    },
  ];
}

function extractIntroListAliases(
  listHtml: string,
  recordsByPackAndId: Map<string, string>,
  recordsByPackAndName: Map<string, string[]>,
  recordsByName: Map<string, string[]>,
  recordsByKey: Map<string, NormalizedIndexRecord>,
): { aliasTexts: string[]; targetRecordKey: string } | null {
  const targets = extractResolvedJournalTargets(
    listHtml,
    recordsByPackAndId,
    recordsByPackAndName,
    recordsByName,
    recordsByKey,
  );
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

      const targets = extractResolvedJournalTargets(
        newCell,
        recordsByPackAndId,
        recordsByPackAndName,
        recordsByName,
        recordsByKey,
      );
      if (targets.length === 0) {
        continue;
      }

      if (targets.length === 1) {
        const oldName = resolveAliasSourceName(
          oldCell,
          recordsByPackAndId,
          recordsByPackAndName,
          recordsByName,
          recordsByKey,
        );
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
      if (
        !targetRecord ||
        embeddedRemaster ||
        !targetRecord.publicationRemaster ||
        shouldIgnoreCompendiumAlias(aliasText, targetRecord.name)
      ) {
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
    const key = [row.canonicalRecordKey, row.normalizedAlias, row.sourceKind, row.sourceRef].join("\u0000");
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
    const key = [row.canonicalRecordKey, row.legacyRecordKey, row.sourceKind, row.sourceRef].join("\u0000");
    if (seen.has(key)) {
      continue;
    }

    seen.add(key);
    deduped.push(row);
  }

  return deduped;
}

function buildIndexedRecordMaps(
  indexedEntries: Array<BuildSourceEntry & { record: NormalizedIndexRecord }>,
): IndexedRecordMaps {
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

  return {
    recordsByKey,
    recordsByPackAndId,
    recordsByPackAndName,
    recordsByName,
  };
}

export async function resolveBuildReferencesAndAliases(options: {
  indexedEntries: Array<BuildSourceEntry & { record: NormalizedIndexRecord }>;
  sourceEntries: BuildSourceEntry[];
  packs: Array<Pick<PackBuildInfo, "name" | "resolvedPath">>;
  rootPath: string;
}): Promise<{ aliasRows: RecordAliasRow[]; legacyLinkRows: RecordLegacyLinkRow[] }> {
  const { indexedEntries, sourceEntries, packs, rootPath } = options;
  const { recordsByKey, recordsByPackAndId, recordsByPackAndName, recordsByName } =
    buildIndexedRecordMaps(indexedEntries);

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
      .map((reference) =>
        resolveExtractedReference(reference, recordsByPackAndId, recordsByPackAndName, recordsByName, recordsByKey),
      )
      .filter((reference): reference is ResolvedBuildReference => Boolean(reference));
  }

  let aliasRows: RecordAliasRow[] = [];
  const legacyLinkRows: RecordLegacyLinkRow[] = [];

  const migrationDir = path.join(rootPath, "src", "module", "migration", "migrations");
  try {
    const migrationFiles = (await readdir(migrationDir))
      .filter((fileName) => fileName.endsWith(".ts"))
      .sort((left, right) => left.localeCompare(right));
    for (const migrationFile of migrationFiles) {
      const migrationSource = await readFile(path.join(migrationDir, migrationFile), "utf8");
      aliasRows.push(...extractMigrationAliases(migrationSource, recordsByName, recordsByKey));
    }
  } catch {
    // Missing migration directory is acceptable in fixtures and alternate corpora.
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

  return {
    aliasRows,
    legacyLinkRows: dedupeLegacyLinkRows(legacyLinkRows),
  };
}
