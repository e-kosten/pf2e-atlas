import { describe, expect, it } from "vitest";

import { normalizeSearchFilters } from "../../src/search/filters/normalization.js";
import type { SearchExecutionFilters } from "../../src/search/contracts.js";

describe("search filter normalization", () => {
  it("collapses singleton boolean wrappers before execution validation", () => {
    const filters: SearchExecutionFilters = {
      filter: {
        kind: "allOf",
        children: [
          {
            kind: "scope",
            category: "creature",
            subcategory: { kind: "present" },
          },
        ],
      },
    };

    expect(normalizeSearchFilters(filters, () => undefined).filter).toEqual({
      kind: "scope",
      category: "creature",
      subcategory: { kind: "present" },
    });
  });

  it("normalizes linkedFrom source record keys", () => {
    const filters: SearchExecutionFilters = {
      filter: {
        kind: "linkedFrom",
        source: "  actions:action-refocus-1  ",
      },
    };

    expect(normalizeSearchFilters(filters, () => undefined).filter).toEqual({
      kind: "linkedFrom",
      source: "actions:action-refocus-1",
    });
  });

  it("rejects empty linkedFrom source record keys", () => {
    const filters: SearchExecutionFilters = {
      filter: {
        kind: "linkedFrom",
        source: "   ",
      },
    };

    expect(() => normalizeSearchFilters(filters, () => undefined)).toThrow("linkedFrom source must not be empty.");
  });
});
