import { describe, expect, it } from "vitest";

import type { NormalizedRecord } from "../../src/domain/record-types.js";
import {
  buildSearchResultRowLine,
  buildTerminalListDetailGroupLine,
  buildTerminalListDetailMetadataLines,
  buildTerminalResultRowLine,
  formatTerminalBreadcrumb,
} from "../../src/tui/list-detail-formatting.js";

function createRecord(overrides: Partial<NormalizedRecord> = {}): NormalizedRecord {
  return {
    recordKey: "creature:test-guardian-adept",
    id: "test-guardian-adept",
    name: "Guardian Adept",
    normalizedName: "guardian adept",
    type: "npc",
    category: "creature",
    subcategory: "npc",
    packName: "pathfinder-monster-core",
    packLabel: "Pathfinder Monster Core",
    documentType: "Actor",
    level: 4,
    rarity: "rare",
    traits: [],
    derivedTags: [],
    publicationTitle: "Pathfinder Monster Core",
    publicationRemaster: true,
    descriptionText: "",
    blurbText: null,
    hasDescription: true,
    descriptionSnippet: "",
    sourceCategory: "core",
    folderId: null,
    families: [],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: "packs/creatures/guardian-adept.json",
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

describe("list detail formatting", () => {
  it("formats shared breadcrumbs through one helper with friendly fallback rendering", () => {
    expect(formatTerminalBreadcrumb(["searchSemantics", "characterCreation", "party_role"])).toBe(
      "Search Semantics > Character Creation > Party Role",
    );
  });

  it("builds compact search-result rows without repeating scope text", () => {
    const line = buildSearchResultRowLine(createRecord(), { selected: false });

    expect(line.text).toBe("Guardian Adept | L4 | Rare | Pathfinder Monster Core");
    expect(line.text).not.toContain("creature/npc");
  });

  it("formats shared row, group, and detail metadata hooks through the shared owner", () => {
    expect(
      buildTerminalResultRowLine("Guardian Adept", {
        selected: false,
        metadata: {
          subtitle: "Creature",
          badges: ["exact"],
          metadataParts: ["L4"],
        },
      }).text,
    ).toBe("Guardian Adept | Creature | L4 | [exact]");
    expect(buildTerminalListDetailGroupLine({ key: "exact", label: "Exact Matches" })).toEqual({
      text: "Exact Matches",
      tone: "section",
      noWrap: true,
    });
    expect(
      buildTerminalListDetailMetadataLines([
        { label: "Showing", value: "result 1/3" },
        { label: "Sort", value: "Ranked" },
      ]),
    ).toEqual([
      { text: "Showing: result 1/3", noWrap: true },
      { text: "Sort: Ranked", noWrap: true },
    ]);
  });
});
