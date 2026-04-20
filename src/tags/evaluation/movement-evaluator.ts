import { DatabaseSync } from "node:sqlite";

import { SearchCategory } from "../../domain/index.js";
import { parseSearchCategoryValue } from "../../data/sql-row-decoding.js";
import { normalizeDerivedTag } from "../runtime/shared.js";

type CategoryStatRow = {
  category: string;
  total: number | bigint;
  tagged: number | bigint;
};

type TagStatRow = {
  tag: string;
  count: number | bigint;
};

type TagRecordRow = {
  tag: string;
  recordKey: string;
  name: string;
};

export type DerivedTagMovementRecord = {
  recordKey: string;
  name: string;
};

export type DerivedTagCategoryMovement = {
  category: SearchCategory;
  baselineTotal: number;
  currentTotal: number;
  baselineTagged: number;
  currentTagged: number;
  baselineCoveragePercent: number;
  currentCoveragePercent: number;
  deltaTagged: number;
  deltaCoveragePoints: number;
};

export type DerivedTagTagMovement = {
  category: SearchCategory;
  tag: string;
  baselineCount: number;
  currentCount: number;
  baselineCoveragePercent: number;
  currentCoveragePercent: number;
  deltaCount: number;
  deltaCoveragePoints: number;
  gainedRecords: DerivedTagMovementRecord[];
  lostRecords: DerivedTagMovementRecord[];
};

export type DerivedTagMovementWarning =
  | {
      kind: "category_drop";
      category: SearchCategory;
      deltaCoveragePoints: number;
      message: string;
    }
  | {
      kind: "category_gain_below";
      category: SearchCategory;
      deltaCoveragePoints: number;
      minimumExpectedGain: number;
      message: string;
    }
  | {
      kind: "tag_drop";
      category: SearchCategory;
      tag: string;
      deltaCount: number;
      deltaCoveragePoints: number;
      message: string;
    }
  | {
      kind: "tag_gain_below";
      category: SearchCategory;
      tag: string;
      deltaCount: number;
      minimumExpectedGain: number;
      message: string;
    };

export type DerivedTagMovementEvaluation = {
  categories: DerivedTagCategoryMovement[];
  tags: DerivedTagTagMovement[];
  warnings: DerivedTagMovementWarning[];
};

export type DerivedTagMovementEvaluationOptions = {
  category?: SearchCategory;
  tags?: string[];
  limit?: number;
  sampleLimit?: number;
  warnCategoryGainBelowPoints?: number;
  warnCategoryDropPoints?: number;
  warnTagGainBelowCount?: number;
  warnTagDropCount?: number;
  warnTagDropPoints?: number;
};

export function evaluateDerivedTagMovement(
  baselineDb: DatabaseSync,
  currentDb: DatabaseSync,
  options: DerivedTagMovementEvaluationOptions = {},
): DerivedTagMovementEvaluation {
  if (options.tags && options.tags.length > 0 && !options.category) {
    throw new Error("Tag-specific movement evaluation requires a category scope.");
  }

  const baselineCategories = loadCategoryStats(baselineDb);
  const currentCategories = loadCategoryStats(currentDb);
  const categories = mergeCategoryStats(baselineCategories, currentCategories, options.category);
  const normalizedTags = normalizeTagList(options.tags);

  const categoryTotals = new Map<SearchCategory, { baselineTotal: number; currentTotal: number }>(
    categories.map((category) => [
      category.category,
      {
        baselineTotal: category.baselineTotal,
        currentTotal: category.currentTotal,
      },
    ]),
  );

  const tags = options.category
    ? mergeTagStats(
        loadTagStats(baselineDb, options.category, normalizedTags),
        loadTagStats(currentDb, options.category, normalizedTags),
        loadTagRecordStats(baselineDb, options.category, normalizedTags, options.sampleLimit),
        loadTagRecordStats(currentDb, options.category, normalizedTags, options.sampleLimit),
        options.category,
        categoryTotals.get(options.category) ?? { baselineTotal: 0, currentTotal: 0 },
        {
          ...options,
          tags: normalizedTags,
        },
      )
    : [];

  const warnings = collectWarnings(categories, tags, options);

  return {
    categories,
    tags,
    warnings,
  };
}

function normalizeTagList(tags?: string[]): string[] {
  return (tags ?? []).map((tag) => normalizeDerivedTag(tag));
}

function loadCategoryStats(db: DatabaseSync): Map<SearchCategory, { total: number; tagged: number }> {
  const rows = db
    .prepare(
      `
    SELECT
      r.category AS category,
      COUNT(*) AS total,
      SUM(CASE WHEN EXISTS (
        SELECT 1
        FROM record_derived_tags d
        WHERE d.record_key = r.record_key
      ) THEN 1 ELSE 0 END) AS tagged
    FROM records r
    WHERE r.is_search_canonical = 1
    GROUP BY r.category
  `,
    )
    .all() as CategoryStatRow[];

  return new Map(
    rows.map((row) => [
      parseSearchCategoryValue(row.category, `derived tag movement category row "${row.category}"`),
      {
        total: toNumber(row.total),
        tagged: toNumber(row.tagged),
      },
    ]),
  );
}

function mergeCategoryStats(
  baseline: Map<SearchCategory, { total: number; tagged: number }>,
  current: Map<SearchCategory, { total: number; tagged: number }>,
  categoryFilter?: SearchCategory,
): DerivedTagCategoryMovement[] {
  const categories = categoryFilter
    ? [categoryFilter]
    : Array.from(new Set([...baseline.keys(), ...current.keys()])).sort();

  return categories.map((category) => {
    const baselineStats = baseline.get(category) ?? { total: 0, tagged: 0 };
    const currentStats = current.get(category) ?? { total: 0, tagged: 0 };
    const baselineCoveragePercent = calculatePercent(baselineStats.tagged, baselineStats.total);
    const currentCoveragePercent = calculatePercent(currentStats.tagged, currentStats.total);

    return {
      category,
      baselineTotal: baselineStats.total,
      currentTotal: currentStats.total,
      baselineTagged: baselineStats.tagged,
      currentTagged: currentStats.tagged,
      baselineCoveragePercent,
      currentCoveragePercent,
      deltaTagged: currentStats.tagged - baselineStats.tagged,
      deltaCoveragePoints: currentCoveragePercent - baselineCoveragePercent,
    };
  });
}

function loadTagStats(db: DatabaseSync, category: SearchCategory, tags?: string[]): Map<string, number> {
  const sql = [
    "SELECT",
    "  d.tag AS tag,",
    "  COUNT(DISTINCT r.record_key) AS count",
    "FROM records r",
    "JOIN record_derived_tags d ON d.record_key = r.record_key",
    "WHERE r.is_search_canonical = 1",
    "  AND r.category = ?",
  ];
  const params: Array<string | number> = [category];

  if ((tags?.length ?? 0) > 0) {
    const placeholders = tags!.map(() => "?").join(", ");
    sql.push(`  AND d.tag IN (${placeholders})`);
    params.push(...tags!);
  }

  sql.push("GROUP BY d.tag");

  const rows = db.prepare(sql.join("\n")).all(...params) as TagStatRow[];
  return new Map(rows.map((row) => [row.tag, toNumber(row.count)]));
}

function loadTagRecordStats(
  db: DatabaseSync,
  category: SearchCategory,
  tags: string[],
  sampleLimit = 0,
): Map<string, Map<string, string>> {
  if (sampleLimit <= 0) {
    return new Map();
  }

  const sql = [
    "SELECT DISTINCT",
    "  d.tag AS tag,",
    "  r.record_key AS recordKey,",
    "  r.name AS name",
    "FROM records r",
    "JOIN record_derived_tags d ON d.record_key = r.record_key",
    "WHERE r.is_search_canonical = 1",
    "  AND r.category = ?",
  ];
  const params: Array<string | number> = [category];

  if (tags.length > 0) {
    const placeholders = tags.map(() => "?").join(", ");
    sql.push(`  AND d.tag IN (${placeholders})`);
    params.push(...tags);
  }

  const rows = db.prepare(sql.join("\n")).all(...params) as TagRecordRow[];
  const recordsByTag = new Map<string, Map<string, string>>();
  for (const row of rows) {
    const current = recordsByTag.get(row.tag) ?? new Map<string, string>();
    current.set(row.recordKey, row.name);
    recordsByTag.set(row.tag, current);
  }

  return recordsByTag;
}

function mergeTagStats(
  baseline: Map<string, number>,
  current: Map<string, number>,
  baselineRecords: Map<string, Map<string, string>>,
  currentRecords: Map<string, Map<string, string>>,
  category: SearchCategory,
  totals: { baselineTotal: number; currentTotal: number },
  options: DerivedTagMovementEvaluationOptions,
): DerivedTagTagMovement[] {
  const explicitTags = options.tags ?? [];
  const tags = explicitTags.length > 0 ? explicitTags : Array.from(new Set([...baseline.keys(), ...current.keys()]));

  const merged = tags
    .map((tag) => {
      const baselineCount = baseline.get(tag) ?? 0;
      const currentCount = current.get(tag) ?? 0;
      const baselineCoveragePercent = calculatePercent(baselineCount, totals.baselineTotal);
      const currentCoveragePercent = calculatePercent(currentCount, totals.currentTotal);

      return {
        category,
        tag,
        baselineCount,
        currentCount,
        baselineCoveragePercent,
        currentCoveragePercent,
        deltaCount: currentCount - baselineCount,
        deltaCoveragePoints: currentCoveragePercent - baselineCoveragePercent,
        gainedRecords: diffRecords(currentRecords.get(tag), baselineRecords.get(tag), options.sampleLimit),
        lostRecords: diffRecords(baselineRecords.get(tag), currentRecords.get(tag), options.sampleLimit),
      };
    })
    .filter((movement) => explicitTags.length > 0 || movement.deltaCount !== 0);

  if (explicitTags.length > 0) {
    const explicitOrder = new Map(explicitTags.map((tag, index) => [tag, index]));
    return merged.sort((left, right) => {
      const leftOrder = explicitOrder.get(left.tag) ?? Number.MAX_SAFE_INTEGER;
      const rightOrder = explicitOrder.get(right.tag) ?? Number.MAX_SAFE_INTEGER;
      return leftOrder - rightOrder;
    });
  }

  const limit = Math.max(1, Math.min(options.limit ?? 20, 200));
  return merged
    .sort((left, right) => {
      const deltaDifference = Math.abs(right.deltaCount) - Math.abs(left.deltaCount);
      if (deltaDifference !== 0) {
        return deltaDifference;
      }

      const coverageDifference = Math.abs(right.deltaCoveragePoints) - Math.abs(left.deltaCoveragePoints);
      if (coverageDifference !== 0) {
        return coverageDifference;
      }

      return left.tag.localeCompare(right.tag);
    })
    .slice(0, limit);
}

function diffRecords(
  source: Map<string, string> | undefined,
  comparison: Map<string, string> | undefined,
  limit = 0,
): DerivedTagMovementRecord[] {
  if (limit <= 0 || !source || source.size === 0) {
    return [];
  }

  const results: DerivedTagMovementRecord[] = [];
  for (const [recordKey, name] of source.entries()) {
    if (comparison?.has(recordKey)) {
      continue;
    }

    results.push({ recordKey, name });
  }

  return results
    .sort((left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey))
    .slice(0, limit);
}

function collectWarnings(
  categories: DerivedTagCategoryMovement[],
  tags: DerivedTagTagMovement[],
  options: DerivedTagMovementEvaluationOptions,
): DerivedTagMovementWarning[] {
  const warnings: DerivedTagMovementWarning[] = [];

  if (options.warnCategoryDropPoints !== undefined) {
    for (const category of categories) {
      if (category.deltaCoveragePoints <= -options.warnCategoryDropPoints) {
        warnings.push({
          kind: "category_drop",
          category: category.category,
          deltaCoveragePoints: category.deltaCoveragePoints,
          message: `${category.category} dropped ${formatSigned(category.deltaCoveragePoints, 1)} coverage points.`,
        });
      }
    }
  }

  if (options.warnCategoryGainBelowPoints !== undefined) {
    for (const category of categories) {
      if (category.deltaCoveragePoints >= options.warnCategoryGainBelowPoints) {
        continue;
      }

      warnings.push({
        kind: "category_gain_below",
        category: category.category,
        deltaCoveragePoints: category.deltaCoveragePoints,
        minimumExpectedGain: options.warnCategoryGainBelowPoints,
        message: `${category.category} net gain was only ${formatSigned(category.deltaCoveragePoints, 1)} coverage points; expected at least +${options.warnCategoryGainBelowPoints.toFixed(1)}.`,
      });
    }
  }

  for (const tag of tags) {
    const gainTriggered = options.warnTagGainBelowCount !== undefined && tag.deltaCount < options.warnTagGainBelowCount;
    const countTriggered = options.warnTagDropCount !== undefined && tag.deltaCount <= -options.warnTagDropCount;
    const pointsTriggered =
      options.warnTagDropPoints !== undefined && tag.deltaCoveragePoints <= -options.warnTagDropPoints;

    if (gainTriggered) {
      warnings.push({
        kind: "tag_gain_below",
        category: tag.category,
        tag: tag.tag,
        deltaCount: tag.deltaCount,
        minimumExpectedGain: options.warnTagGainBelowCount!,
        message: `${tag.category}/${tag.tag} net gain was only ${formatSigned(tag.deltaCount, 0)} records; expected at least +${options.warnTagGainBelowCount}.`,
      });
    }

    if (countTriggered || pointsTriggered) {
      warnings.push({
        kind: "tag_drop",
        category: tag.category,
        tag: tag.tag,
        deltaCount: tag.deltaCount,
        deltaCoveragePoints: tag.deltaCoveragePoints,
        message: `${tag.category}/${tag.tag} dropped ${formatSigned(tag.deltaCount, 0)} records and ${formatSigned(tag.deltaCoveragePoints, 1)} coverage points.`,
      });
    }
  }

  return warnings;
}

function calculatePercent(numerator: number, denominator: number): number {
  if (denominator <= 0) {
    return 0;
  }

  return (numerator / denominator) * 100;
}

function toNumber(value: number | bigint): number {
  return typeof value === "bigint" ? Number(value) : value;
}

function formatSigned(value: number, fractionDigits: number): string {
  const formatted = value.toFixed(fractionDigits);
  return value > 0 ? `+${formatted}` : formatted;
}
