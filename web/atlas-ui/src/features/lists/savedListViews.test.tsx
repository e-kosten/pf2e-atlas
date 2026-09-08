import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  RecordDetailView,
  SavedListDetailView,
  SavedListIndexView,
} from "../../generated/atlas";
import {
  recordDetailFixture as typedRecordDetailFixture,
  recordSummaryFixture,
} from "../../test/recordFixtures";
import { testDeadline } from "../../test/testDeadline";
import { AddToListButton } from "./AddToListButton";
import { ListDetailView } from "./ListDetailView";
import { ListEditView } from "./ListEditView";
import { ListIndexView } from "./ListIndexView";
import { savedListTagOptions } from "./listUtils";

const tenSecondTestDeadline = testDeadline(10_000);

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  createSavedList: vi.fn(),
  deleteSavedList: vi.fn(),
  discoverFilterEditor: vi.fn(),
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
    apiMocks.discoverFilterValues.mockImplementation((request: { field_id: string }) =>
      Promise.resolve({
        field_id: request.field_id,
        matching_record_count: 1n,
        options: [],
      }),
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
    expect(screen.getByText("Standard filters")).toBeInTheDocument();
    await waitFor(() =>
      expect(apiMocks.filterSavedList).toHaveBeenCalledWith({
        list_ref: "research",
        filter: { clauses: [] },
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

  it(
    "opens references from the selected saved-list detail in a popover",
    async () => {
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

      fireEvent.click(await screen.findByRole("link", { name: "Nested Rule" }));

      await waitFor(() =>
        expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
          "rules:nested",
          undefined,
          expect.any(AbortSignal),
        ),
      );
      expect(await screen.findByLabelText("Reference preview")).toBeInTheDocument();
      expect(window.location.pathname).toBe("/lists/research/actions%3AtestAction1");
    },
    tenSecondTestDeadline,
  );

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
        filter: { clauses: [] },
      }),
    );
    fireEvent.change(await screen.findByPlaceholderText("Search records"), {
      target: { value: "Test Action 2" },
    });

    await waitFor(() =>
      expect(apiMocks.filterSavedList).toHaveBeenCalledWith({
        list_ref: "research",
        query: "Test Action 2",
        filter: { clauses: [] },
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
});

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
        item_count: 1n,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
      {
        list_key: "list_encounters",
        slug: "encounters",
        name: "Encounters",
        description: "Fight prep",
        tags: ["arc-two"],
        item_count: 0n,
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
        position: 1n,
        status: "active",
        snapshot: {
          title: "Test Action 1",
          kind: "rule",
        },
        record: recordSummaryFixture("actions:testAction1", "Test Action 1"),
      },
    ],
  };
}

function filterEditorFixture() {
  return {
    matching_record_count: 1n,
    groups: [
      {
        id: "standard",
        label: "Standard filters",
        fields: [
          filterField("level", "Level", "range"),
          filterField("rarity", "Rarity", "option"),
          filterField("kind", "Kind", "option"),
          filterField("traits", "Traits", "option"),
          filterField("pack", "Pack", "option"),
        ],
      },
    ],
  };
}

function filterField(id: string, label: string, controlKind: "option" | "range") {
  return {
    id,
    label,
    placement: "always_visible",
    applicability: "applicable",
    supports_counts: true,
    allowed_operators: ["include_any", "include_all", "exclude_any"],
    default_operator: "include_any",
    control:
      controlKind === "range"
        ? {
            kind: "range",
            min: 0,
            max: 25,
            step: 1,
            min_label: "Min",
            max_label: "Max",
          }
        : { kind: "option" },
  };
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  const detail = typedRecordDetailFixture({
    recordKey,
    title: recordKey === "rules:nested" ? "Nested Rule" : "Test Action 1",
  });
  if (
    recordKey !== "rules:nested" &&
    detail.surface.presentation.presentation_type === "creature"
  ) {
    detail.surface.presentation.body.content = [
      {
        content_key: "nested-reference",
        role: "primary_description",
        authored_order: 0,
        blocks: [
          {
            block_type: "paragraph",
            spans: [
              { span_type: "text", text: "See " },
              {
                span_type: "reference",
                label: "Nested Rule",
                record_key: "rules:nested",
                embedded: false,
              },
            ],
          },
        ],
        content_hash: "nested-reference-hash",
        visibility: "public",
        provenance: {
          source_record_key: recordKey,
          relative_source_path: "fixture.json",
          field_family: "fixture.reference",
        },
      },
    ];
  }
  return detail;
}
