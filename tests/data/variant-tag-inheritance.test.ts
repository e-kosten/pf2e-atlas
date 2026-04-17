import { describe, expect, it } from "vitest";

import type { NormalizedIndexRecord } from "../../src/data/index-types.js";
import { applyVariantBaseTagInheritance } from "../../src/data/variant-tag-inheritance.js";
import { getVariantInheritableTags } from "../../src/tags/index.js";

function createRecord(input: {
  recordKey: string;
  name: string;
  category?: NormalizedIndexRecord["category"];
  subcategory?: NormalizedIndexRecord["subcategory"];
  descriptionText?: string | null;
  traits?: string[];
  derivedTags?: string[];
  variantFamilyKey?: string | null;
  variantBaseName?: string | null;
  variantLabel?: string | null;
  isUnique?: boolean;
}): NormalizedIndexRecord {
  const category = input.category ?? "creature";

  return {
    recordKey: input.recordKey,
    id: input.recordKey,
    name: input.name,
    normalizedName: input.name.toLowerCase(),
    type: category === "creature" ? "npc" : "consumable",
    category,
    subcategory: input.subcategory ?? (category === "equipment" ? "gear" : null),
    packName: category === "creature" ? "pathfinder-monster-core" : "equipment-srd",
    packLabel: category === "creature" ? "Monster Core" : "Equipment SRD",
    documentType: category === "creature" ? "Actor" : "Item",
    level: null,
    rarity: "common",
    traits: input.traits ?? [],
    derivedTags: input.derivedTags ?? [],
    publicationTitle: "Pathfinder Test",
    publicationRemaster: true,
    descriptionText: input.descriptionText ?? null,
    blurbText: null,
    hasDescription: Boolean(input.descriptionText),
    descriptionSnippet: input.descriptionText ?? null,
    sourceCategory: "rules",
    folderId: null,
    families: [],
    variantFamilyKey: input.variantFamilyKey ?? null,
    variantBaseName: input.variantBaseName ?? null,
    variantLabel: input.variantLabel ?? null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "composite",
    sourcePath: `/tmp/${input.recordKey}.json`,
    isUnique: input.isUnique ?? false,
    size: null,
    itemCategory: category === "equipment" ? "gear" : null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: null,
    usage: null,
    hands: null,
    damageTypes: [],
    weaponGroup: null,
    armorGroup: null,
    traditions: [],
    spellKinds: [],
    saveType: null,
    areaType: null,
    rangeText: null,
    durationText: null,
    durationUnit: null,
    targetText: null,
    areaValue: null,
    sustained: false,
    basicSave: false,
    languages: [],
    speedTypes: [],
    senses: [],
    immunities: [],
    resistances: [],
    weaknesses: [],
    disableText: null,
    disableSkills: [],
    isComplex: false,
    actorMetrics: {},
    itemMetrics: {},
    rangeValue: null,
    searchText: input.name,
  };
}

describe("variant base tag inheritance", () => {
  it("treats creature setting tags as opt-in inheritable and leaves encounter-role tags non-inheritable", () => {
    const inheritable = getVariantInheritableTags({ category: "creature" });

    expect(inheritable).toContain("mountain_setting");
    expect(inheritable).toContain("urban_setting");
    expect(inheritable).not.toContain("profession_npc");
    expect(inheritable).not.toContain("enforcer_npc");
  });

  it("inherits opted-in setting tags from an explicit canonical creature base", () => {
    const records = [
      createRecord({
        recordKey: "creature:giant-bat",
        name: "Giant Bat",
        derivedTags: ["ruins_setting", "underground_setting"],
        variantFamilyKey: "creature:family:giant-bat",
        variantBaseName: "Giant Bat",
        variantLabel: null,
      }),
      createRecord({
        recordKey: "creature:albino-giant-bat",
        name: "Albino Giant Bat",
        derivedTags: [],
        variantFamilyKey: "creature:family:giant-bat",
        variantBaseName: "Giant Bat",
        variantLabel: "Albino Giant Bat",
      }),
    ];

    const inherited = applyVariantBaseTagInheritance(records, getVariantInheritableTags({ category: "creature" }));

    expect(records[1]?.derivedTags).toEqual(["ruins_setting", "underground_setting"]);
    expect(inherited.get("creature:albino-giant-bat")).toEqual(["ruins_setting", "underground_setting"]);
  });

  it("uses stable non-unique sibling setting tags when a family has no explicit base record", () => {
    const records = [
      createRecord({
        recordKey: "creature:white-dragon-young",
        name: "White Dragon (Young)",
        derivedTags: ["arctic_setting", "mountain_setting", "underground_setting"],
        variantFamilyKey: "creature:family:white-dragon",
        variantBaseName: "White Dragon",
        variantLabel: "Young",
      }),
      createRecord({
        recordKey: "creature:white-dragon-adult",
        name: "White Dragon (Adult)",
        derivedTags: ["arctic_setting", "mountain_setting", "underground_setting"],
        variantFamilyKey: "creature:family:white-dragon",
        variantBaseName: "White Dragon",
        variantLabel: "Adult",
      }),
      createRecord({
        recordKey: "creature:venexus",
        name: "Venexus",
        derivedTags: ["mountain_setting"],
        variantFamilyKey: "creature:family:white-dragon",
        variantBaseName: "White Dragon",
        variantLabel: "Venexus",
        isUnique: true,
      }),
      createRecord({
        recordKey: "creature:venexus-wyrmling",
        name: "Venexus's Wyrmling",
        derivedTags: [],
        variantFamilyKey: "creature:family:white-dragon",
        variantBaseName: "White Dragon",
        variantLabel: "Venexus's Wyrmling",
        isUnique: true,
      }),
      createRecord({
        recordKey: "creature:venexus-chosen",
        name: "Venexus's Chosen",
        derivedTags: [],
        variantFamilyKey: "creature:family:white-dragon",
        variantBaseName: "White Dragon",
        variantLabel: "Venexus's Chosen",
        isUnique: true,
      }),
    ];

    const inherited = applyVariantBaseTagInheritance(records, getVariantInheritableTags({ category: "creature" }));

    expect(records[2]?.derivedTags).toEqual(["arctic_setting", "mountain_setting", "underground_setting"]);
    expect(inherited.get("creature:venexus")).toEqual(["arctic_setting", "underground_setting"]);
    expect(records[3]?.derivedTags).toEqual(["arctic_setting", "mountain_setting", "underground_setting"]);
    expect(records[4]?.derivedTags).toEqual(["arctic_setting", "mountain_setting", "underground_setting"]);
  });

  it("does not infer from a family without an explicit base and only one non-unique sibling", () => {
    const records = [
      createRecord({
        recordKey: "creature:mystery-beast-scout",
        name: "Mystery Beast (Scout)",
        derivedTags: ["forest_setting"],
        variantFamilyKey: "creature:family:mystery-beast",
        variantBaseName: "Mystery Beast",
        variantLabel: "Scout",
      }),
      createRecord({
        recordKey: "creature:named-mystery-beast",
        name: "Named Mystery Beast",
        derivedTags: [],
        variantFamilyKey: "creature:family:mystery-beast",
        variantBaseName: "Mystery Beast",
        variantLabel: "Named Mystery Beast",
        isUnique: true,
      }),
    ];

    const inherited = applyVariantBaseTagInheritance(records, getVariantInheritableTags({ category: "creature" }));

    expect(records[1]?.derivedTags).toEqual([]);
    expect(inherited.size).toBe(0);
  });

  it("ignores non-creature variant families", () => {
    const records = [
      createRecord({
        recordKey: "equipment:hag-eye",
        name: "Hag Eye",
        category: "equipment",
        derivedTags: ["tracking"],
        variantFamilyKey: "equipment:family:hag-eye",
        variantBaseName: "Hag Eye",
        variantLabel: null,
      }),
      createRecord({
        recordKey: "equipment:smoky-hag-eye",
        name: "Smoky Hag Eye",
        category: "equipment",
        derivedTags: [],
        variantFamilyKey: "equipment:family:hag-eye",
        variantBaseName: "Hag Eye",
        variantLabel: "Smoky",
      }),
    ];

    const inherited = applyVariantBaseTagInheritance(records, getVariantInheritableTags({ category: "creature" }));

    expect(records[1]?.derivedTags).toEqual([]);
    expect(inherited.size).toBe(0);
  });
});
