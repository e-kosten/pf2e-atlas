import { describe, expect, it } from "vitest";

import { buildLiteralQueryWeights, buildSearchQueryAnalysis } from "../src/search-query-analysis.js";

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

  it("builds literal query weights without record-specific heuristic boosts", () => {
    const analysis = buildSearchQueryAnalysis("crawling swarm");
    if (!analysis) {
      throw new Error("Expected query analysis.");
    }

    const weights = buildLiteralQueryWeights(analysis);

    expect([...weights.nameWeights.keys()]).toEqual(["crawling", "swarm"]);
    expect([...weights.traitWeights.keys()]).toEqual(["crawling", "swarm"]);
    expect([...weights.metadataWeights.keys()]).toEqual(["crawling", "swarm"]);
  });
});
