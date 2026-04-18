import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  DerivedTagMigrationRecordDecision,
  DerivedTagMigrationSession,
  DerivedTagMigrationSessionManifest,
  DerivedTagMigrationSessionRecord,
  DerivedTagMigrationSessionReviewState,
} from "./types.js";
import type { SearchCategory, SearchSubcategory } from "../../types.js";

type LegacyDerivedTagMigrationSessionRecord = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  packName: string;
  level: number | null;
  traits: string[];
  families: string[];
  currentDerivedTags: string[];
  currentSources: DerivedTagMigrationSessionRecord["currentSources"];
  descriptionText: string | null;
  blurbText: string | null;
  selectionReasons: DerivedTagMigrationSessionRecord["selectionReasons"];
};

function sessionRoot(rootPath: string): string {
  return path.join(rootPath, "scratch", "migration-sessions");
}

export function migrationSessionDirectory(rootPath: string, sessionId: string): string {
  return path.join(sessionRoot(rootPath), sessionId);
}

function lineSeparatedJson<T>(records: T[]): string {
  return records.map((record) => JSON.stringify(record)).join("\n") + (records.length > 0 ? "\n" : "");
}

function parseJsonLines<T>(value: string): T[] {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => JSON.parse(line) as T);
}

function isLegacySessionRecord(
  record: DerivedTagMigrationSessionRecord | LegacyDerivedTagMigrationSessionRecord,
): record is LegacyDerivedTagMigrationSessionRecord {
  return !("entityRecord" in record);
}

function normalizeSessionRecord(
  record: DerivedTagMigrationSessionRecord | LegacyDerivedTagMigrationSessionRecord,
): DerivedTagMigrationSessionRecord {
  if (!isLegacySessionRecord(record)) {
    return record;
  }

  return {
    entityRecord: {
      recordKey: record.recordKey,
      packName: record.packName,
      name: record.name,
      type: "unknown",
      category: record.category,
      subcategory: record.subcategory,
      documentType: "unknown",
      level: record.level,
      rarity: null,
      traits: record.traits,
      derivedTags: record.currentDerivedTags,
      families: record.families,
      descriptionText: record.descriptionText,
      blurbText: record.blurbText,
      sourceCategory: "unknown",
      publicationTitle: null,
      publicationRemaster: false,
      isUnique: false,
      size: null,
      languages: [],
      speedTypes: [],
      senses: [],
      immunities: [],
      resistances: [],
      weaknesses: [],
      itemCategory: null,
      baseItem: null,
      priceCp: null,
      usage: null,
      hands: null,
      damageTypes: [],
      weaponGroup: null,
      armorGroup: null,
      traditions: [],
      spellKinds: [],
      saveType: null,
      areaType: null,
      rangeText: null,
      durationText: null,
      targetText: null,
      areaValue: null,
      sustained: false,
      basicSave: false,
      disableText: null,
      disableSkills: [],
      isComplex: false,
    },
    currentSources: record.currentSources,
    selectionReasons: record.selectionReasons,
  };
}

export async function writeDerivedTagMigrationSession(
  rootPath: string,
  session: DerivedTagMigrationSession,
): Promise<void> {
  const directory = migrationSessionDirectory(rootPath, session.manifest.id);
  await mkdir(directory, { recursive: true });

  await writeFile(path.join(directory, "manifest.json"), JSON.stringify(session.manifest, null, 2) + "\n", "utf8");
  await writeFile(path.join(directory, "records.jsonl"), lineSeparatedJson(session.records), "utf8");
  await writeFile(path.join(directory, "decisions.jsonl"), lineSeparatedJson(session.decisions), "utf8");
  await writeFile(
    path.join(directory, "review-state.json"),
    JSON.stringify(session.reviewState, null, 2) + "\n",
    "utf8",
  );
}

export async function readDerivedTagMigrationSession(
  rootPath: string,
  sessionId: string,
): Promise<DerivedTagMigrationSession> {
  const directory = migrationSessionDirectory(rootPath, sessionId);
  const [manifestRaw, recordsRaw, decisionsRaw, reviewStateRaw] = await Promise.all([
    readFile(path.join(directory, "manifest.json"), "utf8"),
    readFile(path.join(directory, "records.jsonl"), "utf8"),
    readFile(path.join(directory, "decisions.jsonl"), "utf8"),
    readFile(path.join(directory, "review-state.json"), "utf8"),
  ]);

  return {
    manifest: JSON.parse(manifestRaw) as DerivedTagMigrationSessionManifest,
    records: parseJsonLines<DerivedTagMigrationSessionRecord | LegacyDerivedTagMigrationSessionRecord>(recordsRaw).map(
      (record) => normalizeSessionRecord(record),
    ),
    decisions: parseJsonLines<DerivedTagMigrationRecordDecision>(decisionsRaw),
    reviewState: JSON.parse(reviewStateRaw) as DerivedTagMigrationSessionReviewState,
  };
}
