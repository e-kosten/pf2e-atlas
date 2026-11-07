import type { RuleGroupType } from "react-querybuilder";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import { normalizeBuilderEdit } from "./builderEdits";

const kind = fieldFixture("kind");
const collection = fieldFixture("embedded.equipment", "collection");
const child = { ...fieldFixture("item.name"), scope: collection.path };
const fields = editorFixture([kind, collection, child]).groups[0].fields;

describe("atomic builder edits", () => {
  it("preserves the first value supplied with a new condition", () => {
    const previous: RuleGroupType = { combinator: "and", rules: [] };
    const next: RuleGroupType = {
      ...previous,
      rules: [{ id: "new", field: kind.id, operator: "eq", value: "creature" }],
    };
    expect(normalizeBuilderEdit(previous, next, fields, null)).toEqual(next);
  });
  it("converts a singleton list before the scalar control is rendered", () => {
    const previous = {
      combinator: "and",
      rules: [{ id: "kind", field: kind.id, operator: "in", value: ["companion"] }],
    } satisfies RuleGroupType;
    const next = { ...previous, rules: [{ ...previous.rules[0], operator: "neq" }] };
    expect(normalizeBuilderEdit(previous, next, fields, null).rules[0]).toMatchObject({
      operator: "neq",
      value: "companion",
    });
  });
  it("clears ambiguous lists rather than silently choosing the first value", () => {
    const previous = {
      combinator: "and",
      rules: [
        {
          id: "kind",
          field: kind.id,
          operator: "in",
          value: ["creature", "companion"],
        },
      ],
    } satisfies RuleGroupType;
    const next = { ...previous, rules: [{ ...previous.rules[0], operator: "eq" }] };
    expect(normalizeBuilderEdit(previous, next, fields, null).rules[0]).toMatchObject({
      value: "",
    });
  });
  it("converts restored rules when the library assigns their initial IDs", () => {
    const previous: RuleGroupType = {
      combinator: "and",
      rules: [{ field: kind.id, operator: "in", value: ["companion"] }],
    };
    const next: RuleGroupType = {
      ...previous,
      rules: [
        { id: "assigned-id", field: kind.id, operator: "neq", value: ["companion"] },
      ],
    };
    expect(normalizeBuilderEdit(previous, next, fields, null).rules[0]).toMatchObject({
      id: "assigned-id",
      value: "companion",
    });
  });
  it("replaces availability with a nested group before rendering Exists", () => {
    const previous = {
      combinator: "and",
      rules: [
        { id: "items", field: collection.id, operator: "state", value: "missing" },
      ],
    } satisfies RuleGroupType;
    const next = { ...previous, rules: [{ ...previous.rules[0], operator: "exists" }] };
    expect(normalizeBuilderEdit(previous, next, fields, null).rules[0]).toMatchObject({
      value: { combinator: "and", rules: [] },
    });
  });
  it("applies the same transition policy within a collection child scope", () => {
    const group = {
      combinator: "and",
      rules: [
        { id: "child", field: child.id, operator: "in", value: ["Staff of Fire"] },
      ],
    } satisfies RuleGroupType;
    const previous = {
      combinator: "and",
      rules: [{ id: "items", field: collection.id, operator: "exists", value: group }],
    } satisfies RuleGroupType;
    const next = {
      ...previous,
      rules: [
        {
          ...previous.rules[0],
          value: {
            ...group,
            rules: [{ ...group.rules[0], operator: "eq" }],
          },
        },
      ],
    };
    expect(normalizeBuilderEdit(previous, next, fields, null).rules[0]).toMatchObject({
      value: { rules: [{ value: "Staff of Fire" }] },
    });
  });
});
