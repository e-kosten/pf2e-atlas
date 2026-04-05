import { describe, expect, it } from "vitest";

import {
  formatEvidenceReport,
  parseOptions,
} from "../../src/tags/analyze-derived-tag-evidence.js";

describe("derived-tag evidence CLI helpers", () => {
  it("parses scope, record keys, and untagged mode", () => {
    const options = parseOptions([
      "--category", "equipment",
      "--subcategory", "gear",
      "--record-key", "equipment:1",
      "--exclude-record-key", "equipment:2",
      "--untagged",
      "--limit", "9",
    ]);

    expect(options).toEqual(expect.objectContaining({
      category: "equipment",
      subcategory: "gear",
      recordKeys: ["equipment:1"],
      excludeRecordKeys: ["equipment:2"],
      untaggedOnly: true,
      limit: 9,
    }));
  });

  it("renders a readable evidence report", () => {
    const report = formatEvidenceReport({
      category: "spell",
      subcategory: null,
      cohortSize: 4,
      baselineSize: 20,
      nameTokens: [{ value: "mask", support: 3, cohortSupport: 3, baselineSupport: 4, lift: 3.75, score: 11.25, examples: ["Mask of Cinders"] }],
      namePhrases: [],
      descriptionTokens: [],
      descriptionPhrases: [],
      traits: [],
      references: [],
      representativeRecords: [
        { recordKey: "spell:1", name: "Mask of Cinders", traits: ["fire", "illusion"] },
      ],
    });

    expect(report).toContain("Evidence summary:");
    expect(report).toContain("Representative records:");
    expect(report).toContain("Name tokens:");
  });
});
