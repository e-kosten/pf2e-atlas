import { fireEvent, render, screen } from "@testing-library/react";
import { DEFAULT_SEARCH_STATE } from "../filters/searchState";
import { ResultSortSelect } from "./ResultSortSelect";

it("records explicit intent when the displayed automatic order is selected", () => {
  const setSearch = vi.fn();
  render(<ResultSortSelect search={DEFAULT_SEARCH_STATE} setSearch={setSearch} />);
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Result order" }));
  fireEvent.click(screen.getByRole("option", { name: "Name A–Z" }));
  expect(setSearch).toHaveBeenCalledWith({ ...DEFAULT_SEARCH_STATE, sort: "name_asc" });
});

it("offers Relevance for semantic criteria and Saved order for list filtering", () => {
  const { rerender } = render(
    <ResultSortSelect
      search={{ ...DEFAULT_SEARCH_STATE, query: "fire" }}
      setSearch={vi.fn()}
    />,
  );
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Result order" }));
  expect(screen.getByRole("option", { name: "Relevance" })).toBeInTheDocument();
  expect(screen.queryByRole("option", { name: "Saved order" })).toBeNull();
  rerender(
    <ResultSortSelect
      search={{ ...DEFAULT_SEARCH_STATE, query: "fire" }}
      surface="saved_list"
      setSearch={vi.fn()}
    />,
  );
  expect(screen.getByRole("option", { name: "Saved order" })).toBeInTheDocument();
  expect(screen.queryByRole("option", { name: "Relevance" })).toBeNull();
});

it("keeps the selected numeric order compact while the menu states its full direction", () => {
  render(
    <ResultSortSelect
      search={{ ...DEFAULT_SEARCH_STATE, sort: "level_desc" }}
      setSearch={vi.fn()}
    />,
  );
  expect(screen.getByText("Level / Rank: High")).toBeVisible();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Result order" }));
  expect(
    screen.getByRole("option", { name: "Level / Rank high to low" }),
  ).toBeInTheDocument();
});
