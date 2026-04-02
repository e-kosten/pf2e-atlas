import { describe, expect, it } from "vitest";

import {
  rankDerivedTagGapCandidates,
  type DerivedTagGapRecord,
} from "../src/derived-tag-gap-evaluator.js";

function vector(values: number[]): Float32Array {
  return Float32Array.from(values);
}

describe("derived tag gap evaluator", () => {
  it("ranks untagged records by similarity to tagged exemplars and surfaces common traits", () => {
    const exemplars: DerivedTagGapRecord[] = [
      {
        recordKey: "equipment:1",
        name: "Swallow-Spike",
        category: "equipment",
        subcategory: "gear",
        level: 6,
        traits: ["magical", "armor"],
        descriptionText: "Break free when you become Grabbed.",
        vector: vector([1, 0, 0]),
      },
      {
        recordKey: "equipment:2",
        name: "Implacable",
        category: "equipment",
        subcategory: "gear",
        level: 8,
        traits: ["magical", "armor"],
        descriptionText: "Makes you difficult to hold back.",
        vector: vector([0.9, 0.1, 0]),
      },
      {
        recordKey: "equipment:3",
        name: "Miniaturization Module",
        category: "equipment",
        subcategory: "gear",
        level: 5,
        traits: ["magical", "clockwork"],
        descriptionText: "Gives a bonus to Escape.",
        vector: vector([0.95, 0.05, 0]),
      },
    ];

    const candidates: DerivedTagGapRecord[] = [
      {
        recordKey: "equipment:4",
        name: "Likely Gap",
        category: "equipment",
        subcategory: "gear",
        level: 7,
        traits: ["magical", "armor"],
        descriptionText: "A likely missed restraint-escape tool.",
        vector: vector([0.98, 0.02, 0]),
      },
      {
        recordKey: "equipment:5",
        name: "Unrelated",
        category: "equipment",
        subcategory: "gear",
        level: 7,
        traits: ["magical", "illusion"],
        descriptionText: "A thematically unrelated item.",
        vector: vector([0, 1, 0]),
      },
    ];

    const evaluation = rankDerivedTagGapCandidates(exemplars, candidates, {
      tag: "restraint_escape",
      category: "equipment",
      subcategory: "gear",
      limit: 2,
      exemplarLimit: 2,
      commonTraitLimit: 4,
    });

    expect(evaluation.tag).toBe("restraint_escape");
    expect(evaluation.exemplarCount).toBe(3);
    expect(evaluation.candidateCount).toBe(2);
    expect(evaluation.commonTraits).toEqual(expect.arrayContaining(["magical", "armor"]));
    expect(evaluation.candidates[0]?.name).toBe("Likely Gap");
    expect(evaluation.candidates[0]?.sharedTraits).toEqual(expect.arrayContaining(["magical", "armor"]));
    expect(evaluation.candidates[0]?.similarity).toBeGreaterThan(evaluation.candidates[1]?.similarity ?? 0);
    expect(evaluation.exemplars).toHaveLength(2);
  });
});
