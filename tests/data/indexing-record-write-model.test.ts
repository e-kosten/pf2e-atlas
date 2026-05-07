import { describe, expect, it } from "vitest";

import { createRecordWriteModel } from "../../src/data/indexing/record-write-model.js";
import type { IndexedBuildSourceEntry, NormalizedIndexRecord, PackBuildInfo } from "../../src/data/index-types.js";
import type { DerivedAfflictionBuild } from "../../src/data/derived-afflictions.js";

function createRecord(recordKey: string, name: string): NormalizedIndexRecord {
  return {
    recordKey,
    id: recordKey,
    name,
    normalizedName: name.toLowerCase(),
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spells-srd",
    packLabel: "Spells",
    documentType: "Item",
    level: 1,
    rarity: "common",
    traits: [],
    derivedTags: [],
    publicationTitle: "Player Core",
    publicationRemaster: true,
    descriptionText: null,
    blurbText: null,
    hasDescription: false,
    descriptionSnippet: null,
    sourceCategory: "core",
    folderId: null,
    families: [],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: `packs/spells/${recordKey}.json`,
    isUnique: false,
    size: null,
    itemCategory: null,
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
    searchText: name,
  };
}

function createEntry(recordKey: string, name: string): IndexedBuildSourceEntry {
  const pack: PackBuildInfo = {
    name: "spells-srd",
    label: "Spells",
    documentType: "Item",
    declaredPath: "packs/pf2e/spells-srd",
    resolvedPath: "/tmp/spells-srd",
  };

  return {
    pack,
    filePath: `/tmp/${recordKey}.json`,
    raw: { _id: recordKey, name },
    record: createRecord(recordKey, name),
    actorData: null,
    itemData: null,
    spellData: null,
    references: [],
    resolvedReferences: [],
  };
}

describe("createRecordWriteModel", () => {
  it("turns resolved source artifacts into explicit writable entries", () => {
    const current = createEntry("spell:current", "Current Spell");
    const legacy = createEntry("spell:legacy", "Legacy Spell");
    const derivedRecord = createRecord("affliction:derived", "Derived Affliction");
    const derivedAfflictions: DerivedAfflictionBuild = {
      records: [
        {
          record: derivedRecord,
          raw: { _id: "derived" },
          actorData: null,
          itemData: null,
          spellData: null,
          references: [],
          resolvedReferences: [],
          isSearchCanonical: true,
        },
      ],
      edges: [],
    };

    const result = createRecordWriteModel({
      indexedEntries: [current, legacy],
      derivedAfflictions,
      aliasRows: [
        {
          canonicalRecordKey: "spell:current",
          aliasText: "Zed Alias",
          normalizedAlias: "zed alias",
          sourceKind: "test",
          sourceRef: "test",
        },
        {
          canonicalRecordKey: "spell:current",
          aliasText: "Alpha Alias",
          normalizedAlias: "alpha alias",
          sourceKind: "test",
          sourceRef: "test",
        },
      ],
      legacyLinkRows: [
        {
          canonicalRecordKey: "spell:current",
          legacyRecordKey: "spell:legacy",
          sourceKind: "test",
          sourceRef: "test",
        },
      ],
    });

    expect(result.writableEntries.map((entry) => entry.record.recordKey)).toEqual([
      "spell:current",
      "spell:legacy",
      "affliction:derived",
    ]);
    expect(result.writableEntries[0]?.aliasTexts).toEqual(["Alpha Alias", "Zed Alias"]);
    expect(result.writableEntries[0]?.isSearchCanonical).toBe(true);
    expect(result.writableEntries[1]?.isSearchCanonical).toBe(false);
    expect(result.writableEntries[2]?.isSearchCanonical).toBe(true);
  });
});
