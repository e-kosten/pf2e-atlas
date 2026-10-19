import { fireEvent, render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { ListBrowseHeader } from "./ListBrowseHeader";
import { ListItemsPane } from "./ListItemsPane";

vi.mock("../../shared/filters/FilterPanel", () => ({
  FilterControls: () => <div>List filters</div>,
}));

const infoProps: ComponentProps<typeof ListBrowseHeader> = {
  filterState: {
    search: DEFAULT_SEARCH_STATE,
    setSearch: vi.fn(),
    filterEditor: undefined,
  },
  list: undefined,
  loading: false,
  onAddRecords: vi.fn(),
  onRetry: vi.fn(),
};

describe("saved list local failure states", () => {
  it("shows a query failure and matching retry instead of falsely claiming not found", () => {
    const retry = vi.fn();
    render(
      <ListBrowseHeader
        {...infoProps}
        error={new Error("List request failed")}
        onRetry={retry}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Unable to load this list");
    expect(screen.queryByText("List not found")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry list" }));
    expect(retry).toHaveBeenCalledOnce();
  });

  it("keeps a previously loaded list and its filters next to a refresh failure", () => {
    render(
      <ListBrowseHeader
        {...infoProps}
        error={new Error("Refresh failed")}
        list={{
          list_key: "list_research",
          slug: "research",
          name: "Research",
          tags: [],
          item_count: 0,
          created_at: "2026-10-10",
          updated_at: "2026-10-10",
        }}
      />,
    );
    expect(screen.getByRole("heading", { name: "Research" })).toBeInTheDocument();
    expect(screen.getByText("List filters")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Unable to refresh this list");
  });

  it("shows separate description and tag metadata without a combined disclosure", async () => {
    render(
      <ListBrowseHeader
        {...infoProps}
        list={{
          list_key: "long-list",
          slug: "long-list",
          name: "Long list",
          description: "Detailed preparation notes. ".repeat(30),
          tags: Array.from({ length: 20 }, (_, index) => `Campaign tag ${index}`),
          item_count: 120,
          created_at: "2026-10-10",
          updated_at: "2026-10-10",
        }}
      />,
    );
    expect(screen.getByRole("heading", { name: "Long list" })).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Description and tags" }),
    ).not.toBeInTheDocument();
    expect(screen.getByLabelText("List tags")).toHaveTextContent("Campaign tag 19");
    expect(
      screen.getByText("Detailed preparation notes. ".repeat(30).trim()),
    ).toBeVisible();
  });

  it("keeps a removal failure in the items pane without borrowing detail or query retry", () => {
    render(
      <ListItemsPane
        error={new Error("Removal failed")}
        items={[]}
        loading={false}
        removingKey={null}
        selectedRecordKey={null}
        onRemove={vi.fn()}
        onSelect={vi.fn()}
      />,
    );
    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Unable to remove record");
    expect(alert).toHaveTextContent("Use the record’s Remove button to try again");
    expect(screen.queryByRole("button", { name: /Retry/ })).not.toBeInTheDocument();
    expect(screen.getByRole("region", { name: "List items" })).toBeInTheDocument();
  });
});
