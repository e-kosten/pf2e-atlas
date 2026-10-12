import {
  advancedResidual,
  changeChoiceValue,
  choiceValueState,
  replaceAdvancedResidual,
  clearAllFilters,
  choiceCondition,
  commonCondition,
  replaceCommonCondition,
} from "./filterControls";
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

describe("common filter editing without predicate simplification", () => {
  const kind = fieldFixture("record.kind");
  const level = fieldFixture("record.level_or_rank", "number");
  const traits = fieldFixture("traits", "set");
  const leaf = {
    kind: "in" as const,
    clause_id: "kind",
    field: "record.kind",
    values: ["creature"],
  };
  const other = {
    kind: "compare" as const,
    clause_id: "hp",
    field: "actor.hp.maximum",
    op: "gte" as const,
    value: 80,
  };
  it("preserves conjunction identity and unrelated clauses on change and clear", () => {
    const predicate = {
      kind: "all_of" as const,
      clause_id: "root",
      children: [leaf, other],
    };
    const replacement = choiceCondition(kind, leaf, ["hazard"], true)!;
    expect(replaceCommonCondition(predicate, kind, replacement)).toEqual({
      ...predicate,
      children: [replacement, other],
    });
    expect(replaceCommonCondition(predicate, kind, null)).toEqual({
      ...predicate,
      children: [other],
    });
    expect(replaceCommonCondition(predicate, kind, replacement)?.kind).toBe("all_of");
  });
  it("recognizes imported direct Not(In) and retains the inner clause ID across include/exclude", () => {
    const excluded = { kind: "not" as const, clause_id: "exclude", predicate: leaf };
    const predicate = {
      kind: "all_of" as const,
      clause_id: "root",
      children: [excluded, other],
    };
    expect(commonCondition(predicate, kind)).toEqual({
      status: "editable",
      clause: excluded,
    });
    expect(choiceCondition(kind, excluded, ["spell"], false)).toEqual({
      ...leaf,
      values: ["spell"],
    });
    expect(choiceCondition(kind, excluded, ["spell"], true)).toEqual({
      ...excluded,
      predicate: { ...leaf, values: ["spell"] },
    });
  });
  it("leaves OR, arbitrary NOT groups, scoped clauses and repeated conditions in advanced", () => {
    for (const predicate of [
      { kind: "any_of" as const, children: [leaf, other] },
      {
        kind: "not" as const,
        predicate: { kind: "all_of" as const, children: [leaf] },
      },
      {
        kind: "exists" as const,
        scope_id: "owned",
        collection: "actor.items",
        predicate: leaf,
      },
      { kind: "all_of" as const, children: [leaf, { ...leaf, clause_id: "second" }] },
    ]) {
      expect(commonCondition(predicate, kind).status).toBe("advanced");
      expect(() => replaceCommonCondition(predicate, kind, null)).toThrow("advanced");
    }
  });
  it("recognizes one exact range group without flattening separately authored bounds", () => {
    const low = {
      kind: "compare" as const,
      clause_id: "low",
      field: level.path,
      op: "gte" as const,
      value: 2,
    };
    const high = { ...low, clause_id: "high", op: "lte" as const, value: 8 };
    const range = {
      kind: "all_of" as const,
      clause_id: "range",
      children: [low, high],
    };
    expect(
      commonCondition({ kind: "all_of", children: [range, other] }, level).status,
    ).toBe("editable");
    expect(
      commonCondition({ kind: "all_of", children: [low, high, other] }, level).status,
    ).toBe("advanced");
    expect(
      commonCondition({ ...range, children: [low, { ...high, op: "lt" }] }, level)
        .status,
    ).toBe("advanced");
  });
  it("uses exact typed include/exclude operators for traits", () => {
    expect(choiceCondition(traits, null, ["fire"], false)).toMatchObject({
      kind: "set_match",
      field: "traits",
      op: "includes_any",
      values: ["fire"],
    });
    expect(choiceCondition(traits, null, ["fire"], true)).toMatchObject({
      kind: "set_match",
      field: "traits",
      op: "excludes_any",
      values: ["fire"],
    });
  });
  it("preserves distinct Exists scope and clause identities in the advanced adapter", () => {
    const collection = fieldFixture("actor.items", "collection");
    const child = {
      ...fieldFixture("owned.name"),
      path: "name",
      scope: collection.path,
    };
    const predicate = {
      kind: "all_of" as const,
      children: [
        {
          kind: "exists" as const,
          clause_id: "clause",
          scope_id: "scope",
          collection: collection.path,
          predicate: {
            kind: "all_of" as const,
            children: [
              {
                kind: "compare" as const,
                clause_id: "name",
                field: "name",
                op: "eq" as const,
                value: "Jaws",
              },
            ],
          },
        },
      ],
    };
    expect(
      builderPredicate(predicateBuilder(predicate, [collection, child]), [
        collection,
        child,
      ]),
    ).toEqual(predicate);
  });
});

describe("mixed common values and residual advanced editing", () => {
  const kind = fieldFixture("record.kind");
  const traits = fieldFixture("traits", "set");
  it.each([kind, traits])("keeps mixed states and surviving IDs for $id", (field) => {
    let condition = changeChoiceValue(field, null, "creature", "include");
    condition = changeChoiceValue(field, condition, "spell", "include");
    const includeId = condition?.clause_id;
    condition = changeChoiceValue(field, condition, "hazard", "exclude");
    expect(commonCondition(condition, field).status).toBe("editable");
    expect(choiceValueState(condition, "spell")).toBe("include");
    expect(choiceValueState(condition, "hazard")).toBe("exclude");
    expect(choiceValueState(condition, "missing")).toBe("neutral");
    const excludedId =
      condition?.kind === "all_of"
        ? condition.children[1].kind === "not"
          ? condition.children[1].predicate.clause_id
          : condition.children[1].clause_id
        : null;
    condition = changeChoiceValue(field, condition, "spell", "exclude");
    expect(condition?.kind === "all_of" ? condition.children[0] : null).toMatchObject({
      clause_id: includeId,
      values: ["creature"],
    });
    expect(choiceValueState(condition, "spell")).toBe("exclude");
    const excluded = condition?.kind === "all_of" ? condition.children[1] : null;
    expect(
      excluded?.kind === "not" ? excluded.predicate.clause_id : excluded?.clause_id,
    ).toBe(excludedId);
    condition = changeChoiceValue(field, condition, "hazard", "neutral");
    expect(choiceValueState(condition, "spell")).toBe("exclude");
    expect(choiceValueState(condition, "hazard")).toBe("neutral");
  });
  it("recognizes direct opposite root clauses and retains unrelated group identities", () => {
    const included = {
      kind: "in" as const,
      clause_id: "include",
      field: kind.path,
      values: ["creature"],
    };
    const excluded = {
      kind: "not" as const,
      clause_id: "not",
      predicate: { ...included, clause_id: "exclude", values: ["hazard"] },
    };
    const other = {
      kind: "compare" as const,
      clause_id: "hp",
      field: "actor.hp.maximum",
      op: "gte" as const,
      value: 80,
    };
    const predicate = {
      kind: "all_of" as const,
      clause_id: "root",
      children: [included, other, excluded],
    };
    const projection = advancedResidual(predicate, [kind]);
    expect(projection.residual).toEqual({ ...predicate, children: [other] });
    const updated = replaceAdvancedResidual(predicate, [kind], {
      ...predicate,
      children: [{ ...other, value: 90 }],
    });
    expect(updated).toEqual({
      ...predicate,
      children: [included, { ...other, value: 90 }, excluded],
    });
    const clear = replaceCommonCondition(predicate, kind, null);
    expect(clear).toEqual({ ...predicate, children: [other] });
  });
  it("keeps advanced OR residual grouped under common conjunction", () => {
    const common = {
      kind: "in" as const,
      clause_id: "kind",
      field: kind.path,
      values: ["creature"],
    };
    const a = {
      kind: "compare" as const,
      clause_id: "hp",
      field: "actor.hp.maximum",
      op: "gte" as const,
      value: 80,
    };
    const or = {
      kind: "any_of" as const,
      clause_id: "or",
      children: [a, { ...a, clause_id: "hp2", value: 120 }],
    };
    const predicate = {
      kind: "all_of" as const,
      clause_id: "root",
      children: [common, or],
    };
    expect(
      replaceAdvancedResidual(predicate, [kind], { ...or, children: [a] }),
    ).toEqual({ ...predicate, children: [common, { ...or, children: [a] }] });
    expect(advancedResidual({ kind: "any_of", children: [common, a] }, [kind])).toEqual(
      { common: [], residual: { kind: "any_of", children: [common, a] } },
    );
  });
});

describe("structural boolean constants in the maintained builder", () => {
  it.each([true, false])(
    "roundtrips %s with IDs under root, OR, NOT, nested NOT and Exists",
    (value) => {
      const collection = fieldFixture("actor.items", "collection");
      const constant = {
        kind: "boolean_constant" as const,
        clause_id: "constant",
        value,
      };
      const other = { ...constant, clause_id: "other", value: !value };
      const not = { kind: "not" as const, clause_id: "not", predicate: constant };
      const predicates = [
        constant,
        { kind: "any_of" as const, clause_id: "or", children: [constant, other] },
        not,
        { kind: "not" as const, clause_id: "outer-not", predicate: not },
        {
          kind: "exists" as const,
          clause_id: "exists",
          scope_id: "scope",
          collection: collection.path,
          predicate: constant,
        },
      ];
      for (const predicate of predicates)
        expect(
          builderPredicate(predicateBuilder(predicate, [collection]), [collection]),
        ).toEqual(predicate);
    },
  );
});
