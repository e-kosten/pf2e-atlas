import { describe, expect, it, vi } from "vitest";

import { createPf2eApplicationEntityPageService } from "../../src/app/ontology/entity-page-service.js";
import type { PageRelationsResult } from "../../src/domain/page-relations-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";

function createRecord(overrides: Partial<NormalizedRecord> = {}): NormalizedRecord {
  return {
    recordKey: "spell:test-fireball",
    id: "spell:test-fireball",
    name: "Fireball",
    normalizedName: "fireball",
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spells",
    packLabel: "Spells",
    documentType: "Item",
    level: 3,
    rarity: "common",
    traits: ["concentrate", "fire", "manipulate"],
    derivedTags: ["explosive_magic"],
    publicationTitle: "Pathfinder Player Core",
    publicationRemaster: true,
    descriptionText: "A roaring blast of fire detonates at a spot you designate.",
    blurbText: "A compact burst spell.",
    hasDescription: true,
    descriptionSnippet: "A compact burst spell.",
    sourceCategory: "core",
    folderId: null,
    families: ["damage_burst"],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: "packs/spells/fireball.json",
    isUnique: false,
    size: null,
    itemCategory: null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: 2,
    usage: null,
    hands: null,
    damageTypes: ["fire"],
    weaponGroup: null,
    armorGroup: null,
    traditions: ["arcane", "primal"],
    spellKinds: ["spell"],
    saveType: "reflex",
    areaType: "burst",
    rangeText: "500 feet",
    durationText: null,
    durationUnit: null,
    targetText: null,
    areaValue: 20,
    sustained: false,
    basicSave: true,
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
    rangeValue: 500,
    aliases: [],
    legacyRecordLinks: [],
    raw: {},
    ...overrides,
  };
}

function createRelations(): PageRelationsResult {
  return {
    recordKey: "spell:test-fireball",
    outgoing: {
      records: [
        createRecord({
          recordKey: "spells:fireball-effect",
          id: "spells:fireball-effect",
          name: "Fireball Effect",
          normalizedName: "fireball effect",
          level: null,
          actionCost: null,
          families: [],
          derivedTags: [],
          traits: [],
          traditions: [],
          areaType: null,
          areaValue: null,
          rangeText: null,
          sourcePath: "packs/spells/fireball-effect.json",
        }),
      ],
      edges: [
        {
          fromRecordKey: "spell:test-fireball",
          toRecordKey: "spells:fireball-effect",
          displayText: "Spell Effect: Fireball",
          referenceText: "Fireball",
          direction: "outgoing",
          relationshipType: "references",
          sourcePackName: "spells",
          sourceRecordType: "spell",
          sourceDocumentType: "Item",
          sourceCategory: "core",
        },
      ],
    },
    incoming: {
      records: [],
      edges: [],
    },
    edges: [],
    incomingGroups: [
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
                target: "spell:test-fireball",
              },
            ],
          },
          sort: { kind: "alphabetical" },
          limit: 50,
        },
      },
    ],
  };
}

describe("entity page service", () => {
  it("composes normalized records with page relations into relation-aware preview lines", () => {
    const loadPageRelations = vi.fn(() => createRelations());
    const service = createPf2eApplicationEntityPageService({ loadPageRelations });
    const lines = service.buildDetailLines(createRecord());

    expect(loadPageRelations).toHaveBeenCalledWith("spell:test-fireball");
    expect(lines.some((line) => line.text === "References" && line.tone === "section")).toBe(true);
    expect(lines.some((line) => line.text === "Spell Effect: Fireball")).toBe(true);
    expect(lines.some((line) => line.text === "Referenced By" && line.tone === "section")).toBe(true);
    expect(lines.some((line) => line.text === "Feat (2)")).toBe(true);
    expect(lines.some((line) => line.text === "Derived Tags: Explosive Magic")).toBe(true);
  });

  it("builds a document directly from a record key when a record lookup is available", () => {
    const record = createRecord();
    const service = createPf2eApplicationEntityPageService({
      loadPageRelations: vi.fn(() => createRelations()),
      getRecord: vi.fn((recordKey) => (recordKey === record.recordKey ? record : undefined)),
    });

    const document = service.buildDocumentByRecordKey(record.recordKey);

    expect(document?.recordKey).toBe(record.recordKey);
    expect(document?.title).toBe("Fireball");
  });

  it("can build preview-intent documents from a record key when a record lookup is available", () => {
    const record = createRecord();
    const service = createPf2eApplicationEntityPageService({
      loadPageRelations: vi.fn(() => createRelations()),
      getRecord: vi.fn((recordKey) => (recordKey === record.recordKey ? record : undefined)),
    });

    const document = service.buildDocumentByRecordKey(record.recordKey, {
      recordTargetAction: "preview",
    });

    expect(document?.sections.find((section) => section.title === "References")?.targets).toEqual([
      {
        kind: "record",
        label: "Spell Effect: Fireball",
        recordKey: "spells:fireball-effect",
        action: "preview",
      },
    ]);
  });
});
