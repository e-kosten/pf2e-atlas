import { clearAllFilters } from "./filterControls";
import { DEFAULT_SEARCH_STATE } from "./searchState";
import { builderPredicate, predicateBuilder, numericLiteral } from "./queryBuilder";
import { fieldFixture } from "../../test/fixtures";
describe("typed builder translation", () => {
  it("rejects unsafe integers and nonfinite numbers before query submission", () => {
    for (const value of ["9007199254740993", "NaN", "Infinity", "-Infinity", ""])
      expect(() => numericLiteral(value)).toThrow();
    expect(numericLiteral("0")).toBe(0);
    expect(numericLiteral("0.5")).toBe(0.5);
  });
  it("preserves OR, NOT and one same-child witness", () => {
    const collection = fieldFixture("actor.items", "collection");
    const child = {
      ...fieldFixture("item.name"),
      path: "common.name",
      scope: "actor.items",
    };
    const fields = [collection, child];
    const predicate = builderPredicate(
      {
        combinator: "or",
        not: true,
        rules: [
          {
            id: "child",
            field: collection.id,
            operator: "exists",
            value: {
              combinator: "and",
              rules: [
                { id: "name", field: child.id, operator: "eq", value: "Ghoul Fever" },
              ],
            },
          },
        ],
      },
      fields,
    );
    expect(predicate).toMatchObject({
      kind: "not",
      predicate: {
        kind: "any_of",
        children: [
          {
            kind: "exists",
            collection: "actor.items",
            scope_id: "child",
            predicate: {
              kind: "all_of",
              children: [
                { kind: "compare", field: "common.name", value: "Ghoul Fever" },
              ],
            },
          },
        ],
      },
    });
    expect(builderPredicate(predicateBuilder(predicate, fields), fields)).toEqual(
      predicate,
    );
  });
  it("rejects unknown fields/operators rather than dropping clauses", () => {
    expect(() =>
      builderPredicate(
        {
          combinator: "and",
          rules: [{ field: "old.metric", operator: "eq", value: 1 }],
        },
        [],
      ),
    ).toThrow();
    expect(() =>
      builderPredicate(
        {
          combinator: "and",
          rules: [{ field: "actor.level", operator: "custom", value: 1 }],
        },
        [fieldFixture("actor.level", "number")],
      ),
    ).toThrow();
  });
  it("keeps all five availability states explicit and clears errors", () => {
    const field = fieldFixture("actor.level", "number");
    for (const state of ["value", "missing", "null", "invalid", "not_applicable"])
      expect(
        builderPredicate(
          {
            combinator: "and",
            rules: [{ field: field.id, operator: "state", value: state }],
          },
          [field],
        ),
      ).toMatchObject({ children: [{ kind: "state_match", state }] });
    expect(
      clearAllFilters({ ...DEFAULT_SEARCH_STATE, filterError: "Invalid" }),
    ).toEqual(DEFAULT_SEARCH_STATE);
  });
});
