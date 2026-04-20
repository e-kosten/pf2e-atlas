import { describe, expect, it } from "vitest";

import { describeMetadataNode } from "../../src/tui/search/query-core.js";

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
});
