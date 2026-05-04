import { describe, expect, it, vi } from "vitest";

import { createPf2eApplicationPageRelationsService } from "../../src/app/page-relations-service.js";
import type { PageReferenceCollectionResult } from "../../src/domain/page-relations-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";

function createRecord(
  overrides: Partial<NormalizedRecord> & Pick<NormalizedRecord, "recordKey" | "name" | "category">,
): NormalizedRecord {
  return {
    recordKey: overrides.recordKey,
    id: overrides.recordKey,
    name: overrides.name,
    normalizedName: overrides.name.toLowerCase(),
    type: overrides.category,
    category: overrides.category,
    subcategory: overrides.subcategory ?? null,
    packName: "test-pack",
    packLabel: "Test Pack",
    documentType: "Item",
    level: null,
    rarity: null,
    traits: [],
    derivedTags: [],
    publicationTitle: null,
    publicationRemaster: false,
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
    sourcePath: "test.json",
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
    aliases: [],
    legacyRecordLinks: [],
    raw: {},
    ...overrides,
  };
}

describe("page relations service", () => {
  it("groups incoming relations and seeds canonical browse requests with record keys", () => {
    const graph: PageReferenceCollectionResult = {
      outgoing: { records: [], edges: [] },
      incoming: {
        records: [
          createRecord({
            recordKey: "feat:deep-focus",
            name: "Deep Focus",
            category: "feat",
          }),
          createRecord({
            recordKey: "spell:focus-burst",
            name: "Focus Burst",
            category: "spell",
          }),
          createRecord({
            recordKey: "feat:meditative-well",
            name: "Meditative Well",
            category: "feat",
          }),
        ],
        edges: [],
      },
      edges: [],
    };

    const getReferenceEdges = vi.fn(() => graph);
    const service = createPf2eApplicationPageRelationsService({ getReferenceEdges });
    const relations = service.loadPageRelations("actions:action-refocus-1");

    expect(getReferenceEdges).toHaveBeenCalledWith(["actions:action-refocus-1"], {
      includeOutgoing: true,
      includeIncoming: true,
    });
    expect(relations.incomingGroups).toEqual([
      {
        category: "feat",
        subcategory: null,
        count: 2,
        request: {
          mode: "browse",
          filter: {
            kind: "allOf",
            children: [
              {
                kind: "scope",
                category: "feat",
                subcategory: { kind: "any" },
              },
              {
                kind: "linksTo",
                target: "actions:action-refocus-1",
              },
            ],
          },
          sort: { kind: "alphabetical" },
          limit: 50,
        },
      },
      {
        category: "spell",
        subcategory: null,
        count: 1,
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
                kind: "linksTo",
                target: "actions:action-refocus-1",
              },
            ],
          },
          sort: { kind: "alphabetical" },
          limit: 50,
        },
      },
    ]);
  });
});
