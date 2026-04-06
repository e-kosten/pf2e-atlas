import { describe, expect, it } from "vitest";

import {
  extractDiscoveryNgrams,
  isDiscoveryNoisePhrase,
  isDiscoveryNoiseToken,
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

  it("strips foundry inline markup before tokenization", () => {
    expect(normalizeDiscoveryText("@UUID[Compendium.pf2e.spells-srd.Item.Daze]{Daze} [[/r 1d20+16]]{+16}")).toBe(
      "daze {{number}}",
    );
  });

  it("flags discovery boilerplate and placeholder-heavy phrases as noise", () => {
    expect(isDiscoveryNoiseToken("activate")).toBe(true);
    expect(isDiscoveryNoiseToken("{{number}}")).toBe(true);
    expect(isDiscoveryNoisePhrase("activate {{number}} strike")).toBe(true);
    expect(isDiscoveryNoisePhrase("skyhook harness")).toBe(false);
  });
});
