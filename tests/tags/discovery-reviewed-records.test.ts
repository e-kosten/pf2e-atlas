import { describe, expect, it } from "vitest";

import {
  getReviewedDiscoveryEntries,
  getReviewedDiscoveryReasonCounts,
  getReviewedDiscoveryRecordKeys,
  getReviewedDiscoverySelection,
  summarizeReviewedDiscoverySelection,
  type ReviewedDiscoveryRegistry,
} from "../../src/tags/reviews/discovery-reviewed-records.js";

describe("reviewed discovery records", () => {
  const registry: ReviewedDiscoveryRegistry = {
    creature: {
      setting: {
        not_family_salient: [
          { recordKey: "creature:1", pack: "creature", name: "Record One", note: "No stable habitat cue." },
          { recordKey: "creature:2", pack: "creature", name: "Record Two", subcategory: null },
        ],
        insufficient_evidence: [{ recordKey: "creature:3", pack: "creature", name: "Record Three" }],
      },
    },
    spell: {
      purpose: {
        manual_lore_only: [{ recordKey: "spell:1", pack: "spell", name: "Spell One" }],
      },
    },
  };

  it("filters entries by category, family, subcategory, and reason", () => {
    expect(
      getReviewedDiscoveryEntries(
        {
          category: "creature",
          family: "setting",
        },
        registry,
      )
        .map((entry) => entry.recordKey)
        .sort(),
    ).toEqual(["creature:1", "creature:2", "creature:3"]);

    expect(
      getReviewedDiscoveryEntries(
        {
          category: "creature",
          family: "setting",
          reason: "not_family_salient",
        },
        registry,
      ),
    ).toEqual([
      {
        category: "creature",
        family: "setting",
        reason: "not_family_salient",
        recordKey: "creature:1",
        pack: "creature",
        name: "Record One",
        subcategory: null,
        note: "No stable habitat cue.",
      },
      {
        category: "creature",
        family: "setting",
        reason: "not_family_salient",
        recordKey: "creature:2",
        pack: "creature",
        name: "Record Two",
        subcategory: null,
        note: undefined,
      },
    ]);
  });

  it("dedupes record keys and reports reason counts", () => {
    expect(
      getReviewedDiscoveryRecordKeys(
        {
          category: "creature",
          family: "setting",
        },
        registry,
      ),
    ).toEqual(["creature:1", "creature:2", "creature:3"]);

    expect(
      getReviewedDiscoveryReasonCounts(
        {
          category: "creature",
          family: "setting",
        },
        registry,
      ),
    ).toEqual([
      { reason: "not_family_salient", count: 2 },
      { reason: "insufficient_evidence", count: 1 },
    ]);
  });

  it("builds selection summaries for excluded and filtered modes", () => {
    const excluded = getReviewedDiscoverySelection(
      {
        category: "creature",
        family: "setting",
      },
      registry,
    );
    expect(excluded).toBeDefined();
    expect(excluded?.mode).toBe("excluded");
    expect(excluded?.recordKeys).toEqual(["creature:1", "creature:2", "creature:3"]);

    const filtered = getReviewedDiscoverySelection(
      {
        category: "creature",
        family: "setting",
        includeReviewed: true,
        reviewReason: "not_family_salient",
      },
      registry,
    );
    expect(filtered?.mode).toBe("filtered");
    expect(filtered?.recordKeys).toEqual(["creature:1", "creature:2"]);

    expect(filtered ? summarizeReviewedDiscoverySelection(filtered) : null).toEqual({
      mode: "filtered",
      reviewReason: "not_family_salient",
      scopedCount: 2,
      appliedCount: 2,
      reasonCounts: [{ reason: "not_family_salient", count: 2 }],
    });
  });
});
