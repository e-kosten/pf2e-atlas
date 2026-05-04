import { describe, expect, it } from "vitest";

import { edgeRowToPageReferenceEdge, edgeRowToReferenceEdge, type CandidateRow, rowToRecord } from "../../src/data/rows.js";

function buildCandidateRow(overrides: Partial<CandidateRow> = {}): CandidateRow {
  return {
    recordKey: "spell:test-record",
    id: "test-record",
    name: "Test Record",
    normalizedName: "test record",
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spells-srd",
    packLabel: "Spells",
    documentType: "Item",
    level: 3,
    rarity: "common",
    traitsJson: '["arcane","attack"]',
    derivedTagsJson: '["damage_burst"]',
    publicationTitle: "Player Core",
    publicationRemaster: 1,
    descriptionText: "Test description",
    blurbText: null,
    hasDescription: 1,
    descriptionSnippet: "snippet",
    sourceCategory: "core",
    folderId: null,
    familiesJson: null,
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxesJson: '["tradition"]',
    variantConfidence: null,
    variantSource: "none",
    sourcePath: "packs/spells/test-record.json",
    isUnique: 0,
    size: null,
    itemCategory: null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: 2,
    usage: null,
    hands: null,
    itemMetricsJson:
      '[{"metricKey":"weapon.reload","valueType":"number","numberValue":1,"textValue":null,"boolValue":null}]',
    damageTypesJson: '["fire"]',
    weaponGroup: null,
    armorGroup: null,
    traditionsJson: '["arcane"]',
    spellKindsJson: '["focus"]',
    rangeText: "30 feet",
    saveType: "reflex",
    areaType: "cone",
    durationText: "1 minute",
    durationUnit: "minute",
    targetText: null,
    areaValue: 30,
    sustained: 0,
    basicSave: 1,
    languagesJson: "[]",
    speedTypesJson: "[]",
    sensesJson: "[]",
    immunitiesJson: "[]",
    resistancesJson: "[]",
    weaknessesJson: "[]",
    disableText: null,
    disableSkillsJson: "[]",
    isComplex: 0,
    actorMetricsJson:
      '[{"metricKey":"save.ref","valueType":"number","numberValue":12,"textValue":null,"boolValue":null}]',
    rangeValue: 30,
    rawJson: '{"system":{"slug":"test-record"}}',
    ...overrides,
  };
}

describe("rowToRecord", () => {
  it("decodes canonical row metadata through checked JSON helpers", () => {
    const record = rowToRecord(buildCandidateRow());

    expect(record.category).toBe("spell");
    expect(record.traits).toEqual(["arcane", "attack"]);
    expect(record.derivedTags).toEqual(["damage_burst"]);
    expect(record.variantAxes).toEqual(["tradition"]);
    expect(record.damageTypes).toEqual(["fire"]);
    expect(record.traditions).toEqual(["arcane"]);
    expect(record.spellKinds).toEqual(["focus"]);
    expect(record.basicSave).toBe(true);
    expect(record.sustained).toBe(false);
    expect(record.itemMetrics).toEqual({ "weapon.reload": 1 });
    expect(record.actorMetrics).toEqual({ "save.ref": 12 });
    expect(record.raw).toEqual({ system: { slug: "test-record" } });
  });

  it("rejects mismatched category and subcategory pairs", () => {
    expect(() =>
      rowToRecord(
        buildCandidateRow({
          subcategory: "action",
        }),
      ),
    ).toThrow('Invalid row subcategory "action" for spell record');
  });

  it("rejects malformed metadata arrays", () => {
    expect(() =>
      rowToRecord(
        buildCandidateRow({
          traitsJson: '{"not":"an array"}',
        }),
      ),
    ).toThrow('Expected traitsJson for "spell:test-record" to be a JSON string array.');
  });

  it("rejects invalid variant source values", () => {
    expect(() =>
      rowToRecord(
        buildCandidateRow({
          variantSource: "mystery",
        }),
      ),
    ).toThrow('Invalid row variant source "mystery"');
  });
});

describe("edgeRowToReferenceEdge", () => {
  it("validates reference source categories while decoding edges", () => {
    expect(
      edgeRowToReferenceEdge(
        {
          fromRecordKey: "spell:test-record",
          toRecordKey: "action:raise-shield",
          displayText: null,
          referenceText: "ref",
          fromPackName: "spells-srd",
          fromRecordType: "spell",
          fromDocumentType: "Item",
          fromSourceCategory: "rules",
        },
        "outgoing",
      ),
    ).toMatchObject({
      sourceCategory: "rules",
      relationshipType: "references",
    });
  });
});

describe("edgeRowToPageReferenceEdge", () => {
  it("maps raw incoming edges without the rule-graph-specific direction enum", () => {
    expect(
      edgeRowToPageReferenceEdge(
        {
          fromRecordKey: "spell:test-record",
          toRecordKey: "action:raise-shield",
          displayText: null,
          referenceText: "ref",
          fromPackName: "spells-srd",
          fromRecordType: "spell",
          fromDocumentType: "Item",
          fromSourceCategory: "rules",
        },
        "incoming",
      ),
    ).toMatchObject({
      direction: "incoming",
      relationshipType: "referenced_by",
      sourceCategory: "rules",
    });
  });
});
