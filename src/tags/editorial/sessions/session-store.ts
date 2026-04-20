import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import {
  parseSearchCategoryValue,
  parseSearchSubcategoryForCategory,
  parseSourceCategoryValue,
} from "../../../data/sql-row-decoding.js";
import type {
  DerivedTagMigrationRecordDecision,
  DerivedTagMigrationSession,
  DerivedTagMigrationSessionManifest,
  DerivedTagMigrationSelectionReason,
  DerivedTagMigrationSelectionSource,
  DerivedTagMigrationSessionRecord,
  DerivedTagMigrationSessionReviewState,
  DerivedTagMigrationMode,
  DerivedTagMigrationResolutionStatus,
} from "../types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/index.js";
import type { DerivedTagExemplarPolarity, AuthoredDerivedTagRule } from "../../../domain/index.js";
import type {
  DerivedTagReviewConfidence,
  DerivedTagReviewSource,
  DerivedTagReviewStatus,
} from "../../runtime/derivation/assignments.js";
import type { DerivedTagSource } from "../../runtime/publication/catalog.js";

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

type JsonObject = Record<string, unknown>;

const MIGRATION_MODES: readonly DerivedTagMigrationMode[] = [
  "review_queue",
  "proposal_review",
  "legacy_seed",
  "legacy_rule",
  "exemplar_cleanup",
];
const MIGRATION_RESOLUTION_STATUSES: readonly DerivedTagMigrationResolutionStatus[] = ["complete", "needs_review"];
const MIGRATION_SELECTION_SOURCES: readonly DerivedTagMigrationSelectionSource[] = [
  "authored_review_queue",
  "authored_exemplar_review_queue",
  "llm_assignment_review_queue",
  "llm_exemplar_review_queue",
  "legacy_seed",
  "legacy_rule",
  "exemplar_cleanup",
];
const REVIEW_STATUSES: readonly DerivedTagReviewStatus[] = ["auto_applied", "needs_review", "approved", "rejected"];
const REVIEW_CONFIDENCES: readonly DerivedTagReviewConfidence[] = ["high", "medium", "low"];
const REVIEW_SOURCES: readonly DerivedTagReviewSource[] = ["human", "llm"];
const EXEMPLAR_POLARITIES: readonly DerivedTagExemplarPolarity[] = ["positive", "negative"];
const DECISION_KINDS = ["assignment", "exemplar", "rule"] as const;
const ASSIGNMENT_MODES = ["include", "exclude"] as const;
const EXEMPLAR_ACTIONS = ["keep", "drop"] as const;
const RULE_DECISIONS = ["recreate_authored", "assignment_takeover", "retain_legacy"] as const;
const DERIVED_TAG_SOURCES: readonly DerivedTagSource[] = [
  "authored_rule",
  "legacy_rule",
  "seed_migration",
  "assignment",
  "authored_rule+legacy_rule",
  "authored_rule+seed_migration",
  "authored_rule+assignment",
  "legacy_rule+seed_migration",
  "legacy_rule+assignment",
  "seed_migration+assignment",
  "authored_rule+legacy_rule+seed_migration",
  "authored_rule+legacy_rule+assignment",
  "authored_rule+seed_migration+assignment",
  "legacy_rule+seed_migration+assignment",
  "authored_rule+legacy_rule+seed_migration+assignment",
];

function sessionRoot(rootPath: string): string {
  return path.join(rootPath, "scratch", "migration-sessions");
}

export function migrationSessionDirectory(rootPath: string, sessionId: string): string {
  return path.join(sessionRoot(rootPath), sessionId);
}

function lineSeparatedJson<T>(records: T[]): string {
  return records.map((record) => JSON.stringify(record)).join("\n") + (records.length > 0 ? "\n" : "");
}

function parseJson(raw: string, context: string): unknown {
  try {
    return JSON.parse(raw) as unknown;
  } catch (error) {
    throw new Error(`Invalid JSON in ${context}: ${error instanceof Error ? error.message : String(error)}`, {
      cause: error,
    });
  }
}

function parseJsonLines<T>(value: string, context: string, parseLine: (line: unknown, lineContext: string) => T): T[] {
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line, index) => parseLine(parseJson(line, `${context} line ${index + 1}`), `${context} line ${index + 1}`));
}

function expectObject(value: unknown, context: string): JsonObject {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Expected ${context} to be an object.`);
  }

  return value as JsonObject;
}

function expectArray(value: unknown, context: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`Expected ${context} to be an array.`);
  }

  return value;
}

function expectString(value: unknown, context: string): string {
  if (typeof value !== "string") {
    throw new Error(`Expected ${context} to be a string.`);
  }

  return value;
}

function expectNullableString(value: unknown, context: string): string | null {
  if (value === null) {
    return null;
  }

  return expectString(value, context);
}

function expectBoolean(value: unknown, context: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`Expected ${context} to be a boolean.`);
  }

  return value;
}

function expectInteger(value: unknown, context: string): number {
  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw new Error(`Expected ${context} to be an integer.`);
  }

  return value;
}

function expectNonNegativeInteger(value: unknown, context: string): number {
  const parsed = expectInteger(value, context);
  if (parsed < 0) {
    throw new Error(`Expected ${context} to be non-negative.`);
  }

  return parsed;
}

function expectNullableNumber(value: unknown, context: string): number | null {
  if (value === null) {
    return null;
  }
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`Expected ${context} to be a number or null.`);
  }

  return value;
}

function expectStringArray(value: unknown, context: string): string[] {
  return expectArray(value, context).map((entry, index) => expectString(entry, `${context}[${index}]`));
}

function parseLiteral<T extends string>(value: unknown, allowed: readonly T[], context: string): T {
  const parsed = expectString(value, context);
  if (!allowed.includes(parsed as T)) {
    throw new Error(`Expected ${context} to be one of: ${allowed.join(", ")}.`);
  }

  return parsed as T;
}

function parseCategory(value: unknown, context: string): SearchCategory {
  return parseSearchCategoryValue(expectString(value, context), context);
}

function parseSubcategory(category: SearchCategory, value: unknown, context: string): SearchSubcategory | null {
  if (value === null) {
    return null;
  }

  return parseSearchSubcategoryForCategory(category, expectString(value, context), context);
}

function parseSource(value: unknown, context: string): DerivedTagSource {
  return parseLiteral(value, DERIVED_TAG_SOURCES, context);
}

function parseMigrationMode(value: unknown, context: string): DerivedTagMigrationMode {
  return parseLiteral(value, MIGRATION_MODES, context);
}

function parseMigrationResolutionStatus(value: unknown, context: string): DerivedTagMigrationResolutionStatus {
  return parseLiteral(value, MIGRATION_RESOLUTION_STATUSES, context);
}

function parseSelectionSource(value: unknown, context: string): DerivedTagMigrationSelectionSource {
  return parseLiteral(value, MIGRATION_SELECTION_SOURCES, context);
}

function parseReviewStatus(value: unknown, context: string): DerivedTagReviewStatus {
  return parseLiteral(value, REVIEW_STATUSES, context);
}

function parseReviewConfidence(value: unknown, context: string): DerivedTagReviewConfidence | undefined {
  if (value === undefined) {
    return undefined;
  }

  return parseLiteral(value, REVIEW_CONFIDENCES, context);
}

function parseReviewSource(value: unknown, context: string): DerivedTagReviewSource | undefined {
  if (value === undefined) {
    return undefined;
  }

  return parseLiteral(value, REVIEW_SOURCES, context);
}

function parseExemplarPolarity(value: unknown, context: string): DerivedTagExemplarPolarity {
  return parseLiteral(value, EXEMPLAR_POLARITIES, context);
}

function parseOptionalAuthoredRules(value: unknown, context: string): AuthoredDerivedTagRule[] | undefined {
  if (value === undefined) {
    return undefined;
  }

  const rules = expectArray(value, context).map((entry, index) => expectObject(entry, `${context}[${index}]`));
  return rules as unknown as AuthoredDerivedTagRule[];
}

function parseCurrentSources(value: unknown, context: string): Record<string, DerivedTagSource> {
  const parsed = expectObject(value, context);
  return Object.fromEntries(
    Object.entries(parsed).map(([key, sourceValue]) => [key, parseSource(sourceValue, `${context}.${key}`)]),
  );
}

function parseSelectionReason(value: unknown, context: string): DerivedTagMigrationSelectionReason {
  const parsed = expectObject(value, context);
  return {
    source: parseSelectionSource(parsed.source, `${context}.source`),
    ...(parsed.family !== undefined ? { family: expectString(parsed.family, `${context}.family`) } : {}),
    ...(parsed.tag !== undefined ? { tag: expectString(parsed.tag, `${context}.tag`) } : {}),
    note: expectString(parsed.note, `${context}.note`),
  };
}

function parseSelectionReasons(value: unknown, context: string): DerivedTagMigrationSelectionReason[] {
  return expectArray(value, context).map((entry, index) => parseSelectionReason(entry, `${context}[${index}]`));
}

function parseEntityRecord(value: unknown, context: string): DerivedTagMigrationSessionRecord["entityRecord"] {
  const parsed = expectObject(value, context);
  const category = parseCategory(parsed.category, `${context}.category`);

  return {
    recordKey: expectString(parsed.recordKey, `${context}.recordKey`),
    packName: expectString(parsed.packName, `${context}.packName`),
    name: expectString(parsed.name, `${context}.name`),
    type: expectString(parsed.type, `${context}.type`),
    category,
    subcategory: parseSubcategory(category, parsed.subcategory, `${context}.subcategory`),
    documentType: expectString(parsed.documentType, `${context}.documentType`),
    level: expectNullableNumber(parsed.level, `${context}.level`),
    rarity: expectNullableString(parsed.rarity, `${context}.rarity`),
    traits: expectStringArray(parsed.traits, `${context}.traits`),
    derivedTags: expectStringArray(parsed.derivedTags, `${context}.derivedTags`),
    families: expectStringArray(parsed.families, `${context}.families`),
    descriptionText: expectNullableString(parsed.descriptionText, `${context}.descriptionText`),
    blurbText: expectNullableString(parsed.blurbText, `${context}.blurbText`),
    sourceCategory: parseSourceCategoryValue(
      expectString(parsed.sourceCategory, `${context}.sourceCategory`),
      `${context}.sourceCategory`,
    ),
    publicationTitle: expectNullableString(parsed.publicationTitle, `${context}.publicationTitle`),
    publicationRemaster: expectBoolean(parsed.publicationRemaster, `${context}.publicationRemaster`),
    isUnique: expectBoolean(parsed.isUnique, `${context}.isUnique`),
    size: expectNullableString(parsed.size, `${context}.size`),
    languages: expectStringArray(parsed.languages, `${context}.languages`),
    speedTypes: expectStringArray(parsed.speedTypes, `${context}.speedTypes`),
    senses: expectStringArray(parsed.senses, `${context}.senses`),
    immunities: expectStringArray(parsed.immunities, `${context}.immunities`),
    resistances: expectStringArray(parsed.resistances, `${context}.resistances`),
    weaknesses: expectStringArray(parsed.weaknesses, `${context}.weaknesses`),
    itemCategory: expectNullableString(parsed.itemCategory, `${context}.itemCategory`),
    baseItem: expectNullableString(parsed.baseItem, `${context}.baseItem`),
    priceCp: expectNullableNumber(parsed.priceCp, `${context}.priceCp`),
    usage: expectNullableString(parsed.usage, `${context}.usage`),
    hands: expectNullableNumber(parsed.hands, `${context}.hands`),
    damageTypes: expectStringArray(parsed.damageTypes, `${context}.damageTypes`),
    weaponGroup: expectNullableString(parsed.weaponGroup, `${context}.weaponGroup`),
    armorGroup: expectNullableString(parsed.armorGroup, `${context}.armorGroup`),
    traditions: expectStringArray(parsed.traditions, `${context}.traditions`),
    spellKinds: expectStringArray(parsed.spellKinds, `${context}.spellKinds`),
    saveType: expectNullableString(parsed.saveType, `${context}.saveType`),
    areaType: expectNullableString(parsed.areaType, `${context}.areaType`),
    rangeText: expectNullableString(parsed.rangeText, `${context}.rangeText`),
    durationText: expectNullableString(parsed.durationText, `${context}.durationText`),
    targetText: expectNullableString(parsed.targetText, `${context}.targetText`),
    areaValue: expectNullableNumber(parsed.areaValue, `${context}.areaValue`),
    sustained: expectBoolean(parsed.sustained, `${context}.sustained`),
    basicSave: expectBoolean(parsed.basicSave, `${context}.basicSave`),
    disableText: expectNullableString(parsed.disableText, `${context}.disableText`),
    disableSkills: expectStringArray(parsed.disableSkills, `${context}.disableSkills`),
    isComplex: expectBoolean(parsed.isComplex, `${context}.isComplex`),
  };
}

function parseLegacySessionRecord(value: unknown, context: string): LegacyDerivedTagMigrationSessionRecord {
  const parsed = expectObject(value, context);
  const category = parseCategory(parsed.category, `${context}.category`);

  return {
    recordKey: expectString(parsed.recordKey, `${context}.recordKey`),
    name: expectString(parsed.name, `${context}.name`),
    category,
    subcategory: parseSubcategory(category, parsed.subcategory, `${context}.subcategory`),
    packName: expectString(parsed.packName, `${context}.packName`),
    level: expectNullableNumber(parsed.level, `${context}.level`),
    traits: expectStringArray(parsed.traits, `${context}.traits`),
    families: expectStringArray(parsed.families, `${context}.families`),
    currentDerivedTags: expectStringArray(parsed.currentDerivedTags, `${context}.currentDerivedTags`),
    currentSources: parseCurrentSources(parsed.currentSources, `${context}.currentSources`),
    descriptionText: expectNullableString(parsed.descriptionText, `${context}.descriptionText`),
    blurbText: expectNullableString(parsed.blurbText, `${context}.blurbText`),
    selectionReasons: parseSelectionReasons(parsed.selectionReasons, `${context}.selectionReasons`),
  };
}

function parseSessionRecord(
  value: unknown,
  context: string,
): DerivedTagMigrationSessionRecord | LegacyDerivedTagMigrationSessionRecord {
  const parsed = expectObject(value, context);
  if ("entityRecord" in parsed) {
    return {
      entityRecord: parseEntityRecord(parsed.entityRecord, `${context}.entityRecord`),
      currentSources: parseCurrentSources(parsed.currentSources, `${context}.currentSources`),
      selectionReasons: parseSelectionReasons(parsed.selectionReasons, `${context}.selectionReasons`),
    };
  }

  return parseLegacySessionRecord(parsed, context);
}

function parseRecordDecision(value: unknown, context: string): DerivedTagMigrationRecordDecision {
  const parsed = expectObject(value, context);
  return {
    recordKey: expectString(parsed.recordKey, `${context}.recordKey`),
    name: expectString(parsed.name, `${context}.name`),
    category: parseCategory(parsed.category, `${context}.category`),
    resolutionStatus: parseMigrationResolutionStatus(parsed.resolutionStatus, `${context}.resolutionStatus`),
    ...(parsed.ontologyNotes !== undefined
      ? { ontologyNotes: expectStringArray(parsed.ontologyNotes, `${context}.ontologyNotes`) }
      : {}),
    decisions: expectArray(parsed.decisions, `${context}.decisions`).map((entry, index) =>
      parseMigrationDecision(entry, `${context}.decisions[${index}]`),
    ),
  };
}

function parseMigrationDecision(
  value: unknown,
  context: string,
): DerivedTagMigrationRecordDecision["decisions"][number] {
  const parsed = expectObject(value, context);
  const kind = expectString(parsed.kind, `${context}.kind`);
  if (!DECISION_KINDS.includes(kind as (typeof DECISION_KINDS)[number])) {
    throw new Error(`Expected ${context}.kind to be a valid migration decision kind.`);
  }

  switch (kind) {
    case "assignment": {
      const mode = parseLiteral(parsed.mode, ASSIGNMENT_MODES, `${context}.mode`);
      return {
        kind,
        family: expectString(parsed.family, `${context}.family`),
        tag: expectString(parsed.tag, `${context}.tag`),
        mode,
        status: parseReviewStatus(parsed.status, `${context}.status`),
        confidence: parseReviewConfidence(parsed.confidence, `${context}.confidence`),
        rationale: expectString(parsed.rationale, `${context}.rationale`),
        source: parseReviewSource(parsed.source, `${context}.source`),
      };
    }
    case "exemplar": {
      const action = parseLiteral(parsed.action, EXEMPLAR_ACTIONS, `${context}.action`);

      const currentPolarity =
        parsed.currentPolarity === undefined
          ? undefined
          : parsed.currentPolarity === "none"
            ? "none"
            : parseExemplarPolarity(parsed.currentPolarity, `${context}.currentPolarity`);

      return {
        kind,
        tag: expectString(parsed.tag, `${context}.tag`),
        polarity: parseExemplarPolarity(parsed.polarity, `${context}.polarity`),
        action,
        status: parseReviewStatus(parsed.status, `${context}.status`),
        confidence: parseReviewConfidence(parsed.confidence, `${context}.confidence`),
        rationale: expectString(parsed.rationale, `${context}.rationale`),
        source: parseReviewSource(parsed.source, `${context}.source`),
        ...(currentPolarity !== undefined ? { currentPolarity } : {}),
      };
    }
    case "rule": {
      const decision = parseLiteral(parsed.decision, RULE_DECISIONS, `${context}.decision`);

      return {
        kind,
        tag: expectString(parsed.tag, `${context}.tag`),
        decision,
        status: parseReviewStatus(parsed.status, `${context}.status`),
        rationale: expectString(parsed.rationale, `${context}.rationale`),
        source: parseReviewSource(parsed.source, `${context}.source`),
        ...(parsed.authoredRules !== undefined
          ? { authoredRules: parseOptionalAuthoredRules(parsed.authoredRules, `${context}.authoredRules`) }
          : {}),
      };
    }
    default:
      throw new Error(`Expected ${context}.kind to be a valid migration decision kind.`);
  }
}

function parseSessionManifest(value: unknown, context: string): DerivedTagMigrationSessionManifest {
  const parsed = expectObject(value, context);
  const category = parsed.category === undefined ? undefined : parseCategory(parsed.category, `${context}.category`);
  const subcategory =
    parsed.subcategory === undefined
      ? undefined
      : category
        ? (parseSubcategory(category, parsed.subcategory, `${context}.subcategory`) ?? undefined)
        : (() => {
            throw new Error(`Expected ${context}.category to be set when ${context}.subcategory is present.`);
          })();

  return {
    id: expectString(parsed.id, `${context}.id`),
    mode: parseMigrationMode(parsed.mode, `${context}.mode`),
    ...(category ? { category } : {}),
    ...(subcategory ? { subcategory } : {}),
    ...(parsed.family !== undefined ? { family: expectString(parsed.family, `${context}.family`) } : {}),
    ...(parsed.tag !== undefined ? { tag: expectString(parsed.tag, `${context}.tag`) } : {}),
    createdAt: expectString(parsed.createdAt, `${context}.createdAt`),
    recordCount: expectNonNegativeInteger(parsed.recordCount, `${context}.recordCount`),
  };
}

function parseSessionReviewState(value: unknown, context: string): DerivedTagMigrationSessionReviewState {
  const parsed = expectObject(value, context);
  return {
    currentIndex: expectNonNegativeInteger(parsed.currentIndex, `${context}.currentIndex`),
    unresolvedOnly: expectBoolean(parsed.unresolvedOnly, `${context}.unresolvedOnly`),
    updatedAt: expectString(parsed.updatedAt, `${context}.updatedAt`),
  };
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
    manifest: parseSessionManifest(parseJson(manifestRaw, "manifest.json"), "manifest.json"),
    records: parseJsonLines(recordsRaw, "records.jsonl", (line, lineContext) =>
      parseSessionRecord(line, lineContext),
    ).map((record) => normalizeSessionRecord(record)),
    decisions: parseJsonLines(decisionsRaw, "decisions.jsonl", (line, lineContext) =>
      parseRecordDecision(line, lineContext),
    ),
    reviewState: parseSessionReviewState(parseJson(reviewStateRaw, "review-state.json"), "review-state.json"),
  };
}
