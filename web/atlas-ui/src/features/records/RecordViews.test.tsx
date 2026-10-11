import { detailFixture } from "../../test/fixtures";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import type { RecordDetailView, RecordNavigationView } from "../../generated/atlas";
import { ReaderView, RecordView } from "./RecordViews";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  atlasRoutePath,
  currentAtlasRoute,
} from "../../app/routes";

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedLists: vi.fn(),
}));

vi.mock("../../api/atlasApi", () => ({
  addSavedListItem: apiMocks.addSavedListItem,
  getRecordDetail: apiMocks.getRecordDetail,
  getSavedLists: apiMocks.getSavedLists,
}));

describe("record route views", () => {
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

  it("renders a standalone record detail route", async () => {
    render(<RecordView route={{ kind: "record", recordKey: "spell:heal" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(screen.getByText("spell:heal")).toBeInTheDocument();
    expect(screen.getByText("Reader view")).toBeInTheDocument();
  });

  it("adds a standalone record detail route record to a saved list", async () => {
    render(<RecordView route={{ kind: "record", recordKey: "spell:heal" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();

    await addCurrentRecordToList();

    expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
      list_ref: "research",
      record_ref: "spell:heal",
    });
  });

  it("opens linked references in reader preview without replacing the primary record", async () => {
    history.replaceState(null, "", "/reader/spell%3Aheal");
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Linked Record" }));

    await waitFor(() => expect(window.location.pathname).toBe("/reader/spell%3Aheal"));
    expect(currentAtlasRoute()).toMatchObject({
      kind: "reader",
      recordKey: "spell:heal",
      previewRecordKey: "spell:linked",
      previewSelection: { record_key: "spell:linked" },
    });
    expect(screen.getByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
  });

  it("promotes the reader preview into the primary reader slot", async () => {
    history.replaceState(null, "", "/reader/spell%3Aheal?preview=spell%3Alinked");
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });

    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("link", { name: "Open preview as reader" }));

    await waitFor(() =>
      expect(window.location.pathname).toBe("/reader/spell%3Alinked"),
    );
    expect(window.location.search).toBe("");
  });

  it("adds the reader primary record to a saved list", async () => {
    history.replaceState(null, "", "/reader/spell%3Aheal?preview=spell%3Alinked");
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();

    await addCurrentRecordToList(0);

    expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
      list_ref: "research",
      record_ref: "spell:heal",
    });
  });

  it("adds the reader preview record to a saved list", async () => {
    history.replaceState(null, "", "/reader/spell%3Aheal?preview=spell%3Alinked");
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });

    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();

    await addCurrentRecordToList(1);

    expect(apiMocks.addSavedListItem).toHaveBeenCalledWith({
      list_ref: "research",
      record_ref: "spell:linked",
    });
  });

  it("closes the reader preview without replacing the primary reader record", async () => {
    history.replaceState(null, "", "/reader/spell%3Aheal?preview=spell%3Alinked");
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });

    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Close preview" }));

    await waitFor(() => expect(window.location.pathname).toBe("/reader/spell%3Aheal"));
    expect(window.location.search).toBe("");
    expect(
      screen.getByText("Select a linked record to preview it."),
    ).toBeInTheDocument();
  });

  it("preserves owned field and passage identity through preview follow, close, promotion and history", async () => {
    const selected = (record_key: string, ordinal: number): RecordNavigationView => ({
      record_key,
      owners: [
        { collection: "items", identity: { SnapshotLocal: { index: ordinal } } },
      ],
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
    apiMocks.getRecordDetail.mockImplementation((recordKey: string) => {
      const result = recordDetailFixture(recordKey);
      result.relationships[0].target =
        recordKey === primary.record_key ? first : second;
      return Promise.resolve(result);
    });
    history.replaceState(
      null,
      "",
      atlasRoutePath({
        kind: "reader",
        recordKey: primary.record_key,
        previewRecordKey: null,
        selection: primary,
      }),
    );
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });
    fireEvent.click(await screen.findByRole("button", { name: "Linked Record" }));
    await screen.findByRole("heading", { name: "first" });
    expect(currentAtlasRoute()).toMatchObject({
      selection: primary,
      previewSelection: first,
    });
    fireEvent.click(screen.getAllByRole("button", { name: "Linked Record" })[1]);
    await screen.findByRole("heading", { name: "second" });
    const followedPath = window.location.pathname + window.location.search;
    expect(currentAtlasRoute()).toMatchObject({
      selection: primary,
      previewSelection: second,
    });
    fireEvent.click(screen.getByRole("button", { name: "Close preview" }));
    expect(currentAtlasRoute()).toMatchObject({
      selection: primary,
      previewRecordKey: null,
    });
    // The browser restores the complete saved URL for Back/Forward.
    history.replaceState(null, "", followedPath);
    window.dispatchEvent(new PopStateEvent("popstate"));
    await screen.findByRole("heading", { name: "second" });
    expect(currentAtlasRoute()).toMatchObject({
      selection: primary,
      previewSelection: second,
    });
    fireEvent.click(screen.getByRole("link", { name: "Open preview as reader" }));
    expect(currentAtlasRoute()).toMatchObject({
      recordKey: second.record_key,
      selection: second,
      previewRecordKey: null,
    });
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(second.record_key, {
      ...second,
      fields: [second.field],
    });
  });
});

function ReaderHarness() {
  const [route, setRoute] = useState(currentAtlasRoute);

  useEffect(() => {
    const onRouteChange = () => setRoute(currentAtlasRoute());
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
    window.addEventListener("popstate", onRouteChange);
    return () => {
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
      window.removeEventListener("popstate", onRouteChange);
    };
  }, []);

  return route.kind === "reader" ? <ReaderView route={route} /> : null;
}

async function addCurrentRecordToList(buttonIndex = 0) {
  const buttons = await screen.findAllByRole("button", {
    name: "Add to saved list",
  });
  fireEvent.click(buttons[buttonIndex]!);

  const selector = await screen.findByRole("combobox");
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
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
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
        item_count: 1n,
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
