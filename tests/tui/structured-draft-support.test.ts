import { describe, expect, it } from "vitest";

import {
  buildStructuredDraftEntries,
  getStructuredDraftSelectionIndexForPath,
} from "../../src/tui/search-screen/structured-draft/structured-draft-support.js";
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

  it("falls back to the parent insertion slot when a nested focused path is no longer present", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          {
            kind: "anyOf",
            children: [
              { kind: "pack", value: "monster-core" },
              { kind: "metadataPredicate", field: "rarity", value: "common", operator: "eq" },
            ],
          },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(
      {
        ...query,
        filter: {
          kind: "allOf",
          children: [
            {
              kind: "anyOf",
              children: [{ kind: "pack", value: "monster-core" }],
            },
          ],
        },
      },
      [0, 1],
      {},
    );
    const selectedIndex = getStructuredDraftSelectionIndexForPath(entries, [0, 1], 0);
    const selectedEntry = entries[selectedIndex];

    expect(selectedEntry?.kind).toBe("queryInsertionSlot");
    expect(selectedEntry?.insertionPath).toEqual([0]);
  });

  it("projects active-group shared-explorer clauses into grouped field buckets", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "scope", category: "spell", subcategory: { kind: "any" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "illusion" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "auditory" } },
          {
            kind: "not",
            child: { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "emotion" } },
          },
          { kind: "pack", value: "monster-core" },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(query, [1], {
      groupedFieldValues: new Set(["traits"]),
    });
    const bucketEntries = entries.filter((entry) => entry.kind === "queryFieldBucket");

    expect(bucketEntries).toHaveLength(2);
    expect(bucketEntries[0]?.label).toBe("Traits: Include illusion, auditory");
    expect(bucketEntries[0]?.fieldMemberPaths).toEqual([[1], [2], [3]]);
    expect(bucketEntries[1]?.label).toBe("Traits: !emotion");
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(false);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Auditory")).toBe(false);
  });

  it("selects the grouped field bucket when the focused member path is projected away", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "scope", category: "spell", subcategory: { kind: "any" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "illusion" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "auditory" } },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(query, [2], {
      groupedFieldValues: new Set(["traits"]),
    });
    const selectedIndex = getStructuredDraftSelectionIndexForPath(entries, [2], 0);

    expect(entries[selectedIndex]?.kind).toBe("queryFieldBucket");
    expect(entries[selectedIndex]?.label).toBe("Traits: Include illusion, auditory");
  });

  it("keeps any-of active groups structural instead of projecting shared-explorer buckets", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "scope", category: "spell", subcategory: { kind: "any" } },
          {
            kind: "anyOf",
            children: [
              { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "illusion" } },
              { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "auditory" } },
            ],
          },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(query, [1, 0], {
      groupedFieldValues: new Set(["traits"]),
    });

    expect(entries.some((entry) => entry.kind === "queryFieldBucket")).toBe(false);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(true);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Auditory")).toBe(true);
  });

  it("keeps a root any-of structural instead of projecting shared-explorer buckets", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "anyOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "illusion" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "auditory" } },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(query, [0], {
      groupedFieldValues: new Set(["traits"]),
    });

    expect(entries.some((entry) => entry.kind === "queryFieldBucket")).toBe(false);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(true);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Auditory")).toBe(true);
  });

  it("flattens simple negated nodes into a single inline tree row", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "pack", value: "abomination-vaults" },
          {
            kind: "not",
            child: { kind: "pack", value: "monster-core" },
          },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(query, null, {});
    const menuLabels = entries.map((entry) => entry.menuLabel).filter((label): label is string => Boolean(label));

    expect(menuLabels).toContain("├─ Pack: abomination-vaults");
    expect(menuLabels).toContain("├─ ! Pack: monster-core");
    expect(menuLabels.some((label) => label.includes("Exclude"))).toBe(false);
    expect(entries.filter((entry) => entry.kind === "queryNode" && entry.label === "Pack: monster-core")).toHaveLength(0);
  });
});
