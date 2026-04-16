import { describe, expect, it } from "vitest";

import {
  getReviewedDiscoveryEntries,
  getReviewedDiscoveryReasonCounts,
  getReviewedDiscoveryRecordKeys,
  getReviewedDiscoverySelection,
  summarizeReviewedDiscoverySelection,
  type ReviewedDiscoveryRegistry,
} from "../../src/tags/discovery-reviewed-records.js";

describe("reviewed discovery records", () => {
  const registry: ReviewedDiscoveryRegistry = {
    creature: {
      setting: {
        not_family_salient: [
          { recordKey: "creature:1", note: "No stable habitat cue." },
          { recordKey: "creature:2", subcategory: null },
        ],
        insufficient_evidence: [
          { recordKey: "creature:3" },
        ],
      },
    },
    spell: {
      purpose: {
        manual_lore_only: [
          { recordKey: "spell:1" },
        ],
      },
    },
  };

  it("filters entries by category, family, subcategory, and reason", () => {
    expect(getReviewedDiscoveryEntries({
      category: "creature",
      family: "setting",
    }, registry).map((entry) => entry.recordKey).sort()).toEqual([
      "creature:1",
      "creature:2",
      "creature:3",
    ]);

    expect(getReviewedDiscoveryEntries({
      category: "creature",
      family: "setting",
      reason: "not_family_salient",
    }, registry).map((entry) => entry.recordKey)).toEqual([
      "creature:1",
      "creature:2",
    ]);
  });

  it("dedupes record keys and reports reason counts", () => {
    expect(getReviewedDiscoveryRecordKeys({
      category: "creature",
      family: "setting",
    }, registry)).toEqual([
      "creature:1",
      "creature:2",
      "creature:3",
    ]);

    expect(getReviewedDiscoveryReasonCounts({
      category: "creature",
      family: "setting",
    }, registry)).toEqual([
      { reason: "not_family_salient", count: 2 },
      { reason: "insufficient_evidence", count: 1 },
    ]);
  });

  it("builds selection summaries for excluded and filtered modes", () => {
    const excluded = getReviewedDiscoverySelection({
      category: "creature",
      family: "setting",
    }, registry);
    expect(excluded).toBeDefined();
    expect(excluded?.mode).toBe("excluded");
    expect(excluded?.recordKeys).toEqual([
      "creature:1",
      "creature:2",
      "creature:3",
    ]);

    const filtered = getReviewedDiscoverySelection({
      category: "creature",
      family: "setting",
      includeReviewed: true,
      reviewReason: "not_family_salient",
    }, registry);
    expect(filtered?.mode).toBe("filtered");
    expect(filtered?.recordKeys).toEqual([
      "creature:1",
      "creature:2",
    ]);

    expect(filtered ? summarizeReviewedDiscoverySelection(filtered) : null).toEqual({
      mode: "filtered",
      reviewReason: "not_family_salient",
      scopedCount: 2,
      appliedCount: 2,
      reasonCounts: [{ reason: "not_family_salient", count: 2 }],
    });
  });
});
