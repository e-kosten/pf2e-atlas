import { describe, expect, it } from "vitest";

import { normalizeMetadataAtomicPredicate } from "../../src/search/filters/metadata.js";

describe("metadata predicate normalization", () => {
  it("normalizes text equality predicates even when the public schema only exposes contains operators", () => {
    expect(
      normalizeMetadataAtomicPredicate({
        field: "publicationTitle",
        op: "eq",
        value: "  Player Core  ",
      }),
    ).toEqual({
      field: "publicationTitle",
      op: "eq",
      value: "player core",
    });
  });

  it("normalizes enum-string equality predicates through the shared variant spec", () => {
    expect(
      normalizeMetadataAtomicPredicate({
        field: "saveType",
        op: "eq",
        value: " Reflex ",
      }),
    ).toEqual({
      field: "saveType",
      op: "eq",
      value: "reflex",
    });
  });

  it("preserves hyphenated set values for exact membership predicates", () => {
    expect(
      normalizeMetadataAtomicPredicate({
        field: "families",
        op: "includes",
        value: "  ancestry-npcs  ",
      }),
    ).toEqual({
      field: "families",
      op: "includes",
      value: "ancestry-npcs",
    });
  });
});
