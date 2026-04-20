import { describe, expect, it } from "vitest";

import {
  formatFilterExplorerPolicyCycleCopy,
  formatFilterExplorerPolicySummary,
} from "../../src/tui/filter-explorer/policy-presentation.js";

describe("filter explorer policy presentation", () => {
  it("formats workspace-style policy summaries from the shared presentation seam", () => {
    expect(
      formatFilterExplorerPolicySummary(
        {
          any: ["common"],
          all: ["rare"],
          exclude: ["unique"],
        },
        {
          valueFormatter: (value) => value.toUpperCase(),
        },
      ),
    ).toBe("∪ COMMON | ∩ RARE | ¬ UNIQUE");
  });

  it("formats cycle-copy text from the shared presentation seam", () => {
    expect(formatFilterExplorerPolicyCycleCopy(["any", "all", "exclude"])).toBe(
      "∪ include any, ∩ require all, or ¬ exclude",
    );
    expect(formatFilterExplorerPolicyCycleCopy(["any", "exclude"])).toBe("∪ include any or ¬ exclude");
  });
});
