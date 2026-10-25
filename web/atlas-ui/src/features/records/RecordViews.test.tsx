import { detailFixture, spellFixture, numberFact } from "../../test/fixtures";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useRouterState } from "@tanstack/react-router";
import { RouterTestContext } from "../../test/router";
import type { ReactNode } from "react";
import { ConfigProvider, Grid } from "antd";
import type {
  RecordDetailRequest,
  RecordDetailView,
  RecordNavigationView,
} from "../../generated/atlas";
import { RecordView } from "./RecordViews";
import { destinationHref } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { selectedNavigation } from "../../app/urlState";
import { atlasRouter } from "../../app/router";

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedLists: vi.fn(),
}));

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  addSavedListItem: apiMocks.addSavedListItem,
  getRecordDetail: apiMocks.getRecordDetail,
  getSavedLists: apiMocks.getSavedLists,
}));

describe("record route views", () => {
  afterEach(() => vi.restoreAllMocks());
  beforeEach(() => {
    vi.clearAllMocks();
    history.replaceState(null, "", "/");
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) =>
      Promise.resolve(recordDetailFixture(recordKey)),
    );
    apiMocks.getSavedLists.mockResolvedValue(savedListIndexFixture());
    apiMocks.addSavedListItem.mockImplementation(
      ({ record_ref, list_ref }: { record_ref: string; list_ref: string }) =>
        Promise.resolve({
          list_key: list_ref,
          slug: "research",
          record_key: record_ref,
          outcome: "added",
        }),
    );
  });

  it("changes standalone spell rank without adding history entries or losing its owner", async () => {
    const origin = {
      recordKey: "spell:heal",
      href: "/search/records/spell:heal?q=heal",
    };
    history.replaceState({ atlasOwnerOrigin: origin }, "", "/records/spell:heal");
    apiMocks.getRecordDetail.mockImplementation(
      (key: string, request?: RecordDetailRequest) => {
        const detail = detailFixture(key);
        const rank = request?.spell_rank ?? 3;
        return Promise.resolve({
          ...detail,
          selected: { ...detail.selected, spell_rank: rank },
          presentation: {
            ...detail.presentation,
            body: {
              kind: "spell",
              value: { ...spellFixture(), rank: numberFact(rank) },
            },
          },
        });
      },
    );
    render(<RecordHarness />, { wrapper: queryClientWrapper() });
    await screen.findByRole("button", { name: "Change rank" });
    const entries = history.length;
    for (const rank of [4, 5]) {
      fireEvent.click(screen.getByRole("button", { name: "Change rank" }));
      fireEvent.click(await screen.findByRole("menuitem", { name: "Rank " + rank }));
      await waitFor(() =>
        expect(selectedNavigation(atlasRouter.latestLocation.search)?.spell_rank).toBe(
          rank,
        ),
      );
      await waitFor(() =>
        expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
          "spell:heal",
          expect.objectContaining({ spell_rank: rank }),
          expect.any(AbortSignal),
        ),
      );
      expect(history.length).toBe(entries);
      expect(history.state.atlasOwnerOrigin).toEqual(origin);
    }
  });

  it("renders a standalone record detail route", async () => {
    render(<RecordView route={{ recordKey: "spell:heal" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(screen.queryByText("spell:heal")).toBeNull();
    expect(screen.queryByRole("link", { name: "Reader view" })).toBeNull();
    expect(screen.queryByText("Reference", { exact: true })).toBeNull();
    expect(
      screen.getByRole("heading", { name: "heal" }).closest("header"),
    ).toContainElement(screen.getByRole("button", { name: "Add to saved list" }));
    expect(screen.queryByRole("button", { name: "Back" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Refresh" })).toBeNull();
  });

  it("adds a standalone record detail route record to a saved list", async () => {
    render(<RecordView route={{ recordKey: "spell:heal" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();

    await addCurrentRecordToList();

    expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
      list_ref: "research",
      record_ref: "spell:heal",
    });
  });

  it("keeps an open saved-list dialog and its trigger when the page identity arrives", async () => {
    let resolveDetail!: (detail: RecordDetailView) => void;
    apiMocks.getRecordDetail.mockReturnValue(
      new Promise<RecordDetailView>((resolve) => {
        resolveDetail = resolve;
      }),
    );
    render(<RecordView route={{ recordKey: "spell:heal" }} />, {
      wrapper: queryClientWrapper(),
    });
    const trigger = screen.getByRole("button", { name: "Add to saved list" });
    fireEvent.click(trigger);
    const dialog = await screen.findByRole("dialog", { name: "Add to List" });
    await act(async () => resolveDetail(recordDetailFixture("spell:heal")));
    expect(screen.getByRole("dialog", { name: "Add to List" })).toBe(dialog);
    expect(trigger).toBeInTheDocument();
    await waitFor(() => expect(trigger.closest("header")).toHaveTextContent("heal"));
  });

  it("previews a standalone reference without leaving its parent, with a native full-page link", async () => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xl: true });
    history.replaceState(null, "", "/records/spell:heal");
    render(<RecordView route={{ recordKey: "spell:heal" }} />, {
      wrapper: queryClientWrapper(),
    });
    const trigger = await screen.findByRole("link", { name: "Linked Record" });
    fireEvent.click(trigger);
    await waitFor(() =>
      expect(screen.getByRole("heading", { name: "linked" })).toBeVisible(),
    );
    expect(screen.getByRole("heading", { name: "heal" })).toBeVisible();
    expect(window.location.pathname).toBe("/records/spell:heal");
    expect(
      screen.getByRole("link", { name: "Open reference full page" }),
    ).toHaveAttribute(
      "href",
      destinationHref(
        recordDestination({
          recordKey: "spell:linked",
          selection: recordDetailFixture("spell:linked").selected,
        }),
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    await waitFor(() => expect(trigger).toHaveFocus());
  });

  it("opens a contextual reference as a full page without changing its exact destination", async () => {
    history.replaceState(null, "", "/records/spell:heal");
    render(<RecordHarness />, { wrapper: queryClientWrapper() });
    fireEvent.click(await screen.findByRole("link", { name: "Linked Record" }));
    await screen.findByRole("heading", { name: "linked" });
    fireEvent.click(screen.getByRole("link", { name: "Open reference full page" }));
    await waitFor(() =>
      expect(window.location.pathname).toBe("/records/" + "spell:linked"),
    );
  });

  it("adds the contextual reference to a list without changing the primary selection", async () => {
    history.replaceState(null, "", "/records/spell:heal");
    render(<RecordHarness />, { wrapper: queryClientWrapper() });
    fireEvent.click(await screen.findByRole("link", { name: "Linked Record" }));
    await screen.findByRole("heading", { name: "linked" });
    await addCurrentRecordToList(1);
    expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
      list_ref: "research",
      record_ref: "spell:linked",
    });
    await waitFor(() =>
      expect(window.location.pathname).toBe("/records/" + "spell:heal"),
    );
  });

  it("preserves embedded field, passage and fingerprint through nested reading and Back", async () => {
    const selected = (record_key: string, index: number): RecordNavigationView => ({
      record_key,
      embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index } } }],
      field: "system.description.value",
      passage: {
        kind: "plain_section",
        source_text_sha256: "text",
        selection_version: "v1",
        section_ordinal: 2,
        label: "Passage",
        chunk_bytes: { start: 0, end: 8 },
      },
      source_fingerprint: "snapshot",
    });
    const primary = selected("actors:primary", 0);
    const first = selected("actors:first", 1);
    const second = selected("actors:second", 2);
    apiMocks.getRecordDetail.mockImplementation(
      (recordKey: string, selection?: RecordDetailRequest) => {
        const result = recordDetailFixture(recordKey);
        result.relationships[0].target =
          recordKey === primary.record_key ? first : second;
        if (selection)
          result.selected = {
            record_key: selection.record_key,
            embedded_path: selection.embedded_path,
            field: selection.fields[0] ?? null,
            passage: selection.passage,
            source_fingerprint: selection.source_fingerprint,
          };
        return Promise.resolve(result);
      },
    );
    history.replaceState(
      null,
      "",
      destinationHref(
        recordDestination({
          recordKey: primary.record_key,
          selection: primary,
        }),
      ),
    );
    render(<RecordHarness />, { wrapper: queryClientWrapper() });
    const trigger = await screen.findByRole("link", { name: "Linked Record" });
    fireEvent.click(trigger);
    await screen.findByRole("heading", { name: "first" });
    fireEvent.click(screen.getAllByRole("link", { name: "Linked Record" })[1]);
    await screen.findByRole("heading", { name: "second" });
    expect(selectedNavigation(atlasRouter.latestLocation.search)).toEqual(primary);
    expect(
      screen.getByRole("link", { name: "Open reference full page" }),
    ).toHaveAttribute(
      "href",
      destinationHref(
        recordDestination({
          recordKey: second.record_key,
          selection: second,
        }),
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Back" }));
    await screen.findByRole("heading", { name: "first" });
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    await waitFor(() => expect(trigger).toHaveFocus());
    expect(selectedNavigation(atlasRouter.latestLocation.search)).toEqual(primary);
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
      second.record_key,
      {
        record_key: second.record_key,
        embedded_path: second.embedded_path,
        fields: [second.field],
        passage: second.passage,
        source_fingerprint: second.source_fingerprint,
      },
      expect.any(AbortSignal),
    );
  });
});

function RecordHarness() {
  const location = useRouterState({ select: (state) => state.location });
  const matches = atlasRouter.matchRoutes(location.pathname, location.search);
  const match = matches.find((match) => match.routeId === "/records/$recordKey");
  if (!match || !("recordKey" in match.params)) return null;
  const route = {
    recordKey: match.params.recordKey,
    selection: selectedNavigation(location.search),
    landing: selectedNavigation(location.search, "landing"),
  };
  return <RecordView key={route.recordKey} route={route} />;
}

async function addCurrentRecordToList(buttonIndex = 0) {
  const buttons = await screen.findAllByRole("button", {
    name: "Add to saved list",
  });
  fireEvent.click(buttons[buttonIndex]!);

  const selector = await screen.findByRole("combobox");
  await waitFor(() => expect(selector).toBeEnabled());
  fireEvent.mouseDown(selector);
  fireEvent.click(await screen.findByText("Research"));
  fireEvent.click(screen.getByRole("button", { name: "Add" }));
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
        <RouterTestContext>
          <ConfigProvider theme={{ token: { motion: false } }}>
            {children}
          </ConfigProvider>
        </RouterTestContext>
      </QueryClientProvider>
    );
  };
}

function savedListIndexFixture() {
  return {
    lists: [
      {
        list_key: "list_research",
        slug: "research",
        name: "Research",
        description: "Campaign prep",
        tags: ["arc-one"],
        item_count: 1,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
    ],
  };
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  const result = detailFixture(recordKey, undefined, "spell:linked");
  if (result.presentation.content[0]?.body.kind === "html")
    result.presentation.content[0].body.html =
      result.presentation.content[0].body.html.replace("Nested Rule", "Linked Record");
  return result;
}
