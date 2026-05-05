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

  it("renders active-group shared-explorer metadata clauses as canonical rows", () => {
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
    expect(entries.filter((entry) => entry.kind === "queryFieldBucket")).toHaveLength(0);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(
      true,
    );
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Auditory")).toBe(
      true,
    );
    expect(
      entries.some((entry) => entry.kind === "queryNode" && entry.label === "! Traits: includes Emotion"),
    ).toBe(true);
  });

  it("keeps grouped metadata fields in encounter order instead of synthetic bucket order", () => {
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

    const traitLabels = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([]), {
      groupedFieldValues: new Set(["traits"]),
    })
      .filter((entry) => entry.kind === "queryNode" && entry.label.includes("Traits:"))
      .map((entry) => entry.label);

    expect(traitLabels).toEqual([
      "! Traits: includes Emotion",
      "Traits: includes Illusion",
      "Traits: includes Auditory",
    ]);
  });

  it("renders separate visible rows for multiple excluded grouped-field values", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "scope", category: "creature", subcategory: { kind: "any" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "humanoid" } },
          {
            kind: "not",
            child: { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "evil" } },
          },
          {
            kind: "not",
            child: { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "unholy" } },
          },
        ],
      },
    };

    const traitLabels = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([]), {
      groupedFieldValues: new Set(["traits"]),
    })
      .filter((entry) => entry.kind === "queryNode" && entry.label.includes("Traits:"))
      .map((entry) => entry.label);

    expect(traitLabels).toEqual([
      "Traits: includes Humanoid",
      "! Traits: includes Evil",
      "! Traits: includes Unholy",
    ]);
  });

  it("projects any-of active groups as canonical child rows", () => {
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

    expect(entries.some((entry) => entry.kind === "queryFieldBucket")).toBe(false);
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Illusion")).toBe(
      true,
    );
    expect(entries.some((entry) => entry.kind === "queryNode" && entry.label === "Traits: includes Auditory")).toBe(
      true,
    );
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

  it("keeps nested any-of groups visible when the root group is projected", () => {
    const query: Pf2eTerminalSearchQuery = {
      mode: "browse",
      filter: {
        kind: "allOf",
        children: [
          { kind: "scope", category: "creature", subcategory: { kind: "any" } },
          {
            kind: "anyOf",
            children: [
              { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "evil" } },
              { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "humanoid" } },
            ],
          },
          {
            kind: "not",
            child: { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "chaotic" } },
          },
          {
            kind: "not",
            child: { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "unholy" } },
          },
          { kind: "metric", metric: "ability.cha.mod", op: ">", value: 5 },
        ],
      },
    };

    const rootEntries = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([]), {
      groupedFieldValues: new Set(["traits"]),
    });
    const anyOfEntries = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget([1]), {
      groupedFieldValues: new Set(["traits"]),
    });

    expect(rootEntries.map((entry) => entry.menuLabel)).toContain("├─ Any of");
    expect(rootEntries.map((entry) => entry.menuLabel)).toContain("│  ├─ Traits: includes Evil");
    expect(rootEntries.map((entry) => entry.menuLabel)).toContain("│  ├─ Traits: includes Humanoid");
    expect(rootEntries.map((entry) => entry.menuLabel)).toContain("├─ ! Traits: includes Chaotic");
    expect(rootEntries.map((entry) => entry.menuLabel)).toContain("├─ ! Traits: includes Unholy");
    expect(anyOfEntries).toContainEqual(
      expect.objectContaining({
        kind: "queryNode",
        treePath: [1, 0],
        label: "Traits: includes Evil",
      }),
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
