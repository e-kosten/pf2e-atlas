import { describe, expect, it } from "vitest";

import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { Pf2eTerminalSearchQuery, Pf2eTerminalSearchSession } from "../../src/tui/search/service-types.js";
import { buildResultDetailLines } from "../../src/tui/search-screen/results.js";

function createRecord(overrides: Partial<NormalizedRecord> = {}): NormalizedRecord {
  return {
    recordKey: "spell:test-alarm-ward",
    id: "test-alarm-ward",
    name: "Alarm Ward",
    normalizedName: "alarm ward",
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spell",
    packLabel: "Spells",
    documentType: "Item",
    level: 1,
    rarity: "rare",
    traits: ["fortune", "mental"],
    derivedTags: ["alarm"],
    publicationTitle: "Player Core",
    publicationRemaster: true,
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
    saveType: null,
    areaType: null,
    rangeText: "30 feet",
    durationText: "1 minute",
    durationUnit: "minute",
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
    rangeValue: 30,
    aliases: [],
    legacyRecordLinks: [],
    raw: {},
    ...overrides,
  };
}

function createQuery(): Pf2eTerminalSearchQuery {
  return {
    mode: "search",
    limit: 20,
    queryText: "alarm ward",
    searchProfile: "balanced",
    sourceLabel: null,
    filters: {
      category: "spell",
      parts: [],
    },
  };
}

function createSession(record: NormalizedRecord): Pf2eTerminalSearchSession {
  return {
    windowId: "window-1",
    query: createQuery(),
    results: [record],
    windowOffset: 0,
    resultMode: "hybrid",
    total: 1,
    loadedCount: 1,
    hasMore: false,
    nextOffset: null,
    searchProfile: "balanced",
    sort: "ranked",
    sortSeed: null,
  };
}

describe("search result detail lines", () => {
  it("reuses the shared ontology detail presenter so result previews include the AoN link metadata", () => {
    const lines = buildResultDetailLines(createRecord(), createSession(createRecord()), 0);
    const linkLine = lines.find((line) => "href" in line) as
      | (typeof lines)[number] & { href?: string; plainTextFallback?: string }
      | undefined;

    expect(lines[0]?.text).toBe("Result Preview");
    expect(lines.some((line) => line.text === "Archives of Nethys" && line.tone === "section")).toBe(true);
    expect(linkLine).toMatchObject({
      text: "Open in Archives of Nethys",
      href: expect.stringContaining("https://2e.aonprd.com/Search.aspx?display=short&type=eqs"),
      plainTextFallback: expect.stringContaining("Open in Archives of Nethys: https://2e.aonprd.com"),
    });
    expect(linkLine?.href).toContain("include-traits=fortune+mental");
  });
});
