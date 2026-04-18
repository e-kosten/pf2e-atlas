#!/usr/bin/env node
import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import { AFFLICTION_DERIVED_TAG_CATALOG } from "../../../../src/tags/catalog/affliction.ts";
import { CREATURE_DERIVED_TAG_CATALOG } from "../../../../src/tags/catalog/creature.ts";
import { EQUIPMENT_DERIVED_TAG_CATALOG } from "../../../../src/tags/catalog/equipment.ts";
import { HAZARD_DERIVED_TAG_CATALOG } from "../../../../src/tags/catalog/hazard.ts";
import { SPELL_DERIVED_TAG_CATALOG } from "../../../../src/tags/catalog/spell.ts";
import type { SearchCategory } from "../../../../src/domain/search-types.ts";

type CoveredCategory = Extract<SearchCategory, "affliction" | "creature" | "equipment" | "hazard" | "spell">;

type DenseRecordRow = {
  recordKey: string;
  category: CoveredCategory;
  name: string;
  tagCount: number;
};

type TagRow = {
  tag: string;
};

type BaselineRow = {
  taggedRecords: number;
  avgTags: number;
  maxTags: number;
};

type Options = {
  indexPath: string;
  category?: CoveredCategory;
  name?: string;
  minTags?: number;
  limit: number;
  showTags: boolean;
  json: boolean;
};

const COVERED_CATEGORIES: CoveredCategory[] = ["affliction", "creature", "equipment", "hazard", "spell"];

const DEFAULT_MIN_TAGS: Record<CoveredCategory, number> = {
  affliction: 4,
  creature: 6,
  equipment: 5,
  hazard: 4,
  spell: 6,
};

const TAG_TO_FAMILY = new Map<string, string>();

for (const entry of [
  ...AFFLICTION_DERIVED_TAG_CATALOG,
  ...CREATURE_DERIVED_TAG_CATALOG,
  ...EQUIPMENT_DERIVED_TAG_CATALOG,
  ...HAZARD_DERIVED_TAG_CATALOG,
  ...SPELL_DERIVED_TAG_CATALOG,
]) {
  for (const tag of entry.tags) {
    TAG_TO_FAMILY.set(`${entry.category}:${tag.value}`, entry.family);
  }
}

function parseArgs(argv: string[]): Options {
  let indexPath = path.join(process.cwd(), ".cache", "pf2e-index.sqlite");
  let category: CoveredCategory | undefined;
  let name: string | undefined;
  let minTags: number | undefined;
  let limit = 20;
  let showTags = false;
  let json = false;

  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];
    switch (token) {
      case "--index":
        indexPath = next;
        index += 1;
        break;
      case "--category":
        if (!next || !COVERED_CATEGORIES.includes(next as CoveredCategory)) {
          throw new Error(`--category must be one of: ${COVERED_CATEGORIES.join(", ")}`);
        }
        category = next as CoveredCategory;
        index += 1;
        break;
      case "--name":
        name = next;
        index += 1;
        break;
      case "--min-tags":
        minTags = Number(next);
        index += 1;
        break;
      case "--limit":
        limit = Number(next);
        index += 1;
        break;
      case "--show-tags":
        showTags = true;
        break;
      case "--json":
        json = true;
        break;
      default:
        throw new Error(`Unknown argument: ${token}`);
    }
  }

  if (!Number.isFinite(limit) || limit <= 0) {
    throw new Error("--limit must be a positive number");
  }

  if (minTags !== undefined && (!Number.isFinite(minTags) || minTags <= 0)) {
    throw new Error("--min-tags must be a positive number");
  }

  return { indexPath, category, name, minTags, limit, showTags, json };
}

function getThreshold(options: Options): number {
  if (options.minTags !== undefined) {
    return options.minTags;
  }
  if (options.category) {
    return DEFAULT_MIN_TAGS[options.category];
  }
  return 5;
}

function getBaselines(
  db: DatabaseSync,
  category?: CoveredCategory,
): Array<{ category: CoveredCategory; taggedRecords: number; avgTags: number; maxTags: number }> {
  const categories = category ? [category] : COVERED_CATEGORIES;
  const statement = db.prepare(`
    WITH per_record AS (
      SELECT COUNT(*) AS tag_count
      FROM records r
      JOIN record_derived_tags rdt ON r.record_key = rdt.record_key
      WHERE r.is_search_canonical = 1 AND r.category = ?
      GROUP BY r.record_key
    )
    SELECT
      COUNT(*) AS taggedRecords,
      COALESCE(ROUND(AVG(tag_count), 3), 0) AS avgTags,
      COALESCE(MAX(tag_count), 0) AS maxTags
    FROM per_record
  `);

  return categories.map((entry) => {
    const row = statement.get(entry) as BaselineRow;
    return {
      category: entry,
      taggedRecords: row.taggedRecords,
      avgTags: row.avgTags,
      maxTags: row.maxTags,
    };
  });
}

function getDenseRecords(db: DatabaseSync, options: Options, threshold: number): DenseRecordRow[] {
  const clauses = [
    "r.is_search_canonical = 1",
    `r.category IN (${(options.category ? [options.category] : COVERED_CATEGORIES).map(() => "?").join(", ")})`,
  ];
  const params: Array<string | number> = options.category ? [options.category] : [...COVERED_CATEGORIES];

  if (options.name) {
    clauses.push("r.name = ?");
    params.push(options.name);
  }

  const statement = db.prepare(`
    SELECT
      r.record_key AS recordKey,
      r.category AS category,
      r.name AS name,
      COUNT(*) AS tagCount
    FROM records r
    JOIN record_derived_tags rdt ON r.record_key = rdt.record_key
    WHERE ${clauses.join(" AND ")}
    GROUP BY r.record_key, r.category, r.name
    HAVING COUNT(*) >= ?
    ORDER BY tagCount DESC, r.category ASC, r.name ASC
    LIMIT ?
  `);

  return statement.all(...params, threshold, options.limit) as DenseRecordRow[];
}

function getTagsForRecord(db: DatabaseSync, recordKey: string): string[] {
  return (
    db
      .prepare(
        `
    SELECT tag
    FROM record_derived_tags
    WHERE record_key = ?
    ORDER BY tag ASC
  `,
      )
      .all(recordKey) as TagRow[]
  ).map((row) => row.tag);
}

function summarizeFamilies(category: CoveredCategory, tags: string[]): Array<{ family: string; count: number }> {
  const counts = new Map<string, number>();
  for (const tag of tags) {
    const family = TAG_TO_FAMILY.get(`${category}:${tag}`) ?? "unknown";
    counts.set(family, (counts.get(family) ?? 0) + 1);
  }
  return [...counts.entries()]
    .map(([family, count]) => ({ family, count }))
    .sort((left, right) => right.count - left.count || left.family.localeCompare(right.family));
}

function printHumanReadable(
  baselines: Array<{ category: CoveredCategory; taggedRecords: number; avgTags: number; maxTags: number }>,
  rows: Array<DenseRecordRow & { tags: string[]; familyCounts: Array<{ family: string; count: number }> }>,
  threshold: number,
  showTags: boolean,
): void {
  console.log(`Dense-tag review queue (threshold: ${threshold}+ tags)`);
  console.log("");
  console.log("Category baselines:");
  for (const baseline of baselines) {
    console.log(
      `- ${baseline.category}: ${baseline.taggedRecords} tagged, avg ${baseline.avgTags.toFixed(3)}, max ${baseline.maxTags}`,
    );
  }

  console.log("");
  if (rows.length === 0) {
    console.log("No matching records.");
    return;
  }

  console.log("Candidates:");
  for (const row of rows) {
    const repeatedFamilies = row.familyCounts.filter((entry) => entry.count > 1);
    const familySummary =
      repeatedFamilies.length > 0
        ? repeatedFamilies.map((entry) => `${entry.family}=${entry.count}`).join(", ")
        : row.familyCounts.map((entry) => `${entry.family}=${entry.count}`).join(", ");
    console.log(`- [${row.category}] ${row.name} (${row.tagCount} tags)`);
    console.log(`  ${row.recordKey}`);
    console.log(`  families: ${familySummary}`);
    if (showTags) {
      console.log(`  tags: ${row.tags.join(", ")}`);
    }
  }
}

function main(): void {
  const options = parseArgs(process.argv.slice(2));
  const threshold = options.name ? Math.max(1, options.minTags ?? 1) : getThreshold(options);
  const db = new DatabaseSync(options.indexPath, { readonly: true });

  try {
    const baselines = getBaselines(db, options.category);
    const denseRecords = getDenseRecords(db, options, threshold).map((row) => {
      const tags = getTagsForRecord(db, row.recordKey);
      return {
        ...row,
        tags,
        familyCounts: summarizeFamilies(row.category, tags),
      };
    });

    if (options.json) {
      console.log(
        JSON.stringify(
          {
            threshold,
            baselines,
            records: denseRecords,
          },
          null,
          2,
        ),
      );
      return;
    }

    printHumanReadable(baselines, denseRecords, threshold, options.showTags);
  } finally {
    db.close();
  }
}

main();
