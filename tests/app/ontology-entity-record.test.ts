import { describe, expect, it } from "vitest";

import {
  mapOntologyExplorerEntityRecordRow,
  type OntologyExplorerEntityRecordRow,
} from "../../src/app/ontology/entity-record.js";

function buildRow(overrides: Partial<OntologyExplorerEntityRecordRow> = {}): OntologyExplorerEntityRecordRow {
  return {
    recordKey: "spell:test-record",
    packName: "spells-srd",
    name: "Test Record",
    type: "spell",
    category: "spell",
    subcategory: null,
    documentType: "Item",
    level: 3,
    rarity: "common",
    traitsJson: '["arcane","attack"]',
    derivedTagsJson: '["damage_burst"]',
    familiesJson: "[]",
    descriptionText: "Description",
    blurbText: "Blurb",
    sourceCategory: "core",
    publicationTitle: "Player Core",
    publicationRemaster: 1,
    isUnique: 0,
    size: null,
    languagesJson: "[]",
    speedTypesJson: "[]",
    sensesJson: "[]",
    immunitiesJson: "[]",
    resistancesJson: "[]",
    weaknessesJson: "[]",
    itemCategory: null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: 2,
    usage: null,
    hands: null,
    damageTypesJson: '["fire"]',
    weaponGroup: null,
    armorGroup: null,
    traditionsJson: '["arcane"]',
    spellKindsJson: '["focus"]',
    saveType: "reflex",
    areaType: "cone",
    rangeText: "30 feet",
    durationText: "1 minute",
    targetText: null,
    areaValue: 30,
    sustained: 0,
    basicSave: 1,
    disableText: null,
    disableSkillsJson: "[]",
    isComplex: 0,
    actorMetricsJson:
      '[{"metricKey":"save.ref","valueType":"number","numberValue":12,"textValue":null,"boolValue":null}]',
    itemMetricsJson:
      '[{"metricKey":"weapon.reload","valueType":"number","numberValue":1,"textValue":null,"boolValue":null}]',
    rawJson: '{"system":{"slug":"test-record"}}',
    ...overrides,
  };
}

describe("ontology explorer entity record decoding", () => {
  it("maps validated row payloads into entity records", () => {
    const record = mapOntologyExplorerEntityRecordRow(buildRow());

    expect(record.category).toBe("spell");
    expect(record.sourceCategory).toBe("core");
    expect(record.traits).toEqual(["arcane", "attack"]);
    expect(record.damageTypes).toEqual(["fire"]);
    expect(record.traditions).toEqual(["arcane"]);
    expect(record.spellKinds).toEqual(["focus"]);
    expect(record.basicSave).toBe(true);
    expect(record.actorMetrics).toEqual({ "save.ref": 12 });
    expect(record.itemMetrics).toEqual({ "weapon.reload": 1 });
    expect(record.raw).toEqual({ system: { slug: "test-record" } });
  });

  it("rejects invalid category and subcategory combinations", () => {
    expect(() =>
      mapOntologyExplorerEntityRecordRow(
        buildRow({
          subcategory: "action",
        }),
      ),
    ).toThrow('Invalid search subcategory "action" for spell ontology explorer record');
  });

  it("rejects invalid source categories", () => {
    expect(() =>
      mapOntologyExplorerEntityRecordRow(
        buildRow({
          sourceCategory: "not-real",
        }),
      ),
    ).toThrow('Invalid source category "not-real" for ontology explorer record');
  });

  it("rejects malformed array payloads", () => {
    expect(() =>
      mapOntologyExplorerEntityRecordRow(
        buildRow({
          traitsJson: '{"bad":true}',
        }),
      ),
    ).toThrow('Expected traitsJson for ontology explorer record "spell:test-record" to be a JSON string array.');
  });

  it("preserves empty metric and raw JSON behavior", () => {
    const record = mapOntologyExplorerEntityRecordRow(
      buildRow({
        actorMetricsJson: null,
        itemMetricsJson: "",
        rawJson: null,
      }),
    );

    expect(record.actorMetrics).toEqual({});
    expect(record.itemMetrics).toEqual({});
    expect(record.raw).toEqual({});
  });

  it("rejects malformed metric and raw object payloads", () => {
    expect(() =>
      mapOntologyExplorerEntityRecordRow(
        buildRow({
          actorMetricsJson: '{"bad":true}',
        }),
      ),
    ).toThrow('Expected actorMetricsJson for ontology explorer record "spell:test-record" to be a JSON metric row array.');

    expect(() =>
      mapOntologyExplorerEntityRecordRow(
        buildRow({
          rawJson: "[]",
        }),
      ),
    ).toThrow('Expected rawJson for ontology explorer record "spell:test-record" to be a JSON object.');
  });
});
