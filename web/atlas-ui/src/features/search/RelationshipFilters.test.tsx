import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { RelationshipFilters } from "./RelationshipFilters";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { summaryFixture } from "../../test/fixtures";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import type { RelationshipConstraintView } from "../../generated/atlas";

const constraint: RelationshipConstraintView = {
  direction: "incoming",
  selection: {
    record_key: "equipment:secret-id",
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: "checked-source",
  },
  include_contents: true,
};
function workspace() {
  return {
    search: { ...DEFAULT_SEARCH_STATE, relationships: [constraint], query: "defense" },
    setSearch: vi.fn(),
    resultPage: {
      relationships: [
        {
          constraint,
          title: "Leather Armor",
          record: summaryFixture(constraint.selection.record_key, "Leather Armor"),
        },
      ],
    },
    errorMessage: null,
  } as unknown as SearchWorkspaceState;
}

it("shows the human relationship label at the top level and preserves criteria on scope changes/removal", async () => {
  const state = workspace();
  const view = render(<RelationshipFilters workspace={state} />);
  expect(screen.getByRole("group", { name: "Relationship filter" })).toBeVisible();
  expect(screen.getByRole("link", { name: "Leather Armor" })).toBeVisible();
  expect(view.container).not.toHaveTextContent("secret-id");
  fireEvent.click(screen.getByRole("button", { name: "Relationship scope" }));
  await waitFor(() =>
    expect(
      screen.getByText(
        "Whole entry also checks each matching record’s abilities, spells, equipment, and other nested documents.",
      ),
    ).toBeInTheDocument(),
  );
  fireEvent.click(screen.getByRole("radio", { name: "Main entry only" }));
  expect(state.setSearch).toHaveBeenCalledWith({
    ...state.search,
    relationships: [{ ...constraint, include_contents: false }],
  });
  expect(screen.getByRole("button", { name: "Relationship scope" })).toHaveFocus();
  fireEvent.click(screen.getByRole("button", { name: "Remove relationship filter" }));
  expect(state.setSearch).toHaveBeenLastCalledWith({
    ...state.search,
    relationships: [],
  });
});

it("keeps unavailable targets readable without exposing record keys", () => {
  const state = workspace();
  const view = render(
    <RelationshipFilters
      workspace={{ ...state, resultPage: undefined, errorMessage: "unavailable" }}
    />,
  );
  expect(screen.getByText("Record unavailable")).toBeVisible();
  expect(view.container).not.toHaveTextContent(constraint.selection.record_key);
  expect(
    screen.getByRole("button", { name: "Remove relationship filter" }),
  ).toBeEnabled();
});
