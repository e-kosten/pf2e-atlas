import { describe, expect, it } from "vitest";

import {
  buildEntityPageDocument,
  renderEntityPageDocument,
} from "../../src/app/ontology/entity-page.js";
import type { OntologyExplorerEntityRecord } from "../../src/app/ontology/entity-record.js";
import type { PageRelationsResult } from "../../src/domain/page-relations-types.js";
import { buildPageDocumentModel, renderPageDocumentModel } from "../../src/tui/page-document/model.js";

function createRecord(overrides: Partial<OntologyExplorerEntityRecord> = {}): OntologyExplorerEntityRecord {
  return {
    recordKey: "spell:test-fireball",
    packName: "spells",
    name: "Fireball",
    type: "spell",
    category: "spell",
    subcategory: null,
    documentType: "Item",
    level: 3,
    rarity: "common",
    traits: ["Concentrate", "Fire", "Manipulate"],
    derivedTags: ["explosive_magic"],
    families: ["damage_burst"],
    descriptionText: "A roaring blast of fire detonates at a spot you designate.",
    blurbText: "A compact burst spell.",
    sourceCategory: "core",
    publicationTitle: "Pathfinder Player Core",
    publicationRemaster: true,
    isUnique: false,
    size: null,
    languages: [],
    speedTypes: [],
    senses: [],
    immunities: [],
    resistances: [],
    weaknesses: [],
    itemCategory: null,
    baseItem: null,
    priceCp: null,
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
    targetText: null,
    areaValue: 20,
    sustained: false,
    basicSave: true,
    disableText: null,
    disableSkills: [],
    isComplex: false,
    ...overrides,
  };
}

function createRelations(): PageRelationsResult {
  return {
    recordKey: "spell:test-fireball",
    outgoing: {
      records: [
        {
          recordKey: "spell:fireball-effect",
          id: "spell:fireball-effect",
          name: "Fireball Effect",
          normalizedName: "fireball effect",
          type: "spell",
          category: "spell",
          subcategory: null,
          packName: "spells",
          packLabel: "Spells",
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
          sourcePath: "packs/spells/fireball-effect.json",
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
        },
      ],
      edges: [
        {
          fromRecordKey: "spell:test-fireball",
          toRecordKey: "spell:fireball-effect",
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
                kind: "linkedFrom",
                source: "spell:test-fireball",
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

describe("page document model", () => {
  it("compiles entity page documents into section anchors and target nodes", () => {
    const model = buildPageDocumentModel(buildEntityPageDocument(createRecord(), createRelations()));

    expect(model.sections.map((section) => section.id)).toEqual([
      "summary",
      "description",
      "details",
      "references",
      "backlinks",
      "classification",
    ]);
    expect(model.sectionAnchors.map((anchor) => anchor.sectionId)).toEqual([
      "summary",
      "description",
      "details",
      "references",
      "backlinks",
      "classification",
    ]);
    expect(model.targetNodes.map((node) => node.target.label)).toEqual([
      "Spell Effect: Fireball",
      "Feat (2)",
      "Pack: spells",
      "Category: Spell",
      "Derived Tags: Explosive Magic",
      "Families: Damage Burst",
    ]);
  });

  it("renders the compiled model back to the legacy compatible line output", () => {
    const document = buildEntityPageDocument(createRecord(), createRelations());

    expect(renderPageDocumentModel(buildPageDocumentModel(document))).toEqual(renderEntityPageDocument(document));
  });
});
