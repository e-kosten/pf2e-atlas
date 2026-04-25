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
});
