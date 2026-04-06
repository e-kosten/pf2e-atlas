import { describe, expect, it } from "vitest";

import {
  formatClusterReport,
  parseOptions,
} from "../../src/tags/cluster-derived-tag-candidates.js";
import { formatRuleableCohortReport } from "../../src/tags/discover-ruleable-cohorts.js";

describe("derived-tag cohort CLI helpers", () => {
  it("parses exemplar, tag, and similarity options", () => {
    const options = parseOptions([
      "--category", "creature",
      "--name", "Ghost Commoner",
      "--record-key", "creature:ghost-commoner",
      "--tag", "undead_adjacent",
      "--candidate-limit", "15",
      "--cohort-limit", "4",
      "--min-similarity", "0.84",
    ]);

    expect(options).toEqual(expect.objectContaining({
      category: "creature",
      exemplarNames: ["Ghost Commoner"],
      exemplarRecordKeys: ["creature:ghost-commoner"],
      tag: "undead_adjacent",
      candidateLimit: 15,
      cohortLimit: 4,
      minSimilarity: 0.84,
    }));
  });

  it("renders cluster and ruleability reports", () => {
    const report = {
      category: "creature" as const,
      subcategory: null,
      sourceTag: "undead_adjacent",
      exemplarCount: 2,
      candidateCount: 5,
      resolvedExemplars: [
        { query: "Ghost Commoner", matchedBy: "name" as const, recordKey: "creature:ghost-commoner", name: "Ghost Commoner" },
      ],
      anchorTerms: [
        { kind: "trait" as const, value: "undead", support: 2, cohortSupport: 2, baselineSupport: 3, lift: 2, score: 4, examples: ["Ghost Commoner"] },
      ],
      contrastRecords: [
        { recordKey: "creature:guard", name: "Sunny Guard", similarity: 0.81 },
      ],
      cohorts: [
        {
          signature: ["trait:undead"],
          size: 3,
          averageSimilarity: 0.92,
          sharedTraits: ["undead"],
          sharedAnchors: ["trait:undead"],
          representativeRecords: [
            { recordKey: "creature:wraith", name: "Harbor Wraith", similarity: 0.95 },
          ],
          score: 0.78,
          recommendation: "rule-led" as const,
        },
      ],
    };

    expect(formatClusterReport(report)).toContain("Cohort summary:");
    expect(formatClusterReport(report)).toContain("Cohorts:");
    expect(formatRuleableCohortReport(report)).toContain("Ruleable cohort summary:");
    expect(formatRuleableCohortReport(report)).toContain("Top cohorts:");
  });
});
