import { createMemoryHistory } from "@tanstack/react-router";
import { createAtlasRouter } from "../../app/router";
import { RecordEditionNotice } from "./RecordEditionNotice";
import { remasterInformationFixture } from "../../test/remasterFixtures";
import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import { RouterTestContext } from "../../test/router";
import { RecordReadingWorkspace } from "./RecordReadingWorkspace";
import type { SearchResultsRequest, SearchResultRow } from "../../generated/atlas";
import type { ResultSort } from "../../generated/atlas";
import { linkLeaf } from "../filters/linkPredicates";
import { decodeFilterUrl } from "../filters/filterUrl";
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
    destination: { kind: "resolved_record", target: navigation(seed.record_key) },
  };
  const outgoing = {
    ...incoming,
    source: { record: seed.record_key, embedded_path: [], field: source.field },
    source_navigation: { ...navigation(seed.record_key), field: source.field },
    destination: {
      kind: "resolved_record" as const,
      target: navigation("spell:target"),
    },
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
  providedRouter?: ReturnType<typeof createAtlasRouter>,
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
          <RouterTestContext router={providedRouter}>
            <RecordReadingWorkspace
              identity={(action) => <header>{action}</header>}
              readingNotice={
                <RecordEditionNotice detail={detail} onReference={onReference} />
              }
              explore={<RelatedRecords detail={detail} onReference={onReference} />}
            >
              <input
                aria-label="Retained reading draft"
                defaultValue="Preserved draft"
              />
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
    show_versions: false,
    selection: detail.selected,
    artifact_generation: detail.artifact_generation,
    seed: detail.record,
    references: { kind: "content" as const, content: graphFixture() },
    remaster_relationships: { kind: "empty" as const },
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
      total: total,
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
  expect(
    screen
      .getAllByRole("link")
      .filter((link) => link.hasAttribute("data-record-key"))
      .map((link) => link.getAttribute("data-record-key")),
  ).toEqual([
    "actors:record0",
    "actors:record1",
    "actors:record2",
    "actors:record3",
    "actors:record4",
  ]);
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
        matching_sections: [
          { navigation: expected, label: "Checked ability", preview: null },
        ],
        matches: [
          {
            navigation: expected,
            label: "Raw witness label",
            preview: null,
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
    show_versions: true,
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
      expect(filter).toEqual({
        kind: "relationship",
        clause_id: expect.any(String),
        relationship: {
          direction,
          selection: detail.selected,
          include_contents: !embedded,
        },
      });
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
  history.replaceState(
    null,
    "",
    "/search/records/actors:seed" + searchStateQueryString(state),
  );
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
    filter: {
      kind: "relationship",
      clause_id: expect.any(String),
      relationship: state.filter.relationship,
    },
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
    show_versions: true,
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
  expect(screen.getByRole("heading", { name: "Possible variants" })).toBeVisible();
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

it("shows verified equipment variants with their attributes and omits the current item", async () => {
  const seed = summaryFixture("actors:seed", "Energy Breath Potion (Fire, Lesser)");
  const sibling = summaryFixture(
    "items:sibling",
    "Energy Breath Potion (Sonic, Moderate)",
  );
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    show_versions: true,
    variants: {
      kind: "content",
      content: {
        seed,
        base_name: "Energy Breath Potion",
        variants: [seed, sibling],
        verified: true,
        evidence: [
          {
            record_key: sibling.record_key,
            naming_convention: "reviewed_membership",
            qualifier: "Sonic, Moderate",
            compatibility: "reviewed equipment family",
          },
        ],
        ambiguous: false,
        truncated: false,
      },
    },
  });

  renderRelated();
  openRelated();
  await selectTab("Versions");

  expect(await screen.findByRole("heading", { name: "Variants" })).toBeVisible();
  expect(screen.getByText("Sonic, Moderate", { exact: true })).toBeVisible();
  expect(screen.queryByRole("link", { name: /Fire, Lesser/ })).toBeNull();
  expect(screen.getByRole("button", { name: "Compare" })).toBeVisible();
});

it("renders a scoped general description as the Versions view with exact specific links", async () => {
  const specific = summaryFixture(
    "equipment-srd:evdDpgFpiFZt9UyA",
    "Potion of Cold Resistance (Moderate)",
  );
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    show_versions: true,
    family_context: {
      kind: "general",
      heading: "Damage types",
      members: [{ record: specific, qualifier: "Cold, Moderate" }],
    },
  });

  const onReference = renderRelated();
  openRelated();
  await selectTab("Versions");

  expect(await screen.findByRole("heading", { name: "Damage types" })).toBeVisible();
  expect(screen.queryByText("No other editions or variants found.")).toBeNull();
  expect(screen.queryByRole("heading", { name: "Possible variants" })).toBeNull();
  const link = screen.getByRole("link", { name: /^Potion of Cold Resistance/ });
  expect(link.getAttribute("href")).toContain("evdDpgFpiFZt9UyA");
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith(specific.record_key, link);
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

it("keeps Read Info actions exact and leaves explanation context out of Versions", async () => {
  const target = {
    ...navigation("journals:source"),
    embedded_path: [{ collection: "/pages", identity: { Stable: "page1" } }],
    field: "/text/content",
  };
  const content = remasterInformationFixture();
  content.relationships[0].explanation_links = [
    { label: "Remaster Changes — Class Features", navigation: target },
  ];
  const detail = detailFixture("actors:seed", "Viewed creature");
  detail.remaster_relationships = { kind: "content", content };
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    show_versions: true,
    remaster_relationships: { kind: "content", content },
  });
  const onReference = renderRelated(
    detail,
    undefined,
    createAtlasRouter(
      createMemoryHistory({ initialEntries: ["/records/actors:seed"] }),
    ),
  );
  const info = screen.getByRole("link", { name: "Remaster Changes — Class Features" });
  expect(info).toHaveTextContent("");
  expect(
    selectedNavigation(
      parseSearchParams(new URL(info.getAttribute("href")!, "http://localhost").search),
      "journals:source",
    ),
  ).toEqual(target);
  fireEvent.click(info, { metaKey: true });
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(info);
  expect(onReference).toHaveBeenCalledWith("journals:source", info, target);
  openRelated();
  await selectTab("Versions");
  expect(
    screen.queryByRole("link", { name: "Remaster Changes — Class Features" }),
  ).toBeNull();
  expect(screen.getByText("Renamed to:")).toBeVisible();
});

it("compares a logical relationship's available root without duplicating its name or adding recursive exploration", async () => {
  const content = remasterInformationFixture();
  content.relationships[0].parts = [
    {
      kind: "reference",
      label: "Modern creature",
      navigation: navigation("actors:modern"),
    },
  ];
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    show_versions: true,
    remaster_relationships: { kind: "content", content },
  });
  api.getRecordDetail.mockImplementation(async (key: string) => {
    const isLegacy = key === "actors:seed";
    const detail = detailFixture(
      key,
      isLegacy ? "Viewed creature" : "Modern creature",
      "rules:nested",
    );
    const body = detail.presentation.content[0].body;
    if (body.kind === "html")
      body.html = `<p>${isLegacy ? "Legacy comparison rules." : "Modern comparison rules."}</p>${body.html}`;
    const reciprocal = remasterInformationFixture();
    reciprocal.relationships[0].label = "Reciprocal edition banner";
    reciprocal.relationships[0].parts = [
      {
        kind: "reference",
        label: isLegacy ? "Modern creature" : "Viewed creature",
        navigation: navigation(isLegacy ? "actors:modern" : "actors:seed"),
      },
    ];
    detail.remaster_relationships = { kind: "content", content: reciprocal };
    return detail;
  });
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  const onReference = renderRelated(undefined, client);
  openRelated();
  await selectTab("Versions");
  const line = screen.getByText("Renamed to:").closest("p")!;
  const current = within(line).getByRole("link", { name: "Modern creature" });
  expect(current).toHaveAttribute("href", "/records/actors:modern");
  expect(within(line).getAllByText("Modern creature")).toHaveLength(1);
  fireEvent.click(current, { metaKey: true });
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(current);
  expect(onReference).toHaveBeenCalledWith(
    "actors:modern",
    current,
    navigation("actors:modern"),
  );
  const actions = line.parentElement!.querySelector(".remaster-relationship__actions")!;
  const compare = within(actions as HTMLElement).getByRole("button", {
    name: "Compare",
  });
  expect(line).not.toContainElement(compare);
  expect(
    line.compareDocumentPosition(actions) & Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
  compare.focus();
  fireEvent.click(compare);
  const dialog = await screen.findByRole("dialog", { name: "Variant comparison" });
  expect(
    await within(dialog).findByRole("heading", { name: "Modern creature" }),
  ).toBeVisible();
  expect(
    within(dialog).getByRole("heading", { name: "Viewed creature" }),
  ).toBeVisible();
  expect(
    within(dialog).queryByRole("button", { name: "Explore related records" }),
  ).toBeNull();
  expect(within(dialog).getByText("Legacy comparison rules.")).toBeVisible();
  expect(within(dialog).getByText("Modern comparison rules.")).toBeVisible();
  expect(within(dialog).queryByText("Reciprocal edition banner:")).toBeNull();
  expect(dialog.querySelector(".record-edition-notice")).toBeNull();
  expect(within(dialog).getAllByRole("link", { name: "Nested Rule" })).toHaveLength(2);
  api.getRecordDetail.mockRejectedValueOnce(new Error("Refresh unavailable"));
  await act(async () => {
    await client.refetchQueries({ queryKey: ["record-detail", "actors:modern"] });
  });
  expect(await within(dialog).findByRole("alert")).toHaveTextContent(
    "Unable to refresh this record",
  );
  expect(within(dialog).getByText("Legacy comparison rules.")).toBeVisible();
  expect(within(dialog).getByText("Modern comparison rules.")).toBeVisible();
  expect(within(dialog).getByRole("button", { name: "Retry" })).toBeVisible();
  expect(dialog.querySelector(".record-edition-notice")).toBeNull();
  expect(
    api.getRelatedRecords.mock.calls.every(
      ([request]) => request.selection.record_key === "actors:seed",
    ),
  ).toBe(true);
  fireEvent.keyDown(dialog, { key: "Escape", keyCode: 27 });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(compare).toHaveFocus();
});

it.each([
  "owned host",
  "host field",
  "owned target",
  "target field",
  "same root",
  "unavailable name",
])("does not offer a root-key comparison for %s", async (scenario) => {
  const detail = detailFixture("actors:seed", "Viewed creature");
  const target = navigation("actors:modern");
  const owned = [{ collection: "items", identity: { Stable: "ability" } }];
  if (scenario === "owned host") detail.selected.embedded_path = owned;
  if (scenario === "host field") detail.selected.field = "/system/description/value";
  if (scenario === "owned target") target.embedded_path = owned;
  if (scenario === "target field") target.field = "/system/description/value";
  if (scenario === "same root") target.record_key = detail.selected.record_key;
  const content = remasterInformationFixture();
  content.relationships[0].parts =
    scenario === "unavailable name"
      ? [{ kind: "text", text: "Unavailable earlier name" }]
      : [{ kind: "reference", label: "Modern creature", navigation: target }];
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    show_versions: true,
    remaster_relationships: { kind: "content", content },
  });
  renderRelated(detail);
  openRelated();
  await selectTab("Versions");
  expect(screen.getByText("Renamed to:")).toBeVisible();
  expect(screen.queryByRole("button", { name: "Compare" })).toBeNull();
});

it("lands on the first Explore tab when Versions is available and preserves an explicit tab", async () => {
  let resolve!: (value: ReturnType<typeof relatedFixture>) => void;
  api.getRelatedRecords.mockReturnValue(
    new Promise((yes) => {
      resolve = yes;
    }),
  );
  const router = createAtlasRouter(
    createMemoryHistory({ initialEntries: ["/records/actors:seed"] }),
  );
  renderRelated(detailFixture("actors:seed", "Viewed creature"), undefined, router);
  openRelated();
  await waitFor(() => expect(api.getRelatedRecords).toHaveBeenCalled());
  expect(screen.queryByRole("tab", { name: "Versions" })).toBeNull();
  await act(async () => resolve({ ...relatedFixture(), show_versions: true }));
  const versions = await screen.findByRole("tab", { name: "Versions", selected: true });
  expect(within(screen.getByRole("tablist")).getAllByRole("tab")[0]).toBe(versions);
  expect(router.latestLocation.search.record_tab).toBeUndefined();
  await selectTab("Referenced by");
  await waitFor(() => expect(router.latestLocation.search.record_tab).toBe("incoming"));
  expect(
    screen.getByRole("tab", { name: "Referenced by", selected: true }),
  ).toBeVisible();
});

it.each([
  ["Former name", "Magic Missile"],
  ["Replaces", "Remove Curse, Remove Disease, and Neutralize Poison"],
  ["Merged from", "Aasimar, Aphorite, Ganzi, and Tiefling"],
])(
  "keeps the %s note and journal link without a Versions tab when Rust says no alternatives",
  async (label, text) => {
    const detail = detailFixture("actors:seed", "Viewed creature");
    const content = remasterInformationFixture();
    content.relationships[0] = {
      ...content.relationships[0],
      label,
      parts: [{ kind: "text", text }],
      explanation_links: [
        {
          label: "Remaster Changes — Spells",
          navigation: navigation("journals:changes"),
        },
      ],
    };
    detail.remaster_relationships = { kind: "content", content };
    api.getRelatedRecords.mockResolvedValue({
      ...relatedFixture(),
      show_versions: false,
      remaster_relationships: { kind: "content", content },
    });
    renderRelated(
      detail,
      undefined,
      createAtlasRouter(
        createMemoryHistory({ initialEntries: ["/records/actors:seed"] }),
      ),
    );
    expect(screen.getByText(`${label}:`)).toBeVisible();
    openRelated();
    await waitFor(() => expect(api.getRelatedRecords).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.queryByRole("tab", { name: "Versions" })).toBeNull(),
    );
    expect(screen.queryByText(text)).toBeNull();
    expect(
      screen.queryByRole("link", { name: "Remaster Changes — Spells" }),
    ).toBeNull();
    expect(screen.queryByText("Sources")).toBeNull();
  },
);

it("replaces a successfully unavailable explicit Versions location once and preserves selection, state, hash and drafts", async () => {
  const href =
    "/records/actors:seed?rank=5&record_view=explore&record_tab=versions#retained-field";
  const router = createAtlasRouter(
    createMemoryHistory({
      initialEntries: ["/search?q=retained", href],
      initialIndex: 1,
    }),
  );
  router.history.replace(href, {
    ...router.history.location.state,
    atlasSearchReset: "retained-draft-state",
  });
  const navigate = vi.spyOn(router, "navigate");
  const detail = detailFixture("actors:seed", "Viewed creature");
  const content = remasterInformationFixture();
  content.relationships[0].parts = [{ kind: "text", text: "Unavailable former name" }];
  detail.remaster_relationships = { kind: "content", content };
  let resolve!: (value: ReturnType<typeof relatedFixture>) => void;
  api.getRelatedRecords.mockReturnValue(
    new Promise((yes) => {
      resolve = yes;
    }),
  );
  renderRelated(detail, undefined, router);
  expect(
    await screen.findByRole("tab", { name: "Versions", selected: true }),
  ).toBeVisible();
  expect(screen.queryByText("Unavailable former name")).toBeNull();
  expect(router.latestLocation.search.record_view).toBe("explore");
  await act(async () => resolve(relatedFixture()));
  await waitFor(() => expect(router.latestLocation.search.record_view).toBeUndefined());
  expect(router.latestLocation.search.record_tab).toBeUndefined();
  expect(router.latestLocation.pathname).toBe("/records/actors:seed");
  expect(router.latestLocation.search.rank).toBe("5");
  expect(router.latestLocation.href).toContain("#retained-field");
  expect(router.latestLocation.state.atlasSearchReset).toBe("retained-draft-state");
  expect(router.history.length).toBe(2);
  expect(
    navigate.mock.calls.filter(
      ([options]) => options.to === "." && options.replace === true,
    ),
  ).toHaveLength(1);
  expect(screen.getByText("Unavailable former name")).toBeVisible();
  const draft = screen.getByRole("textbox", { name: "Retained reading draft" });
  fireEvent.change(draft, { target: { value: "Still editing" } });
  await act(async () => router.history.back());
  await waitFor(() => expect(router.latestLocation.pathname).toBe("/search"));
  await act(async () => router.history.forward());
  await waitFor(() =>
    expect(router.latestLocation.pathname).toBe("/records/actors:seed"),
  );
  expect(router.latestLocation.search.record_view).toBeUndefined();
  fireEvent.click(screen.getByRole("button", { name: "Explore related records" }));
  await waitFor(() =>
    expect(
      screen.getByRole("tab", { name: "Referenced by", selected: true }),
    ).toBeVisible(),
  );
  await waitFor(() =>
    expect(screen.queryByRole("tab", { name: "Versions" })).toBeNull(),
  );
  expect(screen.queryByText("Unavailable former name")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Read document" }));
  expect(await screen.findByRole("textbox", { name: "Retained reading draft" })).toBe(
    draft,
  );
  expect(draft).toHaveValue("Still editing");
  navigate.mockRestore();
});

it("keeps an explicit Versions failure recoverable without a floating reader notice", async () => {
  history.replaceState(
    null,
    "",
    "/records/actors:seed?record_view=explore&record_tab=versions",
  );
  const detail = detailFixture("actors:seed", "Viewed creature");
  detail.remaster_relationships = {
    kind: "content",
    content: remasterInformationFixture(),
  };
  api.getRelatedRecords.mockRejectedValue(new Error("Temporary connection failure"));
  renderRelated(detail);
  expect(
    await screen.findByRole("tab", { name: "Versions", selected: true }),
  ).toBeVisible();
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Unable to load connections",
  );
  expect(screen.queryByText("Renamed to:")).toBeNull();
  expect(location.search).toContain("record_view=explore");
  api.getRelatedRecords.mockResolvedValue(relatedFixture());
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await waitFor(() =>
    expect(new URLSearchParams(location.search).has("record_view")).toBe(false),
  );
  expect(screen.getByText("Renamed to:")).toBeVisible();
});

it("retains one Versions relationship and its explanation through a background refetch failure", async () => {
  const detail = detailFixture("actors:seed", "Viewed creature");
  const content = remasterInformationFixture();
  content.relationships[0].explanation_links = [
    {
      label: "Remaster Changes — Class Features",
      navigation: navigation("journals:changes"),
    },
  ];
  detail.remaster_relationships = { kind: "content", content };
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    show_versions: true,
    remaster_relationships: { kind: "content", content },
  });
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  renderRelated(
    detail,
    client,
    createAtlasRouter(
      createMemoryHistory({ initialEntries: ["/records/actors:seed"] }),
    ),
  );
  openRelated();
  await selectTab("Versions");
  expect(await screen.findByRole("link", { name: "New name" })).toBeVisible();
  api.getRelatedRecords.mockRejectedValueOnce(new Error("Background refresh failed"));
  await act(async () => {
    await client.refetchQueries({ queryKey: ["related"] });
  });
  const explore = screen.getByRole("region", { name: "Explore record" });
  expect(await within(explore).findByRole("alert")).toHaveTextContent(
    "Unable to load connections",
  );
  expect(within(explore).getByRole("button", { name: "Retry" })).toBeVisible();
  expect(within(explore).getAllByText("Renamed to:")).toHaveLength(1);
  expect(within(explore).getAllByRole("link", { name: "New name" })).toHaveLength(1);
  expect(
    within(explore).queryByRole("link", { name: "Remaster Changes — Class Features" }),
  ).toBeNull();
  expect(explore.querySelector(".record-edition-notice")).toBeNull();
});

it("names multiple root comparison actions and keeps them below the relationship", async () => {
  const content = remasterInformationFixture();
  content.relationships[0].parts = [
    {
      kind: "reference",
      label: "First alternative",
      navigation: navigation("actors:first"),
    },
    { kind: "text", text: " and " },
    {
      kind: "reference",
      label: "Second alternative",
      navigation: navigation("actors:second"),
    },
  ];
  api.getRelatedRecords.mockResolvedValue({
    ...relatedFixture(),
    show_versions: true,
    remaster_relationships: { kind: "content", content },
  });
  api.getRecordDetail.mockImplementation(async (key: string) =>
    detailFixture(
      key,
      key === "actors:seed"
        ? "Viewed creature"
        : key === "actors:first"
          ? "First alternative"
          : "Second alternative",
    ),
  );
  renderRelated();
  openRelated();
  await selectTab("Versions");
  const line = screen.getByText("Renamed to:").closest("p")!;
  expect(within(line).queryByRole("button")).toBeNull();
  const actions = line.parentElement!.querySelector(".remaster-relationship__actions")!;
  expect(
    within(actions as HTMLElement).getByRole("button", {
      name: "Compare First alternative",
    }),
  ).toBeVisible();
  fireEvent.click(
    within(actions as HTMLElement).getByRole("button", {
      name: "Compare Second alternative",
    }),
  );
  const dialog = await screen.findByRole("dialog", { name: "Variant comparison" });
  expect(
    await within(dialog).findByRole("heading", { name: "Second alternative" }),
  ).toBeVisible();
  expect(
    within(dialog).getByRole("heading", { name: "Viewed creature" }),
  ).toBeVisible();
});
