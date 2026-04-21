import { describe, expect, it } from "vitest";

import { normalizeMetadataFilterNode } from "../../src/search/filters/metadata.js";

describe("metadata filter normalization", () => {
  it("normalizes text equality predicates even when the tool schema only exposes contains operators", () => {
    expect(
      normalizeMetadataFilterNode({
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

  it("normalizes enum-string membership predicates through the shared variant spec", () => {
    expect(
      normalizeMetadataFilterNode({
        field: "saveType",
        op: "in",
        values: ["Reflex", " reflex ", "Fortitude"],
      }),
    ).toEqual({
      field: "saveType",
      op: "in",
      values: ["reflex", "fortitude"],
    });
  });

  it("preserves hyphenated set values for exact membership predicates", () => {
    expect(
      normalizeMetadataFilterNode({
        field: "families",
        op: "includesAny",
        values: ["  ancestry-npcs  ", "ghost"],
      }),
    ).toEqual({
      field: "families",
      op: "includesAny",
      values: ["ancestry-npcs", "ghost"],
    });
  });
});
