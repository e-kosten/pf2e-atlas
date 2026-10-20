import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen } from "@testing-library/react";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { currentAtlasRoute } from "../../app/routes";
import { TraitView } from "./TraitView";
import type { TraitDetailView } from "../../generated/atlas";
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
  api.getTraitRecords.mockReset().mockResolvedValue({
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
  });
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
  const refresh = screen.getByRole("button", { name: "Refresh" });
  refresh.focus();
  api.getTraitDetail.mockRejectedValueOnce(new Error("Refresh failed"));
  await act(async () => {
    await client.invalidateQueries({ queryKey: ["trait-detail"] });
  });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Previously loaded content remains below.",
  );
  expect(screen.getByText("Multiple attack penalty is reduced.")).toBeVisible();
  expect(refresh).toHaveFocus();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByText("Multiple attack penalty is reduced.")).toBeVisible();
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
