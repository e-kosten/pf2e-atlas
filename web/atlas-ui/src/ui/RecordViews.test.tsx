import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import type { RecordDetailView } from "../generated/atlas";
import { ReaderView, RecordView } from "./RecordViews";
import { ATLAS_ROUTE_CHANGE_EVENT, currentAtlasRoute } from "./routes";

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedLists: vi.fn(),
}));

vi.mock("../api/atlasApi", () => ({
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
    apiMocks.getSavedLists.mockResolvedValue({ lists: [] });
    apiMocks.addSavedListItem.mockResolvedValue({
      slug: "research",
      record_key: "spell:heal",
      outcome: "added",
    });
  });

  it("renders a standalone record detail route", async () => {
    render(<RecordView route={{ kind: "record", recordKey: "spell:heal" }} />, {
      wrapper: queryClientWrapper(),
    });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    expect(screen.getByText("spell:heal")).toBeInTheDocument();
    expect(screen.getByText("Reader view")).toBeInTheDocument();
  });

  it("opens linked references in reader preview without replacing the primary record", async () => {
    history.replaceState(null, "", "/reader/spell%3Aheal");
    render(<ReaderHarness />, { wrapper: queryClientWrapper() });

    expect(await screen.findByRole("heading", { name: "heal" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Linked Record" }));

    await waitFor(() => expect(window.location.pathname).toBe("/reader/spell%3Aheal"));
    expect(window.location.search).toBe("?preview=spell%3Alinked");
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

function recordDetailFixture(recordKey: string): RecordDetailView {
  const title = recordKey.split(":")[1] ?? recordKey;
  return {
    record_key: recordKey,
    title,
    kind: "spell",
    presentation: {
      record_key: recordKey,
      kind: "spell",
      title,
      identity: [],
      badges: [],
      sections: [
        {
          kind: "description",
          title: "Description",
          blocks: [
            {
              kind: "relationships",
              content: [
                {
                  kind: "reference",
                  label: "Linked Record",
                  record_key: "spell:linked",
                },
              ],
            },
          ],
        },
      ],
    },
  };
}
