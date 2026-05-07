import { describe, expect, it } from "vitest";

import type { SearchFilterNode } from "../../src/domain/search-request-types.js";
import {
  appendSearchFilterNodesAtPath,
  canLiftSearchFilterNodeAtPath,
  canUnwrapSearchFilterNodeAtPath,
  formatSearchFilterNodePresentationAlias,
  isValidSearchFilterMoveTargetGroupPath,
  liftSearchFilterNodeAtPath,
  moveSearchFilterNodeToGroupPath,
  toggleSearchFilterRootGroupOperator,
  unwrapSearchFilterNodeAtPath,
  wrapSearchFilterNodeAtPath,
} from "../../src/tui/search/query-core.js";

describe("search query-core metric labels", () => {
  it("renders pack clauses with user-facing labels while keeping canonical pack names in the filter node", () => {
    expect(
      formatSearchFilterNodePresentationAlias(
        {
          kind: "pack",
          value: "pathfinder-npc-core",
        },
        {
          packLabelResolver: (packValue) => (packValue === "pathfinder-npc-core" ? "Pathfinder NPC Core" : packValue),
        },
      ),
    ).toBe("Pack: Pathfinder NPC Core");
  });

  it("renders linkedFrom clauses with a stable canonical label", () => {
    expect(
      formatSearchFilterNodePresentationAlias({
        kind: "linkedFrom",
        source: "actions:action-refocus-1",
      }),
    ).toBe("Linked From: actions:action-refocus-1");
  });

  it("infers creature-statistics labels from real metric keys instead of naive prefixes", () => {
    expect(
      formatSearchFilterNodePresentationAlias(
        {
          kind: "metric",
          metric: "hp.value",
          op: "gte",
          value: 10,
        },
        {
          category: "creature",
        },
      ),
    ).toBe("Creature Statistics: hp.value >= 10");

    expect(
      formatSearchFilterNodePresentationAlias(
        {
          kind: "metricCompare",
          leftMetric: "hp.value",
          op: "gte",
          rightMetric: "ac.value",
        },
        {
          category: "creature",
        },
      ),
    ).toBe("Creature Statistics: hp.value >= ac.value");
  });

  it("appends multiple peer canonical nodes into the selected group without wrapping them", () => {
    const tree: SearchFilterNode = {
      kind: "allOf",
      children: [
        {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
        {
          kind: "anyOf",
          children: [{ kind: "pack", value: "alpha" }],
        },
      ],
    };

    expect(
      appendSearchFilterNodesAtPath(
        tree,
        [1],
        [
          { kind: "pack", value: "monster-core" },
          { kind: "pack", value: "pathfinder-npc-core" },
        ],
      ),
    ).toEqual({
      kind: "allOf",
      children: [
        {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
        {
          kind: "anyOf",
          children: [
            { kind: "pack", value: "alpha" },
            { kind: "pack", value: "monster-core" },
            { kind: "pack", value: "pathfinder-npc-core" },
          ],
        },
      ],
    });
  });

  it("preserves explicit single-child canonical groups when wrapping and unwrapping nodes in the editor tree", () => {
    const clause: SearchFilterNode = {
      kind: "pack",
      value: "spells",
    };

    const wrapped = wrapSearchFilterNodeAtPath(clause, [], "allOf");
    expect(wrapped).toEqual({
      kind: "allOf",
      children: [clause],
    });

    expect(
      unwrapSearchFilterNodeAtPath(
        {
          kind: "allOf",
          children: [wrapped!],
        },
        [0],
      ),
    ).toEqual({
      kind: "allOf",
      children: [clause],
    });
  });

  it("moves canonical nodes into later sibling groups without dropping the extracted node", () => {
    const tree: SearchFilterNode = {
      kind: "allOf",
      children: [
        {
          kind: "anyOf",
          children: [{ kind: "pack", value: "alpha" }],
        },
        {
          kind: "pack",
          value: "middle",
        },
        {
          kind: "anyOf",
          children: [{ kind: "pack", value: "beta" }],
        },
      ],
    };

    expect(moveSearchFilterNodeToGroupPath(tree, [0, 0], [2])).toEqual({
      kind: "allOf",
      children: [
        {
          kind: "pack",
          value: "middle",
        },
        {
          kind: "anyOf",
          children: [
            { kind: "pack", value: "beta" },
            { kind: "pack", value: "alpha" },
          ],
        },
      ],
    });
  });

  it("reports valid canonical move, unwrap, and lift targets for structural actions", () => {
    const tree: SearchFilterNode = {
      kind: "allOf",
      children: [
        {
          kind: "not",
          child: {
            kind: "pack",
            value: "alpha",
          },
        },
        {
          kind: "anyOf",
          children: [
            {
              kind: "allOf",
              children: [{ kind: "pack", value: "beta" }],
            },
          ],
        },
      ],
    };

    expect(isValidSearchFilterMoveTargetGroupPath(tree, [0, 0], [0])).toBe(false);
    expect(isValidSearchFilterMoveTargetGroupPath(tree, [0, 0], [1])).toBe(true);
    expect(canUnwrapSearchFilterNodeAtPath(tree, [1, 0])).toBe(true);
    expect(canLiftSearchFilterNodeAtPath(tree, [1, 0, 0])).toBe(true);
    expect(liftSearchFilterNodeAtPath(tree, [1, 0, 0])).toEqual({
      kind: "allOf",
      children: [
        {
          kind: "not",
          child: {
            kind: "pack",
            value: "alpha",
          },
        },
        {
          kind: "anyOf",
          children: [
            {
              kind: "pack",
              value: "beta",
            },
          ],
        },
      ],
    });
  });

  it("formats and toggles canonical boolean-group aliases for workspace and tree views", () => {
    const tree: SearchFilterNode = {
      kind: "allOf",
      children: [
        { kind: "pack", value: "spells" },
        { kind: "pack", value: "feats" },
      ],
    };

    expect(formatSearchFilterNodePresentationAlias(tree, { style: "compact" })).toBe("All of (2 filters)");
    expect(formatSearchFilterNodePresentationAlias(tree, { style: "tree" })).toBe("All of");
    expect(toggleSearchFilterRootGroupOperator(tree)).toEqual({
      kind: "anyOf",
      children: tree.children,
    });
  });

  it("renders simple negation aliases inline with a leading bang", () => {
    const tree: SearchFilterNode = {
      kind: "not",
      child: { kind: "pack", value: "monster-core" },
    };

    expect(formatSearchFilterNodePresentationAlias(tree, { style: "compact" })).toBe("! Pack: monster-core");
    expect(formatSearchFilterNodePresentationAlias(tree, { style: "tree" })).toBe("! Pack: monster-core");
  });
});
