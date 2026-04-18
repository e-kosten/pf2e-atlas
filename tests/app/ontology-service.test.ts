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
  OntologyNode,
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
    expect(saveTypeValueNode?.loadChildren).toBeUndefined();
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
    const dataService: Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues"> = {
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
    };

    const service = createPf2eApplicationOntologyService(createTestConfig(), dataService);
    const domain = service.loadDomain("searchSemantics");
    const traitFieldNode = findNodeById(domain.rootNodes, "spell:field:traits");
    const traitValueNodes = traitFieldNode?.loadChildren?.() ?? [];

    expect(traitValueNodes).toHaveLength(14);
    expect(traitValueNodes.at(-1)?.id).toBe("spell:traits:trait-14");
  });
});
