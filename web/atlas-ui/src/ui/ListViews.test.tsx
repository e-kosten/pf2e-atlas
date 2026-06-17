import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import type {
  RecordDetailView,
  SavedListDetailView,
  SavedListIndexView,
} from "../generated/atlas";
import { AddToListButton, ListDetailView, ListIndexView } from "./ListViews";

const apiMocks = vi.hoisted(() => ({
  addSavedListItem: vi.fn(),
  createSavedList: vi.fn(),
  deleteSavedList: vi.fn(),
  getRecordDetail: vi.fn(),
  getSavedList: vi.fn(),
  getSavedLists: vi.fn(),
  removeSavedListItem: vi.fn(),
}));

vi.mock("../api/atlasApi", () => ({
  addSavedListItem: apiMocks.addSavedListItem,
  createSavedList: apiMocks.createSavedList,
  deleteSavedList: apiMocks.deleteSavedList,
  getRecordDetail: apiMocks.getRecordDetail,
  getSavedList: apiMocks.getSavedList,
  getSavedLists: apiMocks.getSavedLists,
  removeSavedListItem: apiMocks.removeSavedListItem,
}));

describe("list views", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    history.replaceState(null, "", "/lists");
    apiMocks.getSavedLists.mockResolvedValue(savedListIndexFixture());
    apiMocks.getSavedList.mockResolvedValue(savedListDetailFixture());
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
    apiMocks.createSavedList.mockResolvedValue({
      list: {
        list_key: "list_boss_fight_prep",
        slug: "boss-fight-prep",
        name: "Boss Fight Prep",
        description: "Session prep",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
    });
  });

  it("renders saved-list index rows and routes into a list", async () => {
    render(<ListIndexView route={{ kind: "lists" }} />, {
      wrapper: queryClientWrapper(),
    });

    const listLink = await screen.findByRole("link", { name: /Research/ });
    expect(screen.getByText("2 saved lists")).toBeInTheDocument();

    fireEvent.click(listLink);

    await waitFor(() => expect(window.location.pathname).toBe("/lists/research"));
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
    expect(
      screen.getByRole("button", { name: /Test Action 1.*actions:testAction1/ }),
    ).toBeInTheDocument();
    expect(screen.getByText("List")).toBeInTheDocument();
    expect(screen.getByText("Items")).toBeInTheDocument();
    expect(
      screen.queryByRole("columnheader", { name: "Status" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("Active")).not.toBeInTheDocument();
    expect(screen.queryByText("research")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Remove Test Action 1" }));

    await waitFor(() =>
      expect(apiMocks.removeSavedListItem).toHaveBeenCalledWith({
        list_ref: "research",
        record_ref: "actions:testAction1",
      }),
    );
  });

  it("deletes the current list and returns to the list index", async () => {
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

    await screen.findByRole("heading", { name: "Research" });
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() =>
      expect(apiMocks.deleteSavedList).toHaveBeenCalledWith("research"),
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

    await waitFor(() =>
      expect(window.location.pathname).toBe("/lists/encounters"),
    );
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
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-02T00:00:00Z",
      },
      {
        list_key: "list_encounters",
        slug: "encounters",
        name: "Encounters",
        description: "Fight prep",
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
        record: {
          record_key: "actions:testAction1",
          title: "Test Action 1",
          kind: "rule",
          kind_label: "Rule",
        },
      },
    ],
  };
}

function recordDetailFixture(recordKey: string): RecordDetailView {
  return {
    record_key: recordKey,
    title: "Test Action 1",
    kind: "rule",
    presentation: {
      record_key: recordKey,
      kind: "rule",
      title: "Test Action 1",
      identity: [],
      badges: [],
      sections: [],
    },
  };
}
