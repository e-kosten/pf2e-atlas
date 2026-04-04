import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import type { DerivedTagCatalogEntry, DerivedTagRule } from "../../src/types.js";
import { AFFLICTION_DERIVED_TAG_CATALOG } from "../../src/tags/catalog/affliction.js";
import { CREATURE_DERIVED_TAG_CATALOG } from "../../src/tags/catalog/creature.js";
import { EQUIPMENT_DERIVED_TAG_CATALOG } from "../../src/tags/catalog/equipment.js";
import { HAZARD_DERIVED_TAG_CATALOG } from "../../src/tags/catalog/hazard.js";
import { SPELL_DERIVED_TAG_CATALOG } from "../../src/tags/catalog/spell.js";
import { AFFLICTION_DERIVED_TAG_RULES } from "../../src/tags/rules/affliction.js";
import { CREATURE_DERIVED_TAG_RULES } from "../../src/tags/rules/creature.js";
import { EQUIPMENT_DERIVED_TAG_RULES } from "../../src/tags/rules/equipment.js";
import { HAZARD_DERIVED_TAG_RULES } from "../../src/tags/rules/hazard.js";
import { SPELL_DERIVED_TAG_RULES } from "../../src/tags/rules/spell.js";
import { cleanupCreatedRoots, createFixture } from "../helpers/pf2e-service-fixture.js";
import { loadTestService } from "../helpers/pf2e-fixture.js";

const RAW_DERIVED_TAG_CATALOG = [
  ...EQUIPMENT_DERIVED_TAG_CATALOG,
  ...SPELL_DERIVED_TAG_CATALOG,
  ...HAZARD_DERIVED_TAG_CATALOG,
  ...AFFLICTION_DERIVED_TAG_CATALOG,
  ...CREATURE_DERIVED_TAG_CATALOG,
];

const DERIVED_TAG_RULES = [
  ...EQUIPMENT_DERIVED_TAG_RULES,
  ...SPELL_DERIVED_TAG_RULES,
  ...HAZARD_DERIVED_TAG_RULES,
  ...AFFLICTION_DERIVED_TAG_RULES,
  ...CREATURE_DERIVED_TAG_RULES,
];

function buildCatalogTagRows(catalog: DerivedTagCatalogEntry[]) {
  return catalog.flatMap((entry) => entry.tags.map((tag) => ({
    category: entry.category,
    family: entry.family,
    value: tag.value,
    nativeOntologyPolicy: tag.nativeOntologyPolicy ?? "distinct_required",
  })));
}

function keyFor(category: string, value: string): string {
  return `${category}:${value}`;
}

function clauseHasNonTraitEvidence(clause: NonNullable<DerivedTagRule["anyOf"]>[number]): boolean {
  return Boolean(
    clause.textAny?.length
      || clause.textAll?.length
      || clause.textNear?.length
      || clause.referencesAny?.length
      || clause.referencesAll?.length
      || clause.referencesWhere?.length,
  );
}

describe("derived tag native ontology guard", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await cleanupCreatedRoots(createdRoots);
  });

  it("rejects distinct-required tags that only have native-ontology positive evidence", () => {
    const policyRows = buildCatalogTagRows(RAW_DERIVED_TAG_CATALOG);
    const violations: string[] = [];

    for (const row of policyRows) {
      if (row.nativeOntologyPolicy !== "distinct_required") {
        continue;
      }

      const rules = DERIVED_TAG_RULES.filter((rule) => rule.category === row.category && rule.tag === row.value);
      const positiveRules = rules.filter((rule) => (rule.anyOf?.length ?? 0) > 0 || (rule.allOf?.length ?? 0) > 0);
      if (positiveRules.length === 0) {
        continue;
      }

      const hasNonTraitEvidence = positiveRules.some((rule) => [
        ...(rule.anyOf ?? []),
        ...(rule.allOf ?? []),
      ].some((clause) => clauseHasNonTraitEvidence(clause)));

      if (!hasNonTraitEvidence) {
        violations.push(keyFor(row.category, row.value));
      }
    }

    expect(violations).toEqual([]);
  });

  it("rejects distinct-required tags that collapse to a single native trait in the fixture corpus", async () => {
    const fixture = await createFixture();
    createdRoots.push(fixture.root);
    const indexPath = path.join(fixture.root, ".cache", "pf2e-index.sqlite");
    const service = await loadTestService(fixture, { indexPath });
    service.close();

    const db = new DatabaseSync(indexPath, { readonly: true });
    try {
      const tagRows = db.prepare(`
        SELECT r.category AS category, rdt.tag AS tag, r.record_key AS recordKey
        FROM record_derived_tags rdt
        JOIN records r ON r.record_key = rdt.record_key
        WHERE r.is_search_canonical = 1
      `).all() as Array<{ category: string; tag: string; recordKey: string }>;

      const traitRows = db.prepare(`
        SELECT r.category AS category, rt.trait AS trait, r.record_key AS recordKey
        FROM record_traits rt
        JOIN records r ON r.record_key = rt.record_key
        WHERE r.is_search_canonical = 1
      `).all() as Array<{ category: string; trait: string; recordKey: string }>;

      const tagSets = new Map<string, Set<string>>();
      for (const row of tagRows) {
        const key = keyFor(row.category, row.tag);
        const bucket = tagSets.get(key) ?? new Set<string>();
        bucket.add(row.recordKey);
        tagSets.set(key, bucket);
      }

      const traitSets = new Map<string, Set<string>>();
      for (const row of traitRows) {
        const key = keyFor(row.category, row.trait);
        const bucket = traitSets.get(key) ?? new Set<string>();
        bucket.add(row.recordKey);
        traitSets.set(key, bucket);
      }

      const policyRows = buildCatalogTagRows(RAW_DERIVED_TAG_CATALOG);
      const violations: string[] = [];

      for (const row of policyRows) {
        if (row.nativeOntologyPolicy !== "distinct_required") {
          continue;
        }

        const tagSet = tagSets.get(keyFor(row.category, row.value));
        if (!tagSet || tagSet.size < 3) {
          continue;
        }

        for (const [traitKey, traitSet] of traitSets.entries()) {
          const [category, trait] = traitKey.split(":");
          if (category !== row.category || traitSet.size < 3) {
            continue;
          }

          let overlap = 0;
          for (const recordKey of tagSet) {
            if (traitSet.has(recordKey)) {
              overlap += 1;
            }
          }

          const precision = overlap / tagSet.size;
          const recall = overlap / traitSet.size;
          if (precision >= 0.98 && recall >= 0.98) {
            violations.push(`${keyFor(row.category, row.value)} ~= trait:${trait}`);
          }
        }
      }

      expect(violations).toEqual([]);
    } finally {
      db.close();
    }
  });
});
