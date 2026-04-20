import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it, vi } from "vitest";

import { createPf2eApplicationOntologyService } from "../../src/app/ontology-service.js";
import { getMetadataGlossaryArtifactPath } from "../../src/data/metadata-glossary.js";
import type { SearchVocabularyResult } from "../../src/data/vocabulary.js";
import type { Pf2eDataService } from "../../src/data/service.js";
import type {
  FilterValueField,
  SearchFilters,
} from "../../src/domain/search-types.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { MetadataGlossaryArtifact } from "../../src/domain/metadata-glossary-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { OntologyDomainSummary, OntologyNode } from "../../src/domain/ontology-types.js";

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
    recordKey: "spell:test-alarm",
    id: "test-alarm",
    name: "Alarm Ward",
    normalizedName: "alarm ward",
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spell",
    packLabel: "Spells",
    documentType: "Item",
    level: 1,
    rarity: null,
    traits: ["fire"],
    derivedTags: ["alarm"],
    publicationTitle: "Pathfinder Rage of Elements",
    publicationRemaster: false,
    descriptionText: "Warns against intruders.",
    blurbText: null,
    hasDescription: true,
    descriptionSnippet: "Warns against intruders.",
    sourceCategory: "core",
    folderId: null,
    families: ["security"],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: "packs/spells/alarm-ward.json",
    isUnique: false,
    size: null,
    itemCategory: null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: 2,
    usage: null,
    hands: null,
    damageTypes: [],
    weaponGroup: null,
    armorGroup: null,
    traditions: ["arcane"],
    spellKinds: ["spell"],
    saveType: "fortitude",
    areaType: null,
    rangeText: "30 feet",
    durationText: "1 minute",
    durationUnit: "minute",
    targetText: "creature",
    areaValue: null,
    sustained: true,
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
    actorMetrics: { "save.best": "fort" },
    itemMetrics: { "weapon.reload": 1 },
    rangeValue: 30,
    aliases: [],
    legacyRecordLinks: [],
    raw: {},
    ...overrides,
  };
}

function createDataService(): Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords"> {
  const vocabulary: SearchVocabularyResult = {
    categories: [
      { value: "spell", count: 12 },
      { value: "equipment", count: 5 },
      { value: "creature", count: 3 },
    ],
    subcategories: [{ value: "action", count: 6 }],
    rarities: [],
    sizes: [],
    traditions: [],
    spellKinds: [],
    sourceCategories: [],
    commonTraitsByCategory: [{ category: "spell", traits: [{ value: "fire", count: 4 }] }],
    commonDerivedTagsByCategory: [{ category: "spell", tags: [{ value: "alarm", count: 2 }] }],
    derivedTagOntologyFamilies: [],
    derivedTagOntologyTags: [],
    derivedTagCatalog: [],
  };
  const listRecords = vi.fn((filters: SearchFilters) => ({
    searchProfile: null,
    mode: "structured" as const,
    sort: filters.sort ?? "alphabetical",
    total: 1,
    offset: filters.offset ?? 0,
    limit: filters.limit ?? 20,
    hasMore: false,
    nextOffset: null,
    records: [
      createRecord(
        filters.category === "equipment"
          ? {
              recordKey: "equipment:tower-bulwark",
              id: "tower-bulwark",
              name: "Tower Bulwark",
              normalizedName: "tower bulwark",
              type: "armor",
              category: "equipment",
              packName: "equipment",
              packLabel: "Equipment",
              actionCost: null,
              saveType: null,
              sustained: false,
              publicationTitle: "Pathfinder Core",
              actorMetrics: {},
              itemMetrics: { "weapon.reload": 1 },
              traits: [],
              derivedTags: [],
              traditions: [],
              spellKinds: [],
            }
          : filters.category === "creature"
            ? {
                recordKey: "creature:ember-ghost",
                id: "ember-ghost",
                name: "Ember Ghost",
                normalizedName: "ember ghost",
                type: "npc",
                category: "creature",
                packName: "creature",
                packLabel: "Creatures",
                actionCost: null,
                saveType: null,
                sustained: false,
                publicationTitle: "Pathfinder Monster Core",
                actorMetrics: { "save.best": "fort" },
                itemMetrics: {},
                traditions: [],
                spellKinds: [],
              }
            : {},
      ),
    ],
  }));
  return {
    getSearchVocabulary: vi.fn(() => vocabulary),
    listFilterValues: vi.fn(
      ({
        field,
        category,
        metric,
        metricPrefix,
      }: {
        field: FilterValueField;
        category?: string;
        metric?: string;
        metricPrefix?: string;
      }) => {
        const valuesByKey: Partial<Record<`${string}:${FilterValueField}`, Array<{ value: string; count: number }>>> = {
          "spell:subcategories": [{ value: "action", count: 6 }],
          "spell:traits": [{ value: "fire", count: 4 }],
          "spell:saveType": [{ value: "fortitude", count: 3 }],
          "spell:sustained": [{ value: "true", count: 2 }],
          "spell:publicationTitle": [{ value: "Pathfinder Rage of Elements", count: 1 }],
          "equipment:hands": [{ value: "1", count: 5 }],
          "equipment:subcategories": [{ value: "gear", count: 5 }],
          "creature:actorMetrics": metricPrefix === "save." ? [{ value: "save.best", count: 3 }] : [],
          "equipment:itemMetrics": metricPrefix === "weapon." ? [{ value: "weapon.reload", count: 5 }] : [],
        };
        if (field === "actorMetrics" && metric === "save.best") {
          return {
            field,
            values: [{ value: "fort", count: 3 }],
          };
        }
        return {
          field,
          values: valuesByKey[`${category ?? "all"}:${field}`] ?? [],
        };
      },
    ),
    listRecords,
  };
}

describe("application ontology service", () => {
  it("lists available ontology domains", () => {
    const service = createPf2eApplicationOntologyService(createTestConfig(), createDataService());

    expect(service.listDomains().map((domain) => domain.id)).toEqual([
      "catalogCategories",
      "searchSemantics",
    ]);
  });

  it("returns fresh domain summary arrays and objects on each listDomains call", () => {
    const service = createPf2eApplicationOntologyService(createTestConfig(), createDataService());

    const first = service.listDomains();
    const second = service.listDomains();

    expect(first).not.toBe(second);
    expect(first[0]).not.toBe(second[0]);

    (first as OntologyDomainSummary[]).pop();
    (second as OntologyDomainSummary[])[0] = {
      ...second[0]!,
      label: "Mutated Label",
    };

    expect(service.listDomains()).toEqual([
      {
        id: "catalogCategories",
        label: "Categories",
        description:
          "Browse top-level catalog categories and subcategories with live record counts and ready-to-run browse scopes.",
      },
      {
        id: "searchSemantics",
        label: "Search Semantics",
        description: "Explore category-specific metadata fields, live value spaces, and advanced search predicates.",
      },
    ]);
  });

  it("caches ontology domain models across repeated loads", () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(createTestConfig(), dataService);

    const first = service.loadDomain("searchSemantics");
    const second = service.loadDomain("searchSemantics");

    expect(second).toBe(first);
  });

  it("keeps search semantics off the derived-tags domain runtime path", () => {
    const loadDerivedTagOntologyExplorerModel = vi.fn(() => ({ categories: [] }));
    const service = createPf2eApplicationOntologyService(createTestConfig(), createDataService(), {
      loadDerivedTagOntologyExplorerModel,
    });

    service.loadDomain("searchSemantics");

    expect(loadDerivedTagOntologyExplorerModel).not.toHaveBeenCalled();

    service.loadDomain("derivedTags");

    expect(loadDerivedTagOntologyExplorerModel).toHaveBeenCalledTimes(1);
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

    expect(dataService.listFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field: "saveType", category: "spell" }),
    );
    expect(dataService.listFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field: "sustained", category: "spell" }),
    );
    expect(dataService.listFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field: "hands", category: "equipment" }),
    );
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
    const commonTraitNode = findNodeById(domain.rootNodes, "spell:commonTraits")?.children?.[0];
    expect(commonTraitNode?.query?.filters.metadata).toEqual({
      field: "traits",
      op: "includesAny",
      values: ["fire"],
    });
    expect(saveTypeValueNode?.loadChildren?.()[0]?.kind).toBe("record");
    expect(saveTypeValueNode?.detailLines.map((line) => line.text)).toContain(
      "Press Enter or o to open the full matching set in the shared result reader.",
    );
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
      const commonTraitNode = findNodeById(domain.rootNodes, "spell:commonTraits")?.children?.[0];
      const traitFieldNode = findNodeById(domain.rootNodes, "spell:field:traits");
      const traitValueNode = traitFieldNode?.loadChildren?.().find((node) => node.id === "spell:traits:fire");

      expect(commonTraitNode?.label).toBe("Fire");
      expect(commonTraitNode?.listLabel).toBe("Fire | 4");
      expect(commonTraitNode?.detailLines.map((line) => line.text)).toEqual(
        expect.arrayContaining([
          "Fire",
          "Effects with the fire trait deal fire damage or manipulate fire.",
          "Trait: fire",
        ]),
      );

      expect(traitValueNode?.label).toBe("Fire");
      expect(traitValueNode?.detailTitle).toBe("Trait Details");
      expect(traitValueNode?.detailLines.map((line) => line.text)).toEqual(
        expect.arrayContaining(["Fire", "Effects with the fire trait deal fire damage or manipulate fire."]),
      );
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
      listFilterValues: vi.fn(({ field, category }: { field: FilterValueField; category?: string }) => ({
        field,
        values: field === "traits" && category === "spell" ? values : [],
      })),
      listRecords: vi.fn((filters: SearchFilters) => ({
        searchProfile: null,
        mode: "structured" as const,
        sort: filters.sort ?? "alphabetical",
        total: 0,
        offset: filters.offset ?? 0,
        limit: filters.limit ?? 20,
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

  it("uses live record inspection and live metric discovery instead of shallow examples", () => {
    const service = createPf2eApplicationOntologyService(createTestConfig(), createDataService());
    const domain = service.loadDomain("searchSemantics");
    const booleanGroupNode = findNodeById(domain.rootNodes, "spell:booleanGroup:and");
    const actorMetricCompareNode = findNodeById(domain.rootNodes, "creature:advanced:actorMetricCompare");
    const itemMetricCompareNode = findNodeById(domain.rootNodes, "equipment:advanced:itemMetricCompare");
    const actorMetricGroup = findNodeById(domain.rootNodes, "creature:actorMetrics:discovery");
    const actorMetricNamespace = findNodeById(domain.rootNodes, "creature:actorMetrics:namespace:save.");
    const actorMetricNode = actorMetricNamespace
      ?.loadChildren?.()
      .find((node) => node.id === "creature:actorMetrics:save.best");
    const actorMetricValueNode = actorMetricNode
      ?.loadChildren?.()
      .find((node) => node.id === "creature:actorMetrics:save.best:fort");
    const commonTraitNode = findNodeById(domain.rootNodes, "spell:commonTraits")?.children?.[0];
    const saveTypeValueNode = findNodeById(domain.rootNodes, "spell:field:saveType")
      ?.loadChildren?.()
      .find((node) => node.id === "spell:saveType:fortitude");
    const publicationTitleValueNode = findNodeById(domain.rootNodes, "spell:field:publicationTitle")
      ?.loadChildren?.()
      .find((node) => node.id === "spell:publicationTitle:Pathfinder Rage of Elements");
    const itemMetricNamespace = findNodeById(domain.rootNodes, "equipment:itemMetrics:namespace:weapon.");
    const itemMetricNode = itemMetricNamespace
      ?.loadChildren?.()
      .find((node) => node.id === "equipment:itemMetrics:weapon.reload");

    expect(findNodeById(domain.rootNodes, "equipment:example:0")).toBeUndefined();
    expect(findNodeById(domain.rootNodes, "equipment:examples")).toBeUndefined();
    expect(booleanGroupNode?.detailLines.map((line) => line.text)).toContain(
      "Requires every child predicate or group to match. Must contain at least 2 child nodes.",
    );
    expect(actorMetricCompareNode?.query).toEqual({
      kind: "listRecords",
      label: "Browse records matching the actorMetricCompare example",
      filters: {
        category: "creature",
        metadata: {
          field: "actorMetricCompare",
          leftMetric: "ability.int.mod",
          op: ">",
          rightMetric: "ability.cha.mod",
        },
        limit: 20,
      },
    });
    expect(itemMetricCompareNode?.query).toEqual({
      kind: "listRecords",
      label: "Browse records matching the itemMetricCompare example",
      filters: {
        category: "equipment",
        metadata: {
          field: "itemMetricCompare",
          leftMetric: "shield.hp",
          op: ">",
          rightMetric: "shield.bt",
        },
        limit: 20,
      },
    });
    expect(actorMetricGroup?.detailLines.map((line) => line.text)).toContain(
      "Explore live creature statistics namespaces, keys, and exact scalar values from the indexed corpus.",
    );
    expect(actorMetricNamespace?.loadChildren?.().map((node) => node.label)).toContain("Best Save");
    expect(actorMetricNode?.label).toBe("Best Save");
    expect(actorMetricValueNode?.query).toEqual({
      kind: "listRecords",
      label: "Browse records where Best Save = fort",
      filters: {
        category: "creature",
        metadata: {
          field: "actorMetric",
          metric: "save.best",
          op: "==",
          value: "fort",
        },
        limit: 20,
      },
    });
    expect(actorMetricValueNode?.loadChildren?.()[0]?.kind).toBe("record");
    expect(itemMetricNode?.label).toBe("Weapon Reload");
    expect(itemMetricNode?.query).toEqual({
      kind: "listRecords",
      label: "Browse records with Weapon Reload",
      filters: {
        category: "equipment",
        metadata: {
          field: "itemMetricCompare",
          leftMetric: "weapon.reload",
          op: ">=",
          rightMetric: "weapon.reload",
        },
        limit: 20,
      },
    });
    expect(itemMetricNode?.loadChildren).toBeUndefined();
    expect(commonTraitNode?.query).toEqual({
      kind: "listRecords",
      label: "Browse records with this trait",
      filters: {
        category: "spell",
        metadata: {
          field: "traits",
          op: "includesAny",
          values: ["fire"],
        },
        limit: 20,
      },
    });
    expect(saveTypeValueNode?.query).toEqual({
      kind: "listRecords",
      label: "Browse records with this value",
      filters: {
        category: "spell",
        metadata: {
          field: "saveType",
          op: "eq",
          value: "fortitude",
        },
        limit: 20,
      },
    });
    expect(publicationTitleValueNode?.query?.filters.metadata).toEqual({
      field: "publicationTitle",
      op: "eq",
      value: "Pathfinder Rage of Elements",
    });
    expect(saveTypeValueNode?.detailLines.map((line) => line.text)).toContain(
      "Press Enter or o to open the full matching set in the shared result reader.",
    );
  });
});
