import { describe, expect, it } from "vitest";

import { buildStructuredDraftEntries } from "../../src/tui/search-screen/structured-draft/structured-draft-support.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftNodeResumeTarget,
  getStructuredDraftSelectionIndexForResumeTarget,
} from "../../src/tui/search-screen/structured-draft/structured-draft-state.js";
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
      createStructuredDraftNodeResumeTarget([0, 1]),
      {},
    );
    const selectedIndex = getStructuredDraftSelectionIndexForResumeTarget(
      entries,
      createStructuredDraftNodeResumeTarget([0, 1]),
      0,
    );
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

    const entries = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([]), {
      groupedFieldValues: new Set(["traits"]),
    });
    const bucketEntries = entries.filter((entry) => entry.kind === "queryFieldBucket");

    expect(bucketEntries).toHaveLength(2);
    expect(bucketEntries[0]?.label).toBe("Traits: Include auditory, illusion");
    expect(bucketEntries[0]?.fieldMemberPaths).toEqual([[1], [2], [3]]);
    expect(bucketEntries[1]?.label).toBe("Traits: !emotion");
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(
      false,
    );
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Auditory")).toBe(
      false,
    );
  });

  it("orders grouped field buckets by field and then polarity instead of child encounter order", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "scope", category: "spell", subcategory: { kind: "any" } },
          {
            kind: "not",
            child: { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "emotion" } },
          },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "illusion" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "auditory" } },
        ],
      },
    };

    const bucketEntries = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([]), {
      groupedFieldValues: new Set(["traits"]),
    }).filter((entry) => entry.kind === "queryFieldBucket");

    expect(bucketEntries.map((entry) => entry.label)).toEqual([
      "Traits: Include auditory, illusion",
      "Traits: !emotion",
    ]);
  });

  it("keeps focused member paths as exact node targets instead of grouped bucket continuation owners", () => {
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

    const entries = buildStructuredDraftEntries(query, createStructuredDraftNodeResumeTarget([2]), {
      groupedFieldValues: new Set(["traits"]),
    });
    const selectedIndex = getStructuredDraftSelectionIndexForResumeTarget(
      entries,
      createStructuredDraftNodeResumeTarget([2]),
      0,
    );

    expect(entries[selectedIndex]?.kind).toBe("queryNode");
    expect(entries[selectedIndex]?.label).toBe("Traits: includes Auditory");
  });

  it("projects any-of active groups through shared-explorer buckets", () => {
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

    const entries = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([1]), {
      groupedFieldValues: new Set(["traits"]),
    });

    expect(entries).toContainEqual(
      expect.objectContaining({
        kind: "queryFieldBucket",
        groupPath: [1],
        field: "traits",
        memberPaths: [
          [1, 0],
          [1, 1],
        ],
      }),
    );
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(
      false,
    );
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

    const entries = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([]), {
      groupedFieldValues: new Set(["traits"]),
    });

    expect(entries.some((entry) => entry.kind === "queryFieldBucket")).toBe(false);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(
      true,
    );
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Auditory")).toBe(
      true,
    );
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
    expect(entries.filter((entry) => entry.kind === "queryNode" && entry.label === "Pack: monster-core")).toHaveLength(
      0,
    );
  });

  it("does not project a stale not-wrapped group target as the active group", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "scope", category: "creature", subcategory: { kind: "any" } },
          {
            kind: "not",
            child: {
              kind: "allOf",
              children: [
                { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "aquatic" } },
                { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "amphibious" } },
              ],
            },
          },
        ],
      },
    };

    const entries = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([1]), {
      groupedFieldValues: new Set(["traits"]),
    });

    expect(entries).toContainEqual(
      expect.objectContaining({
        kind: "queryTreeRoot",
        treePath: [],
      }),
    );
    expect(entries).not.toContainEqual(
      expect.objectContaining({
        kind: "queryTreeRoot",
        treePath: [1],
      }),
    );
    expect(entries).not.toContainEqual(
      expect.objectContaining({
        kind: "queryFieldBucket",
        groupPath: [1],
      }),
    );
  });
});
