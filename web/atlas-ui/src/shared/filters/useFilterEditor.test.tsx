import { act, renderHook } from "@testing-library/react";
import { useState } from "react";
import type { RuleGroupType } from "react-querybuilder";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "./searchState";
import { useFilterEditor } from "./useFilterEditor";

const hp = fieldFixture("defenses.hp.maximum", "number", "Maximum HP");
const fields = editorFixture([hp]).groups[0].fields;
const initial = {
  ...DEFAULT_SEARCH_STATE,
  filter: { kind: "compare", clause_id: "hp", field: hp.path, op: "gte", value: 80 },
} satisfies SearchFormState;
function harness() {
  return renderHook(() => {
    const [search, setSearch] = useState<SearchFormState>(initial);
    return { search, setSearch, editor: useFilterEditor(search, fields, setSearch) };
  });
}

describe("persistent filter editing lifecycle", () => {
  it("preserves an empty authored group after a valid commit", () => {
    const { result } = harness();
    const query: RuleGroupType = {
      combinator: "and",
      rules: [
        { id: "hp", field: hp.id, operator: "gte", value: 80 },
        { id: "new-group", combinator: "or", rules: [] },
      ],
    };
    act(() => result.current.editor.change(query));
    expect(result.current.editor.query).toBe(query);
    expect(result.current.editor.query?.rules).toHaveLength(2);
    expect(result.current.search.filter).toMatchObject({
      kind: "all_of",
      children: [{ kind: "compare", value: 80 }],
    });
  });
  it("keeps Between as a range after projecting two executable comparisons", () => {
    const { result } = harness();
    const query: RuleGroupType = {
      combinator: "and",
      rules: [{ id: "range", field: hp.id, operator: "between", value: [80, 120] }],
    };
    act(() => result.current.editor.change(query));
    expect(result.current.editor.query).toBe(query);
    expect(result.current.search.filter).toMatchObject({
      kind: "all_of",
      children: [
        {
          kind: "all_of",
          children: [
            { op: "gte", value: 80 },
            { op: "lte", value: 120 },
          ],
        },
      ],
    });
  });
  it("retains an invalid draft through preview navigation with restored clause IDs", () => {
    const { result } = harness();
    const query: RuleGroupType = {
      combinator: "and",
      rules: [{ field: hp.id, operator: "gte", value: "-" }],
    };
    act(() => result.current.editor.change(query));
    expect(result.current.search.filter).toEqual(initial.filter);
    act(() =>
      result.current.setSearch({
        ...result.current.search,
        filter: { ...initial.filter, clause_id: "url-clause-1" },
      }),
    );
    expect(result.current.editor.query).toBe(query);
    expect(result.current.editor.error).toBe("Enter a finite numeric value");
  });
  it("restores external changed criteria and does not revive an old draft on Back", () => {
    const { result } = harness();
    act(() =>
      result.current.editor.change({
        combinator: "and",
        rules: [{ field: hp.id, operator: "gte", value: "-" }],
      }),
    );
    act(() =>
      result.current.setSearch({
        ...initial,
        filter: { ...initial.filter, value: 120 },
      }),
    );
    expect(result.current.editor.query?.rules[0]).toMatchObject({ value: 120 });
    act(() => result.current.setSearch(initial));
    expect(result.current.editor.query?.rules[0]).toMatchObject({ value: 80 });
    expect(result.current.editor.error).toBeNull();
  });
  it("keeps the shared filter draft when query text changes with the same criteria", () => {
    const { result } = harness();
    const query: RuleGroupType = {
      combinator: "and",
      rules: [{ field: hp.id, operator: "gte", value: "-" }],
    };
    act(() => result.current.editor.change(query));
    act(() => result.current.setSearch({ ...initial, query: "zombie", pageSize: 50 }));
    expect(result.current.editor.query).toBe(query);
    expect(result.current.editor.error).toBe("Enter a finite numeric value");
  });
});

it("ANDs a link with the entire committed tree while retaining an unfinished numeric sibling", () => {
  const { result } = harness();
  const query: RuleGroupType = {
    combinator: "and",
    rules: [{ id: "hp", field: hp.id, operator: "gte", value: "-" }],
  };
  act(() => result.current.editor.change(query));
  const relationship = {
    direction: "incoming" as const,
    include_contents: true,
    selection: {
      record_key: "items:armor",
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    },
  };
  act(() => {
    expect(result.current.editor.addLink(relationship, "new-link")).toBeNull();
  });
  expect(result.current.search.filter).toMatchObject({
    kind: "all_of",
    children: [
      initial.filter,
      { kind: "relationship", clause_id: "new-link", relationship },
    ],
  });
  expect(result.current.editor.query?.rules[0]).toMatchObject({ value: "-" });
  expect(result.current.editor.query?.rules[1]).toMatchObject({
    id: "new-link",
    value: relationship,
  });
  expect(result.current.editor.error).toBe("Enter a finite numeric value");
});
