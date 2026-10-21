import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { RelationshipFilters } from "./RelationshipFilters";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import { summaryFixture } from "../../test/fixtures";
import type { SearchWorkspaceState } from "./useSearchWorkspace";
import type { RelationshipConstraintView } from "../../generated/atlas";
import * as routes from "../../app/routes";

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
function workspace(relationship = constraint) {
  return {
    search: {
      ...DEFAULT_SEARCH_STATE,
      relationships: [relationship],
      query: "defense",
    },
    setSearch: vi.fn(),
    resultPage: {
      relationships: [
        {
          constraint: relationship,
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
  const chip = screen.getByRole("button", { name: /Links to Leather Armor/ });
  expect(chip).toHaveTextContent("Links to Leather Armor");
  expect(screen.queryByRole("link", { name: /Leather Armor/ })).toBeNull();
  expect(view.container).not.toHaveTextContent("secret-id");
  fireEvent.click(chip);
  await waitFor(() =>
    expect(
      screen.getByRole("link", { name: "Open Leather Armor" }),
    ).toBeInTheDocument(),
  );
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
  expect(chip).toHaveFocus();
  fireEvent.click(screen.getByRole("button", { name: "Remove relationship filter" }));
  expect(state.setSearch).toHaveBeenLastCalledWith({
    ...state.search,
    relationships: [],
  });
});

it("opens the exact selected record from the combined filter popover", async () => {
  const relationship: RelationshipConstraintView = {
    ...constraint,
    selection: {
      ...constraint.selection,
      embedded_path: [
        { collection: "items", identity: { SnapshotLocal: { index: 0 } } },
      ],
      field: "/system/description/value",
    },
  };
  const state = workspace(relationship);
  const navigate = vi
    .spyOn(routes, "navigateToAtlasRoute")
    .mockImplementation(() => {});
  try {
    render(<RelationshipFilters workspace={state} />);
    fireEvent.click(screen.getByRole("button", { name: /Links to Leather Armor/ }));
    const link = await screen.findByRole("link", { name: "Open Leather Armor" });
    const route = {
      kind: "record",
      recordKey: constraint.selection.record_key,
      selection: relationship.selection,
    } as const;
    expect(link).toHaveAttribute("href", routes.atlasRoutePath(route));
    link.focus();
    fireEvent.keyDown(link, { key: "Escape" });
    expect(
      screen.getByRole("button", { name: /Links to Leather Armor/ }),
    ).toHaveFocus();
    fireEvent.click(screen.getByRole("button", { name: /Links to Leather Armor/ }));
    fireEvent.click(await screen.findByRole("link", { name: "Open Leather Armor" }));
    expect(navigate).toHaveBeenCalledWith(route);
  } finally {
    navigate.mockRestore();
  }
});

it("keeps unavailable targets readable without exposing record keys", () => {
  const state = workspace();
  const view = render(
    <RelationshipFilters
      workspace={{ ...state, resultPage: undefined, errorMessage: "unavailable" }}
    />,
  );
  expect(
    screen.getByRole("button", { name: /Links to Record unavailable/ }),
  ).toBeVisible();
  expect(view.container).not.toHaveTextContent(constraint.selection.record_key);
  expect(
    screen.getByRole("button", { name: "Remove relationship filter" }),
  ).toBeEnabled();
});
