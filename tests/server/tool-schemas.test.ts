import { describe, expect, it } from "vitest";

import {
  browseSortSchema,
  CATEGORY_HINT_DESCRIPTION,
  listRecordsToolInputSchema,
  linksToModeSchema,
  SCOPES_HINT_DESCRIPTION,
  SUBCATEGORY_HINT_DESCRIPTION,
  filterValueFieldSchema,
  recordKeyArraySchema,
  searchFilterSchema,
  searchCategorySchema,
  searchProfileSchema,
  searchScopeSchema,
  searchToolInputSchema,
  searchSubcategorySchema,
} from "../../src/server/tool-schemas.js";

describe("tool schemas", () => {
  it("advertises the user-facing search profiles", () => {
    expect(searchProfileSchema.safeParse("lexical").success).toBe(true);
    expect(searchProfileSchema.safeParse("balanced").success).toBe(true);
    expect(searchProfileSchema.safeParse("concept").success).toBe(true);
    expect(searchProfileSchema.safeParse("semantic_only").success).toBe(false);
  });

  it("validates exact link filter inputs", () => {
    expect(linksToModeSchema.safeParse("any").success).toBe(true);
    expect(linksToModeSchema.safeParse("all").success).toBe(true);
    expect(linksToModeSchema.safeParse("exclude").success).toBe(false);
    expect(recordKeyArraySchema.safeParse(["actionspf2e:action-track-1"]).success).toBe(true);
    expect(recordKeyArraySchema.safeParse(["  actions:action-refocus-1  "])).toMatchObject({
      success: true,
      data: ["actions:action-refocus-1"],
    });
    expect(recordKeyArraySchema.safeParse([]).success).toBe(false);
    expect(recordKeyArraySchema.safeParse(["   "]).success).toBe(false);
  });

  it("accepts the supported filter-value fields", () => {
    expect(filterValueFieldSchema.safeParse("traits")).toMatchObject({ success: true, data: "traits" });
    expect(filterValueFieldSchema.safeParse("derivedTags")).toMatchObject({ success: true, data: "derivedTags" });
    expect(filterValueFieldSchema.safeParse("actorMetrics")).toMatchObject({ success: true, data: "actorMetrics" });
    expect(filterValueFieldSchema.safeParse("itemMetrics")).toMatchObject({ success: true, data: "itemMetrics" });
    expect(filterValueFieldSchema.safeParse("sourceCategory")).toMatchObject({ success: true, data: "sourceCategory" });
    expect(filterValueFieldSchema.safeParse("publicationTitle")).toMatchObject({
      success: true,
      data: "publicationTitle",
    });
    expect(filterValueFieldSchema.safeParse("weaponGroup")).toMatchObject({ success: true, data: "weaponGroup" });
    expect(filterValueFieldSchema.safeParse("actionCost")).toMatchObject({ success: true, data: "actionCost" });
    expect(filterValueFieldSchema.safeParse("hands")).toMatchObject({ success: true, data: "hands" });
    expect(filterValueFieldSchema.safeParse("saveType")).toMatchObject({ success: true, data: "saveType" });
    expect(filterValueFieldSchema.safeParse("areaType")).toMatchObject({ success: true, data: "areaType" });
    expect(filterValueFieldSchema.safeParse("durationUnit")).toMatchObject({ success: true, data: "durationUnit" });
    expect(filterValueFieldSchema.safeParse("rangeValue")).toMatchObject({ success: true, data: "rangeValue" });
    expect(filterValueFieldSchema.safeParse("areaValue")).toMatchObject({ success: true, data: "areaValue" });
    expect(filterValueFieldSchema.safeParse("sustained")).toMatchObject({ success: true, data: "sustained" });
    expect(filterValueFieldSchema.safeParse("basicSave")).toMatchObject({ success: true, data: "basicSave" });
    expect(filterValueFieldSchema.safeParse("senses")).toMatchObject({ success: true, data: "senses" });
    expect(filterValueFieldSchema.safeParse("disableSkills")).toMatchObject({ success: true, data: "disableSkills" });
    expect(filterValueFieldSchema.safeParse("isComplex")).toMatchObject({ success: true, data: "isComplex" });
    expect(filterValueFieldSchema.safeParse("packs")).toMatchObject({ success: true, data: "packs" });
    expect(filterValueFieldSchema.safeParse("foo").success).toBe(false);
  });

  it("accepts canonical filter trees and rejects legacy flat filter payloads", () => {
    expect(
      searchFilterSchema.safeParse({
        kind: "allOf",
        children: [
          {
            kind: "scope",
            category: "creature",
            subcategory: { kind: "any" },
          },
          {
            kind: "metadataPredicate",
            predicate: {
              field: "traits",
              op: "includes",
              value: "undead",
            },
          },
        ],
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "level",
        match: {
          kind: "gt",
          value: 4,
        },
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "actionCost",
        match: {
          kind: "lt",
          value: 2,
        },
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "linkedFrom",
        source: "actions:action-refocus-1",
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        field: "traits",
        op: "includesAny",
        values: ["undead"],
      }).success,
    ).toBe(false);
  });

  it("rejects legacy flat root filter fields on list and search tool inputs", () => {
    expect(browseSortSchema.safeParse({ kind: "alphabetical" }).success).toBe(true);
    expect(browseSortSchema.safeParse({ kind: "random", seed: 7 }).success).toBe(true);
    expect(browseSortSchema.safeParse({ kind: "ranked" }).success).toBe(false);

    expect(
      listRecordsToolInputSchema.safeParse({
        mode: "browse",
        filter: {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
        sort: { kind: "alphabetical" },
        limit: 20,
      }).success,
    ).toBe(true);

    expect(
      listRecordsToolInputSchema.safeParse({
        category: "creature",
        limit: 20,
      }).success,
    ).toBe(false);

    expect(
      listRecordsToolInputSchema.safeParse({
        filter: {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
      }).success,
    ).toBe(false);

    expect(
      searchToolInputSchema.safeParse({
        mode: "search",
        search: {
          query: "ghost sailor ship",
        },
        filter: {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
      }).success,
    ).toBe(true);

    expect(
      searchToolInputSchema.safeParse({
        mode: "search",
        search: {
          query: "ghost sailor ship",
          profile: "balanced",
          exclude: "harbor",
        },
      }).success,
    ).toBe(true);

    expect(
      searchToolInputSchema.safeParse({
        mode: "search",
        filter: {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
      }).success,
    ).toBe(false);

    expect(
      searchToolInputSchema.safeParse({
        mode: "search",
        query: "ghost sailor ship",
        category: "creature",
        levelMin: 1,
      }).success,
    ).toBe(false);

    expect(
      searchToolInputSchema.safeParse({
        mode: "search",
        search: {
          query: "ghost sailor ship",
        },
        searchProfile: "balanced",
      }).success,
    ).toBe(false);

    expect(
      searchToolInputSchema.safeParse({
        mode: "search",
        search: {
          query: "   ",
        },
      }).success,
    ).toBe(false);
  });

  it("validates canonical metadata and metric filter leaves", () => {
    expect(
      searchFilterSchema.safeParse({
        kind: "metadataPredicate",
        predicate: {
          field: "traits",
          op: "includes",
          value: "undead",
        },
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "metadataPredicate",
        predicate: {
          field: "publicationTitle",
          op: "contains",
          value: "Player Core",
        },
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "metadataPredicate",
        predicate: {
          field: "hands",
          op: "between",
          min: 1,
          max: 2,
        },
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "metric",
        metric: "ability.int.mod",
        op: "gte",
        value: 4,
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "metric",
        metric: "save.best",
        op: "eq",
        value: "will",
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "metricCompare",
        leftMetric: "ability.int.mod",
        op: "gt",
        rightMetric: "ability.cha.mod",
      }).success,
    ).toBe(true);

    expect(
      searchFilterSchema.safeParse({
        kind: "metadataPredicate",
        predicate: {
          field: "traits",
          op: "eq",
          value: "undead",
        },
      }).success,
    ).toBe(false);

    expect(
      searchFilterSchema.safeParse({
        kind: "metadataPredicate",
        predicate: {
          field: "publicationTitle",
          op: "eq",
          value: "Player Core",
        },
      }).success,
    ).toBe(true);
  });

  it("accepts the canonical search category labels", () => {
    expect(searchCategorySchema.safeParse("feat")).toMatchObject({ success: true, data: "feat" });
    expect(searchCategorySchema.safeParse("rule")).toMatchObject({ success: true, data: "rule" });
    expect(searchCategorySchema.safeParse("characterCreation").success).toBe(true);
  });

  it("normalizes legacy plural category aliases to canonical singular values", () => {
    expect(searchCategorySchema.safeParse("feats")).toMatchObject({ success: true, data: "feat" });
    expect(searchCategorySchema.safeParse("rules")).toMatchObject({ success: true, data: "rule" });
    expect(searchCategorySchema.safeParse("spells")).toMatchObject({ success: true, data: "spell" });
  });

  it("suggests the parent category when a subcategory is passed as category", () => {
    const result = searchCategorySchema.safeParse("action");
    expect(result.success).toBe(false);
    expect(result.error.issues[0]?.message).toBe(
      'Unknown top-level category "action". Try category:"rule", subcategory:"action".',
    );
  });

  it("falls back to the valid category list for unknown values", () => {
    const result = searchCategorySchema.safeParse("foo");
    expect(result.success).toBe(false);
    expect(result.error.issues[0]?.message).toBe(
      'Unknown top-level category "foo". Valid categories: equipment, feat, creature, hazard, affliction, rule, spell, characterCreation, lore.',
    );
  });

  it("normalizes legacy plural subcategory aliases to canonical singular values", () => {
    expect(searchSubcategorySchema.safeParse("actions")).toMatchObject({ success: true, data: "action" });
    expect(searchSubcategorySchema.safeParse("campaign")).toMatchObject({ success: true, data: "campaignFeature" });
  });

  it("validates scoped multi-category filters", () => {
    const result = searchScopeSchema.safeParse({
      category: "feats",
      subcategories: ["archetypes"],
    });
    expect(result).toMatchObject({
      success: true,
      data: {
        category: "feat",
        subcategories: ["archetype"],
      },
    });
  });

  it("keeps category and subcategory guidance concise and shared", () => {
    expect(CATEGORY_HINT_DESCRIPTION).toBe(
      "Optional top-level category hint. Canonical values: equipment, feat, creature, hazard, affliction, rule, spell, characterCreation, lore. Legacy plural aliases are also accepted.",
    );
    expect(SUBCATEGORY_HINT_DESCRIPTION).toContain("Usually leave unset");
    expect(SCOPES_HINT_DESCRIPTION).toContain("multi-family");
  });
});
