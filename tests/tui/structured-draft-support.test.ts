import { describe, expect, it } from "vitest";

import { buildStructuredDraftEntries } from "../../src/tui/search-screen/structured-draft/structured-draft-support.js";
import type { Pf2eTerminalSearchQuery } from "../../src/tui/search/service.js";

describe("structured draft support", () => {
  it("renders stable tree prefixes when groups have trailing insertion slots", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "anyOf",
        children: [
          { kind: "pack", value: "abomination-vaults" },
          {
            kind: "anyOf",
            children: [
              { kind: "pack", value: "pathfinder-npc-core" },
              { kind: "pack", value: "monster-core" },
            ],
          },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(query, null, {});
    const menuLabels = entries.map((entry) => entry.menuLabel).filter((label): label is string => Boolean(label));

    expect(menuLabels[0]).toBe("Any of");
    expect(menuLabels).toContain("├─ Pack: abomination-vaults");
    expect(menuLabels).toContain("├─ Any of");
    expect(menuLabels).toContain("│  ├─ Pack: pathfinder-npc-core");
    expect(menuLabels).toContain("│  ├─ Pack: monster-core");
    expect(menuLabels).toContain("│  └─ [+ add here]");
    expect(menuLabels).toContain("└─ [+ add here]");
  });
});
