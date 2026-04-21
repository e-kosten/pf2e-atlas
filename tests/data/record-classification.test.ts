import { describe, expect, it } from "vitest";

import { classifyRecordCategory, extractSpellTraditions } from "../../src/data/record-classification.js";

describe("record classification", () => {
  it("maps feat system categories through canonical typed subcategories", () => {
    const classification = classifyRecordCategory({
      documentType: "Item",
      recordType: "feat",
      packName: "feats-srd",
      sourcePath: "packs/feats-srd/feats/general/something.json",
      traits: [],
      traditions: [],
      raw: {
        system: {
          category: "General",
        },
      },
    });

    expect(classification).toEqual({
      category: "feat",
      subcategory: "general",
    });
  });

  it("extracts and normalizes spell traditions from loose raw documents", () => {
    expect(
      extractSpellTraditions({
        system: {
          traits: {
            traditions: ["Arcane", " primal ", 42, null, "Arcane"],
          },
        },
      } as Record<string, unknown>),
    ).toEqual(["arcane", "primal"]);
  });
});
