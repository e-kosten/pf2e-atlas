import { DatabaseSync } from "node:sqlite";

import { groupDerivedTagOntology } from "../tags/runtime/catalog-utils.js";
import { DERIVED_TAG_ONTOLOGY_FAMILIES, DERIVED_TAG_ONTOLOGY_TAGS } from "../tags/index.js";
import type {
  DerivedTagCatalogEntry,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
  FilterValueQuery,
  FilterValueResult,
  PackInfo,
  SearchCategory,
  SearchSubcategory,
  SourceCategory,
} from "../types.js";
import { buildFilterValueQuery } from "../search/sql.js";
import { normalizeText } from "../utils.js";
import type { NormalizedSearchFilters } from "./service-types.js";
import type { ValueCountRow } from "./rows.js";
import {
  parseSearchCategoryValue,
  parseSearchSubcategoryValue,
  parseSourceCategoryValue,
  parseStringArrayJson,
  toSqliteNumber,
} from "./sql-row-decoding.js";

export type SearchVocabularyResult = {
  categories: Array<{ value: SearchCategory; count: number }>;
  subcategories: Array<{ value: string; count: number }>;
  rarities: Array<{ value: string; count: number }>;
  sizes: Array<{ value: string; count: number }>;
  traditions: Array<{ value: string; count: number }>;
  spellKinds: Array<{ value: string; count: number }>;
  sourceCategories: Array<{ value: SourceCategory; count: number }>;
  commonTraitsByCategory: Array<{ category: SearchCategory; traits: Array<{ value: string; count: number }> }>;
  commonDerivedTagsByCategory: Array<{ category: SearchCategory; tags: Array<{ value: string; count: number }> }>;
  derivedTagOntologyFamilies: DerivedTagOntologyFamily[];
  derivedTagOntologyTags: DerivedTagOntologyTag[];
  derivedTagCatalog: DerivedTagCatalogEntry[];
};

type RawValueCountRow = {
  value: string;
  count: number | bigint;
};

type RawCategoryValueCountRow = {
  category: string;
  value: string;
  count: number | bigint;
};

function parseValueCountRows(rows: RawValueCountRow[], context: string): Array<{ value: string; count: number }> {
  return rows.map((row) => ({
    value: row.value,
    count: toSqliteNumber(row.count, `${context} "${row.value}"`),
  }));
}

function parseCategoryValueCountRows(
  rows: RawCategoryValueCountRow[],
  context: string,
): Array<{ category: SearchCategory; value: string; count: number }> {
  return rows.map((row) => ({
    category: parseSearchCategoryValue(row.category, `${context} "${row.value}"`),
    value: row.value,
    count: toSqliteNumber(row.count, `${context} "${row.value}"`),
  }));
}

function parseCategoryCountRows(
  rows: RawValueCountRow[],
  context: string,
): Array<{ value: SearchCategory; count: number }> {
  return rows.map((row) => ({
    value: parseSearchCategoryValue(row.value, context),
    count: toSqliteNumber(row.count, `${context} "${row.value}"`),
  }));
}

function parseSubcategoryCountRows(
  rows: RawValueCountRow[],
  context: string,
): Array<{ value: SearchSubcategory; count: number }> {
  return rows.map((row) => ({
    value: parseSearchSubcategoryValue(row.value, context),
    count: toSqliteNumber(row.count, `${context} "${row.value}"`),
  }));
}

function parseSourceCategoryCountRows(
  rows: RawValueCountRow[],
  context: string,
): Array<{ value: SourceCategory; count: number }> {
  return rows.map((row) => ({
    value: parseSourceCategoryValue(row.value, context),
    count: toSqliteNumber(row.count, `${context} "${row.value}"`),
  }));
}

function getPackAliasValues(pack: PackInfo, value: string): boolean {
  const normalized = normalizeText(value);
  return normalized === normalizeText(pack.name) || normalized === normalizeText(pack.label);
}

export function getSearchVocabulary(
  db: DatabaseSync,
  options: { traitLimitPerCategory?: number } = {},
): SearchVocabularyResult {
  const traitLimit = Math.max(3, Math.min(options.traitLimitPerCategory ?? 12, 25));
  const categories = parseCategoryCountRows(
    db
      .prepare(
        `
        SELECT r.category AS value, COUNT(*) AS count
        FROM records r
        WHERE r.is_search_canonical = 1
        GROUP BY r.category
        ORDER BY count DESC, value ASC
      `,
      )
      .all() as RawValueCountRow[],
    "search vocabulary category",
  );
  const subcategories = parseSubcategoryCountRows(
    db
      .prepare(
        `
        SELECT r.subcategory AS value, COUNT(*) AS count
        FROM records r
        WHERE r.is_search_canonical = 1 AND r.subcategory IS NOT NULL AND r.subcategory <> ''
        GROUP BY r.subcategory
        ORDER BY count DESC, value ASC
      `,
      )
      .all() as RawValueCountRow[],
    "search vocabulary subcategory",
  );
  const sourceCategories = parseSourceCategoryCountRows(
    db
      .prepare(
        `
        SELECT r.source_category AS value, COUNT(*) AS count
        FROM records r
        WHERE r.is_search_canonical = 1
        GROUP BY r.source_category
        ORDER BY count DESC, value ASC
      `,
      )
      .all() as RawValueCountRow[],
    "search vocabulary source category",
  );
  const rarities = parseValueCountRows(
    db
      .prepare(
        `
        SELECT r.rarity AS value, COUNT(*) AS count
        FROM records r
        WHERE r.is_search_canonical = 1 AND r.rarity IS NOT NULL AND r.rarity <> ''
        GROUP BY r.rarity
        ORDER BY count DESC, value ASC
      `,
      )
      .all() as RawValueCountRow[],
    "search vocabulary rarity",
  );
  const sizes = parseValueCountRows(
    db
      .prepare(
        `
        SELECT a.size AS value, COUNT(*) AS count
        FROM actor_records a
        JOIN records r ON r.record_key = a.record_key
        WHERE r.is_search_canonical = 1 AND a.size IS NOT NULL AND a.size <> ''
        GROUP BY a.size
        ORDER BY count DESC, value ASC
      `,
      )
      .all() as RawValueCountRow[],
    "search vocabulary size",
  );
  const categoryTraitRows = parseCategoryValueCountRows(
    db
      .prepare(
        `
        SELECT r.category AS category, rt.trait AS value, COUNT(*) AS count
        FROM record_traits rt
        JOIN records r ON r.record_key = rt.record_key
        WHERE r.is_search_canonical = 1
        GROUP BY r.category, rt.trait
        ORDER BY r.category ASC, count DESC, value ASC
      `,
      )
      .all() as RawCategoryValueCountRow[],
    "search vocabulary trait",
  );
  const commonTraitsByCategory = (() => {
    const grouped = new Map<SearchCategory, Array<{ value: string; count: number }>>();
    for (const row of categoryTraitRows) {
      const bucket = grouped.get(row.category) ?? [];
      if (bucket.length < traitLimit) {
        bucket.push({ value: row.value, count: row.count });
        grouped.set(row.category, bucket);
      }
    }

    return [...grouped.entries()]
      .sort((left, right) => left[0].localeCompare(right[0]))
      .map(([category, traits]) => ({ category, traits }));
  })();
  const categoryDerivedTagRows = parseCategoryValueCountRows(
    db
      .prepare(
        `
        SELECT r.category AS category, rdt.tag AS value, COUNT(*) AS count
        FROM record_derived_tags rdt
        JOIN records r ON r.record_key = rdt.record_key
        WHERE r.is_search_canonical = 1
        GROUP BY r.category, rdt.tag
        ORDER BY r.category ASC, count DESC, value ASC
      `,
      )
      .all() as RawCategoryValueCountRow[],
    "search vocabulary derived tag",
  );
  const commonDerivedTagsByCategory = (() => {
    const grouped = new Map<SearchCategory, Array<{ value: string; count: number }>>();
    for (const row of categoryDerivedTagRows) {
      const bucket = grouped.get(row.category) ?? [];
      if (bucket.length < traitLimit) {
        bucket.push({ value: row.value, count: row.count });
        grouped.set(row.category, bucket);
      }
    }

    return [...grouped.entries()]
      .sort((left, right) => left[0].localeCompare(right[0]))
      .map(([category, tags]) => ({ category, tags }));
  })();

  const traditionCounts = new Map<string, number>();
  const traditionRows = db
    .prepare(
      `
      SELECT s.traditions_json AS traditionsJson
      FROM spell_records s
      JOIN records r ON r.record_key = s.record_key
      WHERE r.is_search_canonical = 1
    `,
    )
    .all() as Array<{ traditionsJson: string }>;
  for (const row of traditionRows) {
    const traditions = parseStringArrayJson(row.traditionsJson, "traditionsJson", "search vocabulary spell traditions");
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

  const spellKindCounts = new Map<string, number>();
  const spellKindRows = db
    .prepare(
      `
      SELECT s.spell_kinds_json AS spellKindsJson
      FROM spell_records s
      JOIN records r ON r.record_key = s.record_key
      WHERE r.is_search_canonical = 1
    `,
    )
    .all() as Array<{ spellKindsJson: string }>;
  for (const row of spellKindRows) {
    const spellKinds = parseStringArrayJson(row.spellKindsJson, "spellKindsJson", "search vocabulary spell kinds");
    for (const spellKind of spellKinds) {
      const normalized = normalizeText(spellKind);
      if (!normalized) {
        continue;
      }

      spellKindCounts.set(normalized, (spellKindCounts.get(normalized) ?? 0) + 1);
    }
  }
  const spellKinds = [...spellKindCounts.entries()]
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
    .map(([value, count]) => ({ value, count }));

  return {
    categories,
    subcategories,
    rarities,
    sizes,
    traditions,
    spellKinds,
    sourceCategories,
    commonTraitsByCategory,
    commonDerivedTagsByCategory,
    derivedTagOntologyFamilies: DERIVED_TAG_ONTOLOGY_FAMILIES,
    derivedTagOntologyTags: DERIVED_TAG_ONTOLOGY_TAGS,
    derivedTagCatalog: groupDerivedTagOntology({
      families: DERIVED_TAG_ONTOLOGY_FAMILIES,
      tags: DERIVED_TAG_ONTOLOGY_TAGS,
    }),
  };
}

export function getPack(packs: PackInfo[], packValue: string): PackInfo | undefined {
  return packs.find((pack) => getPackAliasValues(pack, packValue));
}

export function listPacks(packs: PackInfo[]): PackInfo[] {
  return packs;
}

export function listFilterValues(
  db: DatabaseSync,
  query: FilterValueQuery,
  filters: NormalizedSearchFilters,
): FilterValueResult {
  const { sql, params } = buildFilterValueQuery(query, filters);
  const values = (db.prepare(sql).all(...params) as Array<{ value: string; count: number | bigint }>).map((row) => ({
    value: row.value,
    count: toSqliteNumber(row.count, `filter values for ${query.field}`),
  })) as ValueCountRow[];
  return {
    field: query.field,
    values,
  };
}
