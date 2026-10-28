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
import { RouterTestContext } from "../../test/router";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
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
import { detailFixture, editorFixture, summaryFixture } from "../../test/fixtures";
import { useSearchWorkspace, type SearchWorkspaceState } from "./useSearchWorkspace";
import { SimilarSearchButton } from "./SimilarSearchButton";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
}));
const selectedKey = "equipment:seed-record";
function setup(initial: SearchFormState) {
  history.replaceState(
    {},
    "",
    "/search?" + searchStateQueryString(initial) + "&page=3",
  );
  let live!: SearchWorkspaceState;
  function Harness() {
    live = useSearchWorkspace({ enabled: false, searchProfile: "bge-base" });
    return <SimilarSearchButton workspace={live} />;
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <RouterTestContext>
          <SearchProfileProvider>
            <Harness />
          </SearchProfileProvider>
        </RouterTestContext>
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return () => live;
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
  "chooses a seed independently of the parent %s search and retains filters",
  async (mode) => {
    const current = setup({
      ...DEFAULT_SEARCH_STATE,
      query: mode === "text" ? "fire" : "",
      similarSeed: mode === "similar" ? "equipment:old-seed" : null,
      filter: { kind: "compare", field: "level", op: "gte", value: 5 },
      sort: { kind: "random", seed: 42 },
    });
    const before = current().search;
    const trigger = screen.getByRole("button", {
      name: mode === "similar" ? "Change similar record…" : "Similar to…",
    });
    act(() => trigger.focus());
    fireEvent.click(trigger);
    const dialog = await screen.findByRole("dialog", { name: "Find similar records" });
    expect(vi.mocked(searchPage).mock.calls[0][0].criteria).toEqual({
      kind: "list_records",
      filter: null,
    });
    expect(within(dialog).getByRole("button", { name: "Find similar" })).toBeDisabled();
    fireEvent.click(await within(dialog).findByRole("link", { name: /Leather Armor/ }));
    await waitFor(() =>
      expect(
        within(dialog).getByRole("button", { name: "Find similar" }),
      ).toBeEnabled(),
    );
    expect(current().search).toEqual(before);
    fireEvent.click(within(dialog).getByRole("button", { name: "Find similar" }));
    await waitFor(() =>
      expect(current().search).toMatchObject({
        ...before,
        query: "",
        similarSeed: selectedKey,
        traitReferenceSeed: null,
        mode: "similar_records",
      }),
    );
    await waitFor(() => expect(dialog).not.toBeVisible());
    expect(current().pageNumber).toBe(1);
  },
);
it("cancels without changing the parent criteria or page", async () => {
  const current = setup({ ...DEFAULT_SEARCH_STATE, query: "fire" });
  const before = current().search;
  fireEvent.click(screen.getByRole("button", { name: "Similar to…" }));
  const dialog = await screen.findByRole("dialog", { name: "Find similar records" });
  fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  await waitFor(() => expect(dialog).not.toBeVisible());
  expect(current().search).toEqual(before);
  expect(current().pageNumber).toBe(3);
});
