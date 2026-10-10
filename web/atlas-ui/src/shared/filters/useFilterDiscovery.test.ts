import { findClauseIds } from "./useFilterDiscovery";
import type { QueryPredicate } from "../../generated/atlas";

describe("contextual facet clause selection", () => {
  it("retains every same-field clause and distinguishes root and owned scopes", () => {
    const predicate: QueryPredicate = {
      kind: "all_of",
      children: [
        {
          kind: "compare",
          clause_id: "low",
          field: "actor.level",
          op: "gte",
          value: 3,
        },
        {
          kind: "compare",
          clause_id: "high",
          field: "actor.level",
          op: "lte",
          value: 7,
        },
        {
          kind: "exists",
          collection: "actor.items",
          scope_id: "items",
          predicate: {
            kind: "compare",
            clause_id: "child",
            field: "item.level",
            op: "eq",
            value: 2,
          },
        },
        { kind: "compare", clause_id: "root", field: "item.level", op: "eq", value: 4 },
      ],
    };
    expect(findClauseIds(predicate, "actor.level")).toEqual(["low", "high"]);
    expect(findClauseIds(predicate, "item.level", "actor.items")).toEqual(["child"]);
    expect(findClauseIds(predicate, "item.level")).toEqual(["root"]);
  });
});
