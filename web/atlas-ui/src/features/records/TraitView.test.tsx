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
  searchPage: vi.fn(),
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
  api.searchPage.mockReset();
  api.getTraitRecords.mockReset().mockImplementation(async (request) => ({
    search_profile: null,
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
function setup(query?: string) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <SearchProfileProvider id="embeddinggemma">
        <RouterTestContext>
          <TraitView route={{ traitKey: "trait:agile", query }} />
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
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  expect(api.getTraitDetail).toHaveBeenCalledTimes(3);
  expect(screen.getByText("Multiple attack penalty is reduced.")).toBeVisible();
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
  expect(screen.getByRole("status")).toHaveTextContent("Reference collection changed");
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
  const associations = client.getQueryCache().findAll({ queryKey: ["trait-records"] });
  expect(associations).toHaveLength(2);
  for (const query of associations)
    expect(query.queryKey[1]).toMatchObject({
      trait_key: "trait:agile",
      expected_artifact_generation: "generation-two",
    });
  expect(screen.queryByRole("button", { name: "Reload trait" })).toBeNull();
});

it.each([undefined, "?tab=references", "?tab=similar"])(
  "uses the generic missing destination for a bodyless trait URL %s",
  async (query) => {
    api.getTraitDetail.mockResolvedValueOnce({
      ...definition,
      content: null,
    });
    setup(query);
    expect(await screen.findByText("Link not found")).toBeVisible();
    expect(screen.queryByRole("heading", { name: "Agile" })).toBeNull();
    expect(
      screen.queryByText("No definition is available in this reference collection."),
    ).toBeNull();
    expect(screen.queryByRole("tab")).toBeNull();
    expect(api.getTraitRecords).not.toHaveBeenCalled();
    expect(api.searchPage).not.toHaveBeenCalled();
    expect(document.title).toBe("Link not found · Atlas");
  },
);
it("stops presenting the trait page when a successful refresh removes its content", async () => {
  const client = setup();
  expect(await screen.findByText("Multiple attack penalty is reduced.")).toBeVisible();
  await screen.findByRole("tab", { name: "With this trait" });
  api.getTraitDetail.mockResolvedValue({ ...definition, content: null });
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["trait-detail"] });
  });
  expect(await screen.findByText("Link not found")).toBeVisible();
  expect(screen.queryByText("Multiple attack penalty is reduced.")).toBeNull();
  expect(screen.queryByRole("tab")).toBeNull();
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
it("uses the default association when a stale link selects Similar", async () => {
  setup("?tab=similar");
  expect(
    await screen.findByRole("tab", { name: "With this trait", selected: true }),
  ).toBeVisible();
  expect(screen.queryByRole("tab", { name: "Similar" })).toBeNull();
  expect(api.searchPage).not.toHaveBeenCalled();
});

it.each(["members", "references"] as const)(
  "opens primary Search with exact %s criteria and no inline full-result mode",
  async (kind) => {
    setup();
    await screen.findByRole("tab", { name: "With this trait" });
    if (kind !== "members")
      fireEvent.click(
        screen.getByRole("tab", {
          name: "References",
        }),
      );
    const link = await screen.findByRole("link", {
      name:
        kind === "members"
          ? "Search records with this trait"
          : "Search trait references",
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
        : { kind: "trait_references", trait_key: "trait:agile" },
    );
    expect(fireEvent.click(link, { metaKey: true })).toBe(true);
    expect(screen.queryByRole("button", { name: "Next Page" })).toBeNull();
  },
);

it("hides known-empty associations", async () => {
  api.getTraitRecords.mockImplementation(async (request: TraitRecordsRequest) => ({
    trait_key: request.trait_key,
    kind: request.kind,
    search_profile: null,
    artifact_generation: request.expected_artifact_generation,
    sort: { kind: "relevance" },
    rows: [],
    page: { number: 1, size: 5, count: 0, total: 0n, has_more: false, next_page: null },
    coverage: null,
  }));
  setup();
  await screen.findByText("Multiple attack penalty is reduced.");
  await waitFor(() =>
    expect(screen.queryByLabelText("Loading connections…")).toBeNull(),
  );
  expect(screen.queryByRole("tab", { name: "Similar" })).toBeNull();
  expect(screen.queryByRole("tab", { name: "References" })).toBeNull();
  expect(screen.queryByRole("tab", { name: "With this trait" })).toBeNull();
});
