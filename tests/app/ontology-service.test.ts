import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it, vi } from "vitest";

import { createPf2eApplicationOntologyService } from "../../src/app/ontology-service.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import { getMetadataGlossaryArtifactPath } from "../../src/data/metadata-glossary.js";
import type { SearchSemanticsBootstrapSummaryResult } from "../../src/data/vocabulary.js";
import type { Pf2eDataService } from "../../src/data/service.js";
import { buildAllOfFilter, buildScopeFilter, type SearchFilterNode, type SearchRequest } from "../../src/domain/search-request-types.js";
import type { FilterValueField, SearchResult } from "../../src/domain/search-types.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { MetadataGlossaryArtifact } from "../../src/domain/metadata-glossary-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { OntologyNode } from "../../src/domain/ontology-types.js";

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

function getRequestFilter(request: SearchRequest | undefined): SearchFilterNode | undefined {
  return request?.filter;
}

function createSummary(): SearchSemanticsBootstrapSummaryResult {
  return {
    categories: [
      { value: "spell", count: 12 },
      { value: "equipment", count: 5 },
      { value: "creature", count: 3 },
    ],
    subcategoryCountsByCategory: [
      {
        category: "spell",
        subcategories: [{ value: "action", count: 6 }],
      },
      {
        category: "equipment",
        subcategories: [{ value: "gear", count: 5 }],
      },
      {
        category: "creature",
        subcategories: [],
      },
    ],
    commonTraitsByCategory: [{ category: "spell", traits: [{ value: "fire", count: 4 }] }],
    commonDerivedTagsByCategory: [{ category: "spell", tags: [{ value: "alarm", count: 2 }] }],
    derivedTagCatalog: [],
  };
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

function createDataService(): Pick<Pf2eDataService, "getSearchSemanticsBootstrapSummary" | "listFilterValues" | "listRecords"> {
  const summary = createSummary();
  const listRecords = vi.fn((request: SearchRequest) => ({
    searchProfile: null,
    mode: "structured" as const,
    sort: request.sort ?? "alphabetical",
    total: 1,
    offset: request.offset ?? 0,
    limit: request.limit ?? 20,
    hasMore: false,
    nextOffset: null,
    records: [
      createRecord(
        request.category === "equipment"
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
          : request.category === "creature"
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
  }) satisfies SearchResult);
  return {
    getSearchSemanticsBootstrapSummary: vi.fn(() => summary),
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

function createDiscoveryService(dataService: Pick<Pf2eDataService, "listFilterValues">) {
  return createPf2eApplicationSearchDiscoveryService({
    discoverFilterValues: vi.fn(async (query) => dataService.listFilterValues(query)),
    getPack: vi.fn(() => undefined),
    listFilterValues: dataService.listFilterValues,
  });
}

describe("application ontology service", () => {
  it("exposes an explicit search-semantics loader", () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );

    expect(typeof service.loadSearchSemanticsDomain).toBe("function");
    expect(typeof service.loadSearchFilterExplorerDomain).toBe("function");
  });

  it("caches contextual search filter explorer models by request and discovery mode", async () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );
    const request: SearchRequest = {
      mode: "browse",
      filter: buildScopeFilter("spell"),
      limit: 20,
    };

    const first = await service.loadSearchFilterExplorerDomain({
      request,
      discoveryMode: "matching",
    });
    const second = await service.loadSearchFilterExplorerDomain({
      request,
      discoveryMode: "matching",
    });
    const catalog = await service.loadSearchFilterExplorerDomain({
      request,
      discoveryMode: "catalog",
    });

    expect(second).toBe(first);
    expect(catalog).not.toBe(first);
    expect(first.rootNodes).toEqual([expect.objectContaining({ id: "searchSemantics:spell" })]);
  });

  it("keeps the derived-tag explorer populated for broad creature scope in matching mode", async () => {
    const summary: SearchSemanticsBootstrapSummaryResult = {
      categories: [{ value: "creature", count: 3 }],
      subcategoryCountsByCategory: [{ category: "creature", subcategories: [] }],
      commonTraitsByCategory: [],
      commonDerivedTagsByCategory: [],
      derivedTagCatalog: [
        {
          category: "creature",
          family: "threat_profile",
          axis: "threat",
          description: "Threat-flavored creature tags.",
          tags: [{ value: "undead_adjacent" }],
        },
      ],
    };
    const dataService: Pick<
      Pf2eDataService,
      "getSearchSemanticsBootstrapSummary" | "listFilterValues" | "listRecords"
    > = {
      getSearchSemanticsBootstrapSummary: vi.fn(() => summary),
      listFilterValues: vi.fn(
        ({
          field,
          category,
        }: {
          field: FilterValueField;
          category?: string;
        }) => ({
          field,
          values:
            field === "derivedTags" && category === "creature"
              ? [{ value: "undead_adjacent", count: 3 }]
              : [],
        }),
      ),
      listRecords: vi.fn((request: SearchRequest) => ({
        searchProfile: null,
        mode: "structured" as const,
        sort: request.sort ?? "alphabetical",
        total: 0,
        offset: request.offset ?? 0,
        limit: request.limit ?? 20,
        hasMore: false,
        nextOffset: null,
        records: [],
      }) satisfies SearchResult),
    };
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );

    const domain = await service.loadSearchFilterExplorerDomain({
      request: {
        mode: "search",
        search: { query: "", profile: "balanced" },
        filter: buildScopeFilter("creature"),
        limit: 20,
      },
      discoveryMode: "matching",
    });

    const derivedTagsFieldNode = findNodeById(domain.rootNodes, "creature:field:derivedTags");
    const familyNode = derivedTagsFieldNode?.children?.[0];
    const tagNode = familyNode?.children?.[0];

    expect(derivedTagsFieldNode?.childPresentation).toEqual({
      mode: "grouped",
      groupBy: "axis",
      render: "inline",
    });
    expect(derivedTagsFieldNode?.children).toHaveLength(1);
    expect(familyNode).toEqual(
      expect.objectContaining({
        kind: "family",
        label: "Threat Profile",
      }),
    );
    expect(tagNode).toEqual(
      expect.objectContaining({
        kind: "tag",
        label: "Undead Adjacent",
        listLabel: "Undead Adjacent | 3",
      }),
    );
    expect(dataService.listFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({
        field: "derivedTags",
        category: "creature",
      }),
    );
  });

  it("caches ontology domain models across repeated loads", async () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );

    const first = await service.loadSearchSemanticsDomain();
    const second = await service.loadSearchSemanticsDomain();

    expect(second).toBe(first);
  });

  it("reuses the in-flight ontology domain load", async () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );

    const first = service.loadSearchSemanticsDomain();
    const second = service.loadSearchSemanticsDomain();

    expect(second).toBe(first);
    expect(await second).toBe(await first);
  });

  it("loads search semantics without any derived-tag explorer storage dependency", async () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );

    await service.loadSearchSemanticsDomain();
  });

  it("builds valid field-specific browse queries for search semantics values", async () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );
    const domain = await service.loadSearchSemanticsDomain();
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
      fieldType: "Enumerated String",
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
    expect(saveTypeFieldNode?.listLabel).toBe("Save Type");
    expect(saveTypeFieldNode?.detailLines.map((line) => line.text)).not.toContain("Operators: eq, in, notIn");

    const saveTypeValueNode = saveTypeValueNodes.find((node) => node.id === "spell:saveType:fortitude");

    expect(getRequestFilter(saveTypeValueNode?.query?.request)).toEqual(
      buildAllOfFilter([
        buildScopeFilter("spell"),
        { kind: "metadataPredicate", predicate: { field: "saveType", op: "eq", value: "fortitude" } },
      ]),
    );
    expect(getRequestFilter(sustainedValueNodes.find((node) => node.id === "spell:sustained:true")?.query?.request)).toEqual(
      buildAllOfFilter([
        buildScopeFilter("spell"),
        { kind: "metadataPredicate", predicate: { field: "sustained", op: "eq", value: true } },
      ]),
    );
    expect(getRequestFilter(handsValueNodes.find((node) => node.id === "equipment:hands:1")?.query?.request)).toEqual(
      buildAllOfFilter([
        buildScopeFilter("equipment"),
        { kind: "metadataPredicate", predicate: { field: "hands", op: "eq", value: 1 } },
      ]),
    );
    const commonTraitNode = findNodeById(domain.rootNodes, "spell:commonTraits")?.children?.[0];
    expect(getRequestFilter(commonTraitNode?.query?.request)).toEqual(
      buildAllOfFilter([
        buildScopeFilter("spell"),
        { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "fire" } },
      ]),
    );
    expect(saveTypeValueNode?.loadChildren?.()[0]?.kind).toBe("record");
    expect(saveTypeValueNode?.detailLines.map((line) => line.text)).toContain(
      "Press Enter or o to open the full matching set in the shared result reader.",
    );
  });

  it("enriches trait nodes from the metadata glossary artifact when available", async () => {
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

      const dataService = createDataService();
      const service = createPf2eApplicationOntologyService(config, dataService, createDiscoveryService(dataService));
      const domain = await service.loadSearchSemanticsDomain();
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

  it("loads the full discoverable value set instead of truncating to a common subset", async () => {
    const values = Array.from({ length: 14 }, (_, index) => ({
      value: `trait-${index + 1}`,
      count: index + 1,
    }));
    const dataService: Pick<Pf2eDataService, "getSearchSemanticsBootstrapSummary" | "listFilterValues" | "listRecords"> = {
      getSearchSemanticsBootstrapSummary: vi.fn(() => ({
        categories: [{ value: "spell", count: 14 }],
        subcategoryCountsByCategory: [{ category: "spell", subcategories: [] }],
        commonTraitsByCategory: [],
        commonDerivedTagsByCategory: [],
        derivedTagCatalog: [],
      })),
      listFilterValues: vi.fn(({ field, category }: { field: FilterValueField; category?: string }) => ({
        field,
        values: field === "traits" && category === "spell" ? values : [],
      })),
      listRecords: vi.fn((request: SearchRequest) => ({
        searchProfile: null,
        mode: "structured" as const,
        sort: request.sort ?? "alphabetical",
        total: 0,
        offset: request.offset ?? 0,
        limit: request.limit ?? 20,
        hasMore: false,
        nextOffset: null,
        records: [],
      }) satisfies SearchResult),
    };

    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );
    const domain = await service.loadSearchSemanticsDomain();
    const traitFieldNode = findNodeById(domain.rootNodes, "spell:field:traits");
    const traitValueNodes = traitFieldNode?.loadChildren?.() ?? [];

    expect(traitValueNodes).toHaveLength(14);
    expect(traitValueNodes[0]?.id).toBe("spell:traits:trait-14");
  });

  it("uses live record inspection and live metric discovery instead of shallow examples", async () => {
    const dataService = createDataService();
    const service = createPf2eApplicationOntologyService(
      createTestConfig(),
      dataService,
      createDiscoveryService(dataService),
    );
    const domain = await service.loadSearchSemanticsDomain();
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
    expect(findNodeById(domain.rootNodes, "spell:booleanGroups")).toBeUndefined();
    expect(findNodeById(domain.rootNodes, "creature:advancedPredicates")).toBeUndefined();
    expect(findNodeById(domain.rootNodes, "equipment:advancedPredicates")).toBeUndefined();
    expect(actorMetricGroup?.detailLines.map((line) => line.text)).toContain(
      "Explore live creature statistics namespaces, keys, and exact scalar values from the indexed corpus.",
    );
    expect(actorMetricNamespace?.loadChildren?.().map((node) => node.label)).toContain("Best Save");
    expect(actorMetricNode?.label).toBe("Best Save");
    expect(actorMetricValueNode?.query).toEqual({
      label: "Browse records where Best Save = fort",
      request: {
        mode: "browse",
        filter: {
          kind: "allOf",
          children: [
            {
              kind: "scope",
              category: "creature",
              subcategory: { kind: "any" },
            },
            {
              kind: "metric",
              metric: "save.best",
              op: "eq",
              value: "fort",
            },
          ],
        },
        limit: 20,
      },
    });
    expect(actorMetricValueNode?.loadChildren?.()[0]?.kind).toBe("record");
    expect(itemMetricNode?.label).toBe("Weapon Reload");
    expect(itemMetricNode?.query).toEqual({
      label: "Browse records with Weapon Reload",
      request: {
        mode: "browse",
        filter: {
          kind: "allOf",
          children: [
            {
              kind: "scope",
              category: "equipment",
              subcategory: { kind: "any" },
            },
            {
              kind: "metricCompare",
              leftMetric: "weapon.reload",
              op: "gte",
              rightMetric: "weapon.reload",
            },
          ],
        },
        limit: 20,
      },
    });
    expect(itemMetricNode?.loadChildren).toBeUndefined();
    expect(commonTraitNode?.query).toEqual({
      label: "Browse records with this trait",
      request: {
        mode: "browse",
        filter: {
          kind: "allOf",
          children: [
            {
              kind: "scope",
              category: "spell",
              subcategory: { kind: "any" },
            },
            {
              kind: "metadataPredicate",
              predicate: {
                field: "traits",
                op: "includes",
                value: "fire",
              },
            },
          ],
        },
        limit: 20,
      },
    });
    expect(saveTypeValueNode?.query).toEqual({
      label: "Browse records with this value",
      request: {
        mode: "browse",
        filter: {
          kind: "allOf",
          children: [
            {
              kind: "scope",
              category: "spell",
              subcategory: { kind: "any" },
            },
            {
              kind: "metadataPredicate",
              predicate: {
                field: "saveType",
                op: "eq",
                value: "fortitude",
              },
            },
          ],
        },
        limit: 20,
      },
    });
    expect(getRequestFilter(publicationTitleValueNode?.query?.request)).toEqual(
      buildAllOfFilter([
        buildScopeFilter("spell"),
        {
          kind: "metadataPredicate",
          predicate: {
            field: "publicationTitle",
            op: "eq",
            value: "Pathfinder Rage of Elements",
          },
        },
      ]),
    );
    expect(saveTypeValueNode?.detailLines.map((line) => line.text)).toContain(
      "Press Enter or o to open the full matching set in the shared result reader.",
    );
  });
});
