import {
  changeChoiceValue,
  choiceValueState,
  clearAllFilters,
  choiceCondition,
  commonCondition,
  replaceCommonCondition,
} from "./filterControls";
import { DEFAULT_SEARCH_STATE } from "./searchState";
import {
  builderPredicate,
  predicateBuilder,
  numericLiteral,
  replaceCommonDraft,
  canReplaceCommonDraft,
  commonDraftCondition,
  commonDraftRange,
  replaceCommonRangeDraft,
  type PredicateRule,
} from "./queryBuilder";
import { fieldFixture } from "../../test/fixtures";
import type { QueryPredicate } from "../../generated/atlas";
describe("typed builder translation", () => {
  it("restores atomic NOT as an excluded rule and keeps its toggle across edits", () => {
    const field = fieldFixture("kind");
    const predicate: QueryPredicate = {
      kind: "all_of",
      clause_id: "root",
      children: [
        {
          kind: "not",
          clause_id: "exclude",
          predicate: {
            kind: "in",
            clause_id: "kind",
            field: field.path,
            values: ["companion"],
          },
        },
      ],
    };
    const query = predicateBuilder(predicate, [field]);
    const rule = query.rules[0] as PredicateRule;
    expect(rule).toMatchObject({
      id: "kind",
      meta: { atlasExcluded: true, atlasNotClauseId: "exclude" },
    });
    expect(rule).not.toHaveProperty("rules");
    expect(builderPredicate(query, [field])).toEqual(predicate);
    for (const excluded of [false, true, false, true]) {
      rule.meta = { ...rule.meta, atlasExcluded: excluded };
      const restored = predicateBuilder(builderPredicate(query, [field]), [field]);
      expect(restored.rules).toHaveLength(1);
      expect(restored.rules[0]).not.toHaveProperty("rules");
      expect(restored.rules[0]).toMatchObject({
        id: "kind",
        ...(excluded
          ? { meta: { atlasExcluded: true, atlasNotClauseId: "exclude" } }
          : {}),
      });
    }
  });
  it("preserves authored double negation rather than applying operator inversion", () => {
    const field = fieldFixture("kind");
    const predicate: QueryPredicate = {
      kind: "not",
      clause_id: "outer",
      predicate: {
        kind: "not",
        clause_id: "inner",
        predicate: {
          kind: "compare",
          clause_id: "kind",
          field: field.path,
          op: "eq",
          value: "creature",
        },
      },
    };
    expect(builderPredicate(predicateBuilder(predicate, [field]), [field])).toEqual(
      predicate,
    );
  });
  it("keeps unavailable URL fields in the editor while blocking execution", () => {
    const predicate: QueryPredicate = {
      kind: "not",
      clause_id: "exclude",
      predicate: {
        kind: "compare",
        clause_id: "unknown",
        field: "retired.field",
        op: "eq",
        value: "original value",
      },
    };
    const query = predicateBuilder(predicate, []);
    expect(query.rules[0]).toMatchObject({
      field: "retired.field",
      value: "original value",
      meta: { atlasExcluded: true },
    });
    expect(() => builderPredicate(query, [])).toThrow("available filter field");
  });
  it("retains unavailable closed choices for repair but refuses to execute them", () => {
    const field = { ...fieldFixture("kind"), choices: ["creature", "hazard"] };
    const query = predicateBuilder(
      { kind: "in", field: field.path, values: ["action"] },
      [field],
    );
    expect(query.rules[0]).toMatchObject({ value: ["action"] });
    expect(() => builderPredicate(query, [field])).toThrow("available value");
  });
  it("keeps comma-containing vocabulary literals intact and enforces Includes cardinality", () => {
    const field = {
      ...fieldFixture("traits", "set"),
      operators: ["includes", "includes_any", "state"],
    };
    const query = {
      combinator: "and",
      rules: [
        { id: "trait", field: field.id, operator: "includes", value: "fire, cold" },
      ],
    };
    const predicate = builderPredicate(query, [field]);
    expect(predicate).toMatchObject({
      children: [{ kind: "set_match", op: "includes", values: ["fire, cold"] }],
    });
    expect(predicateBuilder(predicate, [field]).rules[0]).toMatchObject({
      value: "fire, cold",
    });
    expect(() =>
      builderPredicate(
        { ...query, rules: [{ ...query.rules[0], value: ["fire", "cold"] }] },
        [field],
      ),
    ).toThrow("Choose one value");
  });
  it("keeps invalid imported Includes operands without silently choosing their first value", () => {
    const field = {
      ...fieldFixture("traits", "set"),
      operators: ["includes", "includes_any", "state"],
    };
    const query = predicateBuilder(
      {
        kind: "set_match",
        field: field.path,
        op: "includes",
        values: ["fire", "cold"],
      },
      [field],
    );
    expect(query.rules[0]).toMatchObject({ value: ["fire", "cold"] });
    expect(() => builderPredicate(query, [field])).toThrow("Choose one value");
    const empty = predicateBuilder(
      { kind: "set_match", field: field.path, op: "includes", values: [] },
      [field],
    );
    expect(empty.rules[0]).toMatchObject({ value: [] });
    expect(() => builderPredicate(empty, [field])).toThrow("Choose at least one value");
  });
  it("rejects malformed collection operands with a controlled validation error", () => {
    const field = fieldFixture("embedded.equipment", "collection");
    for (const value of ["missing", ["missing"], {}, { rules: [] }])
      expect(() =>
        builderPredicate(
          {
            combinator: "and",
            rules: [{ field: field.id, operator: "exists", value }],
          },
          [field],
        ),
      ).toThrow("Choose a valid condition group");
  });
  it.each(["all_of", "any_of"] as const)(
    "keeps an excluded %s as one editable group across repeated edits",
    (kind) => {
      const field = fieldFixture("defenses.hp.maximum", "number");
      let predicate: QueryPredicate = {
        kind: "not",
        clause_id: "exclude",
        predicate: {
          kind,
          clause_id: "group",
          children: [
            {
              kind: "compare",
              clause_id: "hp",
              field: field.path,
              op: "gte",
              value: 80,
            },
          ],
        },
      };
      for (let index = 0; index < 4; index++) {
        const group = predicateBuilder(predicate, [field]);
        expect(group).toMatchObject({
          id: "group",
          not: true,
          atlasNotClauseId: "exclude",
        });
        expect(group.rules).toHaveLength(1);
        expect(group.rules[0]).toMatchObject({ id: "hp", field: field.id });
        predicate = builderPredicate(group, [field])!;
        expect(predicate).toEqual({
          kind: "not",
          clause_id: "exclude",
          predicate: {
            kind,
            clause_id: "group",
            children: [
              {
                kind: "compare",
                clause_id: "hp",
                field: field.path,
                op: "gte",
                value: 80,
              },
            ],
          },
        });
      }
    },
  );
  it("rejects unsafe integers and nonfinite numbers before query submission", () => {
    for (const value of ["9007199254740993", "NaN", "Infinity", "-Infinity", ""])
      expect(() => numericLiteral(value)).toThrow();
    expect(numericLiteral("0")).toBe(0);
    expect(numericLiteral("0.5")).toBe(0.5);
  });
  it("preserves OR, NOT and one same-child witness", () => {
    const collection = fieldFixture("embedded.equipment", "collection");
    const child = {
      ...fieldFixture("item.name"),
      path: "common.name",
      scope: "embedded.equipment",
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
            collection: "embedded.equipment",
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
          rules: [{ field: "level", operator: "custom", value: 1 }],
        },
        [fieldFixture("level", "number")],
      ),
    ).toThrow();
  });
  it("keeps all five availability states explicit and clears errors", () => {
    const field = fieldFixture("level", "number");
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
  const kind = fieldFixture("kind");
  const level = fieldFixture("level_or_rank", "number");
  const traits = fieldFixture("traits", "set");
  const leaf = {
    kind: "in" as const,
    clause_id: "kind",
    field: "kind",
    values: ["creature"],
  };
  const other = {
    kind: "compare" as const,
    clause_id: "hp",
    field: "defenses.hp.maximum",
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
  it("edits a simple condition within AND wrappers without changing unrelated groups", () => {
    const predicate: QueryPredicate = {
      kind: "all_of",
      clause_id: "root",
      children: [{ kind: "all_of", clause_id: "wrapper", children: [leaf] }, other],
    };
    expect(commonCondition(predicate, kind)).toEqual({
      status: "editable",
      clause: leaf,
    });
    const replacement = { ...leaf, values: ["hazard"] };
    expect(replaceCommonCondition(predicate, kind, replacement)).toEqual({
      ...predicate,
      children: [
        { kind: "all_of", clause_id: "wrapper", children: [replacement] },
        other,
      ],
    });
    expect(replaceCommonCondition(predicate, kind, null)).toEqual({
      ...predicate,
      children: [other],
    });
  });
  it("refreshes the common field in an unfinished builder without dropping an unrelated invalid value", () => {
    const draft = {
      combinator: "and",
      rules: [
        { id: "kind", field: kind.id, operator: "in", value: ["creature"] },
        { id: "hp", field: "defenses.hp.maximum", operator: "gte", value: "-" },
      ],
    };
    const refreshed = replaceCommonDraft(draft, kind, { ...leaf, values: ["hazard"] });
    expect(refreshed.rules).toEqual([
      expect.objectContaining({ id: "kind", value: ["hazard"] }),
      draft.rules[1],
    ]);
    expect(() =>
      builderPredicate(refreshed, [
        kind,
        fieldFixture("defenses.hp.maximum", "number"),
      ]),
    ).toThrow("finite numeric");
  });
  it("reads a valid common condition from the draft rather than its earlier execution value", () => {
    const query = {
      combinator: "and",
      rules: [
        { id: "kind", field: kind.id, operator: "in", value: ["hazard"] },
        { id: "hp", field: "defenses.hp.maximum", operator: "gte", value: "-" },
      ],
    };
    expect(commonDraftCondition(query, kind)).toEqual({
      status: "editable",
      clause: { ...leaf, values: ["hazard"] },
    });
  });
  it("retains the identity of an authored mixed-choice group in the common draft view", () => {
    const choices: QueryPredicate = {
      kind: "all_of",
      clause_id: "choices",
      children: [
        leaf,
        {
          kind: "not",
          predicate: { ...leaf, clause_id: "exclude", values: ["hazard"] },
        },
      ],
    };
    const query = {
      combinator: "and",
      rules: [
        predicateBuilder(choices, [kind]),
        { field: "defenses.hp.maximum", operator: "gte", value: "-" },
      ],
    };
    expect(commonDraftCondition(query, kind)).toEqual({
      status: "editable",
      clause: choices,
    });
  });
  it("keeps raw numeric edits in the shared tree and allows correction without losing siblings", () => {
    const query = {
      id: "root",
      combinator: "and",
      rules: [
        {
          id: "range-wrapper",
          combinator: "and",
          rules: [{ id: "level", field: level.id, operator: "gte", value: 2 }],
        },
        { id: "kind", field: kind.id, operator: "eq", value: "" },
      ],
    };
    const valid = replaceCommonRangeDraft(query, level, ["1.", ""]);
    expect(commonDraftRange(valid, level)).toEqual(["1.", ""]);
    expect(commonDraftCondition(valid, level)).toMatchObject({
      status: "editable",
      clause: { value: 1 },
    });
    const invalid = replaceCommonRangeDraft(valid, level, ["-", "5"]);
    expect(commonDraftRange(invalid, level)).toEqual(["-", "5"]);
    expect(canReplaceCommonDraft(invalid, level)).toBe(true);
    expect(invalid).toMatchObject({
      id: "root",
      rules: [
        {
          id: "range-wrapper",
          rules: [{ id: "level", operator: "between", value: ["-", "5"] }],
        },
        query.rules[1],
      ],
    });
    const corrected = replaceCommonRangeDraft(invalid, level, ["3", "5"]);
    expect(commonDraftRange(corrected, level)).toEqual(["3", "5"]);
    expect(() => builderPredicate(corrected, [level, kind])).toThrow("Enter a value");
    expect(replaceCommonRangeDraft(corrected, level, ["", ""]).rules).toEqual([
      { ...query.rules[0], rules: [] },
      query.rules[1],
    ]);
  });
  it("does not treat numeric OR, NOT, excluded leaves or repeated bounds as a common range", () => {
    const low = { field: level.id, operator: "gte", value: "-" };
    for (const query of [
      { combinator: "or", rules: [low] },
      { combinator: "and", not: true, rules: [low] },
      { combinator: "and", rules: [{ ...low, meta: { atlasExcluded: true } }] },
      { combinator: "and", rules: [low, { ...low, value: "3" }] },
      { combinator: "and", rules: [{ ...low, operator: "gt" }] },
    ]) {
      expect(commonDraftRange(query, level)).toBeNull();
      expect(canReplaceCommonDraft(query, level)).toBe(false);
      expect(() => replaceCommonRangeDraft(query, level, ["3", "5"])).toThrow(
        "advanced filters",
      );
    }
  });
  it("requires Advanced when unfinished rules move Kind into a mixed OR or excluded group", () => {
    for (const shape of [{ combinator: "or" }, { combinator: "and", not: true }]) {
      const draft = {
        ...shape,
        rules: [
          { field: kind.id, operator: "in", value: ["creature"] },
          { field: "defenses.hp.maximum", operator: "gte", value: "-" },
        ],
      };
      expect(canReplaceCommonDraft(draft, kind)).toBe(false);
      expect(() => replaceCommonDraft(draft, kind, leaf)).toThrow("advanced filters");
    }
  });
  it("does not replace same-field unfinished OR, NOT, or repeated conditions", () => {
    for (const shape of [
      { combinator: "or" },
      { combinator: "and", not: true },
      { combinator: "and" },
    ]) {
      const query = {
        ...shape,
        rules: [
          { id: "kind", field: kind.id, operator: "in", value: ["creature"] },
          { id: "draft", field: kind.id, operator: "neq", value: "" },
        ],
      };
      expect(canReplaceCommonDraft(query, kind)).toBe(false);
      expect(() => replaceCommonDraft(query, kind, leaf)).toThrow("advanced filters");
      expect(query.rules[1].value).toBe("");
    }
  });
  it("preserves authored AND wrappers and unrelated empty groups during a quick edit", () => {
    const wrapper = {
      id: "authored",
      combinator: "and",
      rules: [{ id: "kind", field: kind.id, operator: "in", value: ["creature"] }],
    };
    const empty = { id: "unfinished", combinator: "or", rules: [] };
    const query = { id: "root", combinator: "and", rules: [wrapper, empty] };
    expect(canReplaceCommonDraft(query, kind)).toBe(true);
    expect(
      replaceCommonDraft(query, kind, { ...leaf, values: ["hazard"] }),
    ).toMatchObject({
      id: "root",
      rules: [{ id: "authored", rules: [{ id: "kind", value: ["hazard"] }] }, empty],
    });
    expect(replaceCommonDraft(query, kind, null)).toEqual({
      ...query,
      rules: [{ ...wrapper, rules: [] }, empty],
    });
  });
  it("updates a mixed include/exclude group without duplicating its identity", () => {
    const condition: QueryPredicate = {
      kind: "all_of",
      clause_id: "choices",
      children: [
        leaf,
        {
          kind: "not",
          clause_id: "exclude",
          predicate: {
            ...leaf,
            clause_id: "excluded-kind",
            values: ["hazard"],
          },
        },
      ],
    };
    const query = predicateBuilder(condition, [kind]);
    const replacement = changeChoiceValue(kind, condition, "spell", "include");
    const updated = replaceCommonDraft(query, kind, replacement);
    expect(updated.id).toBe("choices");
    expect(updated.rules).toHaveLength(2);
    expect(updated.rules.every((node) => !("rules" in node))).toBe(true);
    expect(builderPredicate(updated, [kind])).toEqual(replacement);
  });
  it("projects the initially empty editor into one common include/exclude conjunction", () => {
    const empty = predicateBuilder(null, [kind]);
    const included = changeChoiceValue(kind, null, "creature", "include");
    const first = replaceCommonDraft(empty, kind, included);
    expect(builderPredicate(first, [kind])).toEqual(included);
    const mixed = changeChoiceValue(kind, included, "hazard", "exclude");
    const second = replaceCommonDraft(first, kind, mixed);
    expect(builderPredicate(second, [kind])).toEqual(mixed);
    expect(builderPredicate(second, [kind])).toMatchObject({
      kind: "all_of",
      children: [{ kind: "in" }, { kind: "not", predicate: { kind: "in" } }],
    });
    const excludedOnly = changeChoiceValue(kind, mixed, "creature", "neutral");
    const third = replaceCommonDraft(second, kind, excludedOnly);
    expect(builderPredicate(third, [kind])).toEqual(excludedOnly);
    expect(builderPredicate(third, [kind])?.kind).toBe("not");
  });
  it("does not strip a restored authored conjunction when common choices shrink to one NOT", () => {
    const authored: QueryPredicate = {
      kind: "all_of",
      clause_id: "authored",
      children: [
        leaf,
        {
          kind: "not",
          predicate: { ...leaf, clause_id: "excluded", values: ["hazard"] },
        },
      ],
    };
    const query = predicateBuilder(authored, [kind]);
    const excluded = changeChoiceValue(kind, authored, "creature", "neutral");
    const updated = replaceCommonDraft(query, kind, excluded);
    expect(builderPredicate(updated, [kind])).toEqual({
      kind: "all_of",
      clause_id: "authored",
      children: [excluded],
    });
  });
  it("assigns a stable identity to the first common numeric draft before mounting a builder", () => {
    const query = replaceCommonRangeDraft(predicateBuilder(null, [level]), level, [
      "-",
      "",
    ]);
    const rule = query.rules[0];
    expect(rule.id).toEqual(expect.any(String));
    expect(rule.id).not.toBe("");
    const updated = replaceCommonRangeDraft(query, level, ["3", "5"]);
    expect(updated.rules[0].id).toBe(rule.id);
    expect(commonDraftRange(updated, level)).toEqual(["3", "5"]);
  });
  it("adds a new quick condition outside an unrelated unfinished OR rather than into it", () => {
    const draft = {
      combinator: "or",
      rules: [
        { field: "defenses.hp.maximum", operator: "gte", value: "-" },
        { field: "level", operator: "eq", value: "12" },
      ],
    };
    expect(replaceCommonDraft(draft, kind, leaf)).toMatchObject({
      combinator: "and",
      rules: [draft, { field: kind.id, value: ["creature"] }],
    });
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
        scope_id: "embedded",
        collection: "embedded.equipment",
        predicate: leaf,
      },
      { kind: "all_of" as const, children: [leaf, { ...leaf, clause_id: "second" }] },
    ]) {
      expect(commonCondition(predicate, kind).status).toBe("advanced");
      expect(() => replaceCommonCondition(predicate, kind, null)).toThrow("advanced");
    }
  });
  it("recognizes grouped and separately authored range bounds without flattening the query", () => {
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
    ).toBe("editable");
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
    const collection = fieldFixture("embedded.equipment", "collection");
    const child = {
      ...fieldFixture("embedded.name"),
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

describe("mixed common values and complete advanced editing", () => {
  const kind = fieldFixture("kind");
  const traits = fieldFixture("traits", "set");
  it("round trips the complete query without growing excluded groups", () => {
    const hp = fieldFixture("defenses.hp.maximum", "number");
    const predicate: QueryPredicate = {
      kind: "all_of",
      clause_id: "root",
      children: [
        { kind: "in", clause_id: "kind", field: kind.path, values: ["creature"] },
        {
          kind: "not",
          clause_id: "exclude",
          predicate: {
            kind: "all_of",
            clause_id: "advanced",
            children: [
              {
                kind: "compare",
                clause_id: "hp",
                field: hp.path,
                op: "gte",
                value: 80,
              },
            ],
          },
        },
      ],
    };
    let next: QueryPredicate = predicate;
    for (let index = 0; index < 4; index++) {
      next = builderPredicate(predicateBuilder(next, [kind, hp]), [kind, hp])!;
      expect(next).toEqual(predicate);
    }
  });
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
      field: "defenses.hp.maximum",
      op: "gte" as const,
      value: 80,
    };
    const predicate = {
      kind: "all_of" as const,
      clause_id: "root",
      children: [included, other, excluded],
    };
    const clear = replaceCommonCondition(predicate, kind, null);
    expect(clear).toEqual({ ...predicate, children: [other] });
  });
});

describe("structural boolean constants in the maintained builder", () => {
  it.each([true, false])(
    "roundtrips %s with IDs under root, OR, NOT, nested NOT and Exists",
    (value) => {
      const collection = fieldFixture("embedded.equipment", "collection");
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
