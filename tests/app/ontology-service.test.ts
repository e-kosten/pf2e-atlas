import { describe, expect, it, vi } from "vitest";

import { createPf2eApplicationOntologyService } from "../../src/app/ontology-service.js";
import type { Pf2eDataService } from "../../src/data/service.js";
import type { AppConfig, FilterValueField, OntologyNode } from "../../src/types.js";

function createTestConfig(): AppConfig {
  return {
    dataPath: "vendor/pf2e",
    rootPath: "vendor/pf2e",
    manifestPath: "vendor/pf2e/system.pf2e.json",
    indexPath: ".cache/pf2e-index.sqlite",
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

function createDataService(): Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues"> {
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

    expect(dataService.listFilterValues).not.toHaveBeenCalled();
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

    expect(saveTypeValueNodes.find((node) => node.id === "spell:saveType:fortitude")?.query?.filters.metadata).toEqual({
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
  });
});
