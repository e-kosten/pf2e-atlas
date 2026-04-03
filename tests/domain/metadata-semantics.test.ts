import { describe, expect, it } from "vitest";

import { getMetadataFilterSemantics } from "../../src/domain/metadata-semantics.js";
import { filterValueFieldSchema } from "../../src/server/tool-schemas.js";

describe("metadata search semantics", () => {
  it("lists meaningful metadata fields by category", () => {
    const semantics = getMetadataFilterSemantics();

    expect(semantics.metadataFieldsByCategory.equipment).toEqual(expect.arrayContaining([
      "usage",
      "hands",
      "weaponGroup",
      "armorGroup",
      "itemCategory",
      "damageTypes",
    ]));
    expect(semantics.metadataFieldsByCategory.creature).toEqual(expect.arrayContaining([
      "families",
      "languages",
      "immunities",
      "size",
    ]));
    expect(semantics.metadataFieldsByCategory.spell).toEqual(expect.arrayContaining([
      "derivedTags",
      "traditions",
      "spellKinds",
      "damageTypes",
      "rangeValue",
    ]));
    expect(semantics.metadataFieldsByCategory.hazard).toEqual(expect.arrayContaining([
      "derivedTags",
    ]));
    expect(semantics.metadataFieldsByCategory.affliction).toEqual(expect.arrayContaining([
      "derivedTags",
    ]));
  });

  it("keeps subcategory narrowing sparse and explicit", () => {
    const semantics = getMetadataFilterSemantics();

    expect(semantics.metadataFieldsByCategoryAndSubcategory.equipment?.weapon).toEqual(["weaponGroup"]);
    expect(semantics.metadataFieldsByCategoryAndSubcategory.equipment?.armor).toEqual(["armorGroup"]);
    expect(semantics.metadataFieldsByCategoryAndSubcategory.creature).toBeUndefined();
  });

  it("advertises discoverable fields that align with list_filter_values", () => {
    const semantics = getMetadataFilterSemantics();
    const discoverableFields = semantics.metadataFields
      .filter((entry) => entry.discoverable)
      .map((entry) => entry.field);

    expect(discoverableFields).toEqual(expect.arrayContaining([
      "traits",
      "families",
      "traditions",
      "spellKinds",
      "weaponGroup",
      "armorGroup",
      "usage",
      "itemCategory",
      "size",
      "rarity",
      "publicationTitle",
    ]));
    expect(discoverableFields).not.toContain("hands");
    expect(discoverableFields).not.toContain("publicationRemaster");

    for (const field of discoverableFields) {
      expect(filterValueFieldSchema.safeParse(field).success).toBe(true);
    }
  });

  it("keeps derived tags category-bounded", () => {
    const semantics = getMetadataFilterSemantics();
    const derivedTags = semantics.metadataFields.find((entry) => entry.field === "derivedTags");

    expect(derivedTags?.categories).toEqual(["equipment", "creature", "hazard", "affliction", "spell"]);
    expect(derivedTags?.notes).toContain("ontology coverage");
  });
});
