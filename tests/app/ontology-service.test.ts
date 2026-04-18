import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it, vi } from "vitest";

import { createPf2eApplicationOntologyService } from "../../src/app/ontology-service.js";
import { getMetadataGlossaryArtifactPath } from "../../src/data/metadata-glossary.js";
import type { Pf2eDataService } from "../../src/data/service.js";
import type {
  AppConfig,
  FilterValueField,
  MetadataGlossaryArtifact,
  NormalizedRecord,
  OntologyNode,
  SearchFilters,
} from "../../src/types.js";

function createTestConfig(indexPath = ".cache/pf2e-index.sqlite"): AppConfig {
  return {
    dataPath: "vendor/pf2e",
    rootPath: "vendor/pf2e",
    manifestPath: "vendor/pf2e/system.pf2e.json",
    indexPath,
    embeddings: {
      provider: "hash",
      modelId: "test-model",
      modelRevision: null,
      cachePath: ".cache/models",
      localModelPath: null,
    },
    ranking: {
      configPath: "pf2e-ranking.json",
    },
  };
}

function findNodeById(nodes: OntologyNode[], id: string): OntologyNode | undefined {
  for (const node of nodes) {
    if (node.id === id) {
      return node;
    }
    if (node.children) {
      const match = findNodeById(node.children, id);
      if (match) {
        return match;
      }
    }
  }
  return undefined;
}

function createRecord(overrides: Partial<NormalizedRecord> = {}): NormalizedRecord {
  return {
    recordKey: overrides.recordKey ?? "spells:burning-hands",
    id: overrides.id ?? "burning-hands",
    name: overrides.name ?? "Burning Hands",
    normalizedName: overrides.normalizedName ?? "burning hands",
    type: overrides.type ?? "spell",
    category: overrides.category ?? "spell",
    subcategory: overrides.subcategory ?? null,
    packName: overrides.packName ?? "spells",
    packLabel: overrides.packLabel ?? "Spells",
    documentType: overrides.documentType ?? "spell",
    level: overrides.level ?? 1,
    rarity: overrides.rarity ?? "common",
    traits: overrides.traits ?? ["fire"],
    derivedTags: overrides.derivedTags ?? ["alarm"],
    publicationTitle: overrides.publicationTitle ?? "Test Source",
    publicationRemaster: overrides.publicationRemaster ?? true,
    descriptionText: overrides.descriptionText ?? "A small cone of fire.",
    blurbText: overrides.blurbText ?? "Fiery burst.",
    hasDescription: overrides.hasDescription ?? true,
    descriptionSnippet: overrides.descriptionSnippet ?? "cone of fire",
    sourceCategory: overrides.sourceCategory ?? "primary",
    folderId: overrides.folderId ?? null,
    families: overrides.families ?? [],
    variantFamilyKey: overrides.variantFamilyKey ?? null,
    variantBaseName: overrides.variantBaseName ?? null,
    variantLabel: overrides.variantLabel ?? null,
    variantAxes: overrides.variantAxes ?? [],
    variantConfidence: overrides.variantConfidence ?? null,
    variantSource: overrides.variantSource ?? "none",
    sourcePath: overrides.sourcePath ?? "packs/spells/burning-hands.json",
    isUnique: overrides.isUnique ?? false,
    size: overrides.size ?? null,
    itemCategory: overrides.itemCategory ?? null,
    baseItem: overrides.baseItem ?? null,
    priceCp: overrides.priceCp ?? null,
    bulkValue: overrides.bulkValue ?? null,
    actionCost: overrides.actionCost ?? 2,
    usage: overrides.usage ?? null,
    hands: overrides.hands ?? null,
    damageTypes: overrides.damageTypes ?? [],
    weaponGroup: overrides.weaponGroup ?? null,
    armorGroup: overrides.armorGroup ?? null,
    traditions: overrides.traditions ?? ["arcane"],
    spellKinds: overrides.spellKinds ?? [],
    saveType: overrides.saveType !== undefined ? overrides.saveType : "fortitude",
    areaType: overrides.areaType !== undefined ? overrides.areaType : "cone",
    rangeText: overrides.rangeText ?? "self",
    durationText: overrides.durationText ?? "instantaneous",
    durationUnit: overrides.durationUnit ?? null,
    targetText: overrides.targetText ?? null,
    areaValue: overrides.areaValue !== undefined ? overrides.areaValue : 15,
    sustained: overrides.sustained ?? false,
    basicSave: overrides.basicSave ?? true,
    languages: overrides.languages ?? [],
    speedTypes: overrides.speedTypes ?? [],
    senses: overrides.senses ?? [],
    immunities: overrides.immunities ?? [],
    resistances: overrides.resistances ?? [],
    weaknesses: overrides.weaknesses ?? [],
    disableText: overrides.disableText ?? null,
    disableSkills: overrides.disableSkills ?? [],
    isComplex: overrides.isComplex ?? false,
    actorMetrics: overrides.actorMetrics ?? {},
    itemMetrics: overrides.itemMetrics ?? {},
    rangeValue: overrides.rangeValue ?? null,
    aliases: overrides.aliases ?? [],
    legacyRecordLinks: overrides.legacyRecordLinks ?? [],
    raw: overrides.raw ?? {},
  };
}

function createDataService(): Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords"> {
  const burningHands = createRecord();
  const mageArmor = createRecord({
    recordKey: "spells:mage-armor",
    id: "mage-armor",
    name: "Mage Armor",
    normalizedName: "mage armor",
    traits: ["abjuration"],
    derivedTags: ["ward"],
    saveType: null,
    areaType: null,
    areaValue: null,
    basicSave: false,
    descriptionText: "Protective force surrounds you.",
    blurbText: "Protective aura.",
  });

  return {
    getSearchVocabulary: vi.fn(() => ({
      categories: [
        { value: "spell", count: 12 },
        { value: "equipment", count: 5 },
      ],
      subcategories: [{ value: "action", count: 6 }],
      rarities: [],
      sizes: [],
      traditions: [],
      spellKinds: [],
      sourceCategories: [],
      commonTraitsByCategory: [
        { category: "spell", traits: [{ value: "fire", count: 4 }] },
      ],
      commonDerivedTagsByCategory: [
        { category: "spell", tags: [{ value: "alarm", count: 2 }] },
      ],
      derivedTagOntologyFamilies: [],
      derivedTagOntologyTags: [],
      derivedTagCatalog: [],
    })),
    listFilterValues: vi.fn(({ field, category }) => {
      const valuesByKey: Partial<Record<`${string}:${FilterValueField}`, Array<{ value: string; count: number }>>> = {
        "spell:subcategories": [{ value: "action", count: 6 }],
        "spell:traits": [{ value: "fire", count: 4 }],
        "spell:saveType": [{ value: "fortitude", count: 3 }],
        "spell:sustained": [{ value: "true", count: 2 }],
        "equipment:hands": [{ value: "1", count: 5 }],
        "equipment:subcategories": [{ value: "gear", count: 5 }],
      };
      return {
        field,
        values: valuesByKey[`${category ?? "all"}:${field}`] ?? [],
      };
    }),
    listRecords: vi.fn((filters: SearchFilters) => {
      const matchingRecords = [burningHands, mageArmor].filter((record) => {
        if (filters.category && record.category !== filters.category) {
          return false;
        }
        if (filters.subcategory && record.subcategory !== filters.subcategory) {
          return false;
        }
        const metadata = filters.metadata;
        if (!metadata || !("field" in metadata)) {
          return true;
        }
        if (metadata.field === "traits" && metadata.op === "includesAny") {
          return metadata.values.some((value) => record.traits.includes(value));
        }
        if (metadata.field === "saveType" && metadata.op === "eq") {
          return record.saveType === metadata.value;
        }
        if (metadata.field === "derivedTags" && metadata.op === "includesAny") {
          return metadata.values.some((value) => record.derivedTags.includes(value));
        }
        return true;
      });

      return {
        searchProfile: null,
        mode: "structured" as const,
        sort: filters.sort ?? "alphabetical",
        total: matchingRecords.length,
        offset: 0,
        limit: filters.limit ?? 20,
        hasMore: false,
        nextOffset: null,
        records: matchingRecords.slice(0, filters.limit ?? 20),
      };
    }),
  };
}

describe("application ontology service", () => {
  it("lists available ontology domains", () => {
    const service = createPf2eApplicationOntologyService(createTestConfig(), createDataService());

    expect(service.listDomains().map((domain) => domain.id)).toEqual([
      "derivedTags",
      "catalogCategories",
      "searchSemantics",
    ]);
  });

  it("caches ontology domain models across repeated loads", () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(createTestConfig(), dataService);

    const first = service.loadDomain("searchSemantics");
    const second = service.loadDomain("searchSemantics");

    expect(second).toBe(first);
  });

  it("builds valid field-specific browse queries for search semantics values", () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(createTestConfig(), dataService);
    const domain = service.loadDomain("searchSemantics");
    const metadataFieldsNode = findNodeById(domain.rootNodes, "spell:metadataFields");

    expect(dataService.listFilterValues).not.toHaveBeenCalledWith({ field: "saveType", category: "spell" });
    expect(dataService.listFilterValues).not.toHaveBeenCalledWith({ field: "hands", category: "equipment" });

    expect(findNodeById(domain.rootNodes, "spell:fieldType:set")).toBeUndefined();
    expect(metadataFieldsNode?.childPresentation).toEqual({
      mode: "grouped",
      groupBy: "fieldType",
      render: "inline",
    });
    expect(findNodeById(domain.rootNodes, "spell:field:saveType")?.groupValues).toEqual({
      fieldType: "enumString",
    });

    const saveTypeFieldNode = findNodeById(domain.rootNodes, "spell:field:saveType");
    const sustainedFieldNode = findNodeById(domain.rootNodes, "spell:field:sustained");
    const handsFieldNode = findNodeById(domain.rootNodes, "equipment:field:hands");

    const saveTypeValueNodes = saveTypeFieldNode?.loadChildren?.() ?? [];
    const sustainedValueNodes = sustainedFieldNode?.loadChildren?.() ?? [];
    const handsValueNodes = handsFieldNode?.loadChildren?.() ?? [];

    expect(dataService.listFilterValues).toHaveBeenCalledWith({ field: "saveType", category: "spell" });
    expect(dataService.listFilterValues).toHaveBeenCalledWith({ field: "sustained", category: "spell" });
    expect(dataService.listFilterValues).toHaveBeenCalledWith({ field: "hands", category: "equipment" });
    expect(dataService.listRecords).not.toHaveBeenCalled();
    expect(saveTypeFieldNode?.listLabel).toBe("saveType");
    expect(saveTypeFieldNode?.detailLines.map((line) => line.text)).not.toContain("Operators: eq, in, notIn");

    const saveTypeValueNode = saveTypeValueNodes.find((node) => node.id === "spell:saveType:fortitude");

    expect(saveTypeValueNode?.query?.filters.metadata).toEqual({
      field: "saveType",
      op: "eq",
      value: "fortitude",
    });
    expect(sustainedValueNodes.find((node) => node.id === "spell:sustained:true")?.query?.filters.metadata).toEqual({
      field: "sustained",
      op: "eq",
      value: true,
    });
    expect(handsValueNodes.find((node) => node.id === "equipment:hands:1")?.query?.filters.metadata).toEqual({
      field: "hands",
      op: "eq",
      value: 1,
    });
    expect(findNodeById(domain.rootNodes, "spell:trait:fire")?.query?.filters.metadata).toEqual({
      field: "traits",
      op: "includesAny",
      values: ["fire"],
    });

    const saveTypeRecordChildren = saveTypeValueNode?.loadChildren?.() ?? [];

    expect(dataService.listRecords).toHaveBeenCalledWith({
      category: "spell",
      metadata: {
        field: "saveType",
        op: "eq",
        value: "fortitude",
      },
      limit: 20,
      sort: "alphabetical",
    });
    expect(saveTypeRecordChildren.map((node) => node.label)).toEqual(["Burning Hands"]);
    expect(saveTypeRecordChildren[0]?.detailTitle).toBe("Record Details");
    expect(saveTypeRecordChildren[0]?.detailLines.map((line) => line.text)).toEqual(expect.arrayContaining([
      "Burning Hands",
      "spells:burning-hands",
    ]));
  });

  it("enriches trait nodes from the metadata glossary artifact when available", () => {
    const tempRoot = mkdtempSync(path.join(tmpdir(), "pf2e-trait-glossary-"));
    const config = createTestConfig(path.join(tempRoot, "pf2e-index.sqlite"));
    const artifact: MetadataGlossaryArtifact = {
      generatedAt: "2026-04-17T00:00:00.000Z",
      fields: {
        traits: {
          fire: {
            value: "fire",
            label: "Fire",
            description: "Effects with the fire trait deal fire damage or manipulate fire.",
          },
        },
      },
    };

    try {
      writeFileSync(
        getMetadataGlossaryArtifactPath(config.indexPath),
        `${JSON.stringify(artifact, null, 2)}\n`,
        "utf8",
      );

      const service = createPf2eApplicationOntologyService(config, createDataService());
      const domain = service.loadDomain("searchSemantics");
      const commonTraitNode = findNodeById(domain.rootNodes, "spell:trait:fire");
      const traitFieldNode = findNodeById(domain.rootNodes, "spell:field:traits");
      const traitValueNode = traitFieldNode?.loadChildren?.().find((node) => node.id === "spell:traits:fire");

      expect(commonTraitNode?.label).toBe("Fire");
      expect(commonTraitNode?.listLabel).toBe("Fire | 4");
      expect(commonTraitNode?.detailLines.map((line) => line.text)).toEqual(expect.arrayContaining([
        "Fire",
        "Effects with the fire trait deal fire damage or manipulate fire.",
        "Trait: fire",
      ]));

      expect(traitValueNode?.label).toBe("Fire");
      expect(traitValueNode?.detailTitle).toBe("Trait Details");
      expect(traitValueNode?.detailLines.map((line) => line.text)).toEqual(expect.arrayContaining([
        "Fire",
        "Effects with the fire trait deal fire damage or manipulate fire.",
      ]));
    } finally {
      rmSync(tempRoot, { recursive: true, force: true });
    }
  });

  it("loads the full discoverable value set instead of truncating to a common subset", () => {
    const values = Array.from({ length: 14 }, (_, index) => ({
      value: `trait-${index + 1}`,
      count: index + 1,
    }));
    const dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords"> = {
      getSearchVocabulary: vi.fn(() => ({
        categories: [{ value: "spell", count: 14 }],
        subcategories: [],
        rarities: [],
        sizes: [],
        traditions: [],
        spellKinds: [],
        sourceCategories: [],
        commonTraitsByCategory: [],
        commonDerivedTagsByCategory: [],
        derivedTagOntologyFamilies: [],
        derivedTagOntologyTags: [],
        derivedTagCatalog: [],
      })),
      listFilterValues: vi.fn(({ field, category }) => ({
        field,
        values: field === "traits" && category === "spell" ? values : [],
      })),
      listRecords: vi.fn(() => ({
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        total: 0,
        offset: 0,
        limit: 20,
        hasMore: false,
        nextOffset: null,
        records: [],
      })),
    };

    const service = createPf2eApplicationOntologyService(createTestConfig(), dataService);
    const domain = service.loadDomain("searchSemantics");
    const traitFieldNode = findNodeById(domain.rootNodes, "spell:field:traits");
    const traitValueNodes = traitFieldNode?.loadChildren?.() ?? [];

    expect(traitValueNodes).toHaveLength(14);
    expect(traitValueNodes.at(-1)?.id).toBe("spell:traits:trait-14");
  });
});
