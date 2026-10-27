import type { ResultSort } from "../../generated/atlas";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { AtlasApiError } from "../../api/atlasApi";
import { RecordIdentity } from "../../shared/records/RecordPresentation";
import { RouterTestContext } from "../../test/router";
import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import {
  decodeSearchStateFromParams,
  buildSearchRequest,
} from "../../shared/filters/searchState";
import { TraitView } from "./TraitView";
import type { TraitDetailView, TraitRecordsRequest } from "../../generated/atlas";
import { detailFixture, summaryFixture } from "../../test/fixtures";

const api = vi.hoisted(() => ({
  getTraitDetail: vi.fn(),
  getTraitRecords: vi.fn(),
  getRecordDetail: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const definition: TraitDetailView = {
  trait_key: "trait:agile",
  label: "Agile",
  artifact_generation: "generation-one",
  content: {
    body: {
      kind: "html",
      inclusions: [],
      html: "<p>Multiple attack penalty is reduced.</p>",
      controls: [],
    },
    references: [],
  },
  catalogs: ["weaponTraits"],
  source_revision: "source-pin",
  indexing_locale: "en",
};
beforeEach(() => {
  history.replaceState(null, "", "/traits/trait:agile");
  api.getRecordDetail
    .mockReset()
    .mockResolvedValue(detailFixture("actors:creature", "Creature"));
  api.getTraitDetail.mockReset().mockResolvedValue(definition);
  api.getTraitRecords.mockReset().mockImplementation(async (request) => ({
    search_profile: request.kind === "similar" ? request.search_profile : null,
    ...(request.kind === "similar"
      ? { sort: request.sort ?? { kind: "relevance" } }
      : {}),
    trait_key: "trait:agile",
    kind: request.kind,
    artifact_generation: "generation-one",
    rows: [
      {
        record: summaryFixture("weapons:dagger", "Dagger"),
        navigation: {
          record_key: "weapons:dagger",
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: null,
        },
      },
    ],
    page: { number: 1, size: 20, count: 1, total: 1, has_more: false, next_page: null },
    coverage: null,
  }));
});
function setup() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <SearchProfileProvider id="embeddinggemma">
        <RouterTestContext>
          <TraitView route={{ traitKey: "trait:agile" }} />
        </RouterTestContext>
      </SearchProfileProvider>
    </QueryClientProvider>,
  );
  return client;
}
it("loads exact membership with generation guard and native record navigation", async () => {
  setup();
  expect(await screen.findByText("Multiple attack penalty is reduced.")).toBeVisible();
  expect(await screen.findByRole("link", { name: /^Dagger/ })).toHaveAttribute(
    "href",
    expect.stringContaining("/records/weapons:dagger"),
  );
  expect(api.getTraitRecords.mock.calls[0][0]).toMatchObject({
    trait_key: "trait:agile",
    kind: "members",
    expected_artifact_generation: "generation-one",
  });
});
it("retains the definition and focus during failed same-trait refresh", async () => {
  const client = setup();
  expect(await screen.findByText("Multiple attack penalty is reduced.")).toBeVisible();
  const member = await screen.findByRole("link", { name: /^Dagger/ });
  member.focus();
  const scroll = member.closest(".detail-panel")!;
  scroll.scrollTop = 120;
  api.getTraitDetail.mockRejectedValueOnce(new Error("Refresh failed"));
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["trait-detail"] });
  });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Previously loaded content remains below.",
  );
  expect(screen.getByText("Multiple attack penalty is reduced.")).toBeVisible();
  expect(member).toHaveFocus();
  expect(scroll.scrollTop).toBe(120);
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByText("Multiple attack penalty is reduced.")).toBeVisible();
});
it("shares the normal document identity chrome without routine navigation or refresh controls", async () => {
  setup();
  const heading = await screen.findByRole("heading", { name: "Agile" });
  expect(heading.tagName).toBe("H2");
  expect(screen.queryByRole("button", { name: "Back" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Refresh" })).toBeNull();
  expect(screen.queryByText("Reference", { exact: true })).toBeNull();
  const traitHeader = heading.closest("header")!;
  expect(traitHeader.querySelector(".record-reference__kind")).toHaveTextContent(
    "Trait",
  );
  expect(heading).not.toHaveTextContent("Trait");
  render(
    <RecordIdentity
      identity={summaryFixture("weapons:rapier", "Rapier")}
      classification="Weapon 0"
      onReference={vi.fn()}
    />,
  );
  const recordHeader = screen
    .getByRole("heading", { name: "Rapier" })
    .closest("header")!;
  expect(recordHeader.className).toBe(traitHeader.className);
  for (const header of [traitHeader, recordHeader]) {
    expect(
      header.querySelector(".record-reference__title > .record-reference__identity h2"),
    ).not.toBeNull();
    expect(
      header.querySelector(
        ".record-reference__tools .record-reference__classification > .record-reference__kind",
      ),
    ).not.toBeNull();
  }
});

it("offers generation recovery only after a stale association and rereads the trait before continuing", async () => {
  const associationSuccess = api.getTraitRecords.getMockImplementation()!;
  api.getTraitRecords.mockRejectedValue(
    new AtlasApiError(409, "Reference data changed", {
      code: "artifact_generation_changed",
      message: "Reference data changed",
      recoverable_action: "reload_record",
    }),
  );
  const client = setup();
  const recovery = await screen.findByRole("button", {
    name: "Reload trait",
  });
  expect(screen.getByRole("alert")).toHaveTextContent("Reference collection changed");
  api.getTraitDetail.mockResolvedValue({
    ...definition,
    artifact_generation: "generation-two",
  });
  api.getTraitRecords.mockImplementation(associationSuccess);
  fireEvent.click(recovery);
  await screen.findByRole("link", { name: /^Dagger/ });
  expect(
    api.getTraitRecords.mock.calls.some(
      ([request]) => request.expected_artifact_generation === "generation-two",
    ),
  ).toBe(true);
  expect(
    client
      .getQueryCache()
      .findAll({ queryKey: ["trait-records"] })
      .every(
        (query) =>
          (query.queryKey[1] as { expected_artifact_generation: string })
            .expected_artifact_generation === "generation-two",
      ),
  ).toBe(true);
  expect(screen.queryByRole("button", { name: "Reload trait" })).toBeNull();
});

it("keeps an unavailable source definition explicit", async () => {
  api.getTraitDetail.mockResolvedValueOnce({
    ...definition,
    content: null,
  });
  setup();
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "No definition is available in this reference collection.",
  );
});

it("keeps exact embedded membership addresses in native links and navigation", async () => {
  const navigation = {
    record_key: "actors:creature",
    embedded_path: [{ collection: "/items", identity: { Stable: "claw" } }],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
  api.getTraitRecords.mockResolvedValueOnce({
    trait_key: "trait:agile",
    kind: "members",
    artifact_generation: "generation-one",
    search_profile: null,
    rows: [
      {
        record: summaryFixture("actors:creature", "Creature"),
        source_label: "Claw",
        navigation,
      },
    ],
    page: { number: 1, size: 20, count: 1, total: 1, has_more: false, next_page: null },
    coverage: null,
  });
  setup();
  const link = await screen.findByRole("link", { name: /Creature.*Trait on Claw/ });
  const target = new URL(link.getAttribute("href")!, window.location.origin);
  expect(target.searchParams.has("selection")).toBe(false);
  expect(
    selectedNavigation(parseSearchParams(target.search), navigation.record_key),
  ).toEqual(navigation);
  expect(fireEvent.click(link, { metaKey: true })).toBe(true);
  expect(screen.queryByRole("dialog")).toBeNull();
  fireEvent.click(link);
  const drawer = await screen.findByRole("dialog");
  expect(drawer).toBeVisible();
  expect(api.getRecordDetail).toHaveBeenCalledWith(
    "actors:creature",
    expect.objectContaining({ embedded_path: navigation.embedded_path }),
    expect.any(AbortSignal),
  );
  expect(location.pathname).toBe("/traits/trait:agile");
});
it("requests stored Similar with the selected profile and no text fallback", async () => {
  setup();
  fireEvent.click(await screen.findByRole("tab", { name: "Similar" }));
  await screen.findByRole("region", { name: "Similar records" });
  expect(
    api.getTraitRecords.mock.calls.some(
      ([request]) =>
        request.kind === "similar" &&
        request.search_profile === "embeddinggemma" &&
        request.trait_key === "trait:agile",
    ),
  ).toBe(true);
});

it("restores Similar order from its URL and carries it into primary Search", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <SearchProfileProvider id="embeddinggemma">
        <RouterTestContext>
          <TraitView
            route={{ traitKey: "trait:agile", query: "?tab=similar&sort=level_desc" }}
          />
        </RouterTestContext>
      </SearchProfileProvider>
    </QueryClientProvider>,
  );
  await screen.findByRole("tab", { name: "Similar", selected: true });
  await waitFor(() =>
    expect(api.getTraitRecords.mock.lastCall?.[0]).toMatchObject({
      kind: "similar",
      page: { number: 1, size: 5 },
      sort: { kind: "attributes", rules: [{ field: "level", direction: "desc" }] },
    } satisfies Partial<TraitRecordsRequest>),
  );
  const link = screen.getByRole("link", { name: "Search similar records" });
  const url = new URL(link.getAttribute("href")!, location.origin);
  expect(url.pathname).toBe("/search");
  const search = decodeSearchStateFromParams(url.searchParams);
  expect(search.sort).toEqual({
    kind: "attributes",
    rules: [{ field: "level", direction: "desc" }],
  } satisfies ResultSort);
  expect(buildSearchRequest(search, 1).criteria).toMatchObject({
    kind: "similar_traits",
    seed_trait_key: "trait:agile",
  });
  fireEvent.click(screen.getByRole("tab", { name: "References" }));
  await screen.findByRole("region", { name: "Trait references" });
  expect(
    api.getTraitRecords.mock.calls
      .filter(([r]) => r.kind !== "similar")
      .every(([r]) => !r.sort),
  ).toBe(true);
});

it.each([
  { label: "attribute order", first: "sort=level_desc", second: "sort=unit_price_asc" },
  { label: "Random seed", first: "sort=random&seed=42", second: "sort=random&seed=43" },
])(
  "keeps a single bounded preview when $label changes A → B → A",
  async ({ first, second }) => {
    api.getTraitRecords.mockImplementation(async (request: TraitRecordsRequest) => ({
      trait_key: request.trait_key,
      kind: request.kind,
      search_profile: request.kind === "similar" ? request.search_profile : null,
      artifact_generation: request.expected_artifact_generation,
      sort: request.sort ?? { kind: "relevance" },
      rows: [
        {
          record: summaryFixture("weapons:dagger", "Dagger"),
          navigation: {
            record_key: "weapons:dagger",
            embedded_path: [],
            field: null,
            passage: null,
            source_fingerprint: null,
          },
        },
      ],
      page: { number: 1, size: 5, count: 1, total: 80n, has_more: true, next_page: 2 },
      coverage: null,
    }));
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const content = (query: string) => (
      <QueryClientProvider client={client}>
        <SearchProfileProvider id="embeddinggemma">
          <RouterTestContext>
            <TraitView
              route={{ traitKey: "trait:agile", query: "?tab=similar&" + query }}
            />
          </RouterTestContext>
        </SearchProfileProvider>
      </QueryClientProvider>
    );
    const view = render(content(first));
    await screen.findByRole("tab", { name: "Similar", selected: true });
    await waitFor(() =>
      expect(api.getTraitRecords.mock.lastCall?.[0].kind).toBe("similar"),
    );
    const original = api.getTraitRecords.mock.lastCall![0].sort;
    view.rerender(content(second));
    await waitFor(() =>
      expect(api.getTraitRecords.mock.lastCall?.[0].sort).not.toEqual(original),
    );
    view.rerender(content(first));
    await waitFor(() =>
      expect(
        new URL(
          screen
            .getByRole("link", { name: "Search similar records" })
            .getAttribute("href")!,
          location.origin,
        ).searchParams.toString(),
      ).toContain(first),
    );
    expect(
      api.getTraitRecords.mock.calls.every(
        ([r]) => r.page.number === 1 && r.page.size === 5,
      ),
    ).toBe(true);
    expect(screen.queryByTitle("Next Page")).toBeNull();
    expect(screen.queryByText(/See more|Browse all/i)).toBeNull();
  },
);

it.each(["members", "references", "similar"] as const)(
  "opens primary Search with exact %s criteria and no inline full-result mode",
  async (kind) => {
    setup();
    await screen.findByRole("tab", { name: "With this trait" });
    if (kind !== "members")
      fireEvent.click(
        screen.getByRole("tab", {
          name: kind === "references" ? "References" : "Similar",
        }),
      );
    const link = await screen.findByRole("link", {
      name:
        kind === "members"
          ? "Search records with this trait"
          : kind === "references"
            ? "Search trait references"
            : "Search similar records",
    });
    expect(link).not.toHaveClass("ant-btn-primary");
    const url = new URL(link.getAttribute("href")!, location.origin);
    expect(url.pathname).toBe("/search");
    const search = decodeSearchStateFromParams(url.searchParams);
    expect(url.searchParams.get("search_profile")).toBe("embeddinggemma");
    expect(buildSearchRequest(search, 1).criteria).toMatchObject(
      kind === "members"
        ? {
            kind: "list_records",
            filter: {
              kind: "set_match",
              field: "traits",
              op: "includes_any",
              values: ["agile"],
            },
          }
        : kind === "references"
          ? { kind: "trait_references", trait_key: "trait:agile" }
          : { kind: "similar_traits", seed_trait_key: "trait:agile" },
    );
    expect(fireEvent.click(link, { metaKey: true })).toBe(true);
    expect(screen.queryByTitle("Next Page")).toBeNull();
  },
);

it("hides known-empty associations while retaining Similar", async () => {
  api.getTraitRecords.mockImplementation(async (request: TraitRecordsRequest) => ({
    trait_key: request.trait_key,
    kind: request.kind,
    search_profile: request.kind === "similar" ? request.search_profile : null,
    artifact_generation: request.expected_artifact_generation,
    sort: { kind: "relevance" },
    rows: [],
    page: { number: 1, size: 5, count: 0, total: 0n, has_more: false, next_page: null },
    coverage: null,
  }));
  setup();
  expect(
    await screen.findByRole("tab", { name: "Similar", selected: true }),
  ).toBeVisible();
  expect(screen.queryByRole("tab", { name: "References" })).toBeNull();
  expect(screen.queryByRole("tab", { name: "With this trait" })).toBeNull();
});
