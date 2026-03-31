import { describe, expect, it } from "vitest";

import { buildCandidateQueryWeights, buildSearchQueryAnalysis } from "../src/search-expansion.js";

describe("search query analysis", () => {
  it("normalizes the literal query without injecting heuristic tokens", () => {
    const analysis = buildSearchQueryAnalysis("ghost ship body horror");

    expect(analysis?.normalizedQuery).toBe("ghost ship body horror");
    expect(analysis?.queryTokens).toEqual(["ghost", "ship", "body", "horror"]);
  });

  it("uses only literal tokens for base trait, name, and metadata weights", () => {
    const analysis = buildSearchQueryAnalysis("haunted manor");
    if (!analysis) {
      throw new Error("Expected query analysis.");
    }

    expect(analysis.baseTraitWeights.get("haunted")).toBe(1);
    expect(analysis.baseNameWeights.get("haunted")).toBe(0.65);
    expect(analysis.baseMetadataWeights.get("haunted")).toBe(0.75);
    expect(analysis.baseTraitWeights.has("haunt")).toBe(false);
    expect(analysis.baseNameWeights.has("hand")).toBe(false);
  });

  it("builds candidate query weights without record-specific heuristic boosts", () => {
    const analysis = buildSearchQueryAnalysis("crawling swarm");
    if (!analysis) {
      throw new Error("Expected query analysis.");
    }

    const weights = buildCandidateQueryWeights(
      {
        recordKey: "record",
        id: "record",
        name: "Crawling Hand Swarm",
        normalizedName: "crawling hand swarm",
        type: "npc",
        category: "creatures",
        subcategories: [],
        packName: "test-pack",
        packLabel: "Test Pack",
        documentType: "Actor",
        level: 1,
        rarity: "common",
        traits: ["swarm", "undead"],
        publicationTitle: null,
        descriptionText: null,
        hasDescription: false,
        descriptionSnippet: null,
        sourceCategory: "core",
        folderId: null,
        sourcePath: "/tmp/record.json",
        isUnique: false,
        size: null,
        itemCategory: null,
        priceCp: null,
        bulkValue: null,
        actionCost: null,
        traditions: [],
        raw: {},
      },
      analysis,
    );

    expect([...weights.nameWeights.keys()]).toEqual(["crawling", "swarm"]);
    expect([...weights.traitWeights.keys()]).toEqual(["crawling", "swarm"]);
    expect([...weights.metadataWeights.keys()]).toEqual(["crawling", "swarm"]);
  });
});
