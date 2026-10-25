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
import { ConfigProvider } from "antd";
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
import { LinkConditionPicker } from "../../shared/filters/LinkConditionPicker";
import { linkClauses } from "../../shared/filters/LinkFilterControls";
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
  history.replaceState(
    { atlasSearchPage: 3 },
    "",
    "/search" + searchStateQueryString(initial),
  );
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
            <LinkPickerContext value={LinkConditionPicker}>
              <Harness />
            </LinkPickerContext>
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
  vi.mocked(getRecordDetail).mockImplementation(async (key) =>
    detailFixture(key, "Leather Armor"),
  );
  vi.mocked(searchPage).mockImplementation(async (request: SearchResultsRequest) => ({
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
it.each(["direction", "scope"])(
  "rejects a safe direct-AND duplicate but permits a different %s",
  async (change) => {
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
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await waitFor(() => expect(linkClauses(current().search.filter)).toHaveLength(2));
  },
);
it("keeps scope explanation optional and Escape closes inner help only", async () => {
  setup();
  const dialog = await openAndSelect();
  const explanation =
    "Whole entry also checks each matching record’s abilities, spells, equipment, and other nested documents.";
  expect(screen.queryByText(explanation)).toBeNull();
  const help = within(dialog).getByRole("button", { name: "About link scope" });
  fireEvent.click(help);
  expect(await screen.findByText(explanation)).toBeInTheDocument();
  fireEvent.keyDown(help, { key: "Escape", keyCode: 27 });
  expect(help).toHaveAttribute("aria-expanded", "false");
  expect(help).toHaveFocus();
  expect(dialog).toBeInTheDocument();
});

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
  fireEvent.click(
    within(drawer).getByRole("button", { name: /Loading record… · Main entry only/ }),
  );
  let dialog = await screen.findByRole("dialog", { name: "Edit link condition" });
  fireEvent.click(within(dialog).getByRole("radio", { name: "Links to" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Apply" }));
  await waitFor(() => expect(dialog).not.toBeVisible());
  expect(linkClauses(current().search.filter)[0].relationship).toEqual({
    ...existing,
    direction: "incoming",
  });
  fireEvent.click(
    within(drawer).getByRole("button", { name: /Loading record… · Main entry only/ }),
  );
  dialog = await screen.findByRole("dialog", { name: "Edit link condition" });
  fireEvent.click(await within(dialog).findByRole("link", { name: /Leather Armor/ }));
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
  fireEvent.click(within(advanced).getByRole("button", { name: "Add condition" }));
  await selectSortOption("Filter field", "Record links");
  const chooseRecord = within(advanced).getByRole("button", { name: "Choose record" });
  act(() => chooseRecord.focus());
  fireEvent.click(chooseRecord);
  const dialogs = await screen.findAllByRole("dialog", { name: "Add link condition" });
  const inner = dialogs[dialogs.length - 1];
  expect(inner).not.toBe(outer);
  fireEvent.keyDown(inner, { key: "Escape", keyCode: 27 });
  await waitFor(() => expect(inner).not.toBeVisible());
  expect(outer).toBeVisible();
  expect(advanced).toBeVisible();
  await waitFor(() => expect(chooseRecord).toHaveFocus());
  expect(current().search.filter).toBeNull();
});

it("rejects a newly selected record from a different reference generation without changing the parent", async () => {
  const lookup = vi.mocked(searchPage).getMockImplementation()!;
  vi.mocked(searchPage).mockImplementation(async (...args) => ({
    ...(await lookup(...args)),
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
