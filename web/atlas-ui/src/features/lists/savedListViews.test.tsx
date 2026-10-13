import {
  detailFixture,
  summaryFixture,
  editorFixture,
  fieldFixture,
  valuesFixture,
} from "../../test/fixtures";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { retireImmutableReferenceQueries } from "../../shared/queries/immutableReferenceQueries";
import { ConfigProvider } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  RecordDetailView,
  SavedListDetailView,
  SavedListIndexView,
  DiscoverFilterValuesRequest,
  FilterVocabularyScope,
} from "../../generated/atlas";
import { useFilterVocabulary } from "../../shared/filters/useFilterDiscovery";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { useSavedListFilterDiscovery } from "./useSavedListFilters";
import { AddToListButton } from "./AddToListButton";
import { ListDetailView } from "./ListDetailView";
import { ListEditView } from "./ListEditView";
import { ListIndexView } from "./ListIndexView";
import { savedListTagOptions } from "./listUtils";

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  createSavedList: vi.fn(),
  deleteSavedList: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
  discoverFilterValues: vi.fn(),
  filterSavedList: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedList: vi.fn(),
  getSavedLists: vi.fn(),
  removeSavedListItem: vi.fn(),
  updateSavedList: vi.fn(),
}));

vi.mock("../../api/atlasApi", () => ({
  addSavedListItem: apiMocks.addSavedListItem,
  createSavedList: apiMocks.createSavedList,
  deleteSavedList: apiMocks.deleteSavedList,
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterCounts: apiMocks.discoverFilterCounts,
  discoverFilterValues: apiMocks.discoverFilterValues,
  filterSavedList: apiMocks.filterSavedList,
  getRecordDetail: apiMocks.getRecordDetail,
  getSavedList: apiMocks.getSavedList,
  getSavedLists: apiMocks.getSavedLists,
  removeSavedListItem: apiMocks.removeSavedListItem,
  updateSavedList: apiMocks.updateSavedList,
}));

describe("list views", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    history.replaceState(null, "", "/lists");
    apiMocks.getSavedLists.mockResolvedValue(savedListIndexFixture());
    apiMocks.getSavedList.mockResolvedValue(savedListDetailFixture());
    apiMocks.filterSavedList.mockResolvedValue(savedListDetailFixture());
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditorFixture());
    apiMocks.discoverFilterCounts.mockResolvedValue({
      counts: { status: "unavailable", reason: "non_conjunctive_context" },
    });
    apiMocks.discoverFilterValues.mockImplementation((request: { field_id: string }) =>
      Promise.resolve(valuesFixture(request.field_id)),
    );
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.addSavedListItem.mockResolvedValue({
      list_key: "list_research",
      slug: "research",
      record_key: "actions:testAction1",
      outcome: "added",
    });
    apiMocks.removeSavedListItem.mockResolvedValue({
      list_key: "list_research",
      slug: "research",
      record_key: "actions:testAction1",
      outcome: "removed",
    });
    apiMocks.deleteSavedList.mockResolvedValue({
      list_key: "list_research",
      slug: "research",
      deleted: true,
    });
    apiMocks.updateSavedList.mockResolvedValue({
      list: {
        list_key: "list_research",
        slug: "renamed-research",
        name: "Renamed Research",
        description: "Updated prep",
        tags: ["arc-one"],
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-04T00:00:00Z",
      },
    });
    apiMocks.createSavedList.mockResolvedValue({
      list: {
        list_key: "list_boss_fight_prep",
        slug: "boss-fight-prep",
        name: "Boss Fight Prep",
        description: "Session prep",
        tags: ["boss"],
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
    });
  });

  it("renders saved-list index rows and routes into a list", async () => {
    render(<ListIndexView route={{ kind: "lists" }} />, {
      wrapper: queryClientWrapper(),
    });

    const listLink = (await screen.findByText("research")).closest("a");
    expect(listLink).not.toBeNull();
    expect(screen.getByText("2 saved lists")).toBeInTheDocument();

    fireEvent.click(listLink!);

    await waitFor(() => expect(window.location.pathname).toBe("/lists/research"));
  });

  it("routes from the saved-list index when clicking the row", async () => {
    render(<ListIndexView route={{ kind: "lists" }} />, {
      wrapper: queryClientWrapper(),
    });

    const row = (await screen.findByText("Campaign prep")).closest("tr");
    expect(row).not.toBeNull();

    fireEvent.click(row!);

    await waitFor(() => expect(window.location.pathname).toBe("/lists/research"));
  });

  it("routes from the saved-list index to edit a list", async () => {
    render(<ListIndexView route={{ kind: "lists" }} />, {
      wrapper: queryClientWrapper(),
    });

    const editLink = await screen.findByRole("link", { name: "Edit Research" });
    fireEvent.click(editLink);

    await waitFor(() => expect(window.location.pathname).toBe("/lists/research/edit"));
  });

  it("filters saved-list index rows by tag", async () => {
    render(<ListIndexView route={{ kind: "lists" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByText("Research")).toBeInTheDocument();
    expect(screen.getByText("Encounters")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("checkbox", { name: "arc-one" }));

    await waitFor(() =>
      expect(screen.queryByText("Encounters")).not.toBeInTheDocument(),
    );
    expect(screen.getByText("Research")).toBeInTheDocument();
    expect(screen.getByText("1 saved list")).toBeInTheDocument();
  });

  it("derives saved-list tag options from existing lists", () => {
    expect(savedListTagOptions(savedListIndexFixture().lists)).toEqual([
      "arc-one",
      "arc-two",
      "boss",
    ]);
  });

  it("renders list contents, loads selected detail, and removes items", async () => {
    render(
      <ListDetailView
        route={{
          kind: "list",
          slug: "research",
          selectedRecordKey: "actions:testAction1",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    expect(
      await screen.findByRole("heading", { name: "Test Action 1" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Test Action 1" })).toBeInTheDocument();
    expect(screen.getByText("List")).toBeInTheDocument();
    expect(screen.getByText("Items")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Advanced filters" }),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(apiMocks.filterSavedList).toHaveBeenCalledWith({
        list_ref: "research",
        filter: undefined,
      }),
    );
    expect(
      screen.queryByRole("columnheader", { name: "Status" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Active")).not.toBeInTheDocument();
    expect(screen.queryByText("research")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Edit" }));
    await waitFor(() => expect(window.location.pathname).toBe("/lists/research/edit"));

    fireEvent.click(screen.getByRole("button", { name: "Remove Test Action 1" }));

    await waitFor(() =>
      expect(apiMocks.removeSavedListItem).toHaveBeenCalledWith({
        list_ref: "research",
        record_ref: "actions:testAction1",
      }),
    );
  });

  it("refreshes mounted list discovery after removing its last item without refreshing other scopes", async () => {
    let removed = false;
    const aliases = ["research", "list_research"];
    apiMocks.discoverFilterEditor.mockResolvedValue(
      editorFixture([fieldFixture("traits", "set", "Traits")]),
    );
    apiMocks.filterSavedList.mockImplementation(async () => ({
      ...savedListDetailFixture(),
      items: removed ? [] : savedListDetailFixture().items,
    }));
    apiMocks.removeSavedListItem.mockImplementation(async () => {
      removed = true;
      return {
        list_key: "list_research",
        slug: "research",
        record_key: "actions:testAction1",
        outcome: "removed",
      };
    });
    apiMocks.discoverFilterValues.mockImplementation(
      async (request: DiscoverFilterValuesRequest) => {
        if (request.kind !== "vocabulary") throw new Error("Expected vocabulary");
        const affected =
          request.scope.kind === "saved_list" &&
          aliases.includes(request.scope.list_ref);
        const empty = affected && removed;
        return {
          kind: "vocabulary",
          scope: request.scope,
          values: {
            field: request.field_id,
            options: empty ? [] : [{ value: "fire" }],
            selected_values: request.selected_values,
            total_values: empty ? 0n : 1n,
            basis: "product_key_scope_before_edition_preference",
          },
        };
      },
    );
    apiMocks.discoverFilterCounts.mockImplementation(async () => ({
      counts: {
        status: "available",
        value: {
          field: "traits",
          states: [],
          exhaustive: true,
          count_basis: "distinct roots",
          minimum: null,
          maximum: null,
        },
      },
    }));
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    render(
      <QueryClientProvider client={client}>
        <ListDetailView
          route={{ kind: "list", slug: "research", selectedRecordKey: null }}
        />
        <ListDiscoveryProbe listRef="research" />
        <ListDiscoveryProbe listRef="list_research" />
        <ListDiscoveryProbe listRef="other-list" />
        <VocabularyProbe scope={{ kind: "artifact" }} label="artifact values" />
      </QueryClientProvider>,
    );
    for (const alias of [...aliases, "other-list"])
      await waitFor(() =>
        expect(screen.getByLabelText(`values ${alias}`)).toHaveTextContent(
          '"options":["fire"]',
        ),
      );
    await waitFor(() =>
      expect(screen.getByLabelText("artifact values")).toHaveTextContent("fire"),
    );
    await waitFor(() => expect(apiMocks.discoverFilterCounts).toHaveBeenCalledTimes(3));
    const otherRequests = apiMocks.discoverFilterValues.mock.calls.filter(
      ([request]) =>
        request.scope.kind === "artifact" || request.scope.list_ref === "other-list",
    ).length;

    fireEvent.click(
      await screen.findByRole("button", { name: "Remove Test Action 1" }),
    );

    for (const alias of aliases) {
      await waitFor(() =>
        expect(screen.getByLabelText(`values ${alias}`)).toHaveTextContent(
          '"options":[],"selected":["retained"],"total":"0"',
        ),
      );
      await waitFor(() =>
        expect(
          apiMocks.discoverFilterCounts.mock.calls.filter(
            ([request]) => request.context.list_ref === alias,
          ),
        ).toHaveLength(2),
      );
    }
    expect(screen.getByLabelText("values other-list")).toHaveTextContent("fire");
    expect(screen.getByLabelText("artifact values")).toHaveTextContent("fire");
    expect(
      apiMocks.discoverFilterValues.mock.calls.filter(
        ([request]) =>
          request.scope.kind === "artifact" || request.scope.list_ref === "other-list",
      ),
    ).toHaveLength(otherRequests);
    expect(apiMocks.discoverFilterCounts).toHaveBeenCalledTimes(5);
  });

  it("opens references from the selected saved-list detail in a popover", async () => {
    history.replaceState(null, "", "/lists/research/actions%3AtestAction1");
    render(
      <ListDetailView
        route={{
          kind: "list",
          slug: "research",
          selectedRecordKey: "actions:testAction1",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    fireEvent.click(await screen.findByRole("button", { name: "Nested Rule" }));

    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "rules:nested",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();
    expect(window.location.pathname).toBe("/lists/research/actions%3AtestAction1");
  });

  it("searches within a saved list through the filter route", async () => {
    render(
      <ListDetailView
        route={{
          kind: "list",
          slug: "research",
          selectedRecordKey: null,
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    await waitFor(() =>
      expect(apiMocks.filterSavedList).toHaveBeenCalledWith({
        list_ref: "research",
        filter: undefined,
      }),
    );
    const search = await screen.findByRole("searchbox", { name: "Search records" });
    fireEvent.change(search, {
      target: { value: "Test Action 2" },
    });
    fireEvent.keyDown(search, { key: "Enter", keyCode: 13 });

    await waitFor(() =>
      expect(apiMocks.filterSavedList).toHaveBeenCalledWith({
        list_ref: "research",
        query: "Test Action 2",
        filter: undefined,
      }),
    );
  });

  it("edits list metadata and navigates to the updated slug", async () => {
    render(
      <ListEditView
        route={{
          kind: "listEdit",
          slug: "research",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    expect(await screen.findByDisplayValue("Research")).toBeInTheDocument();
    expect(screen.queryByLabelText("Slug")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Renamed Research!" },
    });
    fireEvent.change(screen.getByLabelText("Description"), {
      target: { value: "Updated prep" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(apiMocks.updateSavedList).toHaveBeenCalledWith({
        list_key: "list_research",
        slug: "renamed-research",
        name: "Renamed Research!",
        description: "Updated prep",
        tags: ["arc-one", "boss"],
      }),
    );
    await waitFor(() =>
      expect(window.location.pathname).toBe("/lists/renamed-research"),
    );
  });

  it("deletes lists from the edit page and returns to the list index", async () => {
    render(
      <ListEditView
        route={{
          kind: "listEdit",
          slug: "research",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    await screen.findByDisplayValue("Research");
    fireEvent.click(screen.getByRole("button", { name: "Delete List" }));

    await waitFor(() =>
      expect(apiMocks.deleteSavedList).toHaveBeenCalledWith("list_research"),
    );
    await waitFor(() => expect(window.location.pathname).toBe("/lists"));
  });

  it("switches between saved lists from the list pane picker", async () => {
    render(
      <ListDetailView
        route={{
          kind: "list",
          slug: "research",
          selectedRecordKey: null,
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    const selector = await screen.findByRole("combobox", { name: "Selected list" });
    fireEvent.mouseDown(selector);
    fireEvent.click(await screen.findByText("Encounters"));

    await waitFor(() => expect(window.location.pathname).toBe("/lists/encounters"));
  });

  it("explains the saved-list action on keyboard focus without loading lists", async () => {
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <AddToListButton recordKey="actions:testAction1" />
      </ConfigProvider>,
      { wrapper: queryClientWrapper() },
    );
    const action = screen.getByRole("button", { name: "Add to saved list" });
    act(() => action.focus());
    expect(action).toHaveFocus();
    expect(await screen.findByRole("tooltip")).toHaveTextContent("Add to saved list");
    expect(apiMocks.getSavedLists).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    act(() => action.blur());
    await waitFor(() => expect(screen.queryByRole("tooltip")).not.toBeInTheDocument());
  });

  it("adds the current record to a selected list", async () => {
    render(<AddToListButton recordKey="actions:testAction1" />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(screen.getByRole("button", { name: "Add to saved list" }));

    const selector = await screen.findByRole("combobox");
    fireEvent.mouseDown(selector);
    fireEvent.click(await screen.findByText("Research"));
    fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() =>
      expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
        list_ref: "research",
        record_ref: "actions:testAction1",
      }),
    );
  });

  it("creates lists from a name without asking for a slug", async () => {
    render(<ListIndexView route={{ kind: "lists" }} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(screen.getByRole("button", { name: "New List" }));
    expect(screen.queryByLabelText("Slug")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Boss Fight Prep!" },
    });
    expect(screen.queryByLabelText("Description")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() =>
      expect(apiMocks.createSavedList).toHaveBeenCalledWith({
        slug: "boss-fight-prep",
        name: "Boss Fight Prep!",
        tags: [],
      }),
    );
    await waitFor(() =>
      expect(window.location.pathname).toBe("/lists/boss-fight-prep"),
    );
  });
  it("preserves touched list fields during same-identity refresh and resets entity switches", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
      </QueryClientProvider>
    );
    const { rerender } = render(
      <ListEditView route={{ kind: "listEdit", slug: "research" }} />,
      { wrapper },
    );
    const name = await screen.findByLabelText("Name");
    fireEvent.change(name, { target: { value: "My list draft" } });
    client.setQueryData(["record-detail", "old"], {
      artifact_generation: "generation-a",
    });
    await act(async () => retireImmutableReferenceQueries(client));
    expect(client.getQueryData(["record-detail", "old"])).toBeUndefined();
    expect(client.getQueryData(["saved-list", "research"])).toBeDefined();
    expect(name).toHaveValue("My list draft");
    act(() =>
      client.setQueryData(["saved-list", "research"], {
        ...savedListDetailFixture(),
        list: {
          ...savedListDetailFixture().list,
          name: "Server name",
          description: "Server description",
        },
      }),
    );
    expect(name).toHaveValue("My list draft");
    await waitFor(() =>
      expect(screen.getByLabelText("Description")).toHaveValue("Server description"),
    );
    apiMocks.getSavedList.mockResolvedValue({
      ...savedListDetailFixture(),
      list: {
        ...savedListDetailFixture().list,
        list_key: "other-list",
        slug: "other",
        name: "Other list",
      },
    });
    rerender(<ListEditView route={{ kind: "listEdit", slug: "other" }} />);
    expect(await screen.findByLabelText("Name")).toHaveValue("Other list");
  });
  it("retries an initial list failure without discarding later edits", async () => {
    apiMocks.getSavedList.mockRejectedValueOnce(new Error("List unavailable"));
    render(<ListEditView route={{ kind: "listEdit", slug: "research" }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("button", { name: "Retry list" }));
    expect(await screen.findByLabelText("Name")).toHaveValue("Research");
  });
});

function VocabularyProbe({
  scope,
  label,
}: {
  scope: FilterVocabularyScope;
  label: string;
}) {
  const query = useFilterVocabulary(scope, "traits", "", 0, ["retained"]);
  const values = query.data?.values;
  return (
    <output aria-label={label}>
      {JSON.stringify({
        options: values?.options.map((option) => option.value),
        selected: values?.selected_values,
        total: values?.total_values.toString(),
      })}
    </output>
  );
}

function ListDiscoveryProbe({ listRef }: { listRef: string }) {
  const discovery = useSavedListFilterDiscovery(listRef, {
    ...DEFAULT_SEARCH_STATE,
    filter: {
      kind: "set_match",
      clause_id: "trait",
      field: "traits",
      op: "includes_any",
      values: ["retained"],
    },
  });
  return (
    <>
      <VocabularyProbe
        scope={{ kind: "saved_list", list_ref: listRef }}
        label={`values ${listRef}`}
      />
      <output aria-label={`counts ${listRef}`}>
        {discovery.filterCountsByField["traits:trait"]?.counts.status}
      </output>
    </>
  );
}

function queryClientWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

function savedListIndexFixture(): SavedListIndexView {
  return {
    lists: [
      {
        list_key: "list_research",
        slug: "research",
        name: "Research",
        description: "Campaign prep",
        tags: ["arc-one", "boss"],
        item_count: 1,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
      {
        list_key: "list_encounters",
        slug: "encounters",
        name: "Encounters",
        description: "Fight prep",
        tags: ["arc-two"],
        item_count: 0,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-03T00:00:00Z",
      },
    ],
  };
}

function savedListDetailFixture(): SavedListDetailView {
  return {
    list: savedListIndexFixture().lists[0]!,
    items: [
      {
        record_key: "actions:testAction1",
        position: 1,
        status: "active",
        snapshot: {
          title: "Test Action 1",
          kind: "rule",
        },
        record: summaryFixture("actions:testAction1", "Test Action 1"),
      },
    ],
  };
}

function filterEditorFixture() {
  return editorFixture([
    fieldFixture("actor.level", "number", "Level"),
    fieldFixture("common.rarity", "string", "Rarity"),
    fieldFixture("record.kind", "string", "Kind"),
    fieldFixture("common.traits", "set", "Traits"),
    fieldFixture("source.pack", "string", "Pack"),
  ]);
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  return detailFixture(
    recordKey,
    recordKey === "rules:nested" ? "Nested Rule" : "Test Action 1",
    recordKey === "rules:nested" ? undefined : "rules:nested",
  );
}
