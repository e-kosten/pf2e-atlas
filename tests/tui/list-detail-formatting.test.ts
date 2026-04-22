import { describe, expect, it } from "vitest";

import type { NormalizedRecord } from "../../src/domain/record-types.js";
import { buildSearchResultRowLine, formatTerminalBreadcrumb } from "../../src/tui/list-detail-formatting.js";

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
  it("formats shared breadcrumbs through one helper", () => {
    expect(formatTerminalBreadcrumb(["Search Semantics", "Creature", "Derived Tags"])).toBe(
      "Search Semantics > Creature > Derived Tags",
    );
  });

  it("builds compact search-result rows without repeating scope text", () => {
    const line = buildSearchResultRowLine(createRecord(), { selected: false });

    expect(line.text).toBe("Guardian Adept | L4 | Rare | Pathfinder Monster Core");
    expect(line.text).not.toContain("creature/npc");
  });
});
