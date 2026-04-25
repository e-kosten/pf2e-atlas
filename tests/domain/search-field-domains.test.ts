import { describe, expect, it } from "vitest";

import {
  getSearchPromotedFieldValueOrdering,
  normalizeSearchPromotedNumberValue,
  normalizeSearchPromotedStringValue,
} from "../../src/domain/search-field-domains.js";

describe("search promoted field domains", () => {
  it("derives promoted-field ordering from declared domains", () => {
    expect(getSearchPromotedFieldValueOrdering("rarity")).toEqual({
      kind: "canonical",
      order: ["common", "uncommon", "rare", "unique"],
    });
    expect(getSearchPromotedFieldValueOrdering("actionCost")).toEqual({
      kind: "canonical",
      order: ["0", "1", "2", "3"],
    });
  });

  it("normalizes and validates rarity through the shared promoted-field owner", () => {
    expect(normalizeSearchPromotedStringValue("rarity", " Rare ")).toBe("rare");
    expect(normalizeSearchPromotedStringValue("rarity", "mythic", { onInvalid: "null" })).toBeNull();
    expect(() => normalizeSearchPromotedStringValue("rarity", "mythic")).toThrow('Unknown rarity value "mythic".');
  });

  it("normalizes and validates action cost through the shared promoted-field owner", () => {
    expect(normalizeSearchPromotedNumberValue("actionCost", 2)).toBe(2);
    expect(normalizeSearchPromotedNumberValue("actionCost", 4, { onInvalid: "null" })).toBeNull();
    expect(() => normalizeSearchPromotedNumberValue("actionCost", 4)).toThrow('Unsupported actionCost value "4".');
  });
});
