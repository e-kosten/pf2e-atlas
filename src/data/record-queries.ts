import { DatabaseSync } from "node:sqlite";

import type { RuleReferenceEdge } from "../types.js";
import {
  buildCandidateQuery,
  buildLexicalRetrievalQuery,
  buildSemanticRetrievalQuery,
} from "../search/sql.js";
import type { NormalizedSearchFilters } from "./service-types.js";
import { buildPlaceholders, CandidateRow, ReferenceEdgeRow } from "./rows.js";
import type { LexicalRetrievalRow, SemanticRetrievalRow } from "../search/ranking.js";

function encodeVector(vector: Float32Array): Buffer {
  return Buffer.from(vector.buffer.slice(vector.byteOffset, vector.byteOffset + vector.byteLength));
}

function buildRecordSelect(includeRaw = false): string {
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
    "r.families_json AS familiesJson",
    "r.variant_family_key AS variantFamilyKey",
    "r.variant_base_name AS variantBaseName",
    "r.variant_label AS variantLabel",
    "r.variant_axes_json AS variantAxesJson",
    "r.variant_confidence AS variantConfidence",
    "r.variant_source AS variantSource",
    "r.source_path AS sourcePath",
    "r.is_unique AS isUnique",
    "r.is_search_canonical AS isSearchCanonical",
    "a.size AS size",
    "a.languages_json AS languagesJson",
    "a.speed_types_json AS speedTypesJson",
    "a.senses_json AS sensesJson",
    "a.immunities_json AS immunitiesJson",
    "a.resistances_json AS resistancesJson",
    "a.weaknesses_json AS weaknessesJson",
    "a.disable_text AS disableText",
    "a.disable_skills_json AS disableSkillsJson",
    "a.is_complex AS isComplex",
    `COALESCE((
      SELECT json_group_array(json_object(
        'metricKey', am.metric_key,
        'valueType', am.value_type,
        'numberValue', am.number_value,
        'textValue', am.text_value,
        'boolValue', am.bool_value
      ))
      FROM actor_metrics am
      WHERE am.record_key = r.record_key
    ), '[]') AS actorMetricsJson`,
    "i.item_category AS itemCategory",
    "i.base_item AS baseItem",
    "i.price_cp AS priceCp",
    "i.bulk_value AS bulkValue",
    "i.usage_text AS usage",
    "i.hands AS hands",
    `COALESCE((
      SELECT json_group_array(json_object(
        'metricKey', im.metric_key,
        'valueType', im.value_type,
        'numberValue', im.number_value,
        'textValue', im.text_value,
        'boolValue', im.bool_value
      ))
      FROM item_metrics im
      WHERE im.record_key = r.record_key
    ), '[]') AS itemMetricsJson`,
    "COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson",
    "i.weapon_group AS weaponGroup",
    "i.armor_group AS armorGroup",
    "COALESCE(s.action_cost, i.action_cost) AS actionCost",
    "s.traditions_json AS traditionsJson",
    "s.spell_kinds_json AS spellKindsJson",
    "s.range_text AS rangeText",
    "s.save_type AS saveType",
    "s.area_type AS areaType",
    "s.duration_text AS durationText",
    "s.duration_unit AS durationUnit",
    "s.target_text AS targetText",
    "s.area_value AS areaValue",
    "s.sustained AS sustained",
    "s.basic_save AS basicSave",
    "s.range_value AS rangeValue",
  ];

  if (includeRaw) {
    fields.push("r.raw_json AS rawJson");
  }

  return `
    SELECT
      ${fields.join(",\n      ")}
    FROM records r
    LEFT JOIN actor_records a ON a.record_key = r.record_key
    LEFT JOIN item_records i ON i.record_key = r.record_key
    LEFT JOIN spell_records s ON s.record_key = r.record_key
  `;
}

export function fetchCandidates(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  includeSearchText = false,
  includeEmbedding = false,
  options: { recordKeys?: string[] } = {},
): CandidateRow[] {
  const { sql, params } = buildCandidateQuery(filters, includeSearchText, includeEmbedding, options);
  return db.prepare(sql).all(...params) as CandidateRow[];
}

export function fetchLexicalRetrievalRows(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  ftsQuery: string,
  limit: number,
): LexicalRetrievalRow[] {
  const { sql, params } = buildLexicalRetrievalQuery(filters, ftsQuery, limit);
  return db.prepare(sql).all(...params) as LexicalRetrievalRow[];
}

export function fetchSemanticRetrievalRows(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  queryVector: Float32Array,
  limit: number,
): SemanticRetrievalRow[] {
  if (queryVector.length === 0) {
    return [];
  }

  const encodedQuery = encodeVector(queryVector);
  const { sql, params } = buildSemanticRetrievalQuery(filters, limit);
  return db.prepare(sql).all(encodedQuery, ...params) as SemanticRetrievalRow[];
}

export function fetchRecordRowsByKeys(db: DatabaseSync, recordKeys: string[]): CandidateRow[] {
  if (recordKeys.length === 0) {
    return [];
  }

  const placeholders = buildPlaceholders(recordKeys);
  return db
    .prepare(
      `
        ${buildRecordSelect()}
        WHERE r.record_key IN (${placeholders})
      `,
    )
    .all(...recordKeys) as CandidateRow[];
}

export function fetchRecordRow(
  db: DatabaseSync,
  recordKeyOrPack: string,
  maybeId?: string,
): CandidateRow | undefined {
  if (maybeId) {
    return db
      .prepare(
        `
          ${buildRecordSelect(true)}
          WHERE r.pack_name = ? AND r.id = ?
        `,
      )
      .get(recordKeyOrPack, maybeId) as CandidateRow | undefined;
  }

  return db
    .prepare(
      `
        ${buildRecordSelect(true)}
        WHERE r.record_key = ?
      `,
    )
    .get(recordKeyOrPack) as CandidateRow | undefined;
}

export function fetchReferenceEdgeRows(
  db: DatabaseSync,
  direction: RuleReferenceEdge["direction"],
  recordKeys: string[],
  {
    coreOnly = false,
  }: { coreOnly?: boolean } = {},
): ReferenceEdgeRow[] {
  if (recordKeys.length === 0) {
    return [];
  }

  const placeholders = buildPlaceholders(recordKeys);
  const targetFilter = direction === "outgoing"
    ? (coreOnly ? "AND target.source_category = 'core'" : "")
    : (coreOnly ? "AND re.from_source_category = 'core'" : "AND re.from_source_category IN ('core', 'rules')");
  const backlinkFilter = direction === "backlink"
    ? "AND (re.from_record_type = 'action' OR re.from_record_type = 'feat' OR LOWER(re.from_pack_name) = 'classfeatures')"
    : "";
  const keyColumn = direction === "outgoing" ? "re.from_record_key" : "re.to_record_key";

  return db
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
}
