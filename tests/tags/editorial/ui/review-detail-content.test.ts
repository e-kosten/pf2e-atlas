import { describe, expect, it } from "vitest";

import type { OntologyExplorerEntityRecord } from "../../../../src/app/ontology/entity-record.js";
import type { DerivedTagReviewSessionRecord } from "../../../../src/tags/editorial/types.js";
import {
  buildDerivedTagMigrationRecordPageLines,
  buildDerivedTagMigrationRecordPageTextLines,
} from "../../../../src/tags/editorial/ui/review-detail-content.js";

function createEntityRecord(overrides: Partial<OntologyExplorerEntityRecord> = {}): OntologyExplorerEntityRecord {
  return {
    recordKey: "spell:test-alarm-ward",
    packName: "pathfinder-player-core",
    name: "Alarm Ward",
    type: "spell",
    category: "spell",
    subcategory: null,
    documentType: "Item",
    level: 1,
    rarity: "rare",
    traits: ["fortune", "mental"],
    derivedTags: ["alarm"],
    families: ["security"],
    descriptionText: "Warns against intruders.",
    blurbText: "A warning ward.",
    sourceCategory: "core",
    publicationTitle: "Player Core",
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
    damageTypes: [],
    weaponGroup: null,
    armorGroup: null,
    traditions: ["arcane"],
    spellKinds: ["spell"],
    saveType: null,
    areaType: null,
    rangeText: "30 feet",
    durationText: "1 minute",
    targetText: null,
    areaValue: null,
    sustained: false,
    basicSave: false,
    disableText: null,
    disableSkills: [],
    isComplex: false,
    ...overrides,
  };
}

function createReviewRecord(): DerivedTagReviewSessionRecord {
  return {
    entityRecord: createEntityRecord(),
    currentSources: {},
    selectionReasons: [],
  };
}

describe("review detail content", () => {
  it("renders linked ontology lines with a plain-text URL fallback", () => {
    const record = createReviewRecord();
    const lines = buildDerivedTagMigrationRecordPageLines(record);
    const textLines = buildDerivedTagMigrationRecordPageTextLines(record);
    const linkLine = lines.find((line) => line.href);

    expect(linkLine).toMatchObject({
      text: "Search Archives of Nethys for Alarm Ward",
      href: expect.stringContaining("https://2e.aonprd.com/Search.aspx?display=short&type=eqs"),
    });
    expect(
      textLines.some((line) =>
        line.includes("Search Archives of Nethys for Alarm Ward: https://2e.aonprd.com/Search.aspx?display=short&type=eqs"),
      ),
    ).toBe(true);
  });
});
