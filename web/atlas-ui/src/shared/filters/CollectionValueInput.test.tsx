import { builderPredicate, predicateBuilder } from "./queryBuilder";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";
import type { RuleGroupType } from "react-querybuilder";
import { CollectionValueInput, isCompactCollection } from "./CollectionValueInput";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import { DEFAULT_SEARCH_STATE } from "./searchState";

it("shows local errors for a malformed optional amount and clears them on correction", async () => {
  const editor = editorFixture([
    fieldFixture("movement.speeds", "collection", "Movement Speeds"),
    {
      ...fieldFixture("movement.speeds[].type", "string", "Movement type"),
      choices: ["fly"],
    },
    fieldFixture("movement.speeds[].feet", "number", "Speed"),
  ]);
  const field = editor.groups[0].fields[0];
  field.collection_control = {
    category_field_id: "movement.speeds[].type",
    amount_field_id: "movement.speeds[].feet",
  };
  const changed = vi.fn();
  function Harness() {
    const [query, setQuery] = useState<RuleGroupType>({ combinator: "and", rules: [] });
    return (
      <CollectionValueInput
        field={field}
        editor={editor}
        query={query}
        onChange={(next) => {
          changed(next);
          setQuery(next);
        }}
        workspace={{
          search: DEFAULT_SEARCH_STATE,
          setSearch: vi.fn(),
          filterEditor: editor,
        }}
      />
    );
  }
  render(
    <QueryClientProvider client={new QueryClient()}>
      <Harness />
    </QueryClientProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Add speed" }));
  const input = screen.getByRole("textbox", { name: "Speed" });
  fireEvent.change(input, { target: { value: "abc" } });
  expect(screen.getByRole("status")).toHaveTextContent("Enter a finite numeric value");
  fireEvent.change(input, { target: { value: "30" } });
  await waitFor(() => expect(screen.queryByRole("status")).not.toBeInTheDocument());
  expect(input).toHaveValue("30");
  expect(changed).toHaveBeenLastCalledWith({
    combinator: "and",
    rules: [
      {
        id: expect.any(String),
        field: "movement.speeds[].feet",
        operator: "gte",
        value: "30",
      },
    ],
  });
  fireEvent.change(input, { target: { value: "" } });
  expect(changed).toHaveBeenLastCalledWith({ combinator: "and", rules: [] });
});

it("edits one sense and its refinements in the same collection draft", async () => {
  const senses = fieldFixture("senses", "collection", "Senses");
  const member = (
    path: string,
    type: "string" | "number",
    label: string,
    choices: string[] = [],
  ) => ({
    ...fieldFixture(`senses[].${path}`, type, label),
    path,
    scope: "senses",
    choices,
  });
  const editor = editorFixture([
    senses,
    member("type", "string", "Sense", ["scent", "tremorsense", "darkvision"]),
    member("acuity", "string", "Acuity", ["precise", "imprecise"]),
    { ...member("range", "number", "Range"), units: "feet" },
  ]);
  const field = editor.groups[0].fields[0];
  field.collection_control = {
    category_field_id: "senses[].type",
    qualifier_field_id: "senses[].acuity",
    amount_field_id: "senses[].range",
  };
  editor.sense_qualifiers = [
    { sense: "darkvision", acuity: null, has_range: false },
    { sense: "scent", acuity: "precise", has_range: true },
    { sense: "scent", acuity: "imprecise", has_range: true },
    { sense: "tremorsense", acuity: "precise", has_range: true },
    { sense: "tremorsense", acuity: "imprecise", has_range: true },
  ];
  let current: RuleGroupType = {
    id: "same-sense",
    combinator: "and",
    rules: [
      { id: "type", field: "senses[].type", operator: "eq", value: "scent" },
      { id: "acuity", field: "senses[].acuity", operator: "eq", value: "precise" },
      { id: "range", field: "senses[].range", operator: "gte", value: "20" },
    ],
  };
  function Harness() {
    const [query, setQuery] = useState(current);
    return (
      <CollectionValueInput
        field={field}
        editor={editor}
        query={query}
        onChange={(next) => {
          current = next;
          setQuery(next);
        }}
        workspace={{
          search: DEFAULT_SEARCH_STATE,
          setSearch: vi.fn(),
          filterEditor: editor,
          filterCountsEnabled: false,
        }}
      />
    );
  }
  render(
    <QueryClientProvider client={new QueryClient()}>
      <Harness />
    </QueryClientProvider>,
  );
  expect(screen.queryByRole("combobox", { name: "Sense match" })).toBeNull();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Acuity" }));
  fireEvent.click(
    await screen.findByText("imprecise", {
      selector: ".ant-select-item-option-content span",
    }),
  );
  fireEvent.change(screen.getByRole("textbox", { name: "Range" }), {
    target: { value: "30" },
  });
  expect(current).toEqual({
    id: "same-sense",
    combinator: "and",
    rules: [
      {
        id: "type",
        field: "senses[].type",
        operator: "eq",
        value: "scent",
      },
      { id: "acuity", field: "senses[].acuity", operator: "eq", value: "imprecise" },
      { id: "range", field: "senses[].range", operator: "gte", value: "30" },
    ],
  });
  expect(isCompactCollection(field, current)).toBe(true);
  const all = builderPredicate(
    {
      combinator: "and",
      rules: [
        { id: "first", field: "senses", operator: "exists", value: current },
        {
          id: "second",
          field: "senses",
          operator: "exists",
          value: {
            combinator: "and",
            rules: [
              {
                id: "second-type",
                field: "senses[].type",
                operator: "eq",
                value: "darkvision",
              },
            ],
          },
        },
      ],
    },
    editor.groups[0].fields.map((entry) => entry.definition),
  );
  expect(all).toMatchObject({
    kind: "all_of",
    children: [
      {
        kind: "exists",
        scope_id: "first",
        collection: "senses",
        predicate: {
          kind: "all_of",
          children: [
            { kind: "compare", field: "type", op: "eq", value: "scent" },
            { kind: "compare", field: "acuity", value: "imprecise" },
            { kind: "compare", field: "range", op: "gte", value: 30 },
          ],
        },
      },
      {
        kind: "exists",
        scope_id: "second",
        collection: "senses",
        predicate: {
          kind: "all_of",
          children: [{ kind: "compare", field: "type", value: "darkvision" }],
        },
      },
    ],
  });
  expect(isCompactCollection(field, { ...current, combinator: "or" })).toBe(false);
  fireEvent.change(screen.getByRole("textbox", { name: "Range" }), {
    target: { value: "" },
  });
  const acuity = screen
    .getByRole("combobox", { name: "Acuity" })
    .closest(".ant-select")!;
  fireEvent.mouseDown(acuity.querySelector(".ant-select-clear")!);
  fireEvent.click(acuity.querySelector(".ant-select-clear")!);
  await waitFor(() =>
    expect(current.rules).toEqual([
      {
        id: "type",
        field: "senses[].type",
        operator: "eq",
        value: "scent",
      },
    ]),
  );
  fireEvent.mouseDown(screen.getByRole("combobox", { name: /^Sense$/ }));
  fireEvent.click(
    await screen.findByText("darkvision", {
      selector: ".ant-select-item-option-content span",
    }),
  );
  expect(current.rules).toEqual([
    { id: "type", field: "senses[].type", operator: "eq", value: "darkvision" },
  ]);
  expect(screen.queryByRole("combobox", { name: "Acuity" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Add range" })).toBeNull();

  const definitions = editor.groups[0].fields.map((entry) => entry.definition);
  const legacy = {
    kind: "exists" as const,
    collection: "senses",
    scope_id: "legacy",
    predicate: { kind: "in" as const, field: "type", values: ["scent", "darkvision"] },
  };
  expect(
    builderPredicate(predicateBuilder(legacy, definitions), definitions),
  ).toMatchObject({
    kind: "any_of",
    children: ["scent", "darkvision"].map((value) => ({
      kind: "exists",
      collection: "senses",
      predicate: {
        kind: "all_of",
        children: [{ kind: "compare", field: "type", op: "eq", value }],
      },
    })),
  });
  const anySense = {
    ...legacy,
    predicate: { kind: "boolean_constant" as const, value: true },
  };
  const anyDraft = predicateBuilder(anySense, definitions);
  const rule = anyDraft.rules[0] as { value: RuleGroupType };
  expect(isCompactCollection(field, rule.value)).toBe(true);
  expect(builderPredicate(anyDraft, definitions)).toMatchObject(anySense);
  rule.value.rules = [];
  expect(builderPredicate(anyDraft, definitions)).toBeNull();
});
