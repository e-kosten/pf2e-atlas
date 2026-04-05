import { describe, expect, it } from "vitest";

import {
  extractDiscoveryNgrams,
  normalizeDiscoveryText,
  tokenizeDiscoveryText,
} from "../../src/tags/discovery-normalization.js";

describe("discovery normalization", () => {
  it("collapses dice, range, and plain numbers into shared placeholders", () => {
    expect(normalizeDiscoveryText("Deals 2d6 fire damage in a 30-foot cone; DC 17 Reflex.")).toBe(
      "deals {{dice}} fire damage in a {{range}} cone dc {{number}} reflex",
    );
  });

  it("filters stopwords while preserving placeholders", () => {
    expect(tokenizeDiscoveryText("Gain a +10-foot status bonus to your Speed.", { filterStopwords: true })).toEqual([
      "{{range}}",
      "status",
      "bonus",
      "speed",
    ]);
  });

  it("builds normalized ngrams from placeholder-aware text", () => {
    expect(extractDiscoveryNgrams("Deals 4d6 fire damage.", 3, { filterStopwords: true })).toEqual(
      expect.arrayContaining([
        {
          normalized: "{{dice}} fire damage",
          raw: "{{dice}} fire damage",
        },
      ]),
    );
  });
});
