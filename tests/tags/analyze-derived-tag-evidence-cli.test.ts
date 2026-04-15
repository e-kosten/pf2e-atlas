import { describe, expect, it } from "vitest";

import {
  formatHelp,
  formatEvidenceReport,
  parseOptions,
} from "../../src/tags/analyze-derived-tag-evidence.js";

describe("derived-tag evidence CLI helpers", () => {
  it("parses scope, record keys, family filtering, and untagged mode", () => {
    const options = parseOptions([
      "--category", "equipment",
      "--subcategory", "gear",
      "--family", "purpose",
      "--family-gap-signals",
      "--record-key", "equipment:1",
      "--exclude-record-key", "equipment:2",
      "--untagged",
      "--limit", "9",
      "--min-gram-length", "3",
      "--max-gram-length", "5",
    ]);

    expect(options).toEqual(expect.objectContaining({
      category: "equipment",
      subcategory: "gear",
      family: "purpose",
      familyGapSignals: true,
      recordKeys: ["equipment:1"],
      excludeRecordKeys: ["equipment:2"],
      untaggedOnly: true,
      limit: 9,
      minGramLength: 3,
      maxGramLength: 5,
    }));
  });

  it("rejects invalid gram length ranges", () => {
    expect(() => parseOptions([
      "--category", "equipment",
      "--min-gram-length", "1",
    ])).toThrow(/min-gram-length/i);

    expect(() => parseOptions([
      "--category", "equipment",
      "--min-gram-length", "5",
      "--max-gram-length", "4",
    ])).toThrow(/less than or equal/i);
  });

  it("requires category-scoped family filtering and rejects tag plus family", () => {
    expect(() => parseOptions([
      "--family", "setting",
    ])).toThrow(/category/i);

    expect(() => parseOptions([
      "--category", "creature",
      "--tag", "fortress_setting",
      "--family", "setting",
    ])).toThrow(/either --tag .* or --family/i);

    expect(() => parseOptions([
      "--category", "creature",
      "--family-gap-signals",
    ])).toThrow(/family-gap-signals/i);
  });

  it("renders a readable evidence report", () => {
    const report = formatEvidenceReport({
      category: "spell",
      subcategory: null,
      family: "setting",
      cohortSize: 4,
      baselineSize: 20,
      nameTokens: [{ value: "mask", support: 3, cohortSupport: 3, baselineSupport: 4, lift: 3.75, score: 11.25, examples: ["Mask of Cinders"] }],
      namePhrases: [],
      descriptionTokens: [],
      descriptionPhrases: [],
      traits: [],
      references: [],
      familyGap: {
        coveredCount: 7,
        uncoveredCount: 4,
        baselineCount: 20,
        liveTags: ["forest_setting", "underground_setting"],
        likelyNewConcepts: [
          {
            value: "shadow plane",
            kind: "descriptionPhrase",
            support: 3,
            cohortSupport: 3,
            coveredSupport: 0,
            baselineSupport: 3,
            gapLift: 3,
            lift: 2,
            score: 12,
            examples: ["A hunter stalks the shadow plane."],
            bucket: "possible_place_anchor",
            existingTagOverlaps: [],
            suppressionReason: null,
          },
        ],
        existingTagCoverageGaps: [
          {
            value: "jungle",
            kind: "descriptionToken",
            support: 2,
            cohortSupport: 2,
            coveredSupport: 1,
            baselineSupport: 3,
            gapLift: 1.5,
            lift: 1.3,
            score: 5.5,
            examples: ["A jungle prowler waits."],
            bucket: "possible_place_anchor",
            existingTagOverlaps: ["forest_setting"],
            suppressionReason: null,
          },
        ],
        suppressedTerms: [
          {
            value: "dragon",
            kind: "trait",
            support: 4,
            cohortSupport: 4,
            coveredSupport: 4,
            baselineSupport: 8,
            gapLift: 1,
            lift: 1,
            score: 0.9,
            examples: ["dragon"],
            bucket: "taxonomy",
            existingTagOverlaps: [],
            suppressionReason: "taxonomy",
          },
        ],
      },
      representativeRecords: [
        { recordKey: "spell:1", name: "Mask of Cinders", traits: ["fire", "illusion"] },
      ],
    });

    expect(report).toContain("Evidence summary:");
    expect(report).toContain("Family: setting");
    expect(report).toContain("Covered family records: 7");
    expect(report).toContain("Representative records:");
    expect(report).toContain("Name tokens:");
    expect(report).toContain("Likely new concepts:");
    expect(report).toContain("Likely existing-tag coverage gaps:");
    expect(report).toContain("Suppressed generic anchors:");
  });

  it("renders help with family-scoped examples", () => {
    const help = formatHelp();

    expect(help).toContain("Usage:");
    expect(help).toContain("--family <derived-tag-family>");
    expect(help).toContain("--family-gap-signals");
    expect(help).toContain("--category creature --family setting");
  });
});
