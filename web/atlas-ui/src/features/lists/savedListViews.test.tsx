import { recordDetailQueryOptions } from "../../shared/records/recordDetailQuery";
import { RouterTestContext } from "../../test/router";
import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import type { ResultSort } from "../../generated/atlas";
import {
  detailFixture,
  summaryFixture,
  editorFixture,
  fieldFixture,
  valuesFixture,
} from "../../test/fixtures";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { retireReferenceQueries } from "../../shared/queries/referenceQueries";
import { App, ConfigProvider, Grid } from "antd";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { Profiler, type ReactNode } from "react";
import type {
  RecordDetailView,
  RecordDetailRequest,
  RecordNavigationView,
  SavedListDetailView,
  SavedListIndexView,
  DiscoverFilterValuesRequest,
  FilterVocabularyScope,
} from "../../generated/atlas";
import { useFilterVocabulary } from "../../shared/filters/useFilterDiscovery";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { useSavedListFilterDiscovery } from "./useSavedListFilters";
import { AddToRecordButton } from "../record-actions/AddToRecordButton";
import { ListDetailView } from "./ListDetailView";
import { ListEditView } from "./ListEditView";
import { ListIndexView } from "./ListIndexView";
import { savedListTagOptions } from "./listUtils";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";

const apiMocks = vi.hoisted(() => ({
  searchPage: vi.fn(),
  addSavedListItem: vi.fn(),
  createSavedList: vi.fn(),
  deleteSavedList: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
  discoverFilterValues: vi.fn(),
  filterSavedList: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedList: vi.fn(),
  getSavedListItemNote: vi.fn(),
  updateSavedListItemNote: vi.fn(),
  getSavedLists: vi.fn(),
  removeSavedListItem: vi.fn(),
  reorderSavedListItem: vi.fn(),
  updateSavedList: vi.fn(),
}));

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  searchPage: apiMocks.searchPage,
  addSavedListItem: apiMocks.addSavedListItem,
  createSavedList: apiMocks.createSavedList,
  deleteSavedList: apiMocks.deleteSavedList,
  discoverFilterEditor: apiMocks.discoverFilterEditor,
  discoverFilterCounts: apiMocks.discoverFilterCounts,
  discoverFilterValues: apiMocks.discoverFilterValues,
  filterSavedList: apiMocks.filterSavedList,
  getRecordDetail: apiMocks.getRecordDetail,
  getSavedList: apiMocks.getSavedList,
  getSavedListItemNote: apiMocks.getSavedListItemNote,
  updateSavedListItemNote: apiMocks.updateSavedListItemNote,
  getSavedLists: apiMocks.getSavedLists,
  removeSavedListItem: apiMocks.removeSavedListItem,
  reorderSavedListItem: apiMocks.reorderSavedListItem,
  updateSavedList: apiMocks.updateSavedList,
}));

describe("list views", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true, xl: true });
    history.replaceState(null, "", "/lists");
    apiMocks.getSavedLists.mockResolvedValue(savedListIndexFixture());
    apiMocks.getSavedList.mockResolvedValue(savedListDetailFixture());
    apiMocks.getSavedListItemNote.mockImplementation((_list, key) =>
      Promise.resolve({
        list_key: "list_research",
        slug: "research",
        record_key: key,
        title: "Test Action",
        note: null,
      }),
    );
    apiMocks.filterSavedList.mockResolvedValue(savedListDetailFixture());
    apiMocks.discoverFilterEditor.mockResolvedValue(filterEditorFixture());
    apiMocks.discoverFilterCounts.mockResolvedValue({
      counts: { status: "unavailable", reason: "non_conjunctive_context" },
    });
    apiMocks.discoverFilterValues.mockImplementation((request: { field_id: string }) =>
      Promise.resolve(valuesFixture(request.field_id)),
    );
    apiMocks.searchPage.mockResolvedValue({
      sort: {
        kind: "attributes",
        rules: [{ field: "name", direction: "asc" }],
      } satisfies ResultSort,
      search_profile: "bge-base",
      artifact_generation: "generation-a",
      mode: { kind: "list_records" },
      coverage: null,
      page: {
        number: 1,
        size: 25,
        count: 2,
        total: 2n,
        has_more: false,
        next_page: null,
      },
      rows: [
        { record: summaryFixture("actions:testAction1", "Test Action 1"), matches: [] },
        { record: summaryFixture("rules:nested", "Nested Rule"), matches: [] },
      ],
    });
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
        slug: "research",
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
    render(<ListIndexView route={{}} />, {
      wrapper: queryClientWrapper(),
    });

    const listLink = (await screen.findByText("research")).closest("a");
    expect(listLink).not.toBeNull();
    expect(screen.getByText("2 saved lists")).toBeInTheDocument();
    const controls = screen
      .getByRole("textbox", { name: "Search saved lists" })
      .closest<HTMLElement>(".primary-toolbar")!;
    expect(controls.parentElement).toHaveClass("entity-index-page__controls");
    expect(controls).toContainElement(
      screen.getByRole("button", { name: "Edit Tags filter" }),
    );
    expect(controls).toContainElement(screen.getByRole("button", { name: "New list" }));
    expect(
      screen.getByRole("heading", { name: "Saved Lists" }).closest("section"),
    ).not.toContainElement(controls);

    fireEvent.click(listLink!);

    await waitFor(() => expect(window.location.pathname).toBe("/lists/research"));
  });

  it("routes from the saved-list index when clicking the row", async () => {
    render(<ListIndexView route={{}} />, {
      wrapper: queryClientWrapper(),
    });

    const row = (await screen.findByText("Campaign prep")).closest("tr");
    expect(row).not.toBeNull();

    fireEvent.click(row!);

    await waitFor(() => expect(window.location.pathname).toBe("/lists/research"));
  });

  it("routes from the saved-list index to edit a list", async () => {
    render(<ListIndexView route={{}} />, {
      wrapper: queryClientWrapper(),
    });

    const editLink = await screen.findByRole("link", { name: "Edit Research" });
    fireEvent.click(editLink);

    await waitFor(() => expect(window.location.pathname).toBe("/lists/research/edit"));
  });

  it.each(["arc-one", "constructor"])(
    "filters saved-list index rows by the literal tag %s",
    async (tag) => {
      const fixture = savedListIndexFixture();
      fixture.lists[0].tags = [tag, "boss"];
      apiMocks.getSavedLists.mockResolvedValue(fixture);
      render(<ListIndexView route={{}} />, {
        wrapper: queryClientWrapper(),
      });

      expect(await screen.findByText("Research")).toBeInTheDocument();
      expect(screen.getByText("Encounters")).toBeInTheDocument();

      fireEvent.click(screen.getByRole("button", { name: "Edit Tags filter" }));
      fireEvent.click(
        await screen.findByRole("button", {
          name: `Tags: ${tag}, neutral; activate to include`,
        }),
      );

      await waitFor(() =>
        expect(screen.queryByText("Encounters")).not.toBeInTheDocument(),
      );
      expect(screen.getByText("Research")).toBeInTheDocument();
      expect(screen.getByText("1 saved list")).toBeInTheDocument();
    },
  );

  it("combines any included tag with no excluded tags, retaining searchable stable rows", async () => {
    render(<ListIndexView route={{}} />, {
      wrapper: queryClientWrapper(),
    });
    await screen.findByText("Research");
    fireEvent.click(screen.getByRole("button", { name: "Edit Tags filter" }));
    const one = await screen.findByRole("button", {
      name: "Tags: arc-one, neutral; activate to include",
    });
    fireEvent.click(one);
    fireEvent.click(
      screen.getByRole("button", {
        name: "Tags: arc-two, neutral; activate to include",
      }),
    );
    expect(screen.getByText("2 saved lists")).toBeVisible();
    fireEvent.click(one);
    expect(screen.getByText("1 saved list")).toBeVisible();
    expect(screen.queryByText("Campaign prep")).toBeNull();
    expect(screen.getByText("Fight prep")).toBeVisible();
    fireEvent.change(screen.getByRole("textbox", { name: "Search tags choices" }), {
      target: { value: "boss" },
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", {
          name: "Tags: boss, neutral; activate to include",
        }),
      ).toBeVisible(),
    );
    expect(screen.queryByRole("button", { name: /Tags: arc-two/ })).toBeNull();
    fireEvent.keyDown(screen.getByRole("textbox", { name: "Search tags choices" }), {
      key: "Escape",
    });
    expect(screen.getByRole("button", { name: "Edit Tags filter" })).toHaveFocus();
    fireEvent.click(
      screen.getByRole("button", { name: "Remove excluded arc-one from Tags" }),
    );
    expect(screen.getByText("Fight prep")).toBeVisible();
  });

  it("restores list-name, tag and actual index scroll state after unmount and resets scroll on a filter change", async () => {
    const wrapper = queryClientWrapper();
    const view = render(<ListIndexView route={{}} />, { wrapper });
    await screen.findByText("Research");
    fireEvent.change(screen.getByRole("textbox", { name: "Search saved lists" }), {
      target: { value: "Research" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Edit Tags filter" }));
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Tags: boss, neutral; activate to include",
      }),
    );
    const table = view.container.querySelector<HTMLElement>(
      ".entity-index-page__table",
    )!;
    table.scrollTop = 345;
    fireEvent.scroll(table);
    view.unmount();
    const returned = render(<ListIndexView route={{}} />, { wrapper });
    expect(
      await screen.findByRole("textbox", { name: "Search saved lists" }),
    ).toHaveValue("Research");
    expect(
      screen.getByRole("button", { name: "Remove included boss from Tags" }),
    ).toBeVisible();
    const restored = returned.container.querySelector<HTMLElement>(
      ".entity-index-page__table",
    )!;
    expect(restored.scrollTop).toBe(345);
    fireEvent.change(screen.getByRole("textbox", { name: "Search saved lists" }), {
      target: { value: "" },
    });
    expect(restored.scrollTop).toBe(0);
    expect(screen.getByText("Campaign prep")).toBeVisible();
  });

  it("snapshots responsive document scroll before route rendering clamps it, then restores it on return", async () => {
    const scrollTo = vi.spyOn(window, "scrollTo").mockImplementation(() => {});
    const descriptor = Object.getOwnPropertyDescriptor(window, "scrollY")!;
    Object.defineProperty(window, "scrollY", {
      configurable: true,
      writable: true,
      value: 613,
    });
    try {
      const wrapper = queryClientWrapper();
      const view = render(<ListIndexView route={{}} />, { wrapper });
      const link = await screen.findByText("research");
      // Initial restore belongs to the fresh index; emulate subsequent user scrolling.
      Object.defineProperty(window, "scrollY", {
        configurable: true,
        writable: true,
        value: 613,
      });
      fireEvent.click(link.closest("a")!);
      Object.defineProperty(window, "scrollY", {
        configurable: true,
        writable: true,
        value: 0,
      });
      view.unmount();
      render(<ListIndexView route={{}} />, { wrapper });
      await screen.findByText("Campaign prep");
      expect(scrollTo).toHaveBeenLastCalledWith(0, 613);
    } finally {
      scrollTo.mockRestore();
      Object.defineProperty(window, "scrollY", descriptor);
    }
  });

  it("distinguishes initial list failure from empty data and retries", async () => {
    apiMocks.getSavedLists.mockRejectedValueOnce(new Error("Index unavailable"));
    render(<ListIndexView route={{}} />, {
      wrapper: queryClientWrapper(),
    });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Saved lists could not load",
    );
    expect(
      screen.queryByText("No saved lists. Create a list to collect records."),
    ).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Retry lists" }));
    expect(await screen.findByText("Campaign prep")).toBeVisible();
  });

  it("retains loaded rows on index refresh failure and distinguishes filtered emptiness", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={client}>
        <App>
          <ListIndexView route={{}} />
        </App>
      </QueryClientProvider>,
    );
    await screen.findByText("Campaign prep");
    fireEvent.click(screen.getByRole("button", { name: "Edit Tags filter" }));
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Tags: arc-one, neutral; activate to include",
      }),
    );
    apiMocks.getSavedLists.mockRejectedValueOnce(new Error("Refresh unavailable"));
    await act(async () => {
      await client.invalidateQueries({ queryKey: ["saved-lists"] });
    });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Saved lists could not refresh",
    );
    expect(screen.getByText("Campaign prep")).toBeVisible();
    apiMocks.getSavedLists.mockResolvedValue({
      lists: [savedListIndexFixture().lists[1]],
    });
    fireEvent.click(screen.getByRole("button", { name: "Retry lists" }));
    expect(await screen.findByText("No lists match these filters")).toBeVisible();
    expect(
      screen.queryByText("No saved lists. Create a list to collect records."),
    ).toBeNull();
  });

  it("cancels whole-list deletion, retains metadata through a pending rejection and retries once", async () => {
    let fail!: (error: Error) => void;
    apiMocks.deleteSavedList.mockReturnValueOnce(
      new Promise<void>((_ok, reject) => {
        fail = reject;
      }),
    );
    render(<ListEditView route={{ slug: "research" }} />, {
      wrapper: queryClientWrapper(),
    });
    const name = await screen.findByRole("textbox", { name: "Name" });
    fireEvent.change(name, { target: { value: "Unsaved metadata" } });
    fireEvent.click(screen.getByRole("button", { name: "Delete List" }));
    fireEvent.click(await screen.findByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(apiMocks.deleteSavedList).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Delete List" }));
    fireEvent.click(await screen.findByRole("button", { name: "Delete list" }));
    await waitFor(() => expect(apiMocks.deleteSavedList).toHaveBeenCalledOnce());
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
    expect(name).toHaveValue("Unsaved metadata");
    await act(async () => fail(new Error("Delete rejected")));
    expect(
      await screen.findAllByText("Delete list could not be confirmed"),
    ).not.toHaveLength(0);
    expect(name).toHaveValue("Unsaved metadata");
    fireEvent.click(screen.getByRole("button", { name: "Delete list" }));
    await waitFor(() => expect(apiMocks.deleteSavedList).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(window.location.pathname).toBe("/lists"));
  });

  it("keeps a readable primary index column when long titles and twenty tags compete for table width", async () => {
    const fixture = savedListIndexFixture();
    fixture.lists[0].name =
      "Preparation for the planar expedition and the next several sessions";
    fixture.lists[0].tags = Array.from(
      { length: 20 },
      (_, index) => `Preparation tag ${index}`,
    );
    apiMocks.getSavedLists.mockResolvedValue(fixture);
    render(<ListIndexView route={{}} />, {
      wrapper: queryClientWrapper(),
    });
    const title = await screen.findByRole("link", {
      name: /^Preparation for the planar expedition/,
    });
    const table = title.closest("table")!;
    expect(table).toHaveStyle({ tableLayout: "auto" });
    expect(table.querySelector("colgroup col")).toHaveStyle({ minWidth: "240px" });
    expect(title).toHaveAttribute("href", "/lists/research");
    for (const cell of title.closest("tr")!.querySelectorAll("td"))
      expect(cell).toHaveStyle({ verticalAlign: "top" });
    expect(
      title.closest("tr")!.querySelector(".list-index-view__description"),
    ).toHaveAttribute("title", "Campaign prep");
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
          slug: "research",
          selectedRecordKey: "actions:testAction1",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    expect(
      await screen.findByRole("heading", { name: "Test Action 1" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Test Action 1/ })).toHaveAttribute(
      "href",
      "/lists/research/records/actions:testAction1",
    );
    expect(screen.queryByRole("radio")).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Saved records" })).toBeInTheDocument();
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
    fireEvent.click(screen.getByRole("link", { name: "Edit list" }));
    await waitFor(() => expect(window.location.pathname).toBe("/lists/research/edit"));

    fireEvent.click(screen.getByRole("button", { name: "Remove Test Action 1" }));
    expect(apiMocks.removeSavedListItem).not.toHaveBeenCalled();
    fireEvent.click(await screen.findByRole("button", { name: "Remove" }));

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
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false, staleTime: Infinity } },
    });
    render(
      <QueryClientProvider client={client}>
        <App>
          <ListDetailView route={{ slug: "research", selectedRecordKey: null }} />
          <ListDiscoveryProbe listRef="research" />
          <ListDiscoveryProbe listRef="list_research" />
          <ListDiscoveryProbe listRef="other-list" />
          <VocabularyProbe scope={{ kind: "artifact" }} label="artifact values" />
        </App>
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
    expect(apiMocks.discoverFilterCounts).not.toHaveBeenCalled();
    const otherRequests = apiMocks.discoverFilterValues.mock.calls.filter(
      ([request]) =>
        request.scope.kind === "artifact" || request.scope.list_ref === "other-list",
    ).length;

    fireEvent.click(
      await screen.findByRole("button", { name: "Remove Test Action 1" }),
    );
    fireEvent.click(await screen.findByRole("button", { name: "Remove" }));

    for (const alias of aliases) {
      await waitFor(() =>
        expect(screen.getByLabelText(`values ${alias}`)).toHaveTextContent(
          '"options":[],"selected":["retained"],"total":"0"',
        ),
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
    expect(apiMocks.discoverFilterCounts).not.toHaveBeenCalled();
  });

  it("restores an exact list address and rank changes for the same selected member", async () => {
    const key = "actions:testAction1";
    const selection: RecordNavigationView = {
      record_key: key,
      embedded_path: [{ collection: "/items", identity: { Stable: "ability-one" } }],
      field: "/system/description/value",
      passage: { kind: "identity" },
      source_fingerprint: null,
      spell_rank: 0,
    };
    apiMocks.getRecordDetail.mockImplementation(
      async (recordKey: string, request?: RecordDetailRequest) => ({
        ...recordDetailFixture(recordKey),
        selected: request
          ? {
              record_key: recordKey,
              embedded_path: request.embedded_path,
              field: request.fields[0] ?? null,
              passage: request.passage,
              source_fingerprint: request.source_fingerprint,
              spell_rank: request.spell_rank,
            }
          : recordDetailFixture(recordKey).selected,
      }),
    );
    const route = { slug: "research", selectedRecordKey: key, selection };
    const view = render(<ListDetailView route={route} />, {
      wrapper: queryClientWrapper(),
    });
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        key,
        {
          record_key: key,
          embedded_path: selection.embedded_path,
          fields: [selection.field],
          passage: selection.passage,
          source_fingerprint: null,
          spell_rank: 0,
        },
        expect.any(AbortSignal),
      ),
    );
    await waitFor(() =>
      expect(screen.getByRole("link", { name: "Open full page" })).toHaveAttribute(
        "href",
        expect.stringContaining("rank=0"),
      ),
    );
    view.rerender(
      <ListDetailView
        route={{ ...route, selection: { ...selection, spell_rank: 8 } }}
      />,
    );
    await waitFor(() =>
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        key,
        expect.objectContaining({
          embedded_path: selection.embedded_path,
          spell_rank: 8,
        }),
        expect.any(AbortSignal),
      ),
    );
    await waitFor(() =>
      expect(screen.getByRole("link", { name: "Open full page" })).toHaveAttribute(
        "href",
        expect.stringContaining("rank=8"),
      ),
    );
    expect(apiMocks.removeSavedListItem).not.toHaveBeenCalled();
  });

  it("restores landing-only changes for the same list member in native full-page links", async () => {
    const key = "actions:testAction1";
    const first: RecordNavigationView = {
      record_key: key,
      embedded_path: [{ collection: "/items", identity: { Stable: "ability-one" } }],
      field: null,
      passage: null,
      source_fingerprint: null,
    };
    const second: RecordNavigationView = {
      ...first,
      embedded_path: [{ collection: "/items", identity: { Stable: "ability-two" } }],
    };
    const route = { slug: "research", selectedRecordKey: key, landing: first };
    const view = render(<ListDetailView route={route} />, {
      wrapper: queryClientWrapper(),
    });
    function currentLanding() {
      const link = screen.getByRole("link", { name: "Open full page" });
      const url = new URL(link.getAttribute("href")!, location.origin);
      return selectedNavigation(parseSearchParams(url.search), key, "focus");
    }
    await waitFor(() => expect(currentLanding()).toEqual(first));
    view.rerender(<ListDetailView route={{ ...route, landing: second }} />);
    await waitFor(() => expect(currentLanding()).toEqual(second));
    expect(screen.getByRole("link", { name: /Test Action 1/ })).toHaveAttribute(
      "aria-current",
      "true",
    );
    fireEvent.click(screen.getByRole("button", { name: "Return to selection" }));
    await waitFor(() =>
      expect(window.location.pathname).toBe(
        "/lists/research/records/actions:testAction1",
      ),
    );
    expect(new URL(window.location.href).searchParams.has("focus")).toBe(false);
    expect(apiMocks.removeSavedListItem).not.toHaveBeenCalled();
  });

  it("previews references beside the list reader while retaining selection and exact full-page links", async () => {
    history.replaceState(null, "", "/lists/research/actions%3AtestAction1");
    render(
      <ListDetailView
        route={{
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
    expect(
      await screen.findByRole("heading", { name: "Nested Rule" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("dialog", { name: "Nested Rule" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Test Action 1/ })).toHaveAttribute(
      "aria-current",
      "true",
    );
    const fullPage = new URL(
      screen
        .getByRole("link", { name: "Open reference full page" })
        .getAttribute("href")!,
      location.origin,
    );
    expect(fullPage.pathname).toBe("/records/rules:nested");
    expect(fullPage.search).toBe("");
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    expect(
      await screen.findByRole("heading", { name: "Test Action 1" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Nested Rule" }));
    fireEvent.click(
      await screen.findByRole("button", { name: "Close reference preview" }),
    );
    expect(
      await screen.findByRole("heading", { name: "Test Action 1" }),
    ).toBeInTheDocument();
    expect(window.location.pathname).toBe("/lists/research/actions%3AtestAction1");
  });

  it.each([false, true])(
    "recovers the same selected reader when an unresolved member becomes active (filtered omission=%s)",
    async (omitWhileBlocked) => {
      const selection: RecordNavigationView = {
        ...detailFixture("actions:testAction1").selected,
        embedded_path: [{ collection: "/items", identity: { Stable: "ability-one" } }],
        spell_rank: 0,
      };
      const unresolved = savedListDetailFixture();
      unresolved.items[0] = {
        ...unresolved.items[0],
        status: "unresolved",
        record: undefined,
      };
      apiMocks.filterSavedList.mockResolvedValue(unresolved);
      const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
      render(
        <QueryClientProvider client={client}>
          <App>
            <ListDetailView
              route={{
                slug: "research",
                selectedRecordKey: "actions:testAction1",
                selection,
              }}
            />
          </App>
        </QueryClientProvider>,
      );
      await screen.findByText("Record unavailable in this reference collection");
      await waitFor(() =>
        expect(
          screen.queryByRole("heading", { name: "Test Action 1" }),
        ).not.toBeInTheDocument(),
      );
      if (omitWhileBlocked) {
        await act(async () => {
          client.setQueriesData(
            { queryKey: ["saved-list", "research"] },
            { ...unresolved, items: [] },
          );
        });
        expect(
          screen.queryByRole("heading", { name: "Test Action 1" }),
        ).not.toBeInTheDocument();
      }
      await act(async () => {
        client.setQueriesData(
          { queryKey: ["saved-list", "research"] },
          savedListDetailFixture(),
        );
      });
      expect(
        await screen.findByRole("heading", { name: "Test Action 1" }),
      ).toBeVisible();
      expect(screen.getByRole("link", { name: /Test Action 1/ })).toHaveAttribute(
        "aria-current",
        "true",
      );
      expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
        "actions:testAction1",
        expect.objectContaining({
          embedded_path: selection.embedded_path,
          spell_rank: 0,
        }),
        expect.any(AbortSignal),
      );
    },
  );

  it("replaces an unreadable list link with empty Search without requesting broader list data", async () => {
    const query = "?filter=2.obsolete&q=Action&sort=name_asc";
    history.replaceState(
      null,
      "",
      "/lists/research/records/actions:testAction1" + query,
    );
    const length = history.length;
    apiMocks.filterSavedList.mockImplementation(
      async (request: { sort?: ResultSort }) => ({
        ...savedListDetailFixture(),
        sort: request.sort ?? { kind: "saved_order" },
      }),
    );
    render(
      <ListDetailView
        route={{
          slug: "research",
          selectedRecordKey: "actions:testAction1",
          query,
        }}
      />,
      { wrapper: queryClientWrapper() },
    );
    await waitFor(() => expect(window.location.pathname).toBe("/search"));
    expect(window.location.search).toBe("");
    expect(history.state).toMatchObject({
      atlasFilterResetNotice: true,
      atlasSearchReset: expect.any(String),
    });
    expect(history.length).toBe(length);
    expect(apiMocks.filterSavedList).not.toHaveBeenCalled();
    expect(apiMocks.getRecordDetail).not.toHaveBeenCalled();
    expect(apiMocks.reorderSavedListItem).not.toHaveBeenCalled();
  });

  it.each(["click", "Space"])(
    "keeps the current preview mounted until a %s selection is ready",
    async (input) => {
      history.replaceState(null, "", "/lists/research/records/actions:testAction1");
      const list = savedListDetailFixture();
      list.items.push({
        ...list.items[0],
        record_key: "actions:testAction2",
        snapshot: { title: "Test Action 2" },
        record: summaryFixture("actions:testAction2", "Test Action 2"),
      });
      apiMocks.filterSavedList.mockResolvedValue(list);
      let finish!: (detail: RecordDetailView) => void;
      apiMocks.getRecordDetail.mockImplementation((key: string) =>
        key === "actions:testAction2"
          ? new Promise((resolve) => {
              finish = resolve;
            })
          : Promise.resolve(recordDetailFixture(key)),
      );
      const frames: boolean[] = [];
      render(
        <Profiler
          id="list"
          onRender={() =>
            frames.push(
              Boolean(screen.queryByRole("heading", { name: /Test Action [12]/ })),
            )
          }
        >
          <ListDetailView
            route={{ slug: "research", selectedRecordKey: "actions:testAction1" }}
          />
        </Profiler>,
        { wrapper: queryClientWrapper() },
      );
      const original = await screen.findByRole("heading", { name: "Test Action 1" });
      frames.length = 0;
      const link = screen.getByRole("link", { name: /Test Action 2/ });
      if (input === "click") fireEvent.click(link);
      else fireEvent.keyDown(link, { key: " " });
      await waitFor(() =>
        expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
          "actions:testAction2",
          undefined,
          expect.any(AbortSignal),
        ),
      );
      expect(original).toBeVisible();
      expect(window.location.pathname).toBe(
        "/lists/research/records/actions:testAction1",
      );
      await act(async () =>
        finish(detailFixture("actions:testAction2", "Test Action 2")),
      );
      expect(
        await screen.findByRole("heading", { name: "Test Action 2" }),
      ).toBeVisible();
      await waitFor(() =>
        expect(window.location.pathname).toBe(
          "/lists/research/records/actions:testAction2",
        ),
      );
      expect(frames.every(Boolean)).toBe(true);
    },
  );

  it("keeps selection failure and cancellation reachable in narrow results", async () => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue({ lg: false });
    apiMocks.getRecordDetail.mockRejectedValue(new Error("Read failed"));
    render(<ListDetailView route={{ slug: "research", selectedRecordKey: null }} />, {
      wrapper: queryClientWrapper(),
    });
    fireEvent.click(await screen.findByRole("link", { name: /Test Action 1/ }));
    expect(await screen.findByText("Could not load Test Action 1")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Retry loading Test Action 1" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Cancel selection" }));
    expect(screen.queryByText("Could not load Test Action 1")).not.toBeInTheDocument();
    expect(screen.getByText("1 saved records")).toBeVisible();
  });

  it("retains the current preview and note draft when selection fails, then retries in place", async () => {
    history.replaceState(null, "", "/lists/research/records/actions:testAction1");
    const list = savedListDetailFixture();
    list.items.push({
      ...list.items[0],
      record_key: "actions:testAction2",
      snapshot: { title: "Test Action 2" },
      record: summaryFixture("actions:testAction2", "Test Action 2"),
    });
    apiMocks.filterSavedList.mockResolvedValue(list);
    apiMocks.getRecordDetail.mockImplementation((key: string) =>
      key === "actions:testAction2"
        ? Promise.reject(new Error("Read failed"))
        : Promise.resolve(recordDetailFixture(key)),
    );
    render(
      <ListDetailView
        route={{ slug: "research", selectedRecordKey: "actions:testAction1" }}
      />,
      { wrapper: queryClientWrapper() },
    );
    const original = await screen.findByRole("heading", { name: "Test Action 1" });
    fireEvent.click(await screen.findByRole("button", { name: "Add note" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Note" }), {
      target: { value: "Unfinished first note" },
    });
    fireEvent.click(screen.getByRole("link", { name: /Test Action 2/ }));
    expect(await screen.findByText("Could not load Test Action 2")).toBeVisible();
    expect(original).toBeVisible();
    expect(screen.getByRole("textbox", { name: "Note" })).toHaveValue(
      "Unfinished first note",
    );
    expect(window.location.pathname).toBe(
      "/lists/research/records/actions:testAction1",
    );
    apiMocks.getRecordDetail.mockResolvedValue(
      detailFixture("actions:testAction2", "Test Action 2"),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Retry loading Test Action 2" }),
    );
    expect(await screen.findByRole("heading", { name: "Test Action 2" })).toBeVisible();
    expect(screen.queryByText("Could not load Test Action 2")).not.toBeInTheDocument();
  });

  it("keeps the footer populated on every commit when selecting entries before their note refresh completes", async () => {
    const list = savedListDetailFixture();
    list.items = [
      "actions:testAction1",
      "actions:testAction2",
      "actions:testAction3",
    ].map((record_key, index) => ({
      ...list.items[0],
      record_key,
      position: index + 1,
      note: index === 2 ? undefined : "Shared preparation note",
      snapshot: { title: `Test Action ${index + 1}` },
    }));
    apiMocks.filterSavedList.mockResolvedValue(list);
    apiMocks.getSavedListItemNote.mockImplementation(() => new Promise(() => {}));
    const frames: string[] = [];
    const pane = (recordKey: string) => (
      <Profiler
        id="list"
        onRender={() => {
          frames.push(
            screen.queryByRole("region", { name: "List entry note" })?.textContent ??
              "missing",
          );
        }}
      >
        <ListDetailView route={{ slug: "research", selectedRecordKey: recordKey }} />
      </Profiler>
    );
    const view = render(pane("actions:testAction1"), { wrapper: queryClientWrapper() });
    expect(await screen.findByText("Shared preparation note")).toBeVisible();
    frames.length = 0;
    view.rerender(pane("actions:testAction2"));
    await waitFor(() =>
      expect(apiMocks.getSavedListItemNote).toHaveBeenCalledWith(
        "list_research",
        "actions:testAction2",
      ),
    );
    expect(frames.length).toBeGreaterThan(0);
    expect(frames.every((text) => text.includes("Shared preparation note"))).toBe(true);
    expect(screen.getByRole("button", { name: "Edit" })).toBeVisible();
    view.rerender(pane("actions:testAction3"));
    expect(screen.queryByText("Shared preparation note")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Add note" }));
    expect(screen.getByRole("textbox", { name: "Note" })).toHaveValue("");
  });

  it("keeps temporary order in list URLs and native links without enabling reorder writes", async () => {
    apiMocks.filterSavedList.mockImplementation(
      async (request: { sort?: ResultSort }) => ({
        ...savedListDetailFixture(),
        sort: request.sort ?? { kind: "saved_order" },
      }),
    );
    const route = {
      kind: "list" as const,
      slug: "research",
      selectedRecordKey: "actions:testAction1",
      query: "?sort=level_desc",
    };
    const view = render(<ListDetailView route={route} />, {
      wrapper: queryClientWrapper(),
    });
    await screen.findByRole("heading", { name: "Test Action 1" });
    fireEvent.click(await screen.findByRole("button", { name: "Add note" }));
    const note = screen.getByRole("textbox", { name: "Note" });
    fireEvent.change(note, { target: { value: "Keep this unsaved note" } });
    expect(apiMocks.filterSavedList).toHaveBeenCalledWith(
      expect.objectContaining({
        sort: {
          kind: "attributes",
          rules: [{ field: "level", direction: "desc" }],
        } satisfies ResultSort,
      }),
    );
    expect(screen.getByRole("link", { name: /Test Action 1/ })).toHaveAttribute(
      "href",
      "/lists/research/records/actions:testAction1?sort=level_desc",
    );
    expect(screen.queryByRole("button", { name: "Reorder Test Action 1" })).toBeNull();
    expect(apiMocks.reorderSavedListItem).not.toHaveBeenCalled();
    view.rerender(<ListDetailView route={{ ...route, query: "?sort=saved_order" }} />);
    await screen.findByRole("button", { name: "Reorder Test Action 1" });
    expect(screen.getByRole("textbox", { name: "Note" })).toBe(note);
    expect(note).toHaveValue("Keep this unsaved note");
    expect(apiMocks.reorderSavedListItem).not.toHaveBeenCalled();
    expect(apiMocks.updateSavedListItemNote).not.toHaveBeenCalled();
  });

  it("preserves selected membership and nested reading while filters hide the selected member", async () => {
    apiMocks.filterSavedList.mockImplementation(
      async (request: { query?: string }) => ({
        ...savedListDetailFixture(),
        items: request.query ? [] : savedListDetailFixture().items,
      }),
    );
    render(
      <ListDetailView
        route={{
          slug: "research",
          selectedRecordKey: "actions:testAction1",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );
    const listTitle = `${savedListDetailFixture().list.name} · Atlas`;
    await waitFor(() => expect(document.title).toBe(listTitle));
    fireEvent.click(await screen.findByRole("link", { name: "Nested Rule" }));
    await screen.findByRole("heading", { name: "Nested Rule" });
    expect(document.title).toBe(listTitle);
    const input = screen.getByRole("searchbox", { name: "Search records" });
    fireEvent.change(input, { target: { value: "no matching records" } });
    fireEvent.keyDown(input, { key: "Enter", code: "Enter" });
    expect(
      await screen.findByText("No saved records match these filters"),
    ).toBeVisible();
    expect(screen.getByRole("heading", { name: "Nested Rule" })).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    expect(await screen.findByRole("heading", { name: "Test Action 1" })).toBeVisible();
    expect(apiMocks.removeSavedListItem).not.toHaveBeenCalled();
    fireEvent.change(input, { target: { value: "" } });
    await waitFor(() =>
      expect(apiMocks.filterSavedList.mock.lastCall?.[0].query).toBeUndefined(),
    );
    expect(await screen.findByRole("link", { name: /Test Action 1/ })).toBeVisible();
  });

  it("searches within a saved list through the filter route", async () => {
    render(
      <ListDetailView
        route={{
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

  it.each(["Renamed Research!", "🌙"])(
    "edits list metadata to %s while preserving its URL",
    async (name) => {
      render(
        <ListEditView
          route={{
            slug: "research",
          }}
        />,
        { wrapper: queryClientWrapper() },
      );

      expect(await screen.findByDisplayValue("Research")).toBeInTheDocument();
      expect(screen.queryByLabelText("Slug")).not.toBeInTheDocument();

      fireEvent.change(screen.getByLabelText("Name"), {
        target: { value: name },
      });
      fireEvent.change(screen.getByLabelText("Description"), {
        target: { value: "Updated prep" },
      });
      fireEvent.click(screen.getByRole("button", { name: "Save" }));

      await waitFor(() =>
        expect(apiMocks.updateSavedList).toHaveBeenCalledWith({
          list_key: "list_research",
          slug: "research",
          name,
          description: "Updated prep",
          tags: ["arc-one", "boss"],
        }),
      );
      await waitFor(() => expect(window.location.pathname).toBe("/lists/research"));
    },
  );

  it("deletes lists from the edit page and returns to the list index", async () => {
    render(
      <ListEditView
        route={{
          slug: "research",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );

    await screen.findByDisplayValue("Research");
    fireEvent.click(screen.getByRole("button", { name: "Delete List" }));
    expect(apiMocks.deleteSavedList).not.toHaveBeenCalled();
    fireEvent.click(await screen.findByRole("button", { name: "Delete list" }));

    await waitFor(() =>
      expect(apiMocks.deleteSavedList).toHaveBeenCalledWith("list_research"),
    );
    await waitFor(() => expect(window.location.pathname).toBe("/lists"));
  });

  it("returns to the saved-lists index using a native parent link without a duplicate switcher", async () => {
    render(<ListDetailView route={{ slug: "research", selectedRecordKey: null }} />, {
      wrapper: queryClientWrapper(),
    });
    const parent = await screen.findByRole("link", { name: "Saved lists" });
    expect(parent).toHaveAttribute("href", "/lists");
    expect(screen.queryByRole("combobox", { name: "Selected list" })).toBeNull();
    fireEvent.click(parent);
    await waitFor(() => expect(window.location.pathname).toBe("/lists"));
  });

  it("adds records in the shared preview picker, retaining reader, search and candidate through pending failure and retry", async () => {
    let reject!: (error: Error) => void;
    apiMocks.addSavedListItem.mockReturnValueOnce(
      new Promise((_resolve, no) => {
        reject = no;
      }),
    );
    render(
      <ListDetailView
        route={{
          slug: "research",
          selectedRecordKey: "actions:testAction1",
        }}
      />,
      { wrapper: queryClientWrapper() },
    );
    await screen.findByRole("heading", { name: "Test Action 1" });
    fireEvent.click(screen.getByRole("button", { name: "Add records" }));
    const dialog = await screen.findByRole("dialog");
    fireEvent.click(await within(dialog).findByRole("link", { name: /Test Action 1/ }));
    await within(dialog).findByRole("heading", { name: "Test Action 1" });
    const input = within(dialog).getByRole("searchbox", { name: "Search records" });
    fireEvent.change(input, { target: { value: "Unsubmitted search" } });
    const add = within(dialog).getByRole("button", { name: "Add" });
    fireEvent.click(add);
    fireEvent.click(add);
    await waitFor(() => expect(apiMocks.addSavedListItem).toHaveBeenCalledOnce());
    expect(add).toBeDisabled();
    expect(input).toHaveValue("Unsubmitted search");
    await act(async () => reject(new Error("Add rejected")));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(
      "Record could not be added",
    );
    expect(add).toBeEnabled();
    fireEvent.click(add);
    expect(await within(dialog).findByText("Added to list.")).toBeVisible();
    expect(apiMocks.addSavedListItem).toHaveBeenCalledTimes(2);
    expect(screen.getByRole("dialog")).toBeVisible();
    expect(input).toHaveValue("Unsubmitted search");
    expect(add).toBeDisabled();
    fireEvent.click(
      dialog.querySelector<HTMLElement>('[data-record-key="rules:nested"]')!,
    );
    await waitFor(() => expect(add).toBeEnabled());
    apiMocks.addSavedListItem.mockResolvedValueOnce({
      list_key: "list_research",
      slug: "research",
      record_key: "rules:nested",
      outcome: "already_present",
    });
    fireEvent.click(add);
    expect(await within(dialog).findByText("Already in this list.")).toBeVisible();
    expect(apiMocks.addSavedListItem).toHaveBeenLastCalledWith({
      list_ref: "research",
      record_ref: "rules:nested",
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(screen.getByRole("heading", { name: "Test Action 1" })).toBeVisible();
    expect(window.location.pathname).toBe("/lists");
  });

  it("explains the saved-list action on keyboard focus without loading lists", async () => {
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <AddToRecordButton recordKey="actions:testAction1" />
      </ConfigProvider>,
      { wrapper: queryClientWrapper() },
    );
    const action = screen.getByRole("button", { name: "Add to list" });
    act(() => action.focus());
    expect(action).toHaveFocus();
    expect(await screen.findByRole("tooltip")).toHaveTextContent("Add to list");
    expect(apiMocks.getSavedLists).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    act(() => action.blur());
    await waitFor(() => expect(screen.queryByRole("tooltip")).not.toBeInTheDocument());
  });

  it("adds the current record to a selected list", async () => {
    render(<AddToRecordButton recordKey="actions:testAction1" />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(screen.getByRole("button", { name: "Add to list" }));

    const selector = await screen.findByRole("combobox");
    await waitFor(() => expect(selector).toBeEnabled());
    fireEvent.mouseDown(selector);
    fireEvent.click(await screen.findByText("Research"));
    fireEvent.click(screen.getByRole("button", { name: "Add" }));

    await waitFor(() =>
      expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
        list_ref: "list_research",
        record_ref: "actions:testAction1",
      }),
    );
  });

  it("creates lists from a name without asking for a slug", async () => {
    render(<ListIndexView route={{}} />, {
      wrapper: queryClientWrapper(),
    });

    fireEvent.click(screen.getByRole("button", { name: "New list" }));
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
    const { rerender } = render(<ListEditView route={{ slug: "research" }} />, {
      wrapper,
    });
    const name = await screen.findByLabelText("Name");
    fireEvent.change(name, { target: { value: "My list draft" } });
    const oldDetail = recordDetailQueryOptions("old");
    client
      .getQueryCache()
      .build(client, { queryKey: oldDetail.queryKey, meta: oldDetail.meta });
    client.setQueryData(oldDetail.queryKey, detailFixture("old"));
    await act(async () => retireReferenceQueries(client));
    expect(client.getQueryData(oldDetail.queryKey)).toBeUndefined();
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
    rerender(<ListEditView route={{ slug: "other" }} />);
    expect(await screen.findByLabelText("Name")).toHaveValue("Other list");
  });
  it("retries an initial list failure without discarding later edits", async () => {
    apiMocks.getSavedList.mockRejectedValueOnce(new Error("List unavailable"));
    render(<ListEditView route={{ slug: "research" }} />, {
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
  useSavedListFilterDiscovery(listRef, {
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
    <VocabularyProbe
      scope={{ kind: "saved_list", list_ref: listRef }}
      label={`values ${listRef}`}
    />
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
    return (
      <QueryClientProvider client={queryClient}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          <SearchProfileProvider>
            <RouterTestContext>
              <App>{children}</App>
            </RouterTestContext>
          </SearchProfileProvider>
        </ConfigProvider>
      </QueryClientProvider>
    );
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
    sort: { kind: "saved_order" } as const,
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
    fieldFixture("level", "number", "Level"),
    fieldFixture("common.rarity", "string", "Rarity"),
    fieldFixture("kind", "string", "Kind"),
    fieldFixture("common.traits", "set", "Traits"),
    fieldFixture("source.pack", "string", "Pack"),
  ]);
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  return {
    ...detailFixture(
      recordKey,
      recordKey === "rules:nested" ? "Nested Rule" : "Test Action 1",
      recordKey === "rules:nested" ? undefined : "rules:nested",
    ),
    artifact_generation: "generation-a",
  };
}
