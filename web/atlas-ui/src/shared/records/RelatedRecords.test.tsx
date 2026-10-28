import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import { RouterTestContext } from "../../test/router";
import { RecordReadingWorkspace } from "./RecordReadingWorkspace";
import type { SearchResultsRequest, SearchResultRow } from "../../generated/atlas";
import type { ResultSort } from "../../generated/atlas";
import { linkLeaf } from "../filters/linkPredicates";
import { decodeFilterUrl, encodeFilterUrl } from "../filters/filterUrl";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import type {
  GraphContextView,
  RecordNavigationView,
  RecordRelationshipView,
  SearchResultsPage,
} from "../../generated/atlas";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { AtlasApiError } from "../../api/atlasApi";
import { useRecordDetail } from "./useRecordDetail";
import { RelatedRecords } from "./RelatedRecords";
import {
  DEFAULT_SEARCH_STATE,
  decodeSearchStateFromParams,
  searchStateQueryString,
} from "../filters/searchState";

const api = vi.hoisted(() => ({
  getRelatedRecords: vi.fn(),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));

function navigation(key: string): RecordNavigationView {
  return {
    record_key: key,
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
}
function graphFixture(): GraphContextView {
  const seed = summaryFixture("actors:seed", "Viewed creature");
  const source = {
    record: "actors:referrer",
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 0 } } }],
    field: "/system/description/value",
  };
  const incoming: RecordRelationshipView = {
    effect_label: null,
    source,
    source_navigation: {
      ...navigation(source.record),
      embedded_path: source.embedded_path,
      field: source.field,
      source_fingerprint: "snapshot-current",
    },
    ordinal: 4,
    origin: "prepared_content",
    kind: "uuid",
    authored_target: "Compendium.raw.internal.uuid",
    occurrence_path: null,
    audiences: [],
    availability: null,
    status: "resolved_record",
    target: navigation(seed.record_key),
    url: null,
  };
  const outgoing = {
    ...incoming,
    source: { record: seed.record_key, embedded_path: [], field: source.field },
    source_navigation: { ...navigation(seed.record_key), field: source.field },
    target: navigation("spell:target"),
  };
  return {
    seed,
    outgoing: {
      records: [summaryFixture("spell:target", "Referenced spell")],
      occurrences: [outgoing],
      truncated: false,
    },
    backlinks: {
      records: [summaryFixture("actors:referrer", "Referring creature")],
      occurrences: [incoming, { ...incoming, ordinal: 5 }],
      truncated: true,
    },
  };
}
function renderRelated(
  detail = detailFixture("actors:seed", "Viewed creature"),
  providedClient?: QueryClient,
) {
  const onReference = vi.fn();
  const queryClient =
    providedClient ??
    new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 } },
    });
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={queryClient}>
        <SearchProfileProvider>
          <RouterTestContext>
            <RecordReadingWorkspace
              identity={(action) => <header>{action}</header>}
              explore={<RelatedRecords detail={detail} onReference={onReference} />}
            >
              Reading the document
            </RecordReadingWorkspace>
          </RouterTestContext>
        </SearchProfileProvider>
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return onReference;
}
function openRelated() {
  fireEvent.click(screen.getByRole("button", { name: "Explore related records" }));
}

function relatedFixture() {
  const detail = detailFixture("actors:seed", "Viewed creature");
  return {
    selection: detail.selected,
    artifact_generation: detail.artifact_generation,
    seed: detail.record,
    references: { kind: "content" as const, content: graphFixture() },
    counterparts: { kind: "empty" as const },
    variants: { kind: "empty" as const },
  };
}

function page(
  request: SearchResultsRequest,
  rows: SearchResultRow[] = [],
  total = rows.length,
): SearchResultsPage {
  return {
    search_content: "all",
    ...(request.criteria.kind === "similar_records" && request.criteria.similarity
      ? {
          similarity: {
            options: request.criteria.similarity,
            source_sections:
              request.criteria.similarity.method === "baseline" ? null : "all",
            search_content: "all",
            candidate_identity: request.criteria.similarity.method === "baseline",
            core_recipe: null,
            resolved_selectors: [],
            core: null,
            recipe: "test-recipe",
            role_recipe: "typed-content-correspondence/v1",
            diagnostics: {
              seed_units: 1,
              knn_queries: 1,
              candidate_vectors: 1,
              pairwise_comparisons: 1,
              undefined_candidate_centroids: 0,
            },
          },
        }
      : {}),
    sort:
      request.sort ??
      (request.criteria.kind === "list_records"
        ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
        : { kind: "relevance" }),
    ranking: "score",
    search_profile: request.search_profile ?? "bge-base",
    artifact_generation: request.expected_artifact_generation ?? "test-generation",
    mode:
      request.criteria.kind === "similar_records"
        ? { kind: "similar_records", seed: summaryFixture("actors:seed") }
        : { kind: "list_records" },
    rows,
    relationship_clauses: [],
    coverage: null,
    page: {
      number: 1,
      size: 5,
      count: rows.length,
      total: BigInt(total),
      has_more: total > rows.length,
      next_page: total > rows.length ? 2 : null,
    },
  };
}
const row = (key: string, title: string): SearchResultRow => ({
  record: summaryFixture(key, title),
  matches: [],
  relationship_evidence: [],
});
function preview(request: SearchResultsRequest) {
  if (request.criteria.kind === "similar_records") return page(request);
  return page(request, [row("actors:referrer", "Referring creature")], 64);
}
async function selectTab(name: string) {
  fireEvent.click(await screen.findByRole("tab", { name }));
}
beforeEach(() => {
  history.replaceState(null, "", "/records/actors:seed");
  vi.clearAllMocks();
  api.getRelatedRecords.mockResolvedValue(relatedFixture());
  api.searchPage.mockImplementation(async (request: SearchResultsRequest) =>
    preview(request),
  );
});

it("defers exploration until requested, previews five distinct roots, and leaves the full set to Search", async () => {
  api.searchPage.mockImplementation(async (request: SearchResultsRequest) =>
    page(
      request,
      Array.from({ length: 5 }, (_, i) => row("actors:record" + i, "Creature " + i)),
      164,
    ),
  );
  renderRelated();
  expect(api.getRelatedRecords).not.toHaveBeenCalled();
  expect(api.searchPage).not.toHaveBeenCalled();
  openRelated();
  expect(await screen.findByText("5 of 164 records")).toBeVisible();
  expect(screen.getAllByRole("listitem")).toHaveLength(5);
  expect(api.getRelatedRecords).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole("tab", { name: "Versions" })).toBeNull();
  expect(screen.queryByText(/Showing a limited|See more|View all/i)).toBeNull();
  expect(screen.queryByRole("button", { name: /Next page/i })).toBeNull();
  const link = screen.getByRole("link", { name: "Search referring records" });
  expect(link).not.toHaveClass("ant-btn-primary");
  expect(new URL(link.getAttribute("href")!, location.origin).pathname).toBe("/search");
  expect(
    api.searchPage.mock.calls.every(
      ([request]) => request.page.number === 1 && request.page.size === 5,
    ),
  ).toBe(true);
});

it("hides verified-empty reference and version tabs", async () => {
  api.searchPage.mockImplementation(async (request: SearchResultsRequest) =>
    page(request),
  );
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    references: { kind: "empty" },
  });
  renderRelated();
  openRelated();
  expect(
    await screen.findByRole("tab", { name: "Similar", selected: true }),
  ).toBeVisible();
  expect(screen.queryByRole("tab", { name: "References" })).toBeNull();
  expect(screen.queryByRole("tab", { name: "Referenced by" })).toBeNull();
  expect(screen.queryByRole("tab", { name: "Versions" })).toBeNull();
});

it("uses the shared Matches interaction and preserves precise native destinations", async () => {
  const expected = graphFixture().backlinks.occurrences[0].source_navigation;
  api.searchPage.mockImplementation(async (request: SearchResultsRequest) =>
    page(request, [
      {
        ...row("actors:referrer", "Referring creature"),
        matches: [
          {
            navigation: expected,
            label: "Checked ability",
            snippet: null,
            lane: "lexical",
            lexical_rank: null,
            semantic_similarity: null,
          },
        ],
      },
    ]),
  );
  const onReference = renderRelated();
  openRelated();
  const result = await screen.findByRole("link", { name: /^Referring creature/ });
  expect(screen.getAllByRole("link", { name: /^Referring creature/ })).toHaveLength(1);
  fireEvent.click(result);
  expect(onReference).toHaveBeenCalledWith("actors:referrer", result);
  onReference.mockClear();
  expect(fireEvent.click(result, { metaKey: true })).toBe(true);
  expect(onReference).not.toHaveBeenCalled();
  const matches = screen.getByRole("button", {
    name: "1 matches in Referring creature",
  });
  fireEvent.click(matches);
  const match = await screen.findByRole("link", { name: "Checked ability" });
  expect(
    selectedNavigation(
      parseSearchParams(new URL(match.getAttribute("href")!, location.origin).search),
      expected.record_key,
    ),
  ).toEqual(expected);
  fireEvent.keyDown(document, { key: "Escape" });
  expect(matches).toHaveFocus();
  fireEvent.click(matches);
  fireEvent.click(await screen.findByRole("link", { name: "Checked ability" }));
  expect(onReference).toHaveBeenCalledWith(expected.record_key, matches, expected);
  expect(screen.queryByText("Compendium.raw.internal.uuid")).toBeNull();
});

it("reloads the exact record generation after an unavailable association check", async () => {
  const oldDetail = detailFixture("actors:seed", "Viewed creature");
  const newDetail = { ...oldDetail, artifact_generation: "new-preparation" };
  api.getRecordDetail.mockResolvedValueOnce(oldDetail).mockResolvedValue(newDetail);
  api.getRelatedRecords.mockRejectedValueOnce(
    new AtlasApiError(409, "Generation changed", {
      code: "artifact_generation_changed",
      message: "Generation changed",
      recoverable_action: "reload_record",
    }),
  );
  function Reader() {
    const detail = useRecordDetail("actors:seed");
    return detail.data ? (
      <RelatedRecords
        detail={detail.data}
        onReference={vi.fn()}
        onReload={() => void detail.refetch()}
      />
    ) : null;
  }
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <RouterTestContext>
      <QueryClientProvider client={client}>
        <SearchProfileProvider>
          <Reader />
        </SearchProfileProvider>
      </QueryClientProvider>
    </RouterTestContext>,
  );
  await selectTab("References");
  fireEvent.click(await screen.findByRole("button", { name: "Reload record" }));
  await waitFor(() =>
    expect(api.getRelatedRecords).toHaveBeenLastCalledWith(
      { selection: newDetail.selected, artifact_generation: "new-preparation" },
      expect.any(AbortSignal),
    ),
  );
  expect(api.getRecordDetail).toHaveBeenCalledTimes(2);
  client.clear();
});

it("retries failed checks and keeps unavailable versions distinguishable from verified empty", async () => {
  api.getRelatedRecords.mockRejectedValue(new Error("Service unavailable"));
  renderRelated();
  openRelated();
  await selectTab("References");
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Unable to load connections",
  );
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    variants: { kind: "unavailable", reason: "Compatibility facts are unavailable" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await selectTab("Versions");
  expect(await screen.findByText("Compatibility facts are unavailable")).toBeVisible();
  expect(screen.queryByRole("alert")).toBeNull();
});

it.each([false, true])(
  "keeps complete selection and effective contents scope in each Search handoff (embedded: %s)",
  async (embedded) => {
    const detail = detailFixture("actors:seed", "Viewed creature");
    if (embedded)
      detail.selected = {
        ...detail.selected,
        embedded_path: [
          { collection: "/items", identity: { Stable: "checked-ability" } },
        ],
        field: "/system/description/value",
        source_fingerprint: "exact-snapshot",
      };
    renderRelated(detail);
    openRelated();
    for (const [tab, direction, label] of [
      ["Referenced by", "incoming", "Search referring records"],
      ["References", "outgoing", "Search referenced records"],
    ] as const) {
      await selectTab(tab);
      const link = await screen.findByRole("link", { name: label });
      const filter = decodeFilterUrl(
        new URL(link.getAttribute("href")!, location.origin).searchParams.get(
          "filter",
        )!,
      );
      expect(filter).toEqual(
        decodeFilterUrl(
          encodeFilterUrl(
            linkLeaf(
              { direction, selection: detail.selected, include_contents: !embedded },
              "record-explore-reference",
            ),
          ),
        ),
      );
      expect(fireEvent.click(link, { metaKey: true })).toBe(true);
    }
    expect(api.getRelatedRecords).toHaveBeenCalledWith(
      { selection: detail.selected, artifact_generation: detail.artifact_generation },
      expect.any(AbortSignal),
    );
    if (embedded) {
      expect(screen.queryByRole("tab", { name: "Versions" })).toBeNull();
      await selectTab("Similar");
      expect(
        await screen.findByText("Similar to parent record: Viewed creature"),
      ).toBeVisible();
    }
  },
);

it("retains active Search filters and explicit order in the Similar handoff", async () => {
  const state = {
    ...DEFAULT_SEARCH_STATE,
    query: "fire",
    sort: {
      kind: "attributes",
      rules: [{ field: "unit_price", direction: "desc" }],
    } satisfies ResultSort,
    filter: linkLeaf(
      {
        direction: "incoming",
        include_contents: false,
        selection: { ...navigation("items:armor"), source_fingerprint: "source" },
      },
      "link-0",
    ),
  };
  history.replaceState(null, "", "/search" + searchStateQueryString(state));
  renderRelated();
  openRelated();
  await selectTab("Similar");
  const link = await screen.findByRole("link", { name: "Search similar records" });
  const restored = decodeSearchStateFromParams(
    new URL(link.getAttribute("href")!, location.origin).searchParams,
  );
  expect(restored).toMatchObject({
    query: "",
    similarSeed: "actors:seed",
    sort: state.sort,
    filter: decodeFilterUrl(encodeFilterUrl(state.filter)),
  });
  expect(
    api.searchPage.mock.calls.find(([r]) => r.criteria.kind === "similar_records")?.[0],
  ).not.toHaveProperty("sort");
});

it("rejects late Similar responses from the previous model and updates the Search handoff", async () => {
  const detail = detailFixture("actors:seed", "Viewed creature");
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  let finishOld!: (value: SearchResultsPage) => void;
  let finishNew!: (value: SearchResultsPage) => void;
  api.searchPage.mockImplementation((request: SearchResultsRequest) =>
    request.criteria.kind !== "similar_records"
      ? Promise.resolve(preview(request))
      : new Promise((resolve) => {
          if (request.search_profile === "bge-base") finishOld = resolve;
          else finishNew = resolve;
        }),
  );
  const content = (id: string) => (
    <RouterTestContext>
      <QueryClientProvider client={client}>
        <SearchProfileProvider id={id} usable={false}>
          <RelatedRecords detail={detail} onReference={vi.fn()} />
        </SearchProfileProvider>
      </QueryClientProvider>
    </RouterTestContext>
  );
  const view = render(content("bge-base"));
  await selectTab("Similar");
  await waitFor(() => expect(finishOld).toBeTypeOf("function"));
  view.rerender(content("embeddinggemma"));
  await waitFor(() => expect(finishNew).toBeTypeOf("function"));
  const request = (profile: string): SearchResultsRequest =>
    api.searchPage.mock.calls.find(
      ([request]) =>
        request.criteria.kind === "similar_records" &&
        request.search_profile === profile,
    )![0];
  await act(async () =>
    finishNew(
      page(request("embeddinggemma"), [row("actors:current", "Current suggestion")]),
    ),
  );
  await act(async () =>
    finishOld(page(request("bge-base"), [row("actors:old", "Old suggestion")])),
  );
  expect(
    await screen.findByRole("link", { name: /^Current suggestion/ }),
  ).toBeVisible();
  expect(screen.queryByText("Old suggestion")).toBeNull();
  expect(api.getRelatedRecords).toHaveBeenCalledTimes(1);
  expect(
    new URL(
      screen
        .getByRole("link", { name: "Search similar records" })
        .getAttribute("href")!,
      location.origin,
    ).searchParams.get("search_profile"),
  ).toBe("embeddinggemma");
  client.clear();
});

it("renders Versions with shared result cards and compares complete records without recursive exploration", async () => {
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    variants: {
      kind: "content",
      content: {
        seed: summaryFixture("actors:seed"),
        base_name: "Viewed creature",
        variants: [
          summaryFixture("actors:seed", "Viewed creature"),
          summaryFixture("actors:variant", "Greater creature"),
        ],
        evidence: [],
        ambiguous: false,
        truncated: false,
      },
    },
  });
  api.getRecordDetail.mockImplementation((key: string) =>
    Promise.resolve(
      detailFixture(
        key,
        key === "actors:seed" ? "Viewed creature" : "Greater creature",
      ),
    ),
  );
  const onReference = renderRelated();
  openRelated();
  await selectTab("Versions");
  const result = await screen.findByRole("link", { name: /^Greater creature/ });
  expect(result).toHaveAttribute("data-record-key", "actors:variant");
  fireEvent.click(result);
  expect(onReference).toHaveBeenCalledWith("actors:variant", result);
  const compare = screen.getByRole("button", { name: "Compare" });
  compare.focus();
  fireEvent.click(compare);
  const dialog = await screen.findByRole("dialog");
  await waitFor(() =>
    expect(
      within(dialog).getByRole("heading", { name: "Greater creature" }),
    ).toBeVisible(),
  );
  expect(
    within(dialog).getByRole("heading", { name: "Viewed creature" }),
  ).toBeVisible();
  expect(
    within(dialog).queryByRole("button", { name: "Explore related records" }),
  ).toBeNull();
  expect(
    api.getRelatedRecords.mock.calls.every(
      ([request]) => request.selection.record_key === "actors:seed",
    ),
  ).toBe(true);
  fireEvent.keyDown(dialog, { key: "Escape", code: "Escape", keyCode: 27 });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(compare).toHaveFocus();
});

it("retires preview rows after a generation conflict and retains a local recovery action", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  renderRelated(detailFixture("actors:seed", "Viewed creature"), client);
  openRelated();
  expect(
    await screen.findByRole("link", { name: /^Referring creature/ }),
  ).toBeVisible();
  api.searchPage.mockRejectedValue(
    new AtlasApiError(409, "Reference generation changed", {
      code: "artifact_generation_changed",
      message: "Reference generation changed",
      recoverable_action: "reload_record",
    }),
  );
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["record-reference-preview"] });
  });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "The reference collection changed. Refresh before continuing.",
  );
  await waitFor(() =>
    expect(screen.queryByRole("link", { name: /^Referring creature/ })).toBeNull(),
  );
  expect(screen.getByRole("button", { name: "Reload record" })).toBeVisible();
});
