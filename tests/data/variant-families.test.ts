import { describe, expect, it } from "vitest";

import { assignVariantFamilies } from "../../src/data/variant-families.js";
import type {
  BuildSourceEntry,
  NormalizedIndexRecord,
  PackBuildInfo,
} from "../../src/data/index-types.js";

function createRecord(input: {
  recordKey: string;
  name: string;
  category?: NormalizedIndexRecord["category"];
  subcategory?: NormalizedIndexRecord["subcategory"];
  packName?: string;
  sourcePath: string;
}): NormalizedIndexRecord {
  return {
    recordKey: input.recordKey,
    id: input.recordKey,
    name: input.name,
    normalizedName: input.name.toLowerCase(),
    type: "equipment",
    category: input.category ?? "equipment",
    subcategory: input.subcategory ?? "gear",
    packName: input.packName ?? "equipment-srd",
    packLabel: "Equipment",
    documentType: "Item",
    level: null,
    rarity: "common",
    traits: [],
    derivedTags: [],
    publicationTitle: "Pathfinder Treasure Vault",
    publicationRemaster: true,
    descriptionText: null,
    hasDescription: false,
    descriptionSnippet: null,
    sourceCategory: "rules",
    folderId: null,
    families: [],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: input.sourcePath,
    isUnique: false,
    size: null,
    itemCategory: "wand",
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

function createEntry(input: {
  recordKey: string;
  name: string;
  sourcePath: string;
  slug?: string | null;
  baseItem?: string | null;
}): BuildSourceEntry & { record: NormalizedIndexRecord } {
  const pack: PackBuildInfo = {
    name: "equipment-srd",
    label: "Equipment",
    documentType: "Item",
    declaredPath: "packs/pf2e/equipment",
    resolvedPath: "/tmp/equipment",
  };

  return {
    pack,
    filePath: input.sourcePath,
    raw: {
      _id: input.recordKey,
      name: input.name,
      type: "consumable",
      system: {
        slug: input.slug ?? null,
        baseItem: input.baseItem ?? null,
      },
    },
    record: createRecord({
      recordKey: input.recordKey,
      name: input.name,
      sourcePath: input.sourcePath,
    }),
    actorData: null,
    itemData: null,
    spellData: null,
    references: [],
    resolvedReferences: [],
  };
}

describe("variant family normalization", () => {
  it("groups arbitrary parenthetical variants such as Wondrous Figurines", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:figurine-bear",
        name: "Wondrous Figurine (Rubber Bear)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wondrous-figurine-rubber-bear.json",
      }),
      createEntry({
        recordKey: "equipment:figurine-lions",
        name: "Wondrous Figurine (Golden Lions)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wondrous-figurine-golden-lions.json",
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Wondrous Figurine");
    expect(entries[1].record.variantBaseName).toBe("Wondrous Figurine");
    expect(entries[0].record.variantFamilyKey).toBe(entries[1].record.variantFamilyKey);
    expect(entries[0].record.variantLabel).toBe("Rubber Bear");
    expect(entries[1].record.variantLabel).toBe("Golden Lions");
    expect(entries[0].record.variantAxes).toEqual(["other"]);
  });

  it("groups rank variants and marks the rank axis", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:wand-2",
        name: "Wand of Choking Mist (2nd-Rank)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wand-of-choking-mist-2nd-rank.json",
      }),
      createEntry({
        recordKey: "equipment:wand-4",
        name: "Wand of Choking Mist (4th-Rank)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/wand-of-choking-mist-4th-rank.json",
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantBaseName).toBe("Wand of Choking Mist");
    expect(entries[1].record.variantBaseName).toBe("Wand of Choking Mist");
    expect(entries[0].record.variantFamilyKey).toBe(entries[1].record.variantFamilyKey);
    expect(entries[0].record.variantAxes).toEqual(["rank"]);
    expect(entries[1].record.variantAxes).toEqual(["rank"]);
  });

  it("requires at least two siblings before assigning a variant family", () => {
    const entries = [
      createEntry({
        recordKey: "equipment:mask",
        name: "Mask (Ordinary)",
        sourcePath: "vendor/pf2e/packs/pf2e/equipment/mask-ordinary.json",
      }),
    ];

    assignVariantFamilies(entries);

    expect(entries[0].record.variantFamilyKey).toBeNull();
    expect(entries[0].record.variantBaseName).toBeNull();
    expect(entries[0].record.variantSource).toBe("none");
  });
});
