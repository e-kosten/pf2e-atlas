import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { AtlasApiError, searchPage } from "../../api/atlasApi";
import type {
  ReferenceFilterActionView,
  SearchResultsPage,
  SearchResultsRequest,
} from "../../generated/atlas";
import { fieldFixture, summaryFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { decodeFilterUrl } from "../filters/filterUrl";
import { DefinitionAssociations } from "./DefinitionAssociations";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  searchPage: vi.fn(),
}));
const api = vi.mocked(searchPage);
const action: Extract<ReferenceFilterActionView, { kind: "search" }> = {
  kind: "search",
  label: "With this language",
  artifact_generation: "generation-a",
  predicate: {
    kind: "set_match",
    field: "languages",
    op: "includes_any",
    values: ["chthonian"],
  },
};
const numeric: ReferenceFilterActionView = {
  kind: "configure_field",
  label: "Strength Modifier",
  field: fieldFixture("abilities.str.modifier", "number", "Strength Modifier"),
};
function page(request: SearchResultsRequest, count = 5): SearchResultsPage {
  return {
    search_content: "all",
    ranking: "score",
    sort: { kind: "attributes", rules: [{ field: "name", direction: "asc" }] },
    search_profile: request.search_profile ?? "bge-base",
    artifact_generation: request.expected_artifact_generation!,
    mode: { kind: "list_records" },
    coverage: null,
    relationship_clauses: [],
    rows: Array.from({ length: count }, (_, index) => ({
      record: summaryFixture(`actors:speaker-${index}`, `Speaker ${index}`),
      matches: [],
      relationship_evidence: [],
    })),
    page: {
      number: 1,
      size: 5,
      count,
      total: count ? 88n : 0n,
      has_more: count > 0,
      next_page: count ? 2 : null,
    },
  };
}
function setup(actions: ReferenceFilterActionView[] = [action]) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const onReference = vi.fn();
  const onReload = vi.fn();
  const result = render(
    <RouterTestContext>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider client={client}>
          <SearchProfileProvider>
            <DefinitionAssociations
              actions={actions}
              generation="generation-a"
              onReference={onReference}
              onReload={onReload}
            />
          </SearchProfileProvider>
        </QueryClientProvider>
      </ConfigProvider>
    </RouterTestContext>,
  );
  return { ...result, client, onReference, onReload };
}
beforeEach(() => {
  vi.clearAllMocks();
  api.mockImplementation(async (request) => page(request));
});

it("uses the supplied exact predicate for a five-row preview, count, and full Search link", async () => {
  const { onReference } = setup([action, numeric]);
  expect(await screen.findByText("5 of 88 entries")).toBeVisible();
  expect(screen.getAllByRole("listitem")).toHaveLength(5);
  expect(screen.getByRole("tab", { name: "With this language" })).toBeVisible();
  expect(screen.queryByRole("tab", { name: "Similar" })).toBeNull();
  expect(screen.queryByText("Strength Modifier")).toBeNull();
  expect(api).toHaveBeenCalledWith(
    {
      search_profile: "bge-base",
      criteria: { kind: "list_records", filter: action.predicate },
      page: { number: 1, size: 5 },
      expected_artifact_generation: "generation-a",
    },
    expect.any(AbortSignal),
  );
  const search = screen.getByRole("link", { name: "Search records" });
  const url = new URL(search.getAttribute("href")!, window.location.origin);
  expect(decodeFilterUrl(url.searchParams.get("filter")!)).toMatchObject(
    action.predicate,
  );
  expect(url.searchParams.get("search_profile")).toBe("bge-base");
  fireEvent.click(
    within(screen.getAllByRole("listitem")[0]).getByRole("link", { name: /Speaker 0/ }),
  );
  expect(onReference).toHaveBeenCalledWith("actors:speaker-0", expect.any(HTMLElement));
});

it("keeps distinct supplied relations in tabs, including a known-empty result", async () => {
  const alternate: typeof action = {
    ...action,
    label: "With this alternate domain",
    predicate: {
      kind: "set_match",
      field: "deity.alternate_domains",
      op: "includes_any",
      values: ["knowledge"],
    },
  };
  api.mockImplementation(async (request) => page(request, 0));
  setup([action, alternate]);
  expect(await screen.findByText("0 of 0 entries")).toBeVisible();
  expect(screen.getByText("No matching records")).toBeVisible();
  fireEvent.click(screen.getByRole("tab", { name: alternate.label }));
  await waitFor(() =>
    expect(api).toHaveBeenLastCalledWith(
      expect.objectContaining({
        criteria: { kind: "list_records", filter: alternate.predicate },
      }),
      expect.any(AbortSignal),
    ),
  );
  expect(screen.getByRole("tab", { name: action.label })).toBeVisible();
});

it("removes invalidated-generation rows and reloads the definition rather than retrying stale criteria", async () => {
  const { client, onReload } = setup();
  await screen.findByText("5 of 88 entries");
  api.mockRejectedValue(
    new AtlasApiError(409, "Changed", {
      code: "artifact_generation_changed",
      message: "Changed",
      recoverable_action: "refresh_results",
    }),
  );
  await act(async () => {
    await client.refetchQueries({ queryKey: ["definition-association-preview"] });
  });
  expect(await screen.findByText("Reference collection changed")).toBeVisible();
  expect(screen.queryByText("Speaker 0")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Reload definition" }));
  expect(onReload).toHaveBeenCalledOnce();
});

it("does not create an empty standalone block for numeric-only actions", () => {
  setup([numeric]);
  expect(screen.queryByRole("region", { name: "Related records" })).toBeNull();
  expect(api).not.toHaveBeenCalled();
});
