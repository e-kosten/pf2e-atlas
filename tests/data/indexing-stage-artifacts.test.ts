import { describe, expect, it } from "vitest";

import { canonicalizeIndexRecords } from "../../src/data/indexing/canonicalization.js";
import { resolveIndexReferences } from "../../src/data/indexing/reference-resolution.js";
import type {
  ExtractedReference,
  IndexedBuildSourceEntry,
  NormalizedIndexRecord,
  PackBuildInfo,
} from "../../src/data/index-types.js";

const pack: PackBuildInfo = {
  name: "spells-srd",
  label: "Spells",
  documentType: "Item",
  declaredPath: "packs/pf2e/spells-srd",
  resolvedPath: "/tmp/pathfinder-mcp-missing-pack",
};

function createRecord(recordKey: string, name: string): NormalizedIndexRecord {
  const id = recordKey.split(":")[1] ?? recordKey;
  return {
    recordKey,
    id,
    name,
    normalizedName: name.toLowerCase(),
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: pack.name,
    packLabel: pack.label,
    documentType: pack.documentType,
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
    sourcePath: `packs/spells/${id}.json`,
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

function createEntry(recordKey: string, name: string, references: ExtractedReference[] = []): IndexedBuildSourceEntry {
  return {
    pack,
    filePath: `/tmp/${recordKey}.json`,
    raw: { _id: recordKey.split(":")[1] ?? recordKey, name },
    record: createRecord(recordKey, name),
    actorData: null,
    itemData: null,
    spellData: null,
    references,
    resolvedReferences: [],
  };
}

describe("indexing stage artifacts", () => {
  it("returns reference-resolved entries without mutating the family stage entries", async () => {
    const source = createEntry("spells-srd:source", "Source", [
      {
        packName: "spells-srd",
        recordLocator: "target",
        displayText: "Target",
        referenceText: "@UUID[Compendium.pf2e.spells-srd.Item.target]{Target}",
      },
    ]);
    const target = createEntry("spells-srd:target", "Target");

    const result = await resolveIndexReferences({
      indexedEntries: [source, target],
      sourceEntries: [source, target],
      packs: [],
      rootPath: "/tmp/pathfinder-mcp-missing-root",
    });

    expect(source.resolvedReferences).toEqual([]);
    expect(result.referencedEntries[0]).not.toBe(source);
    expect(result.referencedEntries[0]?.record).not.toBe(source.record);
    expect(result.referencedEntries[0]?.resolvedReferences).toHaveLength(1);
    expect(result.referencedEntries[0]?.resolvedReferences[0]?.targetRecordKey).toBe("spells-srd:target");
  });

  it("returns canonical entries with derived tag arrays instead of mutating referenced entries", () => {
    const source = createEntry("spells-srd:source", "Source");
    source.record.derivedTags = ["preexisting"];

    const result = canonicalizeIndexRecords(
      [source],
      {
        referencedEntries: [source],
        aliasRows: [],
        legacyLinkRows: [],
      },
      {},
    );

    expect(source.record.derivedTags).toEqual(["preexisting"]);
    expect(result.canonicalEntries[0]).not.toBe(source);
    expect(result.canonicalEntries[0]?.record).not.toBe(source.record);
    expect(result.canonicalEntries[0]?.record.derivedTags).not.toBe(source.record.derivedTags);
  });
});
