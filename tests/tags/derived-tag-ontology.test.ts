import { describe, expect, it } from "vitest";

import { groupDerivedTagOntology } from "../../src/tags/catalog-utils.js";
import { CREATURE_DERIVED_TAG_ONTOLOGY } from "../../src/tags/ontology/creature.js";
import { flattenDerivedTagAuthoredCategoryOntology } from "../../src/tags/ontology/utils.js";
import {
  DERIVED_TAG_ONTOLOGY_FAMILIES,
  DERIVED_TAG_ONTOLOGY_TAGS,
} from "../../src/tags/index.js";

describe("derived tag ontology", () => {
  it("publishes unique category-scoped families and tags with assignment modes", () => {
    const familiesByCategory = new Map<string, Set<string>>();
    const tagsByCategory = new Map<string, Map<string, string>>();

    for (const family of DERIVED_TAG_ONTOLOGY_FAMILIES) {
      const categoryFamilies = familiesByCategory.get(family.category) ?? new Set<string>();
      expect(categoryFamilies.has(family.family)).toBe(false);
      categoryFamilies.add(family.family);
      familiesByCategory.set(family.category, categoryFamilies);
    }

    for (const tag of DERIVED_TAG_ONTOLOGY_TAGS) {
      expect(tag.assignmentMode).toBeDefined();
      const categoryTags = tagsByCategory.get(tag.category) ?? new Map<string, string>();
      const existingFamily = categoryTags.get(tag.tag);
      if (existingFamily) {
        expect(existingFamily).toBe(tag.family);
      } else {
        categoryTags.set(tag.tag, tag.family);
      }
      tagsByCategory.set(tag.category, categoryTags);
    }

    for (const tag of DERIVED_TAG_ONTOLOGY_TAGS) {
      const categoryTags = tagsByCategory.get(tag.category) ?? new Map<string, string>();
      for (const adjacentTag of tag.adjacentTags ?? []) {
        expect(categoryTags.has(adjacentTag)).toBe(true);
      }
      for (const childTag of tag.compositeOfAnyTags ?? []) {
        expect(categoryTags.has(childTag)).toBe(true);
      }
    }
  });

  it("keeps explicit composite tags and only derives grouped catalog views at the boundary", () => {
    const spellTransformation = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "spell" && tag.tag === "transformation");
    expect(spellTransformation).toEqual(expect.objectContaining({
      tag: "transformation",
      assignmentMode: "composite",
      compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
    }));

    const urbanSetting = DERIVED_TAG_ONTOLOGY_TAGS.find((tag) => tag.category === "creature" && tag.tag === "urban_setting");
    expect(urbanSetting).toEqual(expect.objectContaining({
      family: "setting",
      assignmentMode: "editorial",
      appliesWhen: expect.arrayContaining([
        "The creature is primarily framed as belonging in city or sewer encounter spaces.",
      ]),
      adjacentTags: ["small_settlement_setting", "fortress_setting"],
    }));

    const settingFamily = DERIVED_TAG_ONTOLOGY_FAMILIES.find((family) => family.category === "creature" && family.family === "setting");
    expect(settingFamily?.description).toContain("Creature environment and encounter-setting");

    const groupedCatalog = groupDerivedTagOntology({
      families: DERIVED_TAG_ONTOLOGY_FAMILIES,
      tags: DERIVED_TAG_ONTOLOGY_TAGS,
    });
    const groupedTransformation = groupedCatalog.find((entry) => entry.category === "spell" && entry.family === "transformation");
    expect(groupedTransformation?.tags).toEqual(expect.arrayContaining([
      expect.objectContaining({
        value: "transformation",
        assignmentMode: "composite",
        compositeOfAnyTags: ["battle_form", "animal_form", "elemental_form"],
      }),
    ]));

    const equipmentPurpose = groupedCatalog.find((entry) => entry.category === "equipment" && entry.family === "purpose");
    expect(equipmentPurpose?.tags).toEqual(expect.not.arrayContaining([
      expect.objectContaining({ value: "purpose" }),
    ]));
  });

  it("authors category-scoped ontology with explicit family hierarchy before flattening", () => {
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.category).toBe("creature");
    expect(CREATURE_DERIVED_TAG_ONTOLOGY.families.setting.description).toContain("Creature environment and encounter-setting");

    const urbanSetting = CREATURE_DERIVED_TAG_ONTOLOGY.families.setting.tags.find((tag) => tag.tag === "urban_setting");
    expect(urbanSetting).toEqual(expect.objectContaining({
      tag: "urban_setting",
      assignmentMode: "editorial",
      adjacentTags: ["small_settlement_setting", "fortress_setting"],
    }));

    const flattened = flattenDerivedTagAuthoredCategoryOntology(CREATURE_DERIVED_TAG_ONTOLOGY);
    expect(flattened.families).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "setting",
    }));
    expect(flattened.tags).toContainEqual(expect.objectContaining({
      category: "creature",
      family: "setting",
      tag: "urban_setting",
    }));
  });
});
