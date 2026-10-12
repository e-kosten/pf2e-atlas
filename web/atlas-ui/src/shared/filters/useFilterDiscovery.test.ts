import {
  findClauseIds,
  useFilterDiscovery,
  useFilterVocabulary,
} from "./useFilterDiscovery";
import type { FilterDiscoveryContext, QueryPredicate } from "../../generated/atlas";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { createElement, type ReactNode } from "react";
import { editorFixture, fieldFixture } from "../../test/fixtures";

const api = vi.hoisted(() => ({
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
  discoverFilterValues: vi.fn(),
}));
vi.mock("../../api/atlasApi", () => api);
function wrapper() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return function QueryWrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client }, children);
  };
}

describe("contextual facet clause selection", () => {
  it("retains every same-field clause and distinguishes root and owned scopes", () => {
    const predicate: QueryPredicate = {
      kind: "all_of",
      children: [
        {
          kind: "compare",
          clause_id: "low",
          field: "actor.level",
          op: "gte",
          value: 3,
        },
        {
          kind: "compare",
          clause_id: "high",
          field: "actor.level",
          op: "lte",
          value: 7,
        },
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
        { kind: "compare", clause_id: "root", field: "item.level", op: "eq", value: 4 },
      ],
    };
    expect(findClauseIds(predicate, "actor.level")).toEqual(["low", "high"]);
    expect(findClauseIds(predicate, "item.level", "actor.items")).toEqual(["child"]);
    expect(findClauseIds(predicate, "item.level")).toEqual(["root"]);
  });
});

describe("typed optional discovery and explicit scoped vocabulary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.discoverFilterEditor.mockResolvedValue(
      editorFixture([
        fieldFixture("traits", "set"),
        fieldFixture("record.level_or_rank", "number"),
      ]),
    );
    api.discoverFilterCounts.mockResolvedValue({
      counts: { status: "unavailable", reason: "non_conjunctive_context" },
    });
    api.discoverFilterValues.mockImplementation((request) =>
      Promise.resolve({
        kind: "vocabulary",
        scope: request.scope,
        values: {
          field: request.field_id,
          options: [{ value: "fire" }],
          selected_values: request.selected_values,
          total_values: 90n,
          basis:
            request.scope.kind === "saved_list"
              ? "product_key_scope_before_edition_preference"
              : "product_artifact_before_edition_preference",
        },
      }),
    );
  });
  it("keeps normal count unavailability out of errors and passes the exact negative context", async () => {
    const context = {
      kind: "filtered" as const,
      filter: {
        kind: "not" as const,
        predicate: {
          kind: "set_match" as const,
          clause_id: "traits",
          field: "traits",
          op: "includes_any" as const,
          values: ["fire"],
        },
      },
      text: "dragon",
      mode: "hybrid" as const,
    };
    const { result } = renderHook(
      () => useFilterDiscovery({ context, queryKeyPrefix: ["negative", context] }),
      { wrapper: wrapper() },
    );
    await waitFor(() =>
      expect(result.current.filterCountsByField["traits:traits"]).toBeDefined(),
    );
    expect(result.current.errorMessage).toBeNull();
    expect(api.discoverFilterCounts).toHaveBeenCalledWith(
      {
        context,
        field_id: "traits",
        clause_id: "traits",
      },
      expect.any(AbortSignal),
    );
    expect(api.discoverFilterValues).not.toHaveBeenCalled();
  });
  it("retains real discovery errors instead of converting them into optional unavailability", async () => {
    api.discoverFilterCounts.mockRejectedValue(new Error("Invalid clause target"));
    const context = {
      kind: "filtered" as const,
      filter: {
        kind: "compare" as const,
        field: "record.level_or_rank",
        clause_id: "bad",
        op: "eq" as const,
        value: 2,
      },
      text: null,
      mode: "hybrid" as const,
    };
    const { result } = renderHook(
      () => useFilterDiscovery({ context, queryKeyPrefix: ["bad"] }),
      { wrapper: wrapper() },
    );
    await waitFor(() =>
      expect(result.current.errorMessage).toBe("Invalid clause target"),
    );
  });
  it("never retains counts from a different committed context", async () => {
    const first: FilterDiscoveryContext = {
      kind: "filtered",
      filter: {
        kind: "compare",
        field: "record.level_or_rank",
        clause_id: "level",
        op: "eq",
        value: 2,
      },
      text: null,
      mode: "hybrid",
    };
    const pending = new Promise(() => {});
    const { result, rerender } = renderHook(
      ({ context }) =>
        useFilterDiscovery({ context, queryKeyPrefix: ["counts", context] }),
      { initialProps: { context: first }, wrapper: wrapper() },
    );
    await waitFor(() =>
      expect(
        result.current.filterCountsByField["record.level_or_rank:level"],
      ).toBeDefined(),
    );
    api.discoverFilterCounts.mockReturnValue(pending);
    rerender({ context: { ...first, text: "changed" } });
    expect(
      result.current.filterCountsByField["record.level_or_rank:level"],
    ).toBeUndefined();
  });
  it("requests count-free value search/paging with separately retained selections and explicit list scope", async () => {
    const scope = { kind: "saved_list" as const, list_ref: "research" };
    const { result, rerender } = renderHook(
      ({ offset, text }) =>
        useFilterVocabulary(scope, "traits", text, offset, ["absent-selection"]),
      {
        initialProps: { offset: 0, text: "f" },
        wrapper: wrapper(),
      },
    );
    await waitFor(() =>
      expect(result.current.data?.values.selected_values).toEqual(["absent-selection"]),
    );
    expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
      {
        kind: "vocabulary",
        scope,
        field_id: "traits",
        text: "f",
        offset: 0,
        limit: 40,
        selected_values: ["absent-selection"],
      },
      expect.any(AbortSignal),
    );
    rerender({ offset: 40, text: "fire" });
    await waitFor(() =>
      expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
        expect.objectContaining({
          offset: 40,
          text: "fire",
          selected_values: ["absent-selection"],
        }),
        expect.any(AbortSignal),
      ),
    );
    expect(api.discoverFilterCounts).not.toHaveBeenCalled();
  });
  it("uses artifact scope explicitly and rejects a mismatched contextual response", async () => {
    api.discoverFilterValues.mockResolvedValue({
      kind: "contextual",
      outcome: { status: "unavailable", reason: "negative_condition" },
    });
    const { result } = renderHook(
      () => useFilterVocabulary({ kind: "artifact" }, "traits", "", 0, []),
      { wrapper: wrapper() },
    );
    await waitFor(() =>
      expect(result.current.error?.message).toContain("Unexpected contextual response"),
    );
    expect(api.discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "vocabulary",
        scope: { kind: "artifact" },
        text: null,
      }),
      expect.any(AbortSignal),
    );
  });
});
