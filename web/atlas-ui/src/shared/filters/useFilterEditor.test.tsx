import { act, renderHook } from "@testing-library/react";
import { useState } from "react";
import type { RuleGroupType, RuleType } from "react-querybuilder";
import type { QueryPredicate, QueryRelationship } from "../../generated/atlas";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "./searchState";
import { useFilterEditor } from "./useFilterEditor";
import { decodeFilterUrl, encodeFilterUrl } from "./filterUrl";
import { LINK_CONDITION } from "./queryBuilder";

const hp = fieldFixture("defenses.hp.maximum", "number", "Maximum HP");
const fields = editorFixture([hp]).groups[0].fields;
const initial = {
  ...DEFAULT_SEARCH_STATE,
  filter: { kind: "compare", clause_id: "hp", field: hp.path, op: "gte", value: 80 },
} satisfies SearchFormState;
function harness(start: SearchFormState = initial) {
  return renderHook(() => {
    const [search, setSearch] = useState<SearchFormState>(start);
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

it.each(["edit", "remove"] as const)(
  "keeps a repeated condition independent after a chip %s and later Advanced edit",
  (operation) => {
    const relationship: QueryRelationship = {
      direction: "incoming",
      include_contents: true,
      selection: {
        record_key: "items:armor",
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
    };
    const first: Extract<QueryPredicate, { kind: "relationship" }> = {
      kind: "relationship",
      clause_id: "first",
      relationship,
    };
    const second = { ...first, clause_id: "second" };
    const { result } = harness({
      ...DEFAULT_SEARCH_STATE,
      filter: { kind: "all_of", children: [first, second] },
    });
    const replacement =
      operation === "edit" ? { ...relationship, include_contents: false } : null;
    act(() => {
      expect(result.current.editor.editLink(first, replacement)).toBeNull();
    });
    const expected = replacement
      ? [{ ...first, relationship: replacement }, second]
      : [second];
    expect(result.current.search.filter).toMatchObject({
      kind: "all_of",
      children: expected,
    });
    act(() => result.current.editor.change(result.current.editor.query!));
    expect(result.current.search.filter).toMatchObject({
      kind: "all_of",
      children: expected,
    });
  },
);

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

it.each([
  { selectedIndex: 0, operation: "edit" },
  { selectedIndex: 1, operation: "edit" },
  { selectedIndex: 0, operation: "remove" },
  { selectedIndex: 1, operation: "remove" },
] as const)(
  "$operation affects only restored URL occurrence $selectedIndex and preserves an unfinished sibling",
  ({ selectedIndex, operation }) => {
    const relationship: QueryRelationship = {
      direction: "incoming",
      include_contents: true,
      selection: {
        record_key: "items:armor",
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
    };
    const repeated = ["first", "second"].map((clause_id) => ({
      kind: "relationship" as const,
      clause_id,
      relationship,
    }));
    const initialFilter: QueryPredicate = {
      kind: "all_of",
      clause_id: "root",
      children: [...repeated, initial.filter],
    };
    const { result } = harness({ ...initial, filter: initialFilter });
    const query = result.current.editor.query!;
    const unfinished = {
      ...query,
      rules: query.rules.map((node) =>
        "rules" in node || node.field !== hp.id ? node : { ...node, value: "-" },
      ),
    };
    act(() => result.current.editor.change(unfinished));
    const restored = decodeFilterUrl(encodeFilterUrl(initialFilter));
    act(() => result.current.setSearch({ ...result.current.search, filter: restored }));
    expect(result.current.editor.query).toBe(unfinished);
    if (restored.kind !== "all_of") throw new Error("Expected conjunction");
    const selected = restored.children[selectedIndex];
    if (selected.kind !== "relationship") throw new Error("Expected link");
    const replacement =
      operation === "edit" ? { ...relationship, include_contents: false } : null;
    act(() => {
      expect(result.current.editor.editLink(selected, replacement)).toBeNull();
    });
    const expected = restored.children.flatMap((node, index) =>
      index !== selectedIndex
        ? [node]
        : replacement
          ? [{ ...selected, relationship: replacement }]
          : [],
    );
    expect(result.current.search.filter).toMatchObject({
      kind: "all_of",
      children: expected,
    });
    const expectedDraft = repeated.flatMap((node, index) =>
      index !== selectedIndex
        ? [node]
        : replacement
          ? [{ ...node, relationship: replacement }]
          : [],
    );
    const draftRules = result.current.editor.query!.rules as RuleType[];
    expect(draftRules.find((node) => node.field === hp.id)?.value).toBe("-");
    expect(
      draftRules
        .filter((node) => node.field === LINK_CONDITION)
        .map((node) => ({ id: node.id, scope: node.value.include_contents })),
    ).toEqual(
      expectedDraft.map((node) => ({
        id: node.clause_id,
        scope: node.relationship.include_contents,
      })),
    );
    // A second restoration after removal changes occurrence paths. Its remaining
    // chip must still address the surviving draft rule, never the removed one.
    const again = decodeFilterUrl(encodeFilterUrl(result.current.search.filter!));
    act(() => result.current.setSearch({ ...result.current.search, filter: again }));
    if (again.kind !== "all_of") throw new Error("Expected conjunction");
    const remaining = again.children.filter((node) => node.kind === "relationship");
    const target = remaining[remaining.length - 1];
    const nextRelationship = { ...target.relationship, direction: "outgoing" as const };
    act(() => {
      expect(result.current.editor.editLink(target, nextRelationship)).toBeNull();
    });
    expectedDraft[expectedDraft.length - 1] = {
      ...expectedDraft[expectedDraft.length - 1],
      relationship: nextRelationship,
    };
    const retained = result.current.editor.query!.rules as RuleType[];
    expect(retained.find((node) => node.field === hp.id)?.value).toBe("-");
    expect(
      retained
        .filter((node) => node.field === LINK_CONDITION)
        .map((node) => ({ id: node.id, relationship: node.value })),
    ).toEqual(
      expectedDraft.map((node) => ({
        id: node.clause_id,
        relationship: node.relationship,
      })),
    );
    act(() =>
      result.current.editor.change({
        ...result.current.editor.query!,
        rules: retained.map((node) =>
          node.field === hp.id ? { ...node, value: 90 } : node,
        ),
      }),
    );
    expect(result.current.search.filter).toMatchObject({
      kind: "all_of",
      children: [...expectedDraft, { kind: "compare", field: hp.path, value: 90 }],
    });
  },
);
