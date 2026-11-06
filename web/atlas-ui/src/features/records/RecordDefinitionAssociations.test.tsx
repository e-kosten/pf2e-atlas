import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App, ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import {
  AtlasApiError,
  getDefinitionDetail,
  getRecordDetail,
  searchPage,
} from "../../api/atlasApi";
import type { DefinitionDetailView } from "../../generated/atlas";
import { detailFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { RecordView } from "./RecordViews";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionDetail: vi.fn(),
  getRecordDetail: vi.fn(),
  searchPage: vi.fn(),
}));
const definitionApi = vi.mocked(getDefinitionDetail);
const recordApi = vi.mocked(getRecordDetail);
const searchApi = vi.mocked(searchPage);
const recordKey = "bestiary-ability-glossary:darkvision";
function property(
  generation = "test-generation",
  destinationRecord = recordKey,
): DefinitionDetailView {
  return {
    artifact_generation: generation,
    definition_key: "sense:darkvision",
    label: "Darkvision",
    help: {
      definition_key: "sense:darkvision",
      availability: "available",
      destination: {
        kind: "record",
        reading_context: "general_definition",
        navigation: {
          record_key: destinationRecord,
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: null,
        },
      },
      related_filters: [
        {
          kind: "search",
          label: "With this sense",
          artifact_generation: generation,
          predicate: {
            kind: "exists",
            collection: "senses",
            scope_id: "same-sense",
            predicate: {
              kind: "compare",
              field: "type",
              op: "eq",
              value: "darkvision",
            },
          },
        },
      ],
    },
    content: null,
    attribution: null,
    notices: [],
    references: [],
    references_availability: "available",
    references_truncated: false,
  };
}
function setup() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <RouterTestContext>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>
          <QueryClientProvider client={client}>
            <SearchProfileProvider>
              <RecordView
                route={{
                  recordKey,
                  definitionKey: "sense:darkvision",
                  readingContext: "general_definition",
                }}
              />
            </SearchProfileProvider>
          </QueryClientProvider>
        </App>
      </ConfigProvider>
    </RouterTestContext>,
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  history.replaceState(
    null,
    "",
    `/records/${recordKey}?definition_key=sense%3Adarkvision&reading_context=general_definition`,
  );
  const detail = detailFixture(recordKey, "Darkvision", "dummy:target");
  detail.presentation.content[0].body = {
    kind: "plain",
    text: "Actual existing ability rules.",
  };
  recordApi.mockResolvedValue(detail);
  definitionApi.mockResolvedValue(property());
  searchApi.mockResolvedValue({
    search_content: "all",
    ranking: "score",
    sort: { kind: "attributes", rules: [{ field: "name", direction: "asc" }] },
    search_profile: "bge-base",
    artifact_generation: "test-generation",
    mode: { kind: "list_records" },
    coverage: null,
    relationship_clauses: [],
    rows: [],
    page: { number: 1, size: 5, count: 0, total: 0n, has_more: false, next_page: null },
  });
});

it("adds matched property associations after the actual record body and preserves ordinary Explore", async () => {
  const { container } = setup();
  expect(await screen.findByText("Actual existing ability rules.")).toBeVisible();
  expect(await screen.findByRole("tab", { name: "With this sense" })).toBeVisible();
  expect(container.textContent!.indexOf("Actual existing ability rules.")).toBeLessThan(
    container.textContent!.indexOf("With this sense"),
  );
  expect(screen.getByRole("button", { name: "Explore related records" })).toBeVisible();
  const action = property().help.related_filters[0];
  expect(searchApi).toHaveBeenCalledWith(
    expect.objectContaining({
      criteria: {
        kind: "list_records",
        filter: action.kind === "search" ? action.predicate : null,
      },
    }),
    expect.any(AbortSignal),
  );
});

it("does not lend a property association to a different backend destination", async () => {
  const wrong = property();
  if (wrong.help.destination?.kind === "record")
    wrong.help.destination.navigation.record_key = "other:record";
  definitionApi.mockResolvedValue(wrong);
  setup();
  expect(
    await screen.findByText("This definition no longer points to this entry."),
  ).toBeVisible();
  expect(screen.getByText("Actual existing ability rules.")).toBeVisible();
  expect(screen.queryByRole("tab", { name: "With this sense" })).toBeNull();
  expect(searchApi).not.toHaveBeenCalled();
});

it.each(["matching", "changed"] as const)(
  "refreshes the record before revalidating a %s property mapping on the new generation",
  async (mapping) => {
    const first = detailFixture(recordKey, "Darkvision", "dummy:target");
    first.artifact_generation = "generation-one";
    first.presentation.content[0].body = {
      kind: "plain",
      text: "Generation one rules.",
    };
    const next = { ...first, artifact_generation: "generation-two" };
    recordApi.mockResolvedValueOnce(first).mockResolvedValue(next);
    definitionApi.mockImplementation(async (request) => {
      if (request.expected_artifact_generation === "generation-one")
        throw new AtlasApiError(409, "Changed", {
          code: "artifact_generation_changed",
          message: "Changed",
          recoverable_action: "refresh_results",
        });
      return property(
        request.expected_artifact_generation ?? "generation-one",
        mapping === "changed" &&
          request.expected_artifact_generation === "generation-two"
          ? "other:record"
          : recordKey,
      );
    });
    const searchSuccess = searchApi.getMockImplementation()!;
    searchApi.mockImplementation(async (request, signal) => ({
      ...(await searchSuccess(request, signal)),
      artifact_generation: request.expected_artifact_generation!,
    }));
    setup();
    const url = new URL(window.location.href);
    const refresh = await screen.findByRole("button", { name: "Refresh" });
    const oldRequests = definitionApi.mock.calls.filter(
      ([request]) => request.expected_artifact_generation === "generation-one",
    ).length;
    expect(screen.getByText("Reference collection changed")).toBeVisible();
    fireEvent.click(refresh);
    await waitFor(() =>
      expect(definitionApi).toHaveBeenCalledWith(
        {
          definition_key: "sense:darkvision",
          expected_artifact_generation: "generation-two",
        },
        expect.any(AbortSignal),
      ),
    );
    expect(
      definitionApi.mock.calls.filter(
        ([request]) => request.expected_artifact_generation === "generation-one",
      ),
    ).toHaveLength(oldRequests);
    expect(recordApi).toHaveBeenCalledWith(
      recordKey,
      {
        record_key: recordKey,
        embedded_path: [],
        fields: [],
        passage: null,
        source_fingerprint: null,
        reading_context: "general_definition",
      },
      expect.any(AbortSignal),
    );
    const refreshedUrl = new URL(window.location.href);
    expect(refreshedUrl.pathname).toBe(url.pathname);
    expect([...refreshedUrl.searchParams.entries()].sort()).toEqual(
      [...url.searchParams.entries()].sort(),
    );
    expect(refreshedUrl.hash).toBe(url.hash);
    if (mapping === "matching") {
      expect(await screen.findByRole("tab", { name: "With this sense" })).toBeVisible();
      await waitFor(() =>
        expect(searchApi).toHaveBeenCalledWith(
          expect.objectContaining({ expected_artifact_generation: "generation-two" }),
          expect.any(AbortSignal),
        ),
      );
    } else {
      expect(
        await screen.findByText("This definition no longer points to this entry."),
      ).toBeVisible();
      expect(screen.queryByRole("tab", { name: "With this sense" })).toBeNull();
      expect(
        searchApi.mock.calls.filter(
          ([request]) => request.expected_artifact_generation === "generation-two",
        ),
      ).toHaveLength(0);
    }
    expect(screen.queryByRole("button", { name: "Refresh" })).toBeNull();
  },
);

it("keeps ordinary property request failures on local Retry without reloading the record", async () => {
  let failed = true;
  definitionApi.mockImplementation(async (request) => {
    if (request.expected_artifact_generation && failed)
      throw new Error("Network interrupted");
    return property();
  });
  setup();
  const retry = await screen.findByRole("button", { name: "Retry" });
  const recordRequests = recordApi.mock.calls.length;
  failed = false;
  fireEvent.click(retry);
  expect(await screen.findByRole("tab", { name: "With this sense" })).toBeVisible();
  expect(recordApi).toHaveBeenCalledTimes(recordRequests);
  expect(definitionApi).toHaveBeenLastCalledWith(
    {
      definition_key: "sense:darkvision",
      expected_artifact_generation: "test-generation",
    },
    expect.any(AbortSignal),
  );
});
