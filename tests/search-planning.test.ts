import { describe, expect, it } from "vitest";

import { buildSearchPlan, summarizeExpansionRules } from "../src/search-planning.js";

describe("search planning", () => {
  it("turns broad intent into bounded hybrid and structured backstop queries", () => {
    const plan = buildSearchPlan("ghost ship body horror", {
      recordType: "npc",
      levelMin: 1,
      levelMax: 5,
    });

    expect(plan.recognizedSemantics.map((entry) => entry.id)).toEqual(
      expect.arrayContaining(["spectral-undead", "maritime-depths", "body-horror"]),
    );
    expect(plan.suggestedFilters.traitsAny).toEqual(expect.arrayContaining(["aquatic", "ghost", "undead", "water"]));
    expect(plan.recommendedQueries.map((query) => query.label)).toEqual(
      expect.arrayContaining(["broad_hybrid", "trait_hinted_hybrid", "structured_backstop"]),
    );
  });

  it("surfaces scoped item guidance for gear-oriented intent", () => {
    const plan = buildSearchPlan("infiltration tools lockpick camouflage", {
      documentType: "Item",
      itemCategory: "equipment",
    });

    expect(plan.suggestedFilters.preferredItemCategories).toEqual(expect.arrayContaining(["armor", "consumable", "equipment", "weapon"]));
    expect(plan.recognizedSemantics.map((entry) => entry.id)).toContain("stealth-gear");
  });

  it("treats ambush language as creature-compatible rather than hazard-only", () => {
    const plan = buildSearchPlan("woodland fey ambush", {
      levelMin: 1,
      levelMax: 5,
    });

    expect(plan.recognizedSemantics.map((entry) => entry.id)).toEqual(
      expect.arrayContaining(["encounter-ecology", "stealth-assailants", "wilderness-primal"]),
    );
    expect(plan.suggestedFilters.preferredRecordTypes[0]).toBe("npc");
    expect(plan.suggestedFilters.preferredRecordTypes).toContain("hazard");
  });

  it("does not treat forest spirit as undead planning by default", () => {
    const plan = buildSearchPlan("forest spirit", {
      levelMin: 1,
      levelMax: 5,
    });

    expect(plan.recognizedSemantics.map((entry) => entry.id)).toContain("wilderness-primal");
    expect(plan.recognizedSemantics.map((entry) => entry.id)).not.toContain("spectral-undead");
  });

  it("adds explicit blight semantics for corrupted woodland intent", () => {
    const plan = buildSearchPlan("corrupted woodland blight", {
      levelMin: 1,
      levelMax: 5,
    });

    expect(plan.recognizedSemantics.map((entry) => entry.id)).toEqual(
      expect.arrayContaining(["blighted-wilds", "wilderness-primal"]),
    );
    expect(plan.suggestedFilters.traitsAny).toEqual(expect.arrayContaining(["disease", "fungus", "ooze", "plant"]));
  });

  it("summarizes ontology domains for caller introspection", () => {
    const domains = summarizeExpansionRules();
    expect(domains.some((domain) => domain.id === "storm-spells" && domain.scope?.recordTypes?.includes("spell"))).toBe(true);
    expect(domains.some((domain) => domain.id === "stealth-gear" && domain.scope?.itemCategories?.includes("equipment"))).toBe(true);
  });
});
