import { describe, expect, it } from "vitest";

import { buildAonSearchLink, type AonSearchRecordLike } from "../../src/app/external-links/aon-search.js";
import {
  buildOntologyExplorerEntityDetailLines,
  type OntologyExplorerEntityRecord,
} from "../../src/app/ontology/entity-record.js";

function createAonSearchRecord(overrides: Partial<AonSearchRecordLike> = {}): AonSearchRecordLike {
  return {
    name: "Alarm Ward",
    category: "spell",
    subcategory: null,
    rarity: "rare",
    traits: ["Mental", "auditory", "fortune", "mental"],
    traditions: ["Occult", "arcane", "occult"],
    actionCost: 2,
    ...overrides,
  };
}

function createOntologyEntityRecord(overrides: Partial<OntologyExplorerEntityRecord> = {}): OntologyExplorerEntityRecord {
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
    traits: ["Mental", "auditory", "fortune", "mental"],
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
    traditions: ["Occult", "arcane", "occult"],
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

describe("AoN search links", () => {
  it("builds a spell search link with full trait carry-through and stable filter ordering", () => {
    const link = buildAonSearchLink(createAonSearchRecord());

    expect(link).not.toBeNull();
    expect(link?.appliedFilters).toEqual({
      query: "Alarm Ward",
      includeTypes: ["spell", "cantrip", "ritual"],
      includeTraits: ["auditory", "fortune", "mental"],
      includeRarities: ["rare"],
      includeTraditions: ["arcane", "occult"],
      includeActions: ["2"],
    });

    const url = new URL(link!.url);
    expect(`${url.origin}${url.pathname}`).toBe("https://2e.aonprd.com/Search.aspx");
    expect(url.searchParams.get("display")).toBe("short");
    expect(url.searchParams.get("q")).toBe("Alarm Ward");
    expect(url.searchParams.get("include-types")).toBe("spell cantrip ritual");
    expect(url.searchParams.get("include-traits")).toBe("auditory fortune mental");
    expect(url.searchParams.get("include-rarities")).toBe("rare");
    expect(url.searchParams.get("include-traditions")).toBe("arcane occult");
    expect(url.searchParams.get("include-actions")).toBe("2");
  });

  it("omits empty structured filters but still produces a generic AoN search link", () => {
    const link = buildAonSearchLink(
      createAonSearchRecord({
        name: "Chronicle Note",
        category: "lore",
        rarity: "common",
        traits: [],
        traditions: [],
        actionCost: null,
      }),
    );

    expect(link).not.toBeNull();
    expect(link?.appliedFilters).toEqual({
      query: "Chronicle Note",
      includeTypes: [],
      includeTraits: [],
      includeRarities: [],
      includeTraditions: [],
      includeActions: [],
    });

    const url = new URL(link!.url);
    expect(url.searchParams.get("q")).toBe("Chronicle Note");
    expect(url.searchParams.has("include-types")).toBe(false);
    expect(url.searchParams.has("include-traits")).toBe(false);
    expect(url.searchParams.has("include-rarities")).toBe(false);
    expect(url.searchParams.has("include-traditions")).toBe(false);
    expect(url.searchParams.has("include-actions")).toBe(false);
  });

  it("returns null when the record name is blank", () => {
    expect(
      buildAonSearchLink(
        createAonSearchRecord({
          name: "   ",
        }),
      ),
    ).toBeNull();
  });

  it("adds the AoN section to shared ontology detail lines", () => {
    const lines = buildOntologyExplorerEntityDetailLines(createOntologyEntityRecord());

    expect(lines.some((line) => line.text === "Archives of Nethys" && line.tone === "section")).toBe(true);
    expect(lines.some((line) => line.text.includes("https://2e.aonprd.com/Search.aspx?display=short"))).toBe(true);
    expect(lines.some((line) => line.text.includes("include-traits=auditory+fortune+mental"))).toBe(true);
  });
});
