import { fireEvent, render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { ListInfoPane } from "./ListInfoPane";
import { ListItemsPane } from "./ListItemsPane";

vi.mock("../../shared/filters/FilterPanel", () => ({
  FilterControls: () => <div>List filters</div>,
}));

const infoProps: ComponentProps<typeof ListInfoPane> = {
  currentSlug: "research",
  filterState: {
    search: DEFAULT_SEARCH_STATE,
    setSearch: vi.fn(),
    filterEditor: undefined,
  },
  list: undefined,
  lists: [],
  listsLoading: false,
  loading: false,
  onSelectList: vi.fn(),
  onRetry: vi.fn(),
};

describe("saved list local failure states", () => {
  it("shows a query failure and matching retry instead of falsely claiming not found", () => {
    const retry = vi.fn();
    render(
      <ListInfoPane
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
      <ListInfoPane
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
    expect(screen.getByRole("table")).toBeInTheDocument();
  });
});
