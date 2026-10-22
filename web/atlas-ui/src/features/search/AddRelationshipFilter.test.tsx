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
import type { SearchResultsRequest } from "../../generated/atlas";
import {
  DEFAULT_SEARCH_STATE,
  effectiveResultSort,
  searchStateQueryString,
  type SearchFormState,
} from "../../shared/filters/searchState";
import {
  decodeRelationships,
  MAX_RELATIONSHIP_FILTERS,
} from "../../shared/filters/relationshipUrl";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { detailFixture, editorFixture, summaryFixture } from "../../test/fixtures";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { chooseAttributeSort, selectSortOption } from "../../test/sortControls";
import { AddRelationshipFilter } from "./AddRelationshipFilter";
import { useSearchWorkspace, type SearchWorkspaceState } from "./useSearchWorkspace";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
}));
const selectedKey = "equipment:secret-record-id";
const existing = {
  direction: "outgoing" as const,
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
    return <AddRelationshipFilter workspace={live} />;
  }
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <SearchProfileProvider>
          <Harness />
        </SearchProfileProvider>
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return () => live;
}
async function openAndSelect() {
  const trigger = screen.getByRole("button", { name: "Add link filter" });
  act(() => trigger.focus());
  fireEvent.click(trigger);
  const dialog = await screen.findByRole("dialog", { name: "Add link filter" });
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
    relationships: [],
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
  "looks up independently and appends without replacing parent %s, filters, exact selections or Random order",
  async (mode) => {
    const current = setup({
      ...DEFAULT_SEARCH_STATE,
      query: mode === "text" ? "fire" : "",
      similarSeed: mode === "similar" ? "items:similar-parent" : null,
      filter: { kind: "compare", field: "level", op: "gte", value: 5 },
      relationships: [existing],
      sort: { kind: "random", seed: 42 },
    });
    const before = current().search;
    expect(current().pageNumber).toBe(3);
    const dialog = await openAndSelect();
    const request = vi.mocked(searchPage).mock.calls[0][0];
    expect(request.criteria).toEqual({ kind: "list_records", filter: null });
    expect(request.relationships).toBeUndefined();
    expect(request.sort).toBeUndefined();
    expect(request.page.number).toBe(1);
    expect(within(dialog).getByRole("radio", { name: "Links to" })).toBeChecked();
    expect(
      within(dialog).getByText("Whole entry", { exact: true }),
    ).toBeInTheDocument();
    expect(current().search).toEqual(before);
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await waitFor(() => expect(current().search.relationships).toHaveLength(2));
    expect(current().search).toEqual({
      ...before,
      relationships: [
        existing,
        {
          direction: "incoming",
          include_contents: true,
          selection: rootNavigation(selectedKey),
        },
      ],
    });
    await waitFor(() => expect(current().pageNumber).toBe(1));
    expect(document.body).not.toHaveTextContent("secret-record-id");
  },
);

it("keeps parent paging and criteria untouched on cancel and resets the next lookup to its defaults", async () => {
  const current = setup({
    ...DEFAULT_SEARCH_STATE,
    query: "parent text",
    relationships: [existing],
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
  await waitFor(() =>
    expect(vi.mocked(searchPage).mock.lastCall?.[0].sort).toEqual({
      kind: "attributes",
      rules: [{ field: "name", direction: "desc" }],
    }),
  );
  fireEvent.click(within(dialog).getByRole("radio", { name: "Linked from" }));
  await selectSortOption("Link filter scope", "Main entry only");
  fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  await waitFor(() => expect(dialog).not.toBeVisible());
  expect(screen.getByRole("button", { name: "Add link filter" })).toHaveFocus();
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
  "rejects an exact duplicate without resetting paging, while allowing the same entry with a different %s",
  async (change) => {
    const duplicate = {
      direction: "incoming" as const,
      include_contents: true,
      selection: rootNavigation(selectedKey),
    };
    const current = setup({ ...DEFAULT_SEARCH_STATE, relationships: [duplicate] });
    const before = current().search;
    const dialog = await openAndSelect();
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    expect(
      await within(dialog).findByText("This link filter is already added."),
    ).toHaveAttribute("role", "status");
    expect(current().search).toEqual(before);
    expect(current().pageNumber).toBe(3);
    if (change === "direction")
      fireEvent.click(within(dialog).getByRole("radio", { name: "Linked from" }));
    else await selectSortOption("Link filter scope", "Main entry only");
    fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
    await waitFor(() => expect(current().search.relationships).toHaveLength(2));
    expect(current().search.relationships?.[1]).toEqual({
      ...duplicate,
      direction: change === "direction" ? "outgoing" : "incoming",
      include_contents: change === "direction",
    });
  },
);

it("shows a useful local limit message instead of creating an invalid relationship URL", () => {
  setup({
    ...DEFAULT_SEARCH_STATE,
    relationships: Array.from({ length: MAX_RELATIONSHIP_FILTERS }, (_, index) => ({
      ...existing,
      selection: rootNavigation(`items:${index}`),
    })),
  });
  expect(screen.getByRole("button", { name: "Add link filter" })).toBeDisabled();
  expect(screen.getByRole("status")).toHaveTextContent(
    "Remove a link filter before adding another.",
  );
});

it("recognizes an exact duplicate restored from a URL that omits nullable selection fields", async () => {
  const relationships = decodeRelationships(
    "1." +
      JSON.stringify([
        {
          direction: "incoming",
          include_contents: true,
          selection: { record_key: selectedKey, embedded_path: [] },
        },
      ]),
  );
  const current = setup({ ...DEFAULT_SEARCH_STATE, relationships });
  const before = current().search;
  const dialog = await openAndSelect();
  fireEvent.click(within(dialog).getByRole("button", { name: "Add" }));
  expect(
    await within(dialog).findByText("This link filter is already added."),
  ).toBeVisible();
  expect(current().search).toEqual(before);
  expect(current().pageNumber).toBe(3);
});

it("keeps scope explanation optional and dismisses its inner help without closing the lookup", async () => {
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
