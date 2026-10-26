import { fireEvent, render, screen, within } from "@testing-library/react";
import type { SearchResultRow } from "../../generated/atlas";
import { summaryFixture, activityFixture } from "../../test/fixtures";
import { SearchMatchContext } from "./SearchMatchContext";
import { ReaderMatchNavigation } from "./ReaderMatchNavigation";

const navigation = activityFixture("Breath Fire", "actors:firewyrm").navigation;
const row: SearchResultRow = {
  record: summaryFixture("actors:firewyrm", "Firewyrm"),
  matches: [],
  relationship_evidence: [],
  matching_sections: [{ navigation, label: "Breath Fire", snippet: "A cone of fire." }],
};
function renderMatches(
  props: Parameters<typeof SearchMatchContext>[0],
  onNavigate = vi.fn(),
) {
  return {
    ...render(
      <ReaderMatchNavigation.Provider value={onNavigate}>
        <SearchMatchContext {...props} />
      </ReaderMatchNavigation.Provider>,
    ),
    onNavigate,
  };
}

it("shows named locations and excerpts with exact native destinations and deliberate primary navigation", () => {
  const { onNavigate } = renderMatches({ row, query: "fire" });
  const context = screen.getByRole("region", { name: "Search matches" });
  expect(
    within(context).getByRole("heading", { name: "Search matches for “fire”" }),
  ).toBeVisible();
  expect(within(context).getByText("A cone of fire.")).toBeVisible();
  const link = within(context).getByRole("link", { name: "Breath Fire" });
  expect(
    JSON.parse(
      new URL(link.getAttribute("href")!, "http://localhost").searchParams.get(
        "selection",
      )!,
    ),
  ).toEqual(navigation);
  fireEvent.click(link, { metaKey: true });
  expect(onNavigate).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(onNavigate).toHaveBeenCalledExactlyOnceWith(navigation, link);
});

it.each([undefined, "", "  "])(
  "omits query context without an active query (%s)",
  (query) => {
    const { container } = renderMatches({ row, query });
    expect(container).toBeEmptyDOMElement();
  },
);

it("omits a search section when the server has no meaningful locations", () => {
  const { container } = renderMatches({
    row: { ...row, matching_sections: undefined },
    query: "fire",
  });
  expect(container).toBeEmptyDOMElement();
});
