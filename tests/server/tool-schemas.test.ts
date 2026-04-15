import { describe, expect, it } from "vitest";

import {
  CATEGORY_HINT_DESCRIPTION,
  linksToModeSchema,
  SCOPES_HINT_DESCRIPTION,
  SUBCATEGORY_HINT_DESCRIPTION,
  filterValueFieldSchema,
  metadataFilterSchema,
  recordKeyArraySchema,
  searchCategorySchema,
  searchProfileSchema,
  searchScopeSchema,
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
    expect(filterValueFieldSchema.safeParse("publicationTitle")).toMatchObject({ success: true, data: "publicationTitle" });
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

  it("validates grouped metadata filter predicates", () => {
    expect(metadataFilterSchema.safeParse({
      and: [
        { field: "traits", op: "includesAny", values: ["undead"] },
        { field: "sourceCategory", op: "eq", value: "core" },
      ],
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "publicationTitle",
      op: "contains",
      value: "Player Core",
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "hands",
      op: "between",
      min: 1,
      max: 2,
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "saveType",
      op: "eq",
      value: "reflex",
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "areaType",
      op: "in",
      values: ["burst", "cone"],
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "durationUnit",
      op: "eq",
      value: "minute",
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "rangeText",
      op: "contains",
      value: "feet",
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "sustained",
      op: "eq",
      value: true,
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "senses",
      op: "includesAny",
      values: ["darkvision"],
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "disableSkills",
      op: "includesAny",
      values: ["thievery"],
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "actorMetric",
      metric: "ability.int.mod",
      op: ">=",
      value: 4,
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "actorMetric",
      metric: "save.best",
      op: "==",
      value: "will",
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "actorMetricCompare",
      leftMetric: "ability.int.mod",
      op: ">",
      rightMetric: "ability.cha.mod",
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "itemMetric",
      metric: "weapon.reload",
      op: "==",
      value: 1,
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "itemMetricCompare",
      leftMetric: "shield.hp",
      op: ">",
      rightMetric: "shield.bt",
    }).success).toBe(true);

    expect(metadataFilterSchema.safeParse({
      field: "traits",
      op: "eq",
      value: "undead",
    }).success).toBe(false);
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
    expect(result.error.issues[0]?.message).toBe('Unknown top-level category "action". Try category:"rule", subcategory:"action".');
  });

  it("falls back to the valid category list for unknown values", () => {
    const result = searchCategorySchema.safeParse("foo");
    expect(result.success).toBe(false);
    expect(result.error.issues[0]?.message).toBe(
      "Unknown top-level category \"foo\". Valid categories: equipment, feat, creature, hazard, affliction, rule, spell, characterCreation, lore.",
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
