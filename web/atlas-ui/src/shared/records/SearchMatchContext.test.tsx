import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import { fireEvent, render, screen, within } from "@testing-library/react";
import type { SearchResultRow } from "../../generated/atlas";
import {
  summaryFixture,
  activityFixture,
  similarityEvidenceFixture,
} from "../../test/fixtures";
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
    selectedNavigation(
      parseSearchParams(new URL(link.getAttribute("href")!, "http://localhost").search),
      navigation.record_key,
    ),
  ).toEqual(navigation);
  fireEvent.click(link, { metaKey: true });
  expect(onNavigate).not.toHaveBeenCalled();
  fireEvent.click(link);
  expect(onNavigate).toHaveBeenCalledExactlyOnceWith(navigation, link);
});

it.each([undefined, "", "  "])(
  "shows server-provided matching sections without fabricating a query label (%s)",
  (query) => {
    renderMatches({ row, query });
    const context = screen.getByRole("region", { name: "Search matches" });
    expect(
      within(context).getByRole("heading", { name: "Search matches" }),
    ).toBeVisible();
    expect(within(context).getByRole("link", { name: "Breath Fire" })).toBeVisible();
  },
);

it("omits a search section when the server has no meaningful locations", () => {
  const { container } = renderMatches({
    row: { ...row, matching_sections: undefined },
    query: "fire",
  });
  expect(container).toBeEmptyDOMElement();
});

it("shows similarity coverage, contributions, caveats and both exact passage destinations without a text query", () => {
  const seed = {
    ...navigation,
    record_key: "hazards:brown-mold",
    embedded_path: [],
    field: "/system/description/value",
    source_fingerprint: "seed-source",
  };
  const similarity = similarityEvidenceFixture(navigation, seed);
  const { onNavigate } = renderMatches({
    row: { ...row, similarity },
    showSimilarityDetails: true,
  });
  const context = screen.getByRole("region", { name: "Similarity comparison" });
  expect(
    within(context).getByText("1 of 2 selections have comparable content."),
  ).toBeVisible();
  expect(within(context).getByRole("table")).toHaveTextContent("Description");
  expect(within(context).getByRole("table")).toHaveTextContent(
    "No corresponding content",
  );
  expect(
    within(context).getByText(/Showing part of the comparison evidence/),
  ).toBeVisible();
  expect(
    within(context).getByText(/the score compares averaged vectors/),
  ).toBeVisible();
  expect(within(context).getByText("Source excerpt")).toBeVisible();
  expect(within(context).getByText("Matching excerpt")).toBeVisible();
  for (const [label, target] of [
    ["Seed description", seed],
    ["Candidate description", navigation],
  ] as const) {
    const link = within(context).getByRole("link", { name: label });
    expect(
      JSON.parse(
        new URL(link.getAttribute("href")!, "http://localhost").searchParams.get(
          "selection",
        )!,
      ),
    ).toEqual(target);
    fireEvent.click(link, { metaKey: true });
    expect(onNavigate).not.toHaveBeenCalled();
  }
  const candidate = within(context).getByRole("link", {
    name: "Candidate description",
  });
  fireEvent.click(candidate);
  expect(onNavigate).toHaveBeenCalledExactlyOnceWith(navigation, candidate);
});

it("leaves trait seed excerpts without fabricated record links and native candidates work without a reader host", () => {
  render(
    <SearchMatchContext
      showSimilarityDetails
      row={{ ...row, similarity: similarityEvidenceFixture(navigation, null) }}
    />,
  );
  const context = screen.getByRole("region", { name: "Similarity comparison" });
  expect(within(context).getByText("Source excerpt")).toBeVisible();
  expect(within(context).queryByRole("link", { name: "Seed description" })).toBeNull();
  const candidate = within(context).getByRole("link", {
    name: "Candidate description",
  });
  expect(fireEvent.click(candidate, { metaKey: true })).toBe(true);
});

it("hides similarity details by default while preserving ordinary text match context", () => {
  const props = {
    row: { ...row, similarity: similarityEvidenceFixture(navigation, null) },
    query: "fire",
  };
  const view = render(<SearchMatchContext {...props} />);
  expect(screen.queryByRole("region", { name: "Similarity comparison" })).toBeNull();
  expect(screen.getByRole("region", { name: "Search matches" })).toBeVisible();
  view.rerender(<SearchMatchContext {...props} showSimilarityDetails />);
  expect(screen.getByRole("region", { name: "Similarity comparison" })).toBeVisible();
  view.rerender(<SearchMatchContext {...props} showSimilarityDetails={false} />);
  expect(screen.queryByRole("region", { name: "Similarity comparison" })).toBeNull();
  expect(screen.getByRole("region", { name: "Search matches" })).toBeVisible();
});

it("switches Similar results between compact matches and details without hiding both", () => {
  const props = {
    row: { ...row, similarity: similarityEvidenceFixture(navigation, null) },
  };
  const view = render(<SearchMatchContext {...props} />);
  let context = screen.getByRole("region", { name: "Search matches" });
  const href = within(context)
    .getByRole("link", { name: "Breath Fire" })
    .getAttribute("href");
  expect(within(context).getByText("A cone of fire.")).toBeVisible();
  expect(screen.queryByRole("region", { name: "Similarity comparison" })).toBeNull();
  expect(screen.queryByRole("table")).toBeNull();

  view.rerender(<SearchMatchContext {...props} showSimilarityDetails />);
  expect(screen.queryByRole("region", { name: "Search matches" })).toBeNull();
  context = screen.getByRole("region", { name: "Similarity comparison" });
  expect(within(context).getByRole("table")).toBeVisible();

  view.rerender(<SearchMatchContext {...props} showSimilarityDetails={false} />);
  context = screen.getByRole("region", { name: "Search matches" });
  expect(within(context).getByRole("link", { name: "Breath Fire" })).toHaveAttribute(
    "href",
    href,
  );
  expect(within(context).getByText("A cone of fire.")).toBeVisible();
  expect(screen.queryByRole("region", { name: "Similarity comparison" })).toBeNull();
});

it.each(["parts", "roles"] as const)(
  "collapses only %s that round to 0.0 percent and restores them without changing evidence",
  (kind) => {
    const similarity = similarityEvidenceFixture(navigation, null);
    const weights = [0.99901, 0.00049, 0.0005];
    const labels = ["Largest", "Smaller", "Visible boundary"];
    const roles = weights.map((weight, index) => ({
      key: { kind: "role" as const, role: "description" as const },
      role_label: labels[index],
      weight,
      score: 0.8,
      comparable: true,
    }));
    if (kind === "roles")
      similarity.roles = roles.map((role, index) => ({
        ...role,
        key: {
          kind: "role",
          role: (["description", "ability", "reset"] as const)[index],
        },
      }));
    else {
      similarity.roles = [];
      similarity.parts = roles.map((role, index) => ({
        ...role,
        part: {
          kind: "section",
          selector: `part-${index}`,
          label: labels[index],
          owner_label: "Source",
          owners: [],
          field: "/description",
          section_ordinal: index,
          role: "description",
          role_label: "Description",
          has_vectors: true,
          label_only: false,
          navigation,
        },
      }));
    }
    const original = JSON.stringify(similarity);
    const props = { row: { ...row, similarity }, showSimilarityDetails: true };
    const view = render(<SearchMatchContext {...props} />);
    expect(within(screen.getByRole("table")).getAllByRole("row")).toHaveLength(3);
    expect(screen.getByText("99.9%")).toBeVisible();
    expect(screen.getByText("0.1%")).toBeVisible();
    expect(screen.queryByText("0.0%")).toBeNull();
    expect(screen.getByText("1 smaller weight hidden.")).toBeVisible();
    expect(
      screen.getByText("1 of 2 selections have comparable content."),
    ).toBeVisible();
    view.rerender(<SearchMatchContext {...props} showSmallerWeights />);
    expect(within(screen.getByRole("table")).getAllByRole("row")).toHaveLength(4);
    expect(screen.getByText("0.0%")).toBeVisible();
    expect(screen.queryByText("1 smaller weight hidden.")).toBeNull();
    expect(JSON.stringify(similarity)).toBe(original);
    view.rerender(<SearchMatchContext {...props} />);
    expect(screen.queryByText("0.0%")).toBeNull();
  },
);
