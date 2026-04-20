import { describe, expect, it } from "vitest";

import {
  buildFilterExplorerPolicyBadgeSegments,
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

  it("exposes discoverable long-form tokens for detail summaries", () => {
    expect(
      buildFilterExplorerPolicyBadgeSegments("any", {
        fallback: "discoverable",
      }).map((segment) => segment.text).join(""),
    ).toBe("[includeAny]");
    expect(
      formatFilterExplorerPolicySummary(
        {
          any: ["coastal_setting"],
          all: ["urban_setting"],
          exclude: ["rare"],
        },
        {
          fallback: "discoverable",
        },
      ),
    ).toBe("includeAny coastal_setting | includeAll urban_setting | exclude rare");
  });
});
