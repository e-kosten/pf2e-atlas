import { describe, expect, it } from "vitest";

import { buildSearchQueryAnalysis } from "../src/search-expansion.js";

describe("search expansion", () => {
  it("applies broad creature vibe rules into the expanded query", () => {
    const analysis = buildSearchQueryAnalysis(
      "ghost ship body horror",
      { category: "creatures" },
    );

    expect(analysis?.matchedRules.map((rule) => rule.id)).toEqual(
      expect.arrayContaining(["spectral-undead", "maritime-depths", "body-horror"]),
    );
    expect(analysis?.boostedTraits).toEqual(expect.arrayContaining(["aquatic", "incorporeal", "undead", "water"]));
    expect(analysis?.boostedNameTokens).toEqual(expect.arrayContaining(["crawling", "hand", "limb"]));
    expect(analysis?.boostedMetadataTokens).toEqual(expect.arrayContaining(["curse", "cursed", "drowned", "sailor"]));
  });

  it("does not treat bare spirit language as undead by default", () => {
    const analysis = buildSearchQueryAnalysis(
      "forest spirit",
      { category: "creatures" },
    );

    expect(analysis?.matchedRules.map((rule) => rule.id)).not.toContain("spectral-undead");
    expect(analysis?.matchedRules.map((rule) => rule.id)).toContain("wilderness-primal");
  });

  it("respects scope-aware spell boosts", () => {
    const spellAnalysis = buildSearchQueryAnalysis(
      "storm lightning thunder",
      { category: "spells" },
    );
    expect(spellAnalysis?.matchedRules.map((rule) => rule.id)).toContain("storm-spells");
    expect(spellAnalysis?.boostedTraits).toEqual(expect.arrayContaining(["air", "electricity", "sonic"]));

    const npcAnalysis = buildSearchQueryAnalysis(
      "storm lightning thunder",
      { category: "creatures" },
    );
    expect(npcAnalysis?.matchedRules.map((rule) => rule.id)).not.toContain("storm-spells");
    expect(npcAnalysis?.skippedRules).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: "storm-spells",
          reason: "scope_mismatch",
        }),
      ]),
    );
  });

  it("applies gear-oriented concepts to item categories instead of broad content", () => {
    const gearAnalysis = buildSearchQueryAnalysis(
      "infiltration tools lockpick camouflage",
      { category: "equipment", subcategory: "gear" },
    );
    expect(gearAnalysis?.matchedRules.map((rule) => rule.id)).toContain("stealth-gear");
    expect(gearAnalysis?.boostedMetadataTokens).toEqual(expect.arrayContaining(["concealed", "subtle"]));

    const npcAnalysis = buildSearchQueryAnalysis(
      "stealth infiltration silent",
      { category: "creatures" },
    );
    expect(npcAnalysis?.matchedRules.map((rule) => rule.id)).not.toContain("stealth-gear");
  });

  it("biases encounter ecology language toward npc and hazard content", () => {
    const encounterAnalysis = buildSearchQueryAnalysis(
      "ambush predator lair nest",
      { category: "creatures" },
    );
    expect(encounterAnalysis?.matchedRules.map((rule) => rule.id)).toEqual(
      expect.arrayContaining(["encounter-ecology", "stealth-assailants"]),
    );
    expect(encounterAnalysis?.boostedMetadataTokens).toEqual(expect.arrayContaining(["den", "territory", "stalk", "stealth"]));

    const spellAnalysis = buildSearchQueryAnalysis(
      "ambush predator lair nest",
      { category: "spells" },
    );
    expect(spellAnalysis?.matchedRules.map((rule) => rule.id)).toEqual([]);
    expect(spellAnalysis?.skippedRules).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: "encounter-ecology",
          reason: "scope_mismatch",
        }),
        expect.objectContaining({
          id: "stealth-assailants",
          reason: "scope_mismatch",
        }),
      ]),
    );
  });

  it("can disable expansion while still reporting skipped triggered rules", () => {
    const analysis = buildSearchQueryAnalysis(
      "ghost ship body horror",
      { category: "creatures" },
      { expandQuery: false },
    );

    expect(analysis?.matchedRules).toEqual([]);
    expect(analysis?.boostedTraits).toEqual([]);
    expect(analysis?.boostedNameTokens).toEqual([]);
    expect(analysis?.boostedMetadataTokens).toEqual([]);
    expect(analysis?.skippedRules.map((rule) => rule.reason)).toContain("expansion_disabled");
    expect(analysis?.expandedQuery).toBe("body ghost horror ship");
  });

  it("maps blight language to flora corruption instead of only generic wilderness", () => {
    const analysis = buildSearchQueryAnalysis(
      "corrupted woodland blight",
      { category: "creatures" },
    );

    expect(analysis?.matchedRules.map((rule) => rule.id)).toEqual(
      expect.arrayContaining(["blighted-wilds", "wilderness-primal"]),
    );
    expect(analysis?.boostedTraits).toEqual(expect.arrayContaining(["disease", "fungus", "ooze", "plant"]));
  });

  it("infers category and subcategory from clear user wording", () => {
    const hauntAnalysis = buildSearchQueryAnalysis("haunted manor haunt", {});
    expect(hauntAnalysis?.inferredCategory).toBe("hazards");
    expect(hauntAnalysis?.inferredSubcategory).toBe("haunt");

    const gearAnalysis = buildSearchQueryAnalysis("infiltration elixir", {});
    expect(gearAnalysis?.inferredCategory).toBe("equipment");
    expect(gearAnalysis?.inferredSubcategory).toBe("consumable");

    const deityAnalysis = buildSearchQueryAnalysis("deity of travel", {});
    expect(deityAnalysis?.inferredCategory).toBe("lore");
    expect(deityAnalysis?.inferredSubcategory).toBe("deity");
  });
});
