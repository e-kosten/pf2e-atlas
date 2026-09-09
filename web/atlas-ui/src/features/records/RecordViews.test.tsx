import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import type { RecordDetailView } from "../../generated/atlas";
import {
  heroPointDeckRecordKey,
  recordDetailFixture as typedRecordDetailFixture,
  rollTableRecordDetailFixture,
  rollTableResultOneLocator,
} from "../../test/recordFixtures";
import { ReaderView, RecordView } from "./RecordViews";
import { ATLAS_ROUTE_CHANGE_EVENT, currentAtlasRoute } from "../../app/routes";

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
    expect(screen.queryByText("spell:heal")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Source & provenance/ }));
    expect(screen.getByText("spell:heal")).toBeInTheDocument();
    expect(screen.getByText("Reader view")).toBeInTheDocument();
  });

  it("cold-loads an opaque journal child route through record detail selection", async () => {
    const recordKey = "journals:hero-points";
    const childLocator = "v1~j~s~706167652d73656c6563746564";
    history.replaceState(
      null,
      "",
      `/records/${encodeURIComponent(recordKey)}?child=${encodeURIComponent(childLocator)}`,
    );
    apiMocks.getRecordDetail.mockResolvedValue(
      h8JournalRecordDetailFixture(childLocator),
    );
    const route = currentAtlasRoute();
    expect(route).toEqual({ kind: "record", recordKey, childLocator });
    if (route.kind !== "record") {
      throw new Error("expected the cold URL to parse as a record route");
    }

    render(<RecordView route={route} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByText("Selected page content.")).toBeInTheDocument();
    expect(screen.queryByText("First page content.")).not.toBeInTheDocument();
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
      recordKey,
      { child_locator: childLocator },
      expect.any(AbortSignal),
    );
  });

  it("cold-loads a concise RollTable result route through record detail selection", async () => {
    const recordKey = heroPointDeckRecordKey;
    history.replaceState(
      null,
      "",
      `/records/${encodeURIComponent(recordKey)}?child=${encodeURIComponent(rollTableResultOneLocator)}`,
    );
    apiMocks.getRecordDetail.mockResolvedValue(rollTableRecordDetailFixture());
    const route = currentAtlasRoute();
    expect(route).toEqual({
      kind: "record",
      recordKey,
      childLocator: rollTableResultOneLocator,
    });
    if (route.kind !== "record") {
      throw new Error("expected the cold URL to parse as a record route");
    }

    render(<RecordView route={route} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByText("Second result content.")).toBeInTheDocument();
    expect(screen.getByText("Result 1 selected")).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(screen.getByRole("region", { name: "Selected result 1" })).toHaveFocus();
    expect(apiMocks.getRecordDetail).toHaveBeenCalledWith(
      recordKey,
      { child_locator: rollTableResultOneLocator },
      expect.any(AbortSignal),
    );
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
    fireEvent.click(screen.getByRole("link", { name: "Linked Record" }));
    expect(
      screen.getByRole("dialog", { name: "Linked Record spell details" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Open spell record" }));

    await waitFor(() => expect(window.location.pathname).toBe("/reader/spell%3Aheal"));
    expect(window.location.search).toBe("?preview=spell%3Alinked");
    expect(screen.getByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "linked" })).toBeInTheDocument();
  });

  it("opens a typed child reference from reader content on its exact parent route", async () => {
    const childLocator = "v1~j~s~706167652d31";
    apiMocks.getRecordDetail.mockResolvedValueOnce(
      typedRecordDetailFixture({
        recordKey: "journals:source",
        title: "Source journal",
        contentReferenceLabel: "Exact target page",
        contentReferenceRecordKey: "journals:target",
        contentReferenceChildLocator: childLocator,
      }),
    );
    history.replaceState(null, "", "/reader/journals%3Asource");
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });

    fireEvent.click(await screen.findByRole("button", { name: "Related content" }));
    fireEvent.click(await screen.findByRole("link", { name: "Exact target page" }));

    await waitFor(() =>
      expect(window.location.pathname).toBe("/records/journals%3Atarget"),
    );
    expect(window.location.search).toBe(`?child=${encodeURIComponent(childLocator)}`);
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
});

function ReaderHarness() {
  const [route, setRoute] = useState(currentAtlasRoute);

  useEffect(() => {
    const onRouteChange = () => setRoute(currentAtlasRoute());
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
    return () => window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
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
  const title = recordKey.split(":")[1] ?? recordKey;
  return typedRecordDetailFixture({
    recordKey,
    title,
    referenceLabel: "Linked Record",
    referenceRecordKey: "spell:linked",
  });
}

function h8JournalRecordDetailFixture(selectedLocator: string): RecordDetailView {
  const missing = { state: "missing" } as const;
  const known = <T,>(value: T) => ({ state: "known", value }) as const;
  const page = (
    locator: string,
    name: string,
    content: string,
    sourceOrdinal: number,
  ) => ({
    entry_type: "page" as const,
    page: {
      locator,
      identity_stability: "stable_source_id" as const,
      source_id: known(name.toLowerCase().replace(/ /g, "-")),
      source_ordinal: sourceOrdinal,
      name: known(name),
      page_kind: known("text"),
      sort: known(sourceOrdinal),
      title: known({ show: known(true), level: known(2) }),
      text: known({
        content: known([
          {
            block_type: "paragraph" as const,
            spans: [{ span_type: "text" as const, text: content }],
          },
        ]),
        format: known(1),
        markdown: missing,
      }),
      source: missing,
      image_source: known("{}"),
      image_caption: missing,
      video: missing,
      source_system: known("{}"),
      source_metadata: { ownership: missing, flags: missing, stats: missing },
    },
  });

  return {
    surface: {
      metadata: {
        record_key: "journals:hero-points",
        title: "Hero Points",
        kind: "journal",
        kind_label: "Journal",
      },
      profile: "record_detail",
      presentation: {
        presentation_type: "journal",
        body: {
          source_id: "journal-id",
          pages: known([
            page("v1~j~s~first", "Introduction", "First page content.", 0),
            page(selectedLocator, "Selected Page", "Selected page content.", 1),
          ]),
          source_metadata: {
            folder: missing,
            sort: known(0),
            ownership: missing,
            flags: missing,
            stats: missing,
          },
          provenance: {
            source_path: "packs/journals/hero-point-deck.json",
            source_contract_version: "pf2e-serialized-source/v1",
            source_system_version: "6.12.4",
            source_upstream_commit: "4cbdaa37",
          },
        },
      },
    },
  };
}
