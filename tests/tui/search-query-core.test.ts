import { describe, expect, it } from "vitest";

import type { MetadataFilterNode } from "../../src/domain/metadata-filter-types.js";
import type { SearchFilterNode } from "../../src/domain/search-request-types.js";
import { canonicalFilterToMetadataNode } from "../../src/tui/search/query-parts.js";
import {
  canLiftSearchFilterNodeAtPath,
  canUnwrapSearchFilterNodeAtPath,
  canLiftMetadataNodeAtPath,
  canUnwrapMetadataNodeAtPath,
  describeMetadataNode,
  formatMetadataNodePresentationAlias,
  formatSearchFilterNodePresentationAlias,
  isValidMetadataMoveTargetGroupPath,
  isValidSearchFilterMoveTargetGroupPath,
  liftSearchFilterNodeAtPath,
  moveMetadataNodeToGroupPath,
  moveSearchFilterNodeToGroupPath,
  toggleSearchFilterRootGroupOperator,
  unwrapSearchFilterNodeAtPath,
  unwrapMetadataNodeAtPath,
  wrapSearchFilterNodeAtPath,
  wrapMetadataNodeAtPath,
} from "../../src/tui/search/query-core.js";

describe("search query-core metric labels", () => {
  it("uses friendly creature statistics labels for actor metric predicates", () => {
    expect(
      describeMetadataNode(
        {
          field: "actorMetric",
          metric: "ability.cha.mod",
          op: "!=",
          value: 4,
        },
        {
          rootLabel: "node",
          category: "creature",
        },
      ),
    ).toMatchObject({
      label: "Creature Statistics",
      value: "ability.cha.mod != 4",
      description: "Edit or remove this creature statistics clause.",
    });
  });

  it("uses hazard and item-friendly labels instead of model-language names", () => {
    expect(
      describeMetadataNode(
        {
          field: "actorMetricCompare",
          leftMetric: "perception.mod",
          op: ">=",
          rightMetric: "save.best",
        },
        {
          rootLabel: "node",
          category: "hazard",
        },
      ).label,
    ).toBe("Hazard Statistics");

    expect(
      describeMetadataNode(
        {
          field: "itemMetric",
          metric: "weapon.range_increment",
          op: "==",
          value: 30,
        },
        {
          rootLabel: "node",
          category: "equipment",
        },
      ).label,
    ).toBe("Item Properties");
  });

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
    ).toBe("Creature Statistics: hp.value gte 10");

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
    ).toBe("Creature Statistics: hp.value gte ac.value");
  });

  it("round-trips canonical metric families through metadata conversion using metric inference", () => {
    expect(
      canonicalFilterToMetadataNode({
        kind: "metric",
        metric: "hp.value",
        op: "gte",
        value: 10,
      }),
    ).toMatchObject({
      field: "actorMetric",
      metric: "hp.value",
      op: ">=",
      value: 10,
    });

    expect(
      canonicalFilterToMetadataNode({
        kind: "metricCompare",
        leftMetric: "hp.value",
        op: "gte",
        rightMetric: "ac.value",
      }),
    ).toMatchObject({
      field: "actorMetricCompare",
      leftMetric: "hp.value",
      op: ">=",
      rightMetric: "ac.value",
    });

    expect(
      canonicalFilterToMetadataNode({
        kind: "metric",
        metric: "weapon.range_increment",
        op: "eq",
        value: 30,
      }),
    ).toMatchObject({
      field: "itemMetric",
      metric: "weapon.range_increment",
      op: "==",
      value: 30,
    });
  });

  it("preserves explicit single-child groups when wrapping and unwrapping nodes in the editor tree", () => {
    const predicate: MetadataFilterNode = {
      field: "traits",
      op: "includesAny",
      values: ["fire"],
    };

    const wrapped = wrapMetadataNodeAtPath(predicate, [], "and");
    expect(wrapped).toEqual({
      and: [predicate],
    });

    expect(
      unwrapMetadataNodeAtPath(
        {
          and: [wrapped!],
        },
        [0],
      ),
    ).toEqual({
      and: [predicate],
    });
  });

  it("moves nodes to visible group-bottom insertion targets without collapsing the source group", () => {
    const tree: MetadataFilterNode = {
      and: [
        {
          field: "traits",
          op: "includesAny",
          values: ["fire"],
        },
        {
          or: [
            {
              field: "traits",
              op: "includesAny",
              values: ["cold"],
            },
            {
              field: "traits",
              op: "includesAny",
              values: ["electricity"],
            },
          ],
        },
      ],
    };

    expect(moveMetadataNodeToGroupPath(tree, [1, 1], [])).toEqual({
      and: [
        {
          field: "traits",
          op: "includesAny",
          values: ["fire"],
        },
        {
          or: [
            {
              field: "traits",
              op: "includesAny",
              values: ["cold"],
            },
          ],
        },
        {
          field: "traits",
          op: "includesAny",
          values: ["electricity"],
        },
      ],
    });
  });

  it("keeps sibling-group move targets stable after removing a node from another nested branch", () => {
    const tree: MetadataFilterNode = {
      and: [
        {
          or: [
            {
              field: "traits",
              op: "includesAny",
              values: ["fire"],
            },
            {
              and: [
                {
                  field: "traits",
                  op: "includesAny",
                  values: ["cold"],
                },
                {
                  field: "traits",
                  op: "includesAny",
                  values: ["electricity"],
                },
              ],
            },
            {
              or: [
                {
                  field: "traits",
                  op: "includesAny",
                  values: ["acid"],
                },
              ],
            },
          ],
        },
      ],
    };

    expect(moveMetadataNodeToGroupPath(tree, [0, 1, 0], [0, 2])).toEqual({
      and: [
        {
          or: [
            {
              field: "traits",
              op: "includesAny",
              values: ["fire"],
            },
            {
              and: [
                {
                  field: "traits",
                  op: "includesAny",
                  values: ["electricity"],
                },
              ],
            },
            {
              or: [
                {
                  field: "traits",
                  op: "includesAny",
                  values: ["acid"],
                },
                {
                  field: "traits",
                  op: "includesAny",
                  values: ["cold"],
                },
              ],
            },
          ],
        },
      ],
    });
  });

  it("reports only valid move, unwrap, and lift targets for structural actions", () => {
    const tree: MetadataFilterNode = {
      and: [
        {
          not: {
            field: "traits",
            op: "includesAny",
            values: ["fire"],
          },
        },
        {
          or: [
            {
              field: "traits",
              op: "includesAny",
              values: ["cold"],
            },
          ],
        },
      ],
    };

    expect(isValidMetadataMoveTargetGroupPath(tree, [0, 0], [0])).toBe(false);
    expect(isValidMetadataMoveTargetGroupPath(tree, [0, 0], [1])).toBe(true);
    expect(canUnwrapMetadataNodeAtPath(tree, [1])).toBe(true);
    expect(canLiftMetadataNodeAtPath(tree, [0, 0])).toBe(false);
  });

  it("formats boolean-group presentation aliases for compact workspace and tree views", () => {
    const tree: MetadataFilterNode = {
      or: [
        {
          field: "traits",
          op: "includesAny",
          values: ["fire"],
        },
        {
          not: {
            field: "publicationRemaster",
            op: "eq",
            value: true,
          },
        },
      ],
    };

    expect(formatMetadataNodePresentationAlias(tree, { style: "compact" })).toBe("anyOf(2 filters)");
    expect(formatMetadataNodePresentationAlias(tree, { style: "tree" })).toBe("anyOf");
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

    expect(formatSearchFilterNodePresentationAlias(tree, { style: "compact" })).toBe("allOf(2 filters)");
    expect(formatSearchFilterNodePresentationAlias(tree, { style: "tree" })).toBe("allOf");
    expect(toggleSearchFilterRootGroupOperator(tree)).toEqual({
      kind: "anyOf",
      children: tree.children,
    });
  });
});
