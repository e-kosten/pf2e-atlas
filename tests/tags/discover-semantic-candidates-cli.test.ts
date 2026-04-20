import { describe, expect, it } from "vitest";

import {
  formatSemanticDiscoveryReport,
  parseOptions,
} from "../../src/tags/cli/discovery/discover-semantic-candidates.js";

describe("semantic discovery CLI helpers", () => {
  it("parses repeated exemplar flags and numeric options", () => {
    const options = parseOptions([
      "--category",
      "creature",
      "--name",
      "Ghost Commoner",
      "--name",
      "Ghost Pirate Captain",
      "--record-key",
      "creature:ghost-commoner",
      "--candidate-limit",
      "12",
      "--contrast-limit",
      "4",
      "--min-similarity",
      "0.82",
      "--exclude-derived-tag",
      "haunt_theme",
      "--min-gram-length",
      "4",
      "--max-gram-length",
      "5",
    ]);

    expect(options).toEqual(
      expect.objectContaining({
        category: "creature",
        exemplarNames: ["Ghost Commoner", "Ghost Pirate Captain"],
        exemplarRecordKeys: ["creature:ghost-commoner"],
        limit: 12,
        contrastLimit: 4,
        minSimilarity: 0.82,
        excludeDerivedTag: "haunt_theme",
        minGramLength: 4,
        maxGramLength: 5,
      }),
    );
  });

  it("rejects invalid semantic gram ranges", () => {
    expect(() =>
      parseOptions([
        "--category",
        "creature",
        "--name",
        "Ghost Commoner",
        "--min-gram-length",
        "5",
        "--max-gram-length",
        "2",
      ]),
    ).toThrow(/less than or equal/i);
  });

  it("rejects invalid category scope flags", () => {
    expect(() => parseOptions(["--category", "relic", "--name", "Ghost Commoner"])).toThrow(/Invalid search category/i);

    expect(() => parseOptions(["--category", "creature", "--subcategory", "gear", "--name", "Ghost Commoner"])).toThrow(
      /Invalid search subcategory/i,
    );
  });

  it("renders a readable semantic discovery report", () => {
    const report = formatSemanticDiscoveryReport({
      category: "creature",
      subcategory: null,
      exemplarCount: 2,
      candidateCount: 3,
      matchedCandidateCount: 2,
      commonTraits: ["undead", "human"],
      sharedTokens: [{ value: "mournful", support: 4, exemplarSupport: 2, candidateSupport: 2 }],
      sharedPhrases: [{ value: "abandoned ship", support: 3, exemplarSupport: 1, candidateSupport: 2 }],
      similarityBuckets: [
        { minSimilarity: 0.9, count: 1 },
        { minSimilarity: 0.85, count: 2 },
        { minSimilarity: 0.8, count: 3 },
      ],
      resolvedExemplars: [
        {
          query: "Ghost Commoner",
          matchedBy: "name",
          recordKey: "creature:ghost-commoner",
          name: "Ghost Commoner",
          category: "creature",
          subcategory: null,
          level: 4,
          traits: ["undead", "human"],
        },
      ],
      exemplars: [
        {
          name: "Ghost Commoner",
          recordKey: "creature:ghost-commoner",
          level: 4,
          traits: ["undead", "human"],
          similarityToCentroid: 0.99,
        },
      ],
      candidates: [
        {
          recordKey: "creature:haunted-bosun",
          name: "Haunted Bosun",
          category: "creature",
          subcategory: null,
          level: 6,
          similarity: 0.95,
          traits: ["undead", "human"],
          sharedTraits: ["undead", "human"],
          derivedTags: ["undead_adjacent"],
          descriptionText: "Haunts abandoned decks.",
        },
      ],
      contrastRecords: [
        {
          recordKey: "creature:dockside-ruffian",
          name: "Dockside Ruffian",
          category: "creature",
          subcategory: null,
          level: 5,
          similarity: 0.87,
          traits: ["human"],
          sharedTraits: ["human"],
          derivedTags: [],
          descriptionText: "A dockside thug.",
        },
      ],
    });

    expect(report).toContain("Query summary:");
    expect(report).toContain("Resolved exemplars:");
    expect(report).toContain("Shared evidence:");
    expect(report).toContain("Top candidates:");
    expect(report).toContain("Contrast records:");
    expect(report).toContain("Coverage hints:");
    expect(report).toContain("Interpretation notes:");
  });
});
