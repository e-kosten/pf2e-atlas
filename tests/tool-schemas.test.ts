import { describe, expect, it } from "vitest";

import {
  CATEGORY_HINT_DESCRIPTION,
  SCOPES_HINT_DESCRIPTION,
  SUBCATEGORY_HINT_DESCRIPTION,
  searchCategorySchema,
  searchProfileSchema,
  searchScopeSchema,
  searchSubcategorySchema,
} from "../src/tool-schemas.js";

describe("tool schemas", () => {
  it("advertises the user-facing search profiles", () => {
    expect(searchProfileSchema.safeParse("lexical").success).toBe(true);
    expect(searchProfileSchema.safeParse("balanced").success).toBe(true);
    expect(searchProfileSchema.safeParse("concept").success).toBe(true);
    expect(searchProfileSchema.safeParse("semantic_only").success).toBe(false);
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
