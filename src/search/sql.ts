import {
  FilterValueField,
  NormalizedRecord,
  SearchCategory,
  SearchScope,
  SearchSubcategory,
} from "../types.js";
import {
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import type { NormalizedSearchFilters, NormalizedSearchScope, SqlValue } from "../data/service-types.js";
import { appendMetadataFilterClauses, buildFamiliesArraySql, recordMatchesMetadataFilter } from "./metadata-filters.js";
import { normalizeText, uniqueSorted } from "../utils.js";

function appendWhereClause(sql: string[], params: SqlValue[], clause: string, ...values: SqlValue[]): void {
  sql.push(clause);
  params.push(...values);
}

export function normalizeSearchScope(scope: SearchScope): NormalizedSearchScope {
  const category = normalizeSearchCategory(scope.category);
  if (!category) {
    throw new Error(getSearchCategoryErrorMessage(String(scope.category)));
  }

  const subcategories = scope.subcategories?.map((subcategory) => {
    const canonicalSubcategory = normalizeSearchSubcategory(subcategory);
    if (!canonicalSubcategory) {
      throw new Error(getSearchSubcategoryErrorMessage(String(subcategory)));
    }
    return canonicalSubcategory;
  });

  const uniqueSubcategories = subcategories
    ? uniqueSorted(subcategories) as SearchSubcategory[]
    : undefined;

  return {
    category,
    subcategories: uniqueSubcategories && uniqueSubcategories.length > 0 ? uniqueSubcategories : undefined,
  };
}

export function resolveEffectiveCategory(filters: Pick<NormalizedSearchFilters, "category" | "subcategory" | "scopes">): SearchCategory | null {
  if (filters.scopes && filters.scopes.length > 0) {
    return null;
  }

  const inferredCategoryFromSubcategory = !filters.category && filters.subcategory
    ? getCategoryForSubcategory(filters.subcategory)
    : null;
  return filters.category ?? inferredCategoryFromSubcategory;
}

function appendScopedCategoryClauses(
  sql: string[],
  params: SqlValue[],
  scopes: NormalizedSearchScope[],
  renderTerm: (category: SearchCategory, subcategories: SearchSubcategory[] | undefined) => { clause: string; values: SqlValue[] },
): void {
  const renderedScopes = scopes.map((scope) => renderTerm(scope.category, scope.subcategories));
  appendWhereClause(
    sql,
    params,
    `AND (${renderedScopes.map((entry) => entry.clause).join(" OR ")})`,
    ...renderedScopes.flatMap((entry) => entry.values),
  );
}

function applySearchFilterClauses(
  sql: string[],
  params: SqlValue[],
  filters: NormalizedSearchFilters,
  aliases: {
    records: string;
    actor: string;
    item: string;
    spell: string;
  },
  options: {
    recordKeys?: string[];
  } = {},
): void {
  const recordAlias = aliases.records;
  const actorAlias = aliases.actor;
  const itemAlias = aliases.item;
  const spellAlias = aliases.spell;

  appendWhereClause(sql, params, `AND ${recordAlias}.is_search_canonical = 1`);

  if (options.recordKeys && options.recordKeys.length > 0) {
    const placeholders = options.recordKeys.map(() => "?").join(", ");
    appendWhereClause(sql, params, `AND ${recordAlias}.record_key IN (${placeholders})`, ...options.recordKeys);
  }

  if (filters.pack) {
    appendWhereClause(
      sql,
      params,
      `AND (LOWER(${recordAlias}.pack_name) = LOWER(?) OR LOWER(${recordAlias}.pack_label) = LOWER(?))`,
      filters.pack,
      filters.pack,
    );
  }

  if (filters.scopes && filters.scopes.length > 0) {
    appendScopedCategoryClauses(sql, params, filters.scopes, (category, subcategories) => {
      if (!subcategories || subcategories.length === 0) {
        return {
          clause: `LOWER(${recordAlias}.category) = LOWER(?)`,
          values: [category],
        };
      }

      const placeholders = subcategories.map(() => "?").join(", ");
      return {
        clause: `(LOWER(${recordAlias}.category) = LOWER(?) AND LOWER(COALESCE(${recordAlias}.subcategory, '')) IN (${placeholders}))`,
        values: [category, ...subcategories.map((subcategory) => normalizeText(subcategory))],
      };
    });
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory) {
      appendWhereClause(sql, params, `AND LOWER(${recordAlias}.category) = LOWER(?)`, effectiveCategory);
    }

    if (filters.subcategory) {
      appendWhereClause(sql, params, `AND LOWER(COALESCE(${recordAlias}.subcategory, '')) = LOWER(?)`, filters.subcategory);
    }
  }

  if (filters.levelMin !== undefined) {
    appendWhereClause(sql, params, `AND ${recordAlias}.level >= ?`, filters.levelMin);
  }

  if (filters.levelMax !== undefined) {
    appendWhereClause(sql, params, `AND ${recordAlias}.level <= ?`, filters.levelMax);
  }

  if (filters.rarity) {
    appendWhereClause(sql, params, `AND LOWER(COALESCE(${recordAlias}.rarity, '')) = LOWER(?)`, filters.rarity);
  }

  if (filters.priceMin !== undefined) {
    appendWhereClause(sql, params, `AND ${itemAlias}.price_cp >= ?`, filters.priceMin);
  }

  if (filters.priceMax !== undefined) {
    appendWhereClause(sql, params, `AND ${itemAlias}.price_cp <= ?`, filters.priceMax);
  }

  if (filters.actionCost !== undefined) {
    appendWhereClause(sql, params, `AND COALESCE(${spellAlias}.action_cost, ${itemAlias}.action_cost) = ?`, filters.actionCost);
  }

  appendMetadataFilterClauses(sql, params, filters.metadata, {
    recordKeyExpr: `${recordAlias}.record_key`,
    recordsAlias: recordAlias,
    actorAlias,
    itemAlias,
    spellAlias,
  }, appendWhereClause);
}

export function buildCandidateQuery(
  filters: NormalizedSearchFilters,
  includeSearchText = false,
  includeEmbedding = false,
  options: { recordKeys?: string[] } = {},
): { sql: string; params: SqlValue[] } {
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
    "r.source_path AS sourcePath",
    "r.is_unique AS isUnique",
    "r.is_search_canonical AS isSearchCanonical",
    "a.size AS size",
    "a.languages_json AS languagesJson",
    "a.speed_types_json AS speedTypesJson",
    "a.immunities_json AS immunitiesJson",
    "a.resistances_json AS resistancesJson",
    "a.weaknesses_json AS weaknessesJson",
    "i.item_category AS itemCategory",
    "i.price_cp AS priceCp",
    "i.bulk_value AS bulkValue",
    "i.usage_text AS usage",
    "i.hands AS hands",
    "COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson",
    "i.weapon_group AS weaponGroup",
    "i.armor_group AS armorGroup",
    "COALESCE(s.action_cost, i.action_cost) AS actionCost",
    "s.traditions_json AS traditionsJson",
    "s.spell_kinds_json AS spellKindsJson",
    "s.range_value AS rangeValue",
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
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  }, options);

  return { sql: sql.join("\n"), params };
}

export function buildFilterValueQuery(field: FilterValueField, filters: NormalizedSearchFilters): { sql: string; params: SqlValue[] } {
  const joins = [
    "FROM records r",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
  ];
  const sql: string[] = [];
  const params: SqlValue[] = [];
  const postFilterClauses: string[] = [];
  let valueExpression = "";

  switch (field) {
    case "traits":
      joins.push("JOIN record_traits rt ON rt.record_key = r.record_key");
      valueExpression = "rt.trait";
      break;
    case "families":
      joins.push(`JOIN json_each(${buildFamiliesArraySql("r")}) AS family`);
      valueExpression = "LOWER(family.value)";
      break;
    case "derivedTags":
      joins.push("JOIN record_derived_tags rdt ON rdt.record_key = r.record_key");
      valueExpression = "rdt.tag";
      break;
    case "rarity":
      valueExpression = "r.rarity";
      postFilterClauses.push("AND r.rarity IS NOT NULL AND r.rarity <> ''");
      break;
    case "size":
      valueExpression = "a.size";
      postFilterClauses.push("AND a.size IS NOT NULL AND a.size <> ''");
      break;
    case "publicationTitle":
      valueExpression = "r.publication_title";
      postFilterClauses.push("AND r.publication_title IS NOT NULL AND r.publication_title <> ''");
      break;
    case "traditions":
      joins.push("JOIN json_each(COALESCE(s.traditions_json, '[]')) AS tradition");
      valueExpression = "tradition.value";
      break;
    case "spellKinds":
      joins.push("JOIN json_each(COALESCE(s.spell_kinds_json, '[]')) AS spell_kind");
      valueExpression = "spell_kind.value";
      break;
    case "weaponGroup":
      valueExpression = "i.weapon_group";
      postFilterClauses.push("AND i.weapon_group IS NOT NULL AND i.weapon_group <> ''");
      break;
    case "armorGroup":
      valueExpression = "i.armor_group";
      postFilterClauses.push("AND i.armor_group IS NOT NULL AND i.armor_group <> ''");
      break;
    case "usage":
      valueExpression = "i.usage_text";
      postFilterClauses.push("AND i.usage_text IS NOT NULL AND i.usage_text <> ''");
      break;
    case "damageTypes":
      joins.push("JOIN json_each(COALESCE(s.damage_types_json, i.damage_types_json, '[]')) AS damage_type");
      valueExpression = "damage_type.value";
      break;
    case "languages":
      joins.push("JOIN json_each(COALESCE(a.languages_json, '[]')) AS language");
      valueExpression = "language.value";
      break;
    case "speedTypes":
      joins.push("JOIN json_each(COALESCE(a.speed_types_json, '[]')) AS speed_type");
      valueExpression = "speed_type.value";
      break;
    case "immunities":
      joins.push("JOIN json_each(COALESCE(a.immunities_json, '[]')) AS immunity");
      valueExpression = "immunity.value";
      break;
    case "resistances":
      joins.push("JOIN json_each(COALESCE(a.resistances_json, '[]')) AS resistance");
      valueExpression = "resistance.value";
      break;
    case "weaknesses":
      joins.push("JOIN json_each(COALESCE(a.weaknesses_json, '[]')) AS weakness");
      valueExpression = "weakness.value";
      break;
    case "itemCategory":
      valueExpression = "i.item_category";
      postFilterClauses.push("AND i.item_category IS NOT NULL AND i.item_category <> ''");
      break;
    case "sources":
      valueExpression = "r.source_category";
      break;
    case "categories":
      valueExpression = "r.category";
      break;
    case "subcategories":
      valueExpression = "r.subcategory";
      postFilterClauses.push("AND r.subcategory IS NOT NULL AND r.subcategory <> ''");
      break;
    case "packs":
      valueExpression = "r.pack_label";
      postFilterClauses.push("AND r.pack_label IS NOT NULL AND r.pack_label <> ''");
      break;
  }

  sql.push(`SELECT ${valueExpression} AS value, COUNT(*) AS count`);
  sql.push(...joins);
  sql.push("WHERE 1 = 1");
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  });
  sql.push(...postFilterClauses);
  sql.push("GROUP BY value");
  sql.push("ORDER BY count DESC, value ASC");
  return { sql: sql.join("\n"), params };
}

export function buildLexicalRetrievalQuery(filters: NormalizedSearchFilters, query: string, limit: number): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT r.record_key AS recordKey, bm25(records_fts, 8.0, 1.5) AS rank",
    "FROM records_fts",
    "JOIN records r ON r.record_key = records_fts.record_key",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
    "WHERE records_fts MATCH ?",
  ];
  const params: SqlValue[] = [query];
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  });
  sql.push("ORDER BY rank");
  sql.push("LIMIT ?");
  params.push(limit);
  return { sql: sql.join("\n"), params };
}

export function semanticQueryLimit(baseLimit: number, filters: NormalizedSearchFilters): number {
  return filters.metadata ? Math.min(1000, Math.max(baseLimit * 2, baseLimit + 50)) : baseLimit;
}

function normalizeVectorText(value: string | null | undefined): string {
  return normalizeText(value ?? "") || "";
}

export function buildSemanticRetrievalQuery(filters: NormalizedSearchFilters, limit: number): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT record_key AS recordKey, distance",
    "FROM record_embeddings",
    "WHERE embedding MATCH ?",
    `AND k = ${limit}`,
  ];
  const params: SqlValue[] = [];

  if (filters.scopes && filters.scopes.length > 0) {
    appendScopedCategoryClauses(sql, params, filters.scopes, (category, subcategories) => {
      if (!subcategories || subcategories.length === 0) {
        return {
          clause: "category = ?",
          values: [normalizeVectorText(category)],
        };
      }

      const placeholders = subcategories.map(() => "?").join(", ");
      return {
        clause: `(category = ? AND subcategory IN (${placeholders}))`,
        values: [normalizeVectorText(category), ...subcategories.map((subcategory) => normalizeVectorText(subcategory))],
      };
    });
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory) {
      appendWhereClause(sql, params, "AND category = ?", normalizeVectorText(effectiveCategory));
    }
    if (filters.subcategory) {
      appendWhereClause(sql, params, "AND subcategory = ?", normalizeVectorText(filters.subcategory));
    }
  }
  if (filters.pack) {
    appendWhereClause(sql, params, "AND pack_name = ?", normalizeVectorText(filters.pack));
  }
  if (filters.levelMin !== undefined) {
    appendWhereClause(sql, params, "AND level >= ?", BigInt(filters.levelMin));
  }
  if (filters.levelMax !== undefined) {
    appendWhereClause(sql, params, "AND level <= ?", BigInt(filters.levelMax));
  }
  if (filters.rarity) {
    appendWhereClause(sql, params, "AND rarity = ?", normalizeVectorText(filters.rarity));
  }
  if (filters.priceMin !== undefined) {
    appendWhereClause(sql, params, "AND price_cp >= ?", BigInt(filters.priceMin));
  }
  if (filters.priceMax !== undefined) {
    appendWhereClause(sql, params, "AND price_cp <= ?", BigInt(filters.priceMax));
  }
  if (filters.actionCost !== undefined) {
    appendWhereClause(sql, params, "AND action_cost = ?", BigInt(filters.actionCost));
  }

  appendMetadataFilterClauses(sql, params, filters.metadata, {
    recordKeyExpr: "record_embeddings.record_key",
  }, appendWhereClause);

  return { sql: sql.join("\n"), params };
}

function recordMatchesScope(record: NormalizedRecord, scope: NormalizedSearchScope): boolean {
  if (record.category !== scope.category) {
    return false;
  }

  if (!scope.subcategories || scope.subcategories.length === 0) {
    return true;
  }

  return record.subcategory !== null && scope.subcategories.includes(record.subcategory);
}

export function recordMatchesFilters(record: NormalizedRecord, filters: NormalizedSearchFilters): boolean {
  if (filters.pack) {
    const normalizedPack = normalizeText(filters.pack);
    if (normalizeText(record.packName) !== normalizedPack && normalizeText(record.packLabel) !== normalizedPack) {
      return false;
    }
  }

  if (filters.scopes && filters.scopes.length > 0) {
    if (!filters.scopes.some((scope) => recordMatchesScope(record, scope))) {
      return false;
    }
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory && record.category !== effectiveCategory) {
      return false;
    }
    if (filters.subcategory && record.subcategory !== filters.subcategory) {
      return false;
    }
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
  if (filters.priceMin !== undefined && (record.priceCp === null || record.priceCp < filters.priceMin)) {
    return false;
  }
  if (filters.priceMax !== undefined && (record.priceCp === null || record.priceCp > filters.priceMax)) {
    return false;
  }
  if (filters.actionCost !== undefined && record.actionCost !== filters.actionCost) {
    return false;
  }
  if (filters.metadata && !recordMatchesMetadataFilter(record, filters.metadata)) {
    return false;
  }

  return true;
}
