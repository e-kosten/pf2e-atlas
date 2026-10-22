import { chooseAttributeSort } from "../../test/sortControls";
import type { ResultSort } from "../../generated/atlas";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { AtlasApiError } from "../../api/atlasApi";
import { RecordIdentity } from "../../shared/records/RecordPresentation";
import { currentAtlasRoute } from "../../app/routes";
import { TraitView } from "./TraitView";
import type { TraitDetailView, TraitRecordsRequest } from "../../generated/atlas";
import { summaryFixture } from "../../test/fixtures";

const api = vi.hoisted(() => ({ getTraitDetail: vi.fn(), getTraitRecords: vi.fn() }));
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
  api.getTraitDetail.mockReset().mockResolvedValue(definition);
  api.getTraitRecords.mockReset().mockImplementation(async (request) => ({
    search_profile: request.kind === "similar" ? request.search_profile : null,
    ...(request.kind === "similar"
      ? { sort: request.sort ?? { kind: "relevance" } }
      : {}),
    trait_key: "trait:agile",
    kind: "members",
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
        <TraitView route={{ kind: "trait", traitKey: "trait:agile" }} />
      </SearchProfileProvider>
    </QueryClientProvider>,
  );
  return client;
}
it("loads exact membership with generation guard and native record navigation", async () => {
  setup();
  expect(await screen.findByText("Multiple attack penalty is reduced.")).toBeVisible();
  expect(await screen.findByRole("link", { name: "Dagger" })).toHaveAttribute(
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
  const member = await screen.findByRole("link", { name: "Dagger" });
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
  expect(screen.getByText("Reference", { exact: true })).toBeVisible();
  const traitHeader = heading.closest("header")!;
  expect(traitHeader.querySelector(".record-reference__kind")).toHaveTextContent(
    "Trait",
  );
  expect(heading).not.toHaveTextContent("Trait");
  render(
    <RecordIdentity
      identity={summaryFixture("weapons:rapier", "Rapier")}
      classification="Weapon 0"
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
        ".record-reference__title > .record-reference__classification > .record-reference__kind",
      ),
    ).not.toBeNull();
  }
});

it("offers generation recovery only after a stale association and rereads the trait before continuing", async () => {
  api.getTraitRecords.mockRejectedValueOnce(
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
  fireEvent.click(recovery);
  await screen.findByRole("link", { name: "Dagger" });
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
    source_fingerprint: "current-source",
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
  const link = await screen.findByRole("link", { name: "Claw · Creature" });
  expect(
    new URL(link.getAttribute("href")!, window.location.origin).searchParams.get(
      "selection",
    ),
  ).toBe(JSON.stringify(navigation));
  fireEvent.click(link);
  expect(currentAtlasRoute()).toMatchObject({
    kind: "record",
    recordKey: "actors:creature",
    selection: navigation,
  });
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

it("restores the Similar tab/order from its URL and keeps other association requests unsorted", async () => {
  api.getTraitRecords.mockImplementation(async (request) => ({
    trait_key: request.trait_key,
    kind: request.kind,
    search_profile: request.kind === "similar" ? request.search_profile : null,
    artifact_generation: "generation-one",
    ...(request.kind === "similar"
      ? { sort: request.sort ?? { kind: "relevance" } }
      : {}),
    rows: [],
    page: {
      number: request.page.number,
      size: 20,
      count: 0,
      total: 0n,
      has_more: false,
      next_page: null,
    },
    coverage: null,
  }));
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <SearchProfileProvider id="embeddinggemma">
        <TraitView
          route={{
            kind: "trait",
            traitKey: "trait:agile",
            query: "?tab=similar&sort=level_desc",
          }}
        />
      </SearchProfileProvider>
    </QueryClientProvider>,
  );
  await screen.findByRole("tab", { name: "Similar", selected: true });
  await waitFor(() =>
    expect(api.getTraitRecords.mock.lastCall?.[0]).toMatchObject({
      kind: "similar",
      sort: {
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      } satisfies ResultSort,
      page: { number: 1 },
    }),
  );
  await chooseAttributeSort("unit_price", "asc");
  await waitFor(() =>
    expect(api.getTraitRecords.mock.lastCall?.[0].sort).toEqual({
      kind: "attributes",
      rules: [{ field: "unit_price", direction: "asc" }],
    }),
  );
  expect(location.search).toContain("tab=similar");
  expect(location.search).toContain("sort=unit_price_asc");
  fireEvent.click(screen.getByRole("tab", { name: "References" }));
  await waitFor(() =>
    expect(api.getTraitRecords.mock.lastCall?.[0].kind).toBe("references"),
  );
  expect(api.getTraitRecords.mock.lastCall?.[0]).not.toHaveProperty("sort");
});

it.each([
  { label: "attribute order", first: "sort=level_desc", second: "sort=unit_price_asc" },
  { label: "Random seed", first: "sort=random&seed=42", second: "sort=random&seed=43" },
])(
  "retires a previous page when $label changes A → B → A",
  async ({ first, second }) => {
    api.getTraitRecords.mockImplementation(async (request: TraitRecordsRequest) => ({
      trait_key: request.trait_key,
      kind: request.kind,
      search_profile: request.search_profile,
      artifact_generation: request.expected_artifact_generation,
      sort: request.sort ?? { kind: "relevance" },
      rows: Array.from({ length: 20 }, (_, index) => ({
        record: summaryFixture(
          `weapons:page${request.page.number}-${index}`,
          `Weapon ${request.page.number}:${index}`,
        ),
        navigation: {
          record_key: `weapons:page${request.page.number}-${index}`,
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: null,
        },
      })),
      page: {
        number: request.page.number,
        size: 20,
        count: 20,
        total: 80n,
        has_more: request.page.number < 4,
        next_page: request.page.number < 4 ? request.page.number + 1 : null,
      },
      coverage: null,
    }));
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const content = (query: string) => (
      <QueryClientProvider client={client}>
        <SearchProfileProvider id="embeddinggemma">
          <TraitView
            route={{
              kind: "trait",
              traitKey: "trait:agile",
              query: "?tab=similar&" + query,
            }}
          />
        </SearchProfileProvider>
      </QueryClientProvider>
    );
    const view = render(content(first));
    await screen.findByRole("tab", { name: "Similar", selected: true });
    fireEvent.click(await screen.findByTitle("Next Page"));
    await waitFor(() =>
      expect(api.getTraitRecords.mock.lastCall?.[0].page.number).toBe(2),
    );
    fireEvent.click(screen.getByTitle("Next Page"));
    await waitFor(() =>
      expect(api.getTraitRecords.mock.lastCall?.[0].page.number).toBe(3),
    );
    const original = api.getTraitRecords.mock.lastCall![0].sort;
    view.rerender(content(second));
    await waitFor(() => {
      expect(api.getTraitRecords.mock.lastCall?.[0].sort).not.toEqual(original);
      expect(api.getTraitRecords.mock.lastCall?.[0].page.number).toBe(1);
    });
    view.rerender(content(first));
    await waitFor(() => {
      expect(api.getTraitRecords.mock.lastCall?.[0].sort).toEqual(original);
      expect(api.getTraitRecords.mock.lastCall?.[0].page.number).toBe(1);
    });
  },
);
