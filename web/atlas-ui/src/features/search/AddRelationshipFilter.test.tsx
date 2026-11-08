import { RouterTestContext } from "../../test/router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ConfigProvider, Grid } from "antd";
import {
  discoverFilterCounts,
  discoverFilterEditor,
  getRecordDetail,
  searchPage,
} from "../../api/atlasApi";
import type { SearchResultsRequest, QueryRelationship } from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  effectiveResultSort,
  searchStateQueryString,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import {
  detailFixture,
  editorFixture,
  fieldFixture,
  summaryFixture,
} from "../../test/fixtures";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { chooseAttributeSort, selectSortOption } from "../../test/sortControls";
import { FilterControls } from "../../shared/filters/FilterPanel";
import { LinkPickerContext } from "../../shared/filters/LinkPickerContext";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import { LinkConditionPicker } from "../../shared/filters/LinkConditionPicker";
import { linkClauses } from "../../shared/filters/linkPredicates";
import { linkLeaf } from "../../shared/filters/linkPredicates";
import { useSearchWorkspace, type SearchWorkspaceState } from "./useSearchWorkspace";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
}));
const selectedKey = "equipment:secret-record-id";
const existing: QueryRelationship = {
  direction: "outgoing",
  include_contents: false,
  selection: {
    ...rootNavigation("creatures:parent-id"),
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 4 } } }],
    field: "/description",
    source_fingerprint: "exact-current-source",
  },
};
function setup(initial: SearchFormState = DEFAULT_SEARCH_STATE) {
  const params = new URLSearchParams(searchStateQueryString(initial));
  params.set("page", "3");
  history.replaceState({}, "", "/search?" + params.toString());
  let live!: SearchWorkspaceState;
  function Harness() {
    live = useSearchWorkspace({ enabled: false, searchProfile: "bge-base" });
    return (
      <FilterControls
        filterState={{
          ...live,
          filterEditor: editorFixture([fieldFixture("level", "number", "Level")]),
        }}
      />
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <RouterTestContext>
          <SearchProfileProvider>
            <ReferenceHelpHost>
              <LinkPickerContext value={LinkConditionPicker}>
                <Harness />
              </LinkPickerContext>
            </ReferenceHelpHost>
          </SearchProfileProvider>
        </RouterTestContext>
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return () => live;
}
async function openAndSelect() {
  const trigger = screen.getByRole("button", { name: "Links" });
  act(() => trigger.focus());
  fireEvent.click(trigger);
  const dialog = await screen.findByRole("dialog", { name: "Add link condition" });
  fireEvent.click(await within(dialog).findByRole("link", { name: /Leather Armor/ }));
  await waitFor(() =>
    expect(within(dialog).getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  return dialog;
}
beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  vi.mocked(discoverFilterEditor).mockResolvedValue({
    ...editorFixture(),
    groups: [],
    common_field_ids: [],
  });
  vi.mocked(discoverFilterCounts).mockResolvedValue({
    search_profile: "bge-base",
    artifact_generation: "generation-a",
    counts: { status: "unavailable", reason: "non_conjunctive_context" },
  });
  vi.mocked(getRecordDetail).mockImplementation(async (key) => ({
    ...detailFixture(key, "Leather Armor"),
    artifact_generation: "generation-a",
  }));
  vi.mocked(searchPage).mockImplementation(async (request: SearchResultsRequest) => ({
    search_content: "all",
    artifact_generation: "generation-a",
    search_profile: "bge-base",
    ranking: "score",
    relationship_clauses: [],
    coverage: null,
    sort: effectiveResultSort({
      ...DEFAULT_SEARCH_STATE,
      query: request.criteria.kind === "text_search" ? request.criteria.query : "",
      sort: request.sort,
    }),
    mode:
      request.criteria.kind === "text_search"
        ? { kind: "text_search", query: request.criteria.query, mode: "hybrid" }
        : { kind: "list_records" },
    page: {
      number: request.page.number,
      size: request.page.size,
      count: 1,
      total: 1n,
      has_more: false,
      next_page: null,
    },
    rows: [
      {
        record: summaryFixture(selectedKey, "Leather Armor"),
        matches: [],
        relationship_evidence: [],
      },
    ],
  }));
});
it.each(["text", "similar"])(
  "looks up independently and ANDs a leaf without replacing parent %s, exact selections or Random order",
  async (mode) => {
    const current = setup({
      ...DEFAULT_SEARCH_STATE,
      query: mode === "text" ? "fire" : "",
      similarSeed: mode === "similar" ? "items:similar-parent" : null,
      filter: {
        kind: "all_of",
        children: [
          { kind: "compare", field: "level", op: "gte", value: 5 },
          linkLeaf(existing, "existing"),
        ],
      },
      sort: { kind: "random", seed: 42 },
    });
    const before = current().search;
    const dialog = await openAndSelect();
    expect(vi.mocked(searchPage).mock.calls[0][0].criteria).toEqual({
      kind: "list_records",
      filter: null,
    });
    expect(vi.mocked(searchPage).mock.calls[0][0].sort).toBeUndefined();
    expect(current().search).toEqual(before);
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await waitFor(() => expect(linkClauses(current().search.filter)).toHaveLength(2));
    expect(current().search).toMatchObject({
      ...before,
      filter: {
        kind: "all_of",
        children: [
          before.filter,
          {
            kind: "relationship",
            relationship: {
              direction: "incoming",
              include_contents: true,
              selection: rootNavigation(selectedKey),
            },
          },
        ],
      },
    });
    await waitFor(() => expect(current().pageNumber).toBe(1));
    expect(document.body).not.toHaveTextContent("secret-record-id");
  },
);
it("keeps parent paging and criteria untouched on cancel and resets lookup defaults", async () => {
  const current = setup({
    ...DEFAULT_SEARCH_STATE,
    query: "parent text",
    filter: linkLeaf(existing, "existing"),
  });
  const before = current().search;
  const dialog = await openAndSelect();
  const lookup = within(dialog).getByRole("searchbox", { name: "Search records" });
  fireEvent.change(lookup, { target: { value: "armor" } });
  fireEvent.keyDown(lookup, { key: "Enter", keyCode: 13 });
  await waitFor(() =>
    expect(vi.mocked(searchPage).mock.lastCall?.[0].criteria).toMatchObject({
      kind: "text_search",
      query: "armor",
      filter: null,
    }),
  );
  const results = within(dialog).queryByRole("button", { name: "Results" });
  if (results) fireEvent.click(results);
  await chooseAttributeSort("name", "desc");
  fireEvent.click(within(dialog).getByRole("radio", { name: "Linked from" }));
  await selectSortOption("Link filter scope", "Main entry only");
  fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  await waitFor(() => expect(dialog).not.toBeVisible());
  expect(current().search).toEqual(before);
  expect(current().pageNumber).toBe(3);
  const reopened = await openAndSelect();
  expect(
    within(reopened).getByRole("searchbox", { name: "Search records" }),
  ).toHaveValue("");
  expect(within(reopened).getByRole("radio", { name: "Links to" })).toBeChecked();
  expect(
    within(reopened).getByText("Whole entry", { exact: true }),
  ).toBeInTheDocument();
});
it.each([
  { change: "direction", direction: "outgoing", include_contents: true },
  { change: "scope", direction: "incoming", include_contents: false },
] as const)(
  "rejects a safe direct-AND duplicate but permits a different $change",
  async ({ change, direction, include_contents }) => {
    const duplicate = linkLeaf(
      {
        direction: "incoming",
        include_contents: true,
        selection: rootNavigation(selectedKey),
      },
      "already",
    );
    const current = setup({ ...DEFAULT_SEARCH_STATE, filter: duplicate });
    const before = current().search;
    const dialog = await openAndSelect();
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    expect(
      await within(dialog).findByText("This link condition is already added."),
    ).toBeInTheDocument();
    expect(current().search).toEqual(before);
    expect(current().pageNumber).toBe(3);
    if (change === "direction")
      fireEvent.click(within(dialog).getByRole("radio", { name: "Linked from" }));
    else await selectSortOption("Link filter scope", "Main entry only");
    expect(within(dialog).getByRole("button", { name: "Add" })).toBeDisabled();
    fireEvent.click(await within(dialog).findByRole("link", { name: /Leather Armor/ }));
    await waitFor(() =>
      expect(within(dialog).getByRole("button", { name: "Add" })).toBeEnabled(),
    );
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await waitFor(() => expect(linkClauses(current().search.filter)).toHaveLength(2));
    expect(
      linkClauses(current().search.filter).map((leaf) => leaf.relationship),
    ).toEqual([
      {
        direction: "incoming",
        include_contents: true,
        selection: rootNavigation(selectedKey),
      },
      { direction, include_contents, selection: rootNavigation(selectedKey) },
    ]);
  },
);
it("keeps scope explanation optional and Escape closes inner help only", async () => {
  setup();
  const dialog = await openAndSelect();
  const explanation =
    "Also check links in each result’s contents, such as its abilities, spells, and equipment.";
  expect(screen.queryByText(explanation)).toBeNull();
  const help = within(dialog).getByRole("button", { name: "About link scope" });
  fireEvent.click(help);
  expect(await screen.findByText(explanation)).toBeInTheDocument();
  fireEvent.click(within(dialog).getByRole("radio", { name: "Linked from" }));
  expect(
    await screen.findByText("finds records that your selection links to."),
  ).toBeInTheDocument();
  expect(
    screen.getByText(
      "Also check links in the selected entry’s contents, such as its abilities, spells, and equipment.",
    ),
  ).toBeInTheDocument();
  fireEvent.keyDown(help, { key: "Escape", keyCode: 27 });
  expect(help).toHaveAttribute("aria-expanded", "false");
  expect(help).toHaveFocus();
  expect(dialog).toBeInTheDocument();
});

it.each([true, false])(
  "clears an existing link target without restoring it or applying a parent filter change (wide=%s)",
  async (wide) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: wide });
    const current = setup({
      ...DEFAULT_SEARCH_STATE,
      filter: linkLeaf(existing, "existing"),
    });
    const before = current().search;
    fireEvent.click(screen.getByRole("button", { name: /^Linked from/ }));
    const dialog = await screen.findByRole("dialog", { name: "Edit link condition" });
    fireEvent.click(
      await within(dialog).findByRole("button", { name: "Clear selection" }),
    );
    expect(within(dialog).getByRole("button", { name: "Apply" })).toBeDisabled();
    expect(
      within(dialog).queryByRole("button", { name: "Clear selection" }),
    ).toBeNull();
    expect(current().search).toEqual(before);
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(current().search).toEqual(before);
  },
);

it("edits an exact embedded selection without rebinding it until a parent row is explicitly selected", async () => {
  const lookup = vi.mocked(searchPage).getMockImplementation()!;
  vi.mocked(searchPage).mockImplementation(async (...args) => {
    const page = await lookup(...args);
    return {
      ...page,
      rows: page.rows.map((row) => ({
        ...row,
        record: summaryFixture(existing.selection.record_key, "Leather Armor"),
      })),
    };
  });
  const current = setup({
    ...DEFAULT_SEARCH_STATE,
    filter: linkLeaf(existing, "embedded"),
  });
  fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
  const drawer = screen.getByRole("dialog", { name: "Advanced filters" });
  fireEvent.click(within(drawer).getByRole("button", { name: "Edit related entries" }));
  fireEvent.click(screen.getByRole("button", { name: "Choose record" }));
  let dialog = await screen.findByRole("dialog", { name: "Edit link condition" });
  await waitFor(() =>
    expect(within(dialog).getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Apply" }));
  await waitFor(() => expect(dialog).not.toBeVisible());
  expect(linkClauses(current().search.filter)[0].relationship).toEqual(existing);
  fireEvent.click(within(drawer).getByRole("button", { name: "Edit related entries" }));
  fireEvent.click(screen.getByRole("button", { name: "Choose record" }));
  dialog = await screen.findByRole("dialog", { name: "Edit link condition" });
  fireEvent.click(within(dialog).getByRole("button", { name: "Clear selection" }));
  fireEvent.click(await within(dialog).findByRole("link", { name: /Leather Armor/ }));
  await within(dialog).findByRole("heading", { name: "Leather Armor" });
  await waitFor(() =>
    expect(within(dialog).getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(linkClauses(current().search.filter)[0].relationship.selection).toEqual(
      rootNavigation(existing.selection.record_key),
    ),
  );
  expect(drawer).toBeVisible();
});

it("composes a nested picker through the same provider and Escape preserves the outer lookup", async () => {
  const current = setup();
  const outer = await openAndSelect();
  fireEvent.click(within(outer).getByRole("button", { name: "Advanced filters" }));
  const advanced = screen.getByRole("dialog", { name: "Advanced filters" });
  await selectSortOption("Add condition", "Related entries");
  const configure = within(advanced).getByRole("button", {
    name: "Configure related entries",
  });
  fireEvent.click(configure);
  await selectSortOption("Relationship", "Links to");
  await selectSortOption("Match related entry", "Choose a record");
  fireEvent.click(screen.getByRole("button", { name: "Choose record" }));
  const dialogs = await screen.findAllByRole("dialog", { name: "Add link condition" });
  const inner = dialogs[dialogs.length - 1];
  expect(inner).not.toBe(outer);
  fireEvent.keyDown(inner, { key: "Escape", keyCode: 27 });
  await waitFor(() => expect(inner).not.toBeVisible());
  expect(outer).toBeVisible();
  expect(advanced).toBeVisible();
  await waitFor(() => expect(configure).toHaveFocus());
  expect(current().search.filter).toBeNull();
});

it("requires a new exact selection when scope changes in the compact editor", async () => {
  const current = setup({
    ...DEFAULT_SEARCH_STATE,
    filter: linkLeaf(existing, "existing"),
  });
  fireEvent.click(screen.getByRole("button", { name: "Advanced filters" }));
  fireEvent.click(screen.getByRole("button", { name: "Edit related entries" }));
  await screen.findByRole("dialog", { name: "Related entries" });
  await selectSortOption(
    existing.direction === "outgoing"
      ? "Check links in the source entries"
      : "Check links in results",
    existing.include_contents ? "Main entry only" : "Whole entry",
  );
  fireEvent.click(screen.getByRole("button", { name: "Choose record" }));
  const picker = await screen.findByRole("dialog", { name: "Add link condition" });
  expect(within(picker).getByRole("button", { name: "Add" })).toBeDisabled();
  expect(linkClauses(current().search.filter)[0].relationship).toEqual(existing);
});

it("rejects a newly selected record from a different reference generation without changing the parent", async () => {
  const lookup = vi.mocked(searchPage).getMockImplementation()!;
  vi.mocked(searchPage).mockImplementation(async (...args) => ({
    ...(await lookup(...args)),
    artifact_generation: "generation-b",
  }));
  vi.mocked(getRecordDetail).mockImplementation(async (key) => ({
    ...detailFixture(key, "Leather Armor"),
    artifact_generation: "generation-b",
  }));
  const current = setup();
  const dialog = await openAndSelect();
  fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
  expect(
    await within(dialog).findByText(
      "The reference collection changed. Refresh results before changing this condition.",
    ),
  ).toBeInTheDocument();
  expect(current().search.filter).toBeNull();
  expect(current().pageNumber).toBe(3);
});

it("uses the selected preview as identity and retains the revised direction and scope", async () => {
  const current = setup();
  const picker = await openAndSelect();
  fireEvent.click(within(picker).getByRole("radio", { name: "Linked from" }));
  await selectSortOption("Link filter scope", "Main entry only");
  expect(within(picker).getByRole("button", { name: "Add" })).toBeDisabled();
  fireEvent.click(await within(picker).findByRole("link", { name: /Leather Armor/ }));
  await within(picker).findByRole("heading", { name: "Leather Armor" });
  await waitFor(() =>
    expect(within(picker).getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  expect(picker.querySelector(".reference-picker__footer a")).toBeNull();
  expect(location.pathname).toBe("/search");
  expect(picker).toBeInTheDocument();
  expect(within(picker).getByRole("radio", { name: "Linked from" })).toBeChecked();
  expect(within(picker).getByText("Main entry only", { exact: true })).toBeVisible();
  fireEvent.click(within(picker).getByRole("button", { name: "Add" }));
  await waitFor(() => expect(linkClauses(current().search.filter)).toHaveLength(1));
  expect(linkClauses(current().search.filter)[0].relationship).toEqual({
    direction: "outgoing",
    include_contents: false,
    selection: rootNavigation(selectedKey),
  });
});

it("excludes an existing link in the modal without rebinding the exact target", async () => {
  const current = setup({
    ...DEFAULT_SEARCH_STATE,
    filter: linkLeaf(existing, "existing"),
  });
  const [original] = linkClauses(current().search.filter);
  fireEvent.click(screen.getByRole("button", { name: /^Linked from/ }));
  const dialog = await screen.findByRole("dialog", { name: "Edit link condition" });
  fireEvent.click(
    within(dialog).getByRole("checkbox", { name: "Exclude matching records" }),
  );
  const apply = within(dialog).getByRole("button", { name: "Apply" });
  await waitFor(() => expect(apply).toBeEnabled());
  fireEvent.click(apply);
  await waitFor(() =>
    expect(current().search.filter).toMatchObject({
      kind: "not",
      predicate: original,
    }),
  );
});
