import { DatabaseSync } from "node:sqlite";

import { buildPlaceholders } from "../../../data/rows.js";
import {
  parseSearchCategoryValue,
  parseSearchSubcategoryForCategory,
  parseStringArrayJson,
  toSqliteNumber,
} from "../../../data/sql-row-decoding.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/index.js";
import { normalizeText } from "../../../shared/utils.js";
import {
  buildOntologyExplorerEntityRecordSelectColumns,
  mapOntologyExplorerEntityRecordRow,
  type OntologyExplorerEntityRecord,
  type OntologyExplorerEntityRecordRow,
} from "../../entity-record.js";

export type DerivedTagMigrationReference = {
  recordKey: string;
  packName: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: string[];
};

export type DerivedTagMigrationRecord = {
  entityRecord: OntologyExplorerEntityRecord;
  references: DerivedTagMigrationReference[];
};

type LoadedReferenceRow = {
  fromRecordKey: string;
  targetRecordKey: string;
  targetPackName: string | null;
  targetName: string;
  targetCategory: string;
  targetSubcategory: string | null;
  targetTraitsJson: string;
};

const FULL_RECORD_ENTITY_COLUMNS = [
  "pack_name",
  "record_type",
  "document_type",
  "rarity",
  "traits_json",
  "derived_tags_json",
  "families_json",
  "description_text",
  "blurb_text",
  "source_category",
  "publication_title",
  "publication_remaster",
  "is_unique",
] as const;

function hasTable(db: DatabaseSync, tableName: string): boolean {
  const row = db
    .prepare(
      `
    SELECT 1 AS present
    FROM sqlite_master
    WHERE type = 'table' AND name = ?
    LIMIT 1
  `,
    )
    .get(tableName) as { present: number } | undefined;
  return Boolean(row?.present);
}

function listTableColumns(db: DatabaseSync, tableName: string): Set<string> {
  const rows = db.prepare(`PRAGMA table_info(${tableName})`).all() as Array<{ name: string }>;
  return new Set(rows.map((row) => row.name));
}

function buildFallbackSelectColumns(options: {
  includeActor: boolean;
  includeItem: boolean;
  includeSpell: boolean;
  recordColumns: Set<string>;
}): string[] {
  const { includeActor, includeItem, includeSpell, recordColumns } = options;
  const optionalRecordColumn = (column: string, alias: string, fallbackSql: string): string =>
    recordColumns.has(column) ? `r.${column} AS ${alias}` : `${fallbackSql} AS ${alias}`;
  const optionalJoinedColumn = (enabled: boolean, expression: string, alias: string): string =>
    enabled ? `${expression} AS ${alias}` : `NULL AS ${alias}`;

  return [
    "r.record_key AS recordKey",
    optionalRecordColumn("pack_name", "packName", "NULL"),
    "r.name AS name",
    optionalRecordColumn("record_type", "type", "'unknown'"),
    "r.category AS category",
    optionalRecordColumn("subcategory", "subcategory", "NULL"),
    optionalRecordColumn("document_type", "documentType", "'unknown'"),
    optionalRecordColumn("level", "level", "NULL"),
    optionalRecordColumn("rarity", "rarity", "NULL"),
    optionalRecordColumn("traits_json", "traitsJson", "'[]'"),
    optionalRecordColumn("derived_tags_json", "derivedTagsJson", "'[]'"),
    optionalRecordColumn("families_json", "familiesJson", "NULL"),
    optionalRecordColumn("description_text", "descriptionText", "NULL"),
    optionalRecordColumn("blurb_text", "blurbText", "NULL"),
    optionalRecordColumn("source_category", "sourceCategory", "'unknown'"),
    optionalRecordColumn("publication_title", "publicationTitle", "NULL"),
    optionalRecordColumn("publication_remaster", "publicationRemaster", "0"),
    optionalRecordColumn("is_unique", "isUnique", "0"),
    optionalJoinedColumn(includeActor, "a.size", "size"),
    optionalJoinedColumn(includeActor, "a.languages_json", "languagesJson"),
    optionalJoinedColumn(includeActor, "a.speed_types_json", "speedTypesJson"),
    optionalJoinedColumn(includeActor, "a.senses_json", "sensesJson"),
    optionalJoinedColumn(includeActor, "a.immunities_json", "immunitiesJson"),
    optionalJoinedColumn(includeActor, "a.resistances_json", "resistancesJson"),
    optionalJoinedColumn(includeActor, "a.weaknesses_json", "weaknessesJson"),
    optionalJoinedColumn(includeActor, "a.disable_text", "disableText"),
    optionalJoinedColumn(includeActor, "a.disable_skills_json", "disableSkillsJson"),
    optionalJoinedColumn(includeActor, "a.is_complex", "isComplex"),
    optionalJoinedColumn(includeItem, "i.item_category", "itemCategory"),
    optionalJoinedColumn(includeItem, "i.base_item", "baseItem"),
    optionalJoinedColumn(includeItem, "i.price_cp", "priceCp"),
    optionalJoinedColumn(includeItem, "i.usage_text", "usage"),
    optionalJoinedColumn(includeItem, "i.hands", "hands"),
    optionalJoinedColumn(includeItem, "i.damage_types_json", "damageTypesJson"),
    optionalJoinedColumn(includeItem, "i.weapon_group", "weaponGroup"),
    optionalJoinedColumn(includeItem, "i.armor_group", "armorGroup"),
    optionalJoinedColumn(includeSpell, "s.traditions_json", "traditionsJson"),
    optionalJoinedColumn(includeSpell, "s.spell_kinds_json", "spellKindsJson"),
    optionalJoinedColumn(includeSpell, "s.save_type", "saveType"),
    optionalJoinedColumn(includeSpell, "s.area_type", "areaType"),
    optionalJoinedColumn(includeSpell, "s.range_text", "rangeText"),
    optionalJoinedColumn(includeSpell, "s.duration_text", "durationText"),
    optionalJoinedColumn(includeSpell, "s.target_text", "targetText"),
    optionalJoinedColumn(includeSpell, "s.area_value", "areaValue"),
    optionalJoinedColumn(includeSpell, "s.sustained", "sustained"),
    optionalJoinedColumn(includeSpell, "s.basic_save", "basicSave"),
  ];
}

function loadReferences(db: DatabaseSync, recordKeys: string[]): Map<string, DerivedTagMigrationReference[]> {
  if (recordKeys.length === 0) {
    return new Map();
  }

  const rows = db
    .prepare(
      `
    SELECT
      re.from_record_key AS fromRecordKey,
      re.to_record_key AS targetRecordKey,
      target.pack_name AS targetPackName,
      target.name AS targetName,
      target.category AS targetCategory,
      target.subcategory AS targetSubcategory,
      target.traits_json AS targetTraitsJson
    FROM reference_edges re
    JOIN records target ON target.record_key = re.to_record_key
    WHERE re.from_record_key IN (${buildPlaceholders(recordKeys)})
  `,
    )
    .all(...recordKeys) as LoadedReferenceRow[];

  const referencesByRecordKey = new Map<string, DerivedTagMigrationReference[]>();
  for (const row of rows) {
    const bucket = referencesByRecordKey.get(row.fromRecordKey) ?? [];
    const category = parseSearchCategoryValue(row.targetCategory, `migration reference "${row.targetRecordKey}"`);
    bucket.push({
      recordKey: row.targetRecordKey,
      packName: row.targetPackName ?? row.targetRecordKey.split(":")[0] ?? "",
      name: row.targetName,
      category,
      subcategory: parseSearchSubcategoryForCategory(
        category,
        row.targetSubcategory,
        `migration reference "${row.targetRecordKey}"`,
      ),
      traits: parseStringArrayJson(
        row.targetTraitsJson,
        "targetTraitsJson",
        `migration reference "${row.targetRecordKey}"`,
      ),
    });
    referencesByRecordKey.set(row.fromRecordKey, bucket);
  }

  return referencesByRecordKey;
}

export type LoadDerivedTagMigrationRecordsOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  recordKeys?: string[];
  requireTag?: string;
  untaggedOnly?: boolean;
  limit?: number;
};

function appendRecordFilters(
  sql: string[],
  params: Array<string | number>,
  options: LoadDerivedTagMigrationRecordsOptions,
): void {
  if (options.category) {
    sql.push("AND r.category = ?");
    params.push(options.category);
  }
  if (options.subcategory) {
    sql.push("AND r.subcategory = ?");
    params.push(options.subcategory);
  }
  if (options.requireTag) {
    sql.push("AND EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag = ?)");
    params.push(normalizeText(options.requireTag));
  }
  if (options.untaggedOnly) {
    sql.push("AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key)");
  }
  if (options.recordKeys) {
    if (options.recordKeys.length === 0) {
      sql.push("AND 0 = 1");
    } else {
      sql.push(`AND r.record_key IN (${buildPlaceholders(options.recordKeys)})`);
      params.push(...options.recordKeys);
    }
  }
}

export function countDerivedTagMigrationRecords(
  db: DatabaseSync,
  options: LoadDerivedTagMigrationRecordsOptions,
): number {
  const sql = ["SELECT COUNT(*) AS count", "FROM records r", "WHERE r.is_search_canonical = 1"];
  const params: Array<string | number> = [];
  appendRecordFilters(sql, params, options);

  const row = db.prepare(sql.join("\n")).get(...params) as { count: number | bigint } | undefined;
  return row ? toSqliteNumber(row.count, "migration record count") : 0;
}

export function loadDerivedTagMigrationRecords(
  db: DatabaseSync,
  options: LoadDerivedTagMigrationRecordsOptions,
): DerivedTagMigrationRecord[] {
  const includeActor = hasTable(db, "actor_records");
  const includeItem = hasTable(db, "item_records");
  const includeSpell = hasTable(db, "spell_records");
  const recordColumns = listTableColumns(db, "records");
  const hasFullRecordProjection = FULL_RECORD_ENTITY_COLUMNS.every((column) => recordColumns.has(column));
  const selectColumns = hasFullRecordProjection
    ? buildOntologyExplorerEntityRecordSelectColumns({
        includeActor,
        includeItem,
        includeSpell,
      })
    : buildFallbackSelectColumns({
        includeActor,
        includeItem,
        includeSpell,
        recordColumns,
      });
  const sql = ["SELECT", `  ${selectColumns.join(",\n  ")}`, "FROM records r", "WHERE r.is_search_canonical = 1"];
  if (includeActor) {
    sql.splice(3, 0, "LEFT JOIN actor_records a ON a.record_key = r.record_key");
  }
  if (includeItem) {
    sql.splice(includeActor ? 4 : 3, 0, "LEFT JOIN item_records i ON i.record_key = r.record_key");
  }
  if (includeSpell) {
    sql.splice(
      (includeActor ? 1 : 0) + (includeItem ? 1 : 0) + 3,
      0,
      "LEFT JOIN spell_records s ON s.record_key = r.record_key",
    );
  }
  const params: Array<string | number> = [];
  appendRecordFilters(sql, params, options);
  sql.push("ORDER BY r.name ASC, r.record_key ASC");
  if (options.limit && options.limit > 0) {
    sql.push(`LIMIT ${Math.trunc(options.limit)}`);
  }

  const rows = db.prepare(sql.join("\n")).all(...params) as OntologyExplorerEntityRecordRow[];
  const referencesByRecordKey = loadReferences(
    db,
    rows.map((row) => row.recordKey),
  );

  return rows.map((row) => ({
    entityRecord: mapOntologyExplorerEntityRecordRow(row),
    references: referencesByRecordKey.get(row.recordKey) ?? [],
  }));
}
