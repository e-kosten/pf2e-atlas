import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { ConfigProvider, Grid } from "antd";
import { useState, type ComponentProps } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { FilterControls } from "./FilterPanel";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "./searchState";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import type { FilterEditorView, QueryPredicate } from "../../generated/atlas";

const actorMetricCatalog = () =>
  editorFixture([
    fieldFixture("abilities.str.modifier", "number", "Strength Modifier"),
    fieldFixture("abilities.dex.modifier", "number", "Dexterity Modifier"),
    fieldFixture("skills.athletics.modifier", "number", "Athletics Modifier"),
    fieldFixture("skills.athletics.rank", "number", "Athletics Rank"),
    fieldFixture("skills.arcana.modifier", "number", "Arcana Modifier"),
    fieldFixture("skills.arcana.rank", "number", "Arcana Rank"),
  ]);
function chooseActorMetricOption(label: string, option: string) {
  const control = screen.getByRole("combobox", { name: label });
  fireEvent.mouseDown(control);
  fireEvent.click(screen.getAllByText(option, { exact: true }).slice(-1)[0]);
}

describe("combined ability and skill conditions", () => {
  beforeEach(() => vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true }));
  afterEach(() => vi.restoreAllMocks());

  it("offers two families, waits for a complete selection, and retains numeric criteria when changing entity", async () => {
    const change = renderControls(actorMetricCatalog());
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.click(screen.getByRole("button", { name: "Add condition" }));
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Filter field" }));
    expect(screen.getByRole("option", { name: "Ability" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Skill" })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "Strength Modifier" })).toBeNull();
    expect(screen.queryByRole("option", { name: "Athletics Rank" })).toBeNull();
    fireEvent.click(screen.getAllByText("Ability", { exact: true }).slice(-1)[0]);
    expect(change).not.toHaveBeenCalled();
    chooseActorMetricOption("Ability", "Strength");
    expect(change).not.toHaveBeenCalled();
    fireEvent.change(screen.getByRole("textbox", { name: "Strength Modifier" }), {
      target: { value: "3" },
    });
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({
            kind: "compare",
            field: "abilities.str.modifier",
            value: 3,
          }),
        }),
      ),
    );
    chooseActorMetricOption("Ability", "Dexterity");
    expect(screen.getByRole("textbox", { name: "Dexterity Modifier" })).toHaveValue(
      "3",
    );
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({
            field: "abilities.dex.modifier",
            value: 3,
          }),
        }),
      ),
    );
  });

  it("uses a skill/statistic picker and named ranks without sending a UI family field to the query", async () => {
    const change = renderControls(actorMetricCatalog());
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.click(screen.getByRole("button", { name: "Add condition" }));
    chooseActorMetricOption("Filter field", "Skill");
    chooseActorMetricOption("Skill", "Athletics");
    fireEvent.change(screen.getByRole("textbox", { name: "Athletics Modifier" }), {
      target: { value: "20" },
    });
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({
            field: "skills.athletics.modifier",
            value: 20,
          }),
        }),
      ),
    );
    change.mockClear();
    chooseActorMetricOption("Skill statistic", "Proficiency rank");
    expect(change).not.toHaveBeenCalled();
    chooseActorMetricOption("Athletics Rank", "Expert");
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({ field: "skills.athletics.rank", value: 2 }),
        }),
      ),
    );
    chooseActorMetricOption("Skill", "Arcana");
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({ field: "skills.arcana.rank", value: 2 }),
        }),
      ),
    );
    expect(
      screen.getByRole("combobox", { name: "Arcana Rank" }).closest(".ant-select"),
    ).toHaveTextContent("Expert");
  });

  it("restores an existing rank condition, including an unrecognized authored numeric rank", async () => {
    const change = renderControls(actorMetricCatalog(), {
      kind: "compare",
      field: "skills.arcana.rank",
      op: "gte",
      value: 5,
      clause_id: "retained-rank",
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    expect(
      screen.getByRole("combobox", { name: "Skill" }).closest(".ant-select"),
    ).toHaveTextContent("Arcana");
    expect(
      screen.getByRole("combobox", { name: "Skill statistic" }).closest(".ant-select"),
    ).toHaveTextContent("Proficiency rank");
    expect(
      screen.getByRole("combobox", { name: "Arcana Rank" }).closest(".ant-select"),
    ).toHaveTextContent("Rank 5");
    expect(change).not.toHaveBeenCalled();
    chooseActorMetricOption("Arcana Rank", "Master");
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({
            field: "skills.arcana.rank",
            op: "gte",
            value: 3,
            clause_id: "retained-rank",
          }),
        }),
      ),
    );
  });

  it("keeps named rank ranges incomplete until both bounds exist", async () => {
    const change = renderControls(actorMetricCatalog(), {
      kind: "compare",
      field: "skills.athletics.rank",
      op: "gte",
      value: 2,
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    chooseActorMetricOption("Condition for Athletics Rank", "between");
    chooseActorMetricOption("Athletics Rank minimum", "Expert");
    expect(change).not.toHaveBeenCalled();
    chooseActorMetricOption("Athletics Rank maximum", "Legendary");
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({
            kind: "all_of",
            children: [
              expect.objectContaining({
                kind: "compare",
                field: "skills.athletics.rank",
                op: "gte",
                value: 2,
              }),
              expect.objectContaining({
                kind: "compare",
                field: "skills.athletics.rank",
                op: "lte",
                value: 4,
              }),
            ],
          }),
        }),
      ),
    );
  });

  it("retains an existing skill-rank availability condition alongside the family pickers", async () => {
    const change = renderControls(actorMetricCatalog(), {
      kind: "state_match",
      field: "skills.athletics.rank",
      state: "null",
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    chooseActorMetricOption("Athletics Rank availability", "Missing from source");
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({
          filter: expect.objectContaining({
            kind: "state_match",
            field: "skills.athletics.rank",
            state: "missing",
          }),
        }),
      ),
    );
  });
});

function renderControls(
  editor: FilterEditorView,
  predicate: QueryPredicate | null = null,
  filterError: string | null = null,
  presentation: Pick<
    ComponentProps<typeof FilterControls>,
    "layout" | "includeSearch"
  > = {},
) {
  const onChange = vi.fn();
  function Harness() {
    const [search, setSearch] = useState<SearchFormState>({
      ...DEFAULT_SEARCH_STATE,
      filter: predicate,
      filterError,
    });
    return (
      <FilterControls
        includeSearch={false}
        {...presentation}
        filterState={{
          search,
          setSearch: (next) => {
            setSearch(next);
            onChange(next);
          },
          filterEditor: editor,
        }}
      />
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return onChange;
}
describe("shared common and maintained advanced filters", () => {
  beforeEach(() => vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true }));
  afterEach(() => vi.restoreAllMocks());
  it("distinguishes open bounds, exact values and ranges while editing and clearing", async () => {
    const change = renderControls(
      editorFixture([fieldFixture("level_or_rank", "number", "Level / rank")]),
      null,
      null,
      { layout: "toolbar" },
    );
    const trigger = screen.getByRole("button", {
      name: "Edit Level / rank range",
    });
    expect(trigger).toHaveTextContent("Level / rank: Any");
    fireEvent.click(trigger);
    const minimum = await screen.findByRole("textbox", {
      name: "Level / rank minimum",
    });
    const maximum = screen.getByRole("textbox", {
      name: "Level / rank maximum",
    });
    await waitFor(() =>
      expect(screen.getByText("Minimum", { exact: true })).toBeVisible(),
    );
    expect(screen.getByText("Maximum", { exact: true })).toBeVisible();
    fireEvent.change(maximum, { target: { value: "6" } });
    expect(trigger).toHaveTextContent("Level / rank: 6 or lower");
    fireEvent.change(minimum, { target: { value: "6" } });
    expect(trigger).toHaveTextContent("Level / rank: Exactly 6");
    fireEvent.change(minimum, { target: { value: "6.0" } });
    expect(trigger).toHaveTextContent("Level / rank: Exactly 6.0");
    fireEvent.change(maximum, { target: { value: "" } });
    expect(trigger).toHaveTextContent("Level / rank: 6.0 or higher");
    fireEvent.change(minimum, { target: { value: "0" } });
    expect(trigger).toHaveTextContent("Level / rank: 0 or higher");
    fireEvent.change(minimum, { target: { value: "-1" } });
    fireEvent.change(maximum, { target: { value: "6" } });
    expect(trigger).toHaveTextContent("Level / rank: -1 – 6");
    fireEvent.click(screen.getByRole("button", { name: "Clear range" }));
    expect(trigger).toHaveTextContent("Level / rank: Any");
    expect(minimum).toHaveValue("");
    expect(maximum).toHaveValue("");
    await waitFor(() =>
      expect(change).toHaveBeenLastCalledWith(
        expect.objectContaining({ filter: null }),
      ),
    );
  });
  it("keeps the search draft, focus and fixed controls mounted while filter definitions arrive", () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const setSearch = vi.fn();
    const catalog = editorFixture([
      fieldFixture("level_or_rank", "number", "Level / rank"),
    ]);
    const view = (filterEditor?: FilterEditorView, loading = false) => (
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider client={client}>
          <FilterControls
            layout="toolbar"
            filterState={{
              search: DEFAULT_SEARCH_STATE,
              setSearch,
              filterEditor,
              filterDiscoveryLoading: loading,
            }}
          />
        </QueryClientProvider>
      </ConfigProvider>
    );
    const { rerender } = render(view(undefined, true));
    const input = screen.getByRole("searchbox", { name: "Search records" });
    const advanced = screen.getByRole("button", { name: "Advanced filters" });
    const pending = screen.getByRole("status", { name: "Loading filters" });
    expect(input.closest(".filter-controls__search-row")).not.toContainElement(pending);
    expect(pending.closest(".filter-controls__criteria")).not.toBeNull();
    input.focus();
    fireEvent.change(input, { target: { value: "unfinished spell search" } });
    rerender(view(catalog));
    expect(screen.getByRole("searchbox", { name: "Search records" })).toBe(input);
    expect(input).toHaveValue("unfinished spell search");
    expect(input).toHaveFocus();
    expect(screen.getByRole("button", { name: "Advanced filters" })).toBe(advanced);
    expect(screen.queryByRole("status", { name: "Loading filters" })).toBeNull();
    const level = screen.getByRole("button", { name: "Edit Level / rank range" });
    rerender(view(catalog, true));
    expect(screen.getByRole("button", { name: "Edit Level / rank range" })).toBe(level);
    expect(screen.queryByRole("status", { name: "Loading filters" })).toBeNull();
    expect(input).toHaveFocus();
    expect(setSearch).not.toHaveBeenCalled();
  });
  it("does not announce pending changes for an empty filter draft", () => {
    renderControls(
      editorFixture([fieldFixture("defenses.hp.maximum", "number", "Maximum HP")]),
    );
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.click(screen.getByRole("button", { name: "Add condition" }));
    expect(screen.queryByText("Changes pending")).not.toBeInTheDocument();
  });
  it("composes inline search and the same common/advanced controls once while preserving the predicate on submit", () => {
    const field = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature"],
    };
    const predicate = {
      kind: "in" as const,
      field: "kind",
      values: ["creature"],
    };
    const change = renderControls(editorFixture([field]), predicate, null, {
      layout: "inline-toolbar",
      includeSearch: true,
    });
    const input = screen.getByRole("searchbox", { name: "Search records" });
    const row = input.closest(".primary-toolbar")!;
    expect(row).toHaveClass("filter-controls__inline");
    expect(row).toContainElement(
      screen.getByRole("button", { name: "Edit Kind filter" }),
    );
    expect(row).toContainElement(
      screen.getByRole("button", { name: "Advanced filters" }),
    );
    expect(screen.getAllByRole("searchbox", { name: "Search records" })).toHaveLength(
      1,
    );
    fireEvent.change(input, { target: { value: "masked spirit" } });
    expect(change).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { key: "Enter", keyCode: 13, code: "Enter" });
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({ query: "masked spirit", filter: predicate }),
    );
  });

  it("keeps individual exclusion reversible after successful edits", () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["companion"],
    };
    const change = renderControls(editorFixture([kind]), {
      kind: "not",
      predicate: { kind: "in", field: kind.path, values: ["companion"] },
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    const operator = screen.getByRole("combobox", { name: "Condition for Kind" });
    fireEvent.mouseDown(operator);
    fireEvent.click(screen.getAllByText("in", { exact: true }).slice(-1)[0]);
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({ filter: expect.objectContaining({ kind: "in" }) }),
    );
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Condition for Kind" }));
    fireEvent.click(screen.getAllByText("is none of", { exact: true }).slice(-1)[0]);
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({ filter: expect.objectContaining({ kind: "not" }) }),
    );
  });
  it("converts a selected singleton when changing In to Is not", () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["companion"],
    };
    const change = renderControls(editorFixture([kind]), {
      kind: "in",
      field: kind.path,
      values: ["companion"],
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Condition for Kind" }));
    fireEvent.click(screen.getAllByText("neq", { exact: true }).slice(-1)[0]);
    expect(screen.queryByText("Enter a value")).toBeNull();
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({ op: "neq", value: "companion" }),
      }),
    );
  });
  it("renders restored boolean choices and commits their typed values", () => {
    const field = fieldFixture("actor.has_reactions", "boolean", "Has reactions");
    const editor = editorFixture([field]);
    editor.groups[0].fields[0].choices = [
      { value: true, label: "Yes" },
      { value: false, label: "No" },
    ];
    const change = renderControls(editor, {
      kind: "compare",
      field: field.path,
      op: "eq",
      value: false,
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    const value = screen.getByRole("combobox", { name: "Has reactions" });
    expect(value.parentElement).toHaveTextContent("No");
    fireEvent.mouseDown(value);
    fireEvent.click(screen.getAllByText("Yes", { exact: true }).slice(-1)[0]);
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({ kind: "compare", value: true }),
      }),
    );
  });
  it("keeps a newly added empty group available for its first condition", () => {
    renderControls(
      editorFixture([fieldFixture("defenses.hp.maximum", "number", "Maximum HP")]),
      {
        kind: "compare",
        field: "defenses.hp.maximum",
        op: "gte",
        value: 80,
      },
    );
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.click(screen.getByRole("button", { name: "Add group" }));
    expect(screen.getAllByRole("button", { name: "Add condition" })).toHaveLength(2);
    fireEvent.click(screen.getAllByRole("button", { name: "Add condition" })[1]);
    expect(screen.getAllByRole("combobox", { name: "Filter field" })).toHaveLength(2);
  });
  it("shares unfinished common numeric input with Advanced and corrects it there", () => {
    const level = fieldFixture("level_or_rank", "number", "Level / rank");
    const change = renderControls(editorFixture([level]), {
      kind: "compare",
      field: level.path,
      op: "gte",
      value: 1,
    });
    fireEvent.change(screen.getByRole("textbox", { name: "Level / rank minimum" }), {
      target: { value: "-" },
    });
    expect(change).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    expect(screen.getByRole("textbox", { name: "Level / rank" })).toHaveValue("-");
    fireEvent.change(screen.getByRole("textbox", { name: "Level / rank" }), {
      target: { value: "2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(screen.getByRole("textbox", { name: "Level / rank minimum" })).toHaveValue(
      "2",
    );
    expect(change).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({
        filter: expect.objectContaining({
          kind: "compare",
          field: "level_or_rank",
          op: "gte",
          value: 2,
        }),
        filterError: null,
      }),
    );
  });
  it("shows the current Kind draft while an unrelated numeric condition is unfinished", async () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature", "spell"],
    };
    const hp = fieldFixture("defenses.hp.maximum", "number", "Maximum HP");
    const change = renderControls(editorFixture([kind, hp]), {
      kind: "all_of",
      children: [
        { kind: "compare", field: kind.path, op: "eq", value: "creature" },
        { kind: "compare", field: hp.path, op: "gte", value: 80 },
      ],
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Maximum HP" }), {
      target: { value: "-" },
    });
    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Kind" }));
    fireEvent.click(screen.getAllByText("spell", { exact: true }).slice(-1)[0]);
    expect(change).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    fireEvent.click(screen.getByRole("button", { name: "Edit Kind filter" }));
    await waitFor(() =>
      expect(
        screen.getByRole("button", {
          name: "Kind: Spell, included; activate to exclude",
        }),
      ).toBeVisible(),
    );
  });
  it("explains a malformed URL with the drawer closed and lets Clear recover", () => {
    const onChange = renderControls(
      editorFixture([fieldFixture("kind")]),
      null,
      "Invalid structured filter URL",
    );
    expect(screen.getByText("Invalid structured filter URL")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(screen.queryByText("Invalid structured filter URL")).toBeNull();
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ filter: null, filterError: null }),
    );
  });
  it("uses ordered backend common labels and keeps invalid ranges local", async () => {
    const editor = editorFixture([
      fieldFixture("level_or_rank", "number", "Level / rank"),
      fieldFixture("rarity"),
    ]);
    const onChange = renderControls(editor, {
      kind: "compare",
      clause_id: "level",
      field: "level_or_rank",
      op: "gte",
      value: 1,
    });
    const input = screen.getByRole("textbox", { name: "Level / rank minimum" });
    fireEvent.change(input, { target: { value: "9007199254740993" } });
    expect(await screen.findByText(/exact numeric range/)).toBeInTheDocument();
    expect(onChange).not.toHaveBeenCalled();
    fireEvent.change(input, { target: { value: "2" } });
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filterError: null,
        filter: expect.objectContaining({ clause_id: "level", value: 2 }),
      }),
    );
    fireEvent.change(input, { target: { value: "-" } });
    expect(await screen.findByText("Enter a finite numeric value")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(screen.getByRole("textbox", { name: "Level / rank minimum" })).toHaveValue(
      "",
    );
    expect(screen.queryByText("Enter a finite numeric value")).toBeNull();
    expect(screen.queryByRole("button", { name: "Clear filters" })).toBeNull();
    expect(screen.getByRole("button", { name: "Advanced filters" })).toHaveFocus();
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ filter: null, filterError: null }),
    );
  });
  it("retains invalid range input while the catalog reloads and adopts new definitions", async () => {
    const onChange = vi.fn();
    const editor = editorFixture([
      fieldFixture("level_or_rank", "number", "Level / rank"),
    ]);
    function Harness({ catalog }: { catalog?: FilterEditorView }) {
      const [search, setSearch] = useState<SearchFormState>({
        ...DEFAULT_SEARCH_STATE,
        filter: { kind: "compare", field: "level_or_rank", op: "gte", value: 1 },
      });
      return (
        <FilterControls
          includeSearch={false}
          filterState={{
            search,
            setSearch: (next) => {
              setSearch(next);
              onChange(next);
            },
            filterEditor: catalog,
            filterDiscoveryLoading: !catalog,
          }}
        />
      );
    }
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const view = (catalog?: FilterEditorView) => (
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider client={client}>
          <Harness catalog={catalog} />
        </QueryClientProvider>
      </ConfigProvider>
    );
    const { rerender } = render(view(editor));
    fireEvent.change(screen.getByRole("textbox", { name: "Level / rank minimum" }), {
      target: { value: "9007199254740993" },
    });
    expect(await screen.findByText(/exact numeric range/)).toBeVisible();
    rerender(view());
    expect(screen.queryByRole("textbox", { name: "Level / rank minimum" })).toBeNull();
    const updated = editorFixture([
      fieldFixture("level_or_rank", "number", "Updated level"),
    ]);
    rerender(view(updated));
    expect(screen.getByRole("textbox", { name: "Updated level minimum" })).toHaveValue(
      "9007199254740993",
    );
    expect(screen.getByText(/exact numeric range/)).toBeVisible();
    expect(screen.queryByText("Level / rank")).toBeNull();
    expect(onChange).not.toHaveBeenCalled();
  });
  it("retains a complex expression and offers the advanced editor instead of flattening it", () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature", "spell"],
    };
    const predicate: QueryPredicate = {
      kind: "any_of",
      clause_id: "root",
      children: [
        { kind: "in", clause_id: "kind", field: kind.path, values: ["creature"] },
      ],
    };
    const change = renderControls(editorFixture([kind]), predicate);
    fireEvent.click(screen.getByRole("button", { name: "Kind: Advanced" }));
    expect(
      screen.getByRole("combobox", { name: "Match conditions" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Filter field" })).toBeInTheDocument();
    expect(change).not.toHaveBeenCalled();
  });
  it("keeps negative filters editable without unavailable-count notices", () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature"],
    };
    renderControls(editorFixture([kind]), {
      kind: "not",
      predicate: {
        kind: "in",
        clause_id: "kind",
        field: kind.path,
        values: ["creature"],
      },
    });
    expect(
      screen.queryByText("Counts unavailable for this condition"),
    ).not.toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Edit Kind filter" }));
    expect(
      screen.getByRole("button", {
        name: "Kind: Creature, excluded; activate to clear",
      }),
    ).toBeEnabled();
  });
  it("keeps unsafe advanced numeric drafts visible without changing committed criteria", async () => {
    const change = renderControls(
      editorFixture([fieldFixture("level", "number", "Creature level")]),
      {
        kind: "compare",
        clause_id: "level",
        field: "level",
        op: "gte",
        value: 1,
      },
    );
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Creature level" }), {
      target: { value: "-" },
    });
    expect(await screen.findByText("Enter a finite numeric value")).toBeInTheDocument();
    expect(change).not.toHaveBeenCalled();
    expect(screen.getByRole("textbox", { name: "Creature level" })).toHaveValue("-");
  });
  it("retains match-all trait semantics in the advanced editor without offering a simplified picker", () => {
    const traits = fieldFixture("traits", "set", "Traits");
    const predicate: QueryPredicate = {
      kind: "set_match",
      clause_id: "all-traits",
      field: "traits",
      op: "includes_all",
      values: ["fire", "magical"],
    };
    const change = renderControls(editorFixture([traits]), predicate);
    expect(
      screen.queryByRole("button", { name: "Edit Traits filter" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Traits: Advanced" }));
    expect(
      screen.getByRole("combobox", { name: "Condition for Traits" }),
    ).toBeInTheDocument();
    expect(screen.getByText("includes_all", { exact: true })).toBeInTheDocument();
    expect(change).not.toHaveBeenCalled();
  });
  it("uses a narrow Ant Drawer and returns focus after closing", async () => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue({ xs: true });
    renderControls(editorFixture());
    const trigger = screen.getByRole("button", { name: "Advanced filters" });
    trigger.focus();
    fireEvent.click(trigger);
    expect(
      await screen.findByRole("dialog", { name: "Advanced filters" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    await waitFor(() => expect(trigger).toHaveFocus());
  });
  it("offers scoped fields without another collection inside an Exists condition", async () => {
    const collection = fieldFixture(
      "embedded.equipment",
      "collection",
      "Embedded items",
    );
    const child = {
      ...fieldFixture("embedded.name", "string", "Embedded item name"),
      path: "name",
      scope: "embedded.equipment",
      choices: ["Jaws"],
    };
    const nested = {
      ...fieldFixture("nested", "collection", "Nested collection"),
      scope: "embedded.equipment",
    };
    renderControls(editorFixture([collection, child, nested]), {
      kind: "exists",
      clause_id: "embedded",
      scope_id: "scope",
      collection: "embedded.equipment",
      predicate: {
        kind: "compare",
        field: "name",
        clause_id: "name",
        op: "eq",
        value: "Jaws",
      },
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    const selectors = screen.getAllByRole("combobox", { name: "Filter field" });
    fireEvent.mouseDown(selectors[1]);
    await waitFor(() =>
      expect(
        screen.getByRole("option", { name: "Embedded item name" }),
      ).toBeInTheDocument(),
    );
    expect(
      screen.queryByRole("option", { name: "Nested collection" }),
    ).not.toBeInTheDocument();
  });
});

describe("per-value states and the complete query", () => {
  beforeEach(() => vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true }));
  afterEach(() => vi.restoreAllMocks());
  function choose(label: string, state: string) {
    const control = screen.getByRole("combobox", { name: label });
    fireEvent.mouseDown(control);
    const listbox = document.getElementById(control.getAttribute("aria-controls")!)!;
    fireEvent.click(within(listbox).getByRole("option", { name: state }));
  }
  it("includes and excludes individual Kind values without a field-wide mode", async () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature", "spell"],
    };
    const change = renderControls(editorFixture([kind]));
    expect(screen.queryByText("creature")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Edit Kind filter" }));
    fireEvent.click(
      screen.getByRole("button", {
        name: "Kind: Creature, neutral; activate to include",
      }),
    );
    const spell = screen.getByRole("button", {
      name: "Kind: Spell, neutral; activate to include",
    });
    spell.focus();
    fireEvent.click(spell);
    await waitFor(() =>
      expect(spell).toHaveAccessibleName("Kind: Spell, included; activate to exclude"),
    );
    expect(spell).toHaveFocus();
    fireEvent.click(spell);
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({
          kind: "all_of",
          children: [
            expect.objectContaining({ kind: "in", values: ["creature"] }),
            expect.objectContaining({
              kind: "not",
              predicate: expect.objectContaining({ values: ["spell"] }),
            }),
          ],
        }),
      }),
    );
    expect(screen.getByRole("button", { name: "Edit Kind filter" })).toHaveTextContent(
      "Kind",
    );
    expect(
      screen.queryByRole("combobox", { name: "Kind include or exclude" }),
    ).not.toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Kind: Creature, included; activate to exclude",
      }),
    );
    fireEvent.click(
      screen.getByRole("button", {
        name: "Kind: Creature, excluded; activate to clear",
      }),
    );
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({
          kind: "not",
          predicate: expect.objectContaining({ values: ["spell"] }),
        }),
      }),
    );
  });
  it("shows common clauses in Advanced and adds exactly one additional condition", async () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature", "spell"],
    };
    const hp = fieldFixture("defenses.hp.maximum", "number", "Maximum HP");
    const common: QueryPredicate = {
      kind: "in",
      clause_id: "kind",
      field: kind.path,
      values: ["creature"],
    };
    const change = renderControls(editorFixture([kind, hp]), common);
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    const advanced = screen.getByRole("region", { name: "Advanced filters" });
    expect(
      within(advanced).getByRole("combobox", { name: "Filter field" }),
    ).toBeInTheDocument();
    fireEvent.click(within(advanced).getByRole("button", { name: "Add condition" }));
    expect(change).not.toHaveBeenCalled();
    expect(screen.queryByText("Complete this filter")).not.toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(
      within(advanced).getAllByRole("combobox", { name: "Filter field" }),
    ).toHaveLength(2);
    const field = within(advanced).getAllByRole("combobox", {
      name: "Filter field",
    })[1];
    fireEvent.mouseDown(field);
    fireEvent.click(screen.getAllByText("Maximum HP", { exact: true }).slice(-1)[0]);
    fireEvent.change(screen.getByRole("textbox", { name: "Maximum HP" }), {
      target: { value: "12" },
    });
    await waitFor(() => expect(change).toHaveBeenCalled());
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({
          kind: "all_of",
          children: [
            common,
            expect.objectContaining({
              field: hp.path,
              value: 12,
            }),
          ],
        }),
      }),
    );
  });
  it("shows and edits No records as a structural condition", () => {
    const change = renderControls(editorFixture(), {
      kind: "boolean_constant",
      clause_id: "constant",
      value: false,
    });
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    choose("Structural condition", "All records");
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: { kind: "boolean_constant", clause_id: "constant", value: true },
      }),
    );
  });
  it("shows the original Kind immediately and never introduces a hidden duplicate while adding Kind", async () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature", "spell"],
    };
    const original: QueryPredicate = {
      kind: "in",
      clause_id: "original-kind",
      field: kind.path,
      values: ["creature"],
    };
    const change = renderControls(editorFixture([kind]), original);
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    const advanced = screen.getByRole("region", { name: "Advanced filters" });
    expect(
      within(advanced).getAllByRole("combobox", { name: "Filter field" }),
    ).toHaveLength(1);
    fireEvent.click(within(advanced).getByRole("button", { name: "Add condition" }));
    expect(
      within(advanced).getAllByRole("combobox", { name: "Filter field" }),
    ).toHaveLength(2);
    const field = within(advanced).getAllByRole("combobox", {
      name: "Filter field",
    })[1];
    fireEvent.mouseDown(field);
    fireEvent.click(screen.getAllByText("Kind", { exact: true }).slice(-1)[0]);
    await waitFor(() =>
      expect(within(advanced).getAllByRole("combobox", { name: "Kind" })).toHaveLength(
        2,
      ),
    );
    const value = within(advanced).getAllByRole("combobox", { name: "Kind" })[1];
    fireEvent.mouseDown(value);
    fireEvent.click(screen.getAllByText("creature", { exact: true }).slice(-1)[0]);
    await waitFor(() => expect(change).toHaveBeenCalled());
    expect(
      within(advanced).getAllByRole("combobox", { name: "Filter field" }),
    ).toHaveLength(2);
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({
          kind: "all_of",
          children: [
            original,
            expect.objectContaining({ field: kind.path, value: "creature" }),
          ],
        }),
      }),
    );
    expect(screen.queryByText("Defined in advanced filters")).toBeNull();
    fireEvent.click(
      within(advanced).getAllByRole("button", { name: "Remove condition" })[1],
    );
    expect(
      within(advanced).getAllByRole("combobox", { name: "Filter field" }),
    ).toHaveLength(1);
    expect(
      screen.getByRole("button", { name: "Edit Kind filter" }),
    ).toBeInTheDocument();
  });
  it("updates the shared Kind row while preserving an unfinished advanced number", async () => {
    const kind = {
      ...fieldFixture("kind", "string", "Kind"),
      choices: ["creature", "spell"],
    };
    const hp = fieldFixture("defenses.hp.maximum", "number", "Maximum HP");
    const original: QueryPredicate = {
      kind: "all_of",
      children: [
        { kind: "in", clause_id: "kind", field: kind.path, values: ["creature"] },
        { kind: "compare", clause_id: "hp", field: hp.path, op: "gte", value: 80 },
      ],
    };
    const change = renderControls(editorFixture([kind, hp]), original);
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Maximum HP" }), {
      target: { value: "-" },
    });
    expect(await screen.findByText("Enter a finite numeric value")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    fireEvent.click(screen.getByRole("button", { name: "Edit Kind filter" }));
    fireEvent.click(
      screen.getByRole("button", {
        name: "Kind: Creature, included; activate to exclude",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
    expect(screen.getByRole("textbox", { name: "Maximum HP" })).toHaveValue("-");
    expect(
      screen.getByRole("button", { name: "Remove excluded Creature from Kind" }),
    ).toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Maximum HP" }), {
      target: { value: "90" },
    });
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({
          kind: "all_of",
          children: [
            expect.objectContaining({
              kind: "not",
              predicate: expect.objectContaining({ values: ["creature"] }),
            }),
            expect.objectContaining({ field: hp.path, value: 90 }),
          ],
        }),
      }),
    );
  });
});

describe("field applicability while editing Boolean groups", () => {
  it.each(["Match any", "Match none", "Not all match"])(
    "keeps fields available when %s can broaden the current result",
    (match) => {
      const editor = editorFixture([
        { ...fieldFixture("kind", "string", "Kind"), choices: ["creature"] },
        fieldFixture("hazard.hardness", "number", "Hardness"),
      ]);
      editor.groups[0].fields[1].applicable = false;
      renderControls(editor, {
        kind: "in",
        field: "kind",
        values: ["creature"],
      });
      fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
      fireEvent.click(screen.getByRole("button", { name: "Add condition" }));
      fireEvent.mouseDown(screen.getByRole("combobox", { name: "Match conditions" }));
      fireEvent.click(screen.getAllByText(match, { exact: true }).slice(-1)[0]);
      fireEvent.mouseDown(
        screen.getAllByRole("combobox", { name: "Filter field" }).slice(-1)[0],
      );
      fireEvent.click(screen.getAllByText("Hardness", { exact: true }).slice(-1)[0]);
      expect(screen.getByRole("textbox", { name: "Hardness" })).toBeEnabled();
    },
  );
});
