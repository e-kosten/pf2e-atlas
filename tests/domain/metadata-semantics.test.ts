import { describe, expect, it } from "vitest";

import { getMetadataFilterSemantics } from "../../src/search/filters/semantics.js";
import { filterValueFieldSchema } from "../../src/server/tool-schemas.js";

function matcherArrayContaining(values: unknown[]): unknown {
  return expect.arrayContaining(values);
}

function matcherObjectContaining(value: Record<string, unknown>): unknown {
  return expect.objectContaining(value);
}

describe("metadata search semantics", () => {
  it("lists meaningful metadata fields by category", () => {
    const semantics = getMetadataFilterSemantics();

    expect(semantics.metadataFieldsByCategory.equipment).toEqual(
      expect.arrayContaining([
        "usage",
        "hands",
        "weaponGroup",
        "armorGroup",
        "itemCategory",
        "baseItem",
        "damageTypes",
        "variantAxes",
        "variantFamilyKey",
        "variantBaseName",
        "variantLabel",
      ]),
    );
    expect(semantics.metadataFieldsByCategory.creature).toEqual(
      expect.arrayContaining(["families", "languages", "immunities", "size"]),
    );
    expect(semantics.metadataFieldsByCategory.spell).toEqual(
      expect.arrayContaining([
        "derivedTags",
        "traditions",
        "spellKinds",
        "saveType",
        "areaType",
        "durationUnit",
        "damageTypes",
        "rangeValue",
        "areaValue",
        "sustained",
        "basicSave",
        "variantAxes",
        "variantFamilyKey",
        "variantBaseName",
        "variantLabel",
      ]),
    );
    expect(semantics.metadataFieldsByCategory.hazard).toEqual(
      expect.arrayContaining(["derivedTags", "disableSkills", "isComplex"]),
    );
    expect(semantics.metadataFieldsByCategory.affliction).toEqual(expect.arrayContaining(["derivedTags"]));
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

    expect(discoverableFields).toEqual(
      expect.arrayContaining([
        "traits",
        "families",
        "sourceCategory",
        "traditions",
        "spellKinds",
        "saveType",
        "areaType",
        "weaponGroup",
        "armorGroup",
        "usage",
        "actionCost",
        "hands",
        "rangeValue",
        "areaValue",
        "durationUnit",
        "sustained",
        "basicSave",
        "senses",
        "disableSkills",
        "isComplex",
        "itemCategory",
        "baseItem",
        "size",
        "rarity",
        "publicationTitle",
        "variantAxes",
        "variantFamilyKey",
        "variantBaseName",
        "variantLabel",
      ]),
    );
    expect(discoverableFields).not.toContain("publicationRemaster");

    for (const field of discoverableFields) {
      expect(filterValueFieldSchema.safeParse(field).success).toBe(true);
    }
  });

  it("routes promoted-field ordering through the shared promoted-field domain owner", () => {
    const semantics = getMetadataFilterSemantics();

    expect(semantics.metadataFields.find((entry) => entry.field === "rarity")?.valueOrdering).toEqual({
      kind: "canonical",
      order: ["common", "uncommon", "rare", "unique"],
    });
    expect(semantics.metadataFields.find((entry) => entry.field === "actionCost")?.valueOrdering).toEqual({
      kind: "canonical",
      order: ["0", "1", "2", "3"],
    });
  });

  it("keeps derived tags category-bounded", () => {
    const semantics = getMetadataFilterSemantics();
    const derivedTags = semantics.metadataFields.find((entry) => entry.field === "derivedTags");

    expect(derivedTags?.categories).toEqual(["equipment", "creature", "hazard", "affliction", "spell"]);
    expect(derivedTags?.notes).toContain("ontology coverage");
  });

  it("documents advanced creature metric predicates separately from fixed metadata fields", () => {
    const semantics = getMetadataFilterSemantics();

    expect(semantics.advancedPredicates).toEqual(
      matcherArrayContaining([
        matcherObjectContaining({
          name: "actorMetric",
          categories: ["creature", "hazard"],
          operators: matcherArrayContaining([">=", "=="]),
        }),
        matcherObjectContaining({
          name: "actorMetricCompare",
          categories: ["creature", "hazard"],
          operators: matcherArrayContaining([">", "!="]),
        }),
        matcherObjectContaining({
          name: "itemMetric",
          categories: ["equipment"],
          operators: matcherArrayContaining([">=", "=="]),
        }),
        matcherObjectContaining({
          name: "itemMetricCompare",
          categories: ["equipment"],
          operators: matcherArrayContaining([">", "!="]),
        }),
      ]),
    );
    expect(semantics.actorMetricDiscovery?.filterValueField).toBe("actorMetrics");
    expect(semantics.actorMetricDiscovery?.namespaces.map((entry) => entry.prefix)).toEqual(
      expect.arrayContaining([
        "ability.",
        "save.",
        "skill.",
        "perception.",
        "ac.",
        "hp.",
        "hardness.",
        "stealth.",
        "speed.",
        "sense.",
        "disable.",
      ]),
    );
    expect(semantics.itemMetricDiscovery?.filterValueField).toBe("itemMetrics");
    expect(semantics.itemMetricDiscovery?.namespaces.map((entry) => entry.prefix)).toEqual(
      expect.arrayContaining(["weapon.", "armor.", "shield."]),
    );
  });

  it("keeps example filters on the canonical SearchFilterNode model", () => {
    const semantics = getMetadataFilterSemantics();

    expect(semantics.examplesByCategory.creature?.[0]?.filter).toEqual({
      kind: "allOf",
      children: expect.arrayContaining([
        {
          kind: "metadataPredicate",
          predicate: { field: "sourceCategory", op: "eq", value: "core" },
        },
        {
          kind: "metadataPredicate",
          predicate: { field: "traits", op: "includes", value: "undead" },
        },
        {
          kind: "not",
          child: {
            kind: "metadataPredicate",
            predicate: { field: "traits", op: "includes", value: "water" },
          },
        },
      ]),
    });
    expect(semantics.advancedPredicates.find((entry) => entry.name === "actorMetricCompare")?.example).toEqual({
      kind: "metricCompare",
      leftMetric: "ability.int.mod",
      op: "gt",
      rightMetric: "ability.cha.mod",
    });
  });
});
