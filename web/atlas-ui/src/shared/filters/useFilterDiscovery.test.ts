import {
  findClauseIds,
  useFilterDiscovery,
  useFilterOptionCounts,
  useFilterVocabulary,
} from "./useFilterDiscovery";
import type { FilterDiscoveryContext, QueryPredicate } from "../../generated/atlas";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { createElement, type ReactNode } from "react";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import { invalidateSavedListQueries } from "../../features/lists/savedListQueries";

const api = vi.hoisted(() => ({
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
  discoverFilterValues: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
function wrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return function QueryWrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client }, children);
  };
}
const context: FilterDiscoveryContext = {
  kind: "filtered",
  search_profile: "bge-base",
  expected_artifact_generation: "generation-a",
  filter: null,
  text: null,
  mode: "hybrid",
};
function response(profile = "bge-base", generation = "generation-a", count = 3n) {
  return {
    search_profile: profile,
    artifact_generation: generation,
    counts: {
      status: "available",
      value: {
        field: "traits",
        options: [
          { value: "fire", distinct_roots: count, selected: false },
          { value: "zero", distinct_roots: 0n, selected: false },
        ],
        total_values: 2n,
        exhaustive: true,
        count_basis: "exact counterfactual displayed roots",
      },
    },
  };
}
beforeEach(() => {
  vi.clearAllMocks();
  api.discoverFilterEditor.mockResolvedValue(
    editorFixture([fieldFixture("traits", "set")]),
  );
  api.discoverFilterCounts.mockResolvedValue(response());
  api.discoverFilterValues.mockImplementation((request) =>
    Promise.resolve({
      kind: "vocabulary",
      scope: request.scope,
      values: {
        field: request.field_id,
        options: [{ value: "fire" }],
        selected_values: request.selected_values,
        total_values: 90n,
        basis: "product_artifact_before_edition_preference",
      },
    }),
  );
});
it("addresses root and child clauses separately without semantic eligibility guesses", () => {
  const predicate: QueryPredicate = {
    kind: "all_of",
    children: [
      { kind: "compare", clause_id: "root", field: "item.level", op: "eq", value: 4 },
      {
        kind: "exists",
        collection: "actor.items",
        scope_id: "items",
        predicate: {
          kind: "compare",
          clause_id: "child",
          field: "item.level",
          op: "eq",
          value: 2,
        },
      },
    ],
  };
  expect(findClauseIds(predicate, "item.level")).toEqual(["root"]);
  expect(findClauseIds(predicate, "item.level", "actor.items")).toEqual(["child"]);
});
it("loads catalog without page-load count fanout", async () => {
  const { result } = renderHook(
    () => useFilterDiscovery({ context, queryKeyPrefix: ["catalog"] }),
    { wrapper: wrapper() },
  );
  await waitFor(() => expect(result.current.filterEditor).toBeDefined());
  expect(api.discoverFilterCounts).not.toHaveBeenCalled();
  expect(result.current.errorMessage).toBeNull();
});
it("requests only active bounded page literals and preserves known zero versus missing", async () => {
  const { result, rerender } = renderHook(
    ({ open, values }) =>
      useFilterOptionCounts(context, "traits", null, values, open, 3),
    {
      wrapper: wrapper(),
      initialProps: {
        open: false,
        values: ["fire", "zero", "fire", "retained", "overflow"],
      },
    },
  );
  expect(api.discoverFilterCounts).not.toHaveBeenCalled();
  rerender({ open: true, values: ["fire", "zero", "fire", "retained", "overflow"] });
  await waitFor(() => expect(result.current.get("fire")).toBe(3n));
  expect(result.current.get("zero")).toBe(0n);
  expect(result.current.has("retained")).toBe(false);
  expect(api.discoverFilterCounts).toHaveBeenCalledWith(
    {
      context,
      field_id: "traits",
      clause_id: null,
      values: ["fire", "zero", "retained"],
    },
    expect.any(AbortSignal),
  );
});
it("does not request text-aware or unavailable execution contexts", () => {
  renderHook(
    () =>
      useFilterOptionCounts(
        { ...context, text: "dragon" },
        "traits",
        null,
        ["fire"],
        true,
        128,
      ),
    { wrapper: wrapper() },
  );
  renderHook(
    () => useFilterOptionCounts(undefined, "traits", null, ["fire"], true, 128),
    { wrapper: wrapper() },
  );
  expect(api.discoverFilterCounts).not.toHaveBeenCalled();
});
it("abandons a count read when its popup closes without issuing a disabled request", async () => {
  let signal!: AbortSignal;
  api.discoverFilterCounts.mockImplementationOnce((_request, activeSignal) => {
    signal = activeSignal;
    return new Promise(() => {});
  });
  const { result, rerender } = renderHook(
    ({ open }) => useFilterOptionCounts(context, "traits", null, ["fire"], open, 128),
    { wrapper: wrapper(), initialProps: { open: true } },
  );
  await waitFor(() => expect(api.discoverFilterCounts).toHaveBeenCalledTimes(1));
  rerender({ open: false });
  expect(signal.aborted).toBe(true);
  expect(result.current.size).toBe(0);
  expect(api.discoverFilterCounts).toHaveBeenCalledTimes(1);
});
it.each(["unavailable", "error"])(
  "keeps %s optional counts absent while vocabulary is usable",
  async (kind) => {
    if (kind === "unavailable")
      api.discoverFilterCounts.mockResolvedValue({
        ...response(),
        counts: { status: "unavailable", reason: "negative_condition" },
      });
    else api.discoverFilterCounts.mockRejectedValue(new Error("SQL failure"));
    const { result } = renderHook(
      () => ({
        counts: useFilterOptionCounts(
          context,
          "traits",
          "negative",
          ["fire"],
          true,
          128,
        ),
        values: useFilterVocabulary({ kind: "artifact" }, "traits", "", 0, [
          "retained",
        ]),
      }),
      { wrapper: wrapper() },
    );
    await waitFor(() => expect(result.current.values.data).toBeDefined());
    expect(result.current.counts.size).toBe(0);
    expect(result.current.values.data?.values.selected_values).toEqual(["retained"]);
  },
);
it("isolates late context/profile/generation/page completions and aborts abandoned requests", async () => {
  let finish!: (value: unknown) => void;
  let oldSignal!: AbortSignal;
  api.discoverFilterCounts
    .mockImplementationOnce((_request, signal) => {
      oldSignal = signal;
      return new Promise((resolve) => {
        finish = resolve;
      });
    })
    .mockResolvedValue(response("embeddinggemma", "generation-b", 9n));
  const { result, rerender } = renderHook(
    ({ activeContext, values }) =>
      useFilterOptionCounts(activeContext, "traits", null, values, true, 128),
    { wrapper: wrapper(), initialProps: { activeContext: context, values: ["fire"] } },
  );
  await waitFor(() => expect(api.discoverFilterCounts).toHaveBeenCalledTimes(1));
  rerender({
    activeContext: {
      ...context,
      search_profile: "embeddinggemma",
      expected_artifact_generation: "generation-b",
      filter: { kind: "in", field: "record.kind", values: ["spell"] },
    },
    values: ["fire", "zero"],
  });
  expect(result.current.size).toBe(0);
  await waitFor(() => expect(result.current.get("fire")).toBe(9n));
  expect(oldSignal.aborted).toBe(true);
  await act(async () => finish(response()));
  expect(result.current.get("fire")).toBe(9n);
});
it.each([
  ["embeddinggemma", "generation-a"],
  ["bge-base", "old-generation"],
])("never shows mismatched response identity %s/%s", async (profile, generation) => {
  api.discoverFilterCounts.mockResolvedValue(response(profile, generation));
  const { result } = renderHook(
    () => useFilterOptionCounts(context, "traits", null, ["fire"], true, 128),
    { wrapper: wrapper() },
  );
  await waitFor(() => expect(api.discoverFilterCounts).toHaveBeenCalledTimes(1));
  await act(async () => {
    await Promise.resolve();
  });
  expect(result.current.size).toBe(0);
});
it("keeps count-free scoped search/paging and separately retained selections", async () => {
  const scope = { kind: "saved_list" as const, list_ref: "research" };
  const { result, rerender } = renderHook(
    ({ offset, text }) =>
      useFilterVocabulary(scope, "traits", text, offset, ["absent"]),
    { wrapper: wrapper(), initialProps: { offset: 0, text: "f" } },
  );
  await waitFor(() =>
    expect(result.current.data?.values.selected_values).toEqual(["absent"]),
  );
  rerender({ offset: 40, text: "fire" });
  await waitFor(() =>
    expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
      {
        kind: "vocabulary",
        scope,
        field_id: "traits",
        text: "fire",
        offset: 40,
        limit: 40,
        selected_values: ["absent"],
      },
      expect.any(AbortSignal),
    ),
  );
  expect(api.discoverFilterCounts).not.toHaveBeenCalled();
});
it("refreshes active saved-list option counts through the existing membership invalidation family", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const listContext: FilterDiscoveryContext = {
    ...context,
    kind: "saved_list",
    list_ref: "research",
  };
  function ListWrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client }, children);
  }
  const { result } = renderHook(
    () =>
      useFilterOptionCounts(listContext, "traits", null, ["fire"], true, 128, [
        "saved-list-filter-discovery",
        "research",
      ]),
    { wrapper: ListWrapper },
  );
  await waitFor(() => expect(result.current.get("fire")).toBe(3n));
  api.discoverFilterCounts.mockResolvedValue(response("bge-base", "generation-a", 7n));
  await act(async () => {
    await invalidateSavedListQueries(client, ["unrelated"]);
  });
  expect(api.discoverFilterCounts).toHaveBeenCalledTimes(1);
  await act(async () => {
    await invalidateSavedListQueries(client, ["research"]);
  });
  await waitFor(() => expect(result.current.get("fire")).toBe(7n));
  expect(api.discoverFilterCounts).toHaveBeenCalledTimes(2);
});

it("keeps displayed counts during a filter refresh and replaces them in place", async () => {
  const { result, rerender } = renderHook(
    ({ activeContext }) =>
      useFilterOptionCounts(activeContext, "traits", null, ["fire"], true, 128),
    { wrapper: wrapper(), initialProps: { activeContext: context } },
  );
  await waitFor(() => expect(result.current.get("fire")).toBe(3n));
  let finish!: (value: unknown) => void;
  api.discoverFilterCounts.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  rerender({
    activeContext: {
      ...context,
      filter: { kind: "in", field: "record.kind", values: ["spell"] },
    },
  });
  await waitFor(() => expect(api.discoverFilterCounts).toHaveBeenCalledTimes(2));
  expect(result.current.get("fire")).toBe(3n);
  await act(async () => finish(response("bge-base", "generation-a", 7n)));
  await waitFor(() => expect(result.current.get("fire")).toBe(7n));
});

it.each(["artifact_generation_changed", "internal_error"])(
  "preserves contextual %s errors instead of silently using unfiltered vocabulary",
  async (message) => {
    api.discoverFilterValues.mockRejectedValue(new Error(message));
    const { result } = renderHook(
      () =>
        useFilterVocabulary({ kind: "artifact" }, "traits", "", 0, [], true, {
          context,
          clauseId: null,
        }),
      { wrapper: wrapper() },
    );
    await waitFor(() => expect(result.current.error?.message).toBe(message));
    expect(api.discoverFilterValues).toHaveBeenCalledTimes(1);
    expect(api.discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ kind: "contextual" }),
      expect.any(AbortSignal),
    );
  },
);
