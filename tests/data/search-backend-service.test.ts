import type { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it, vi } from "vitest";

import { Pf2eSearchBackendService } from "../../src/data/backend/search-service.js";
import type { EmbeddingProvider } from "../../src/embeddings.js";
import type { SearchWindowSnapshot } from "../../src/search/runtime-search-snapshot.js";
import { buildSearchWindowSnapshot } from "../../src/search/runtime-search.js";

vi.mock("../../src/search/runtime-search.js", async () => {
  const actual = await vi.importActual<typeof import("../../src/search/runtime-search.js")>(
    "../../src/search/runtime-search.js",
  );
  return {
    ...actual,
    buildSearchWindowSnapshot: vi.fn(),
  };
});

function createCatalog() {
  return {
    decorateRecord: vi.fn((record) => record),
    getAliases: vi.fn(() => []),
    getPack: vi.fn(() => undefined),
    getRankingConfigStatus: vi.fn(() => ({
      path: "default",
      source: "default",
      revision: 1,
      loadedAt: "2026-04-24T00:00:00.000Z",
      lastError: null,
    })),
    getRecordsByKeys: vi.fn(() => []),
    listFilterValues: vi.fn((_query, _filters, options: { recordKeys?: string[] } = {}) => ({
      field: "traits",
      values: options.recordKeys?.length ? [{ value: "undead", count: options.recordKeys.length }] : [],
    })),
  };
}

function createSnapshot(recordKeys: string[]): SearchWindowSnapshot {
  return {
    searchProfile: "balanced",
    mode: "hybrid",
    sort: "ranked",
    records: recordKeys.map((recordKey) => ({ recordKey }) as SearchWindowSnapshot["records"][number]),
    explanations: [],
  };
}

function createTraceSink() {
  const spans: Array<{ name: string; metadata: Record<string, unknown>; endMetadata?: Record<string, unknown> }> = [];
  return {
    spans,
    trace: {
      startSpan: (name: string, metadata: Record<string, unknown> = {}) => {
        const span = { name, metadata };
        spans.push(span);
        return {
          end: (endMetadata: Record<string, unknown> = {}) => {
            span.endMetadata = endMetadata;
          },
        };
      },
    },
  };
}

describe("Pf2eSearchBackendService discovery caching", () => {
  afterEach(() => {
    vi.resetAllMocks();
  });

  it("reuses one ranked snapshot across concurrent matching-discovery fields for the same request", async () => {
    vi.mocked(buildSearchWindowSnapshot).mockResolvedValue(createSnapshot(["creature-1", "creature-2"]));
    const catalog = createCatalog();
    const service = new Pf2eSearchBackendService(
      {} as DatabaseSync,
      catalog as never,
      { embed: vi.fn(async () => new Float32Array()) } as EmbeddingProvider,
      null,
    );
    const request = {
      mode: "search" as const,
      search: { query: "ghost", profile: "balanced" as const },
      filter: {
        kind: "scope" as const,
        category: "creature" as const,
        subcategory: { kind: "any" as const },
      },
    };

    await Promise.all([
      service.discoverFilterValues({ field: "traits", category: "creature" }, request),
      service.discoverFilterValues({ field: "rarity", category: "creature" }, request),
    ]);

    expect(buildSearchWindowSnapshot).toHaveBeenCalledTimes(1);
    expect(catalog.listFilterValues).toHaveBeenCalledTimes(2);
    expect(catalog.listFilterValues).toHaveBeenNthCalledWith(
      1,
      { field: "traits", category: "creature" },
      expect.any(Object),
      { recordKeys: ["creature-1", "creature-2"] },
    );
    expect(catalog.listFilterValues).toHaveBeenNthCalledWith(
      2,
      { field: "rarity", category: "creature" },
      expect.any(Object),
      { recordKeys: ["creature-1", "creature-2"] },
    );
  });

  it("emits opt-in trace spans for matching discovery and matching-set resolution", async () => {
    vi.mocked(buildSearchWindowSnapshot).mockResolvedValue(createSnapshot(["creature-1", "creature-2"]));
    const catalog = createCatalog();
    const { spans, trace } = createTraceSink();
    const service = new Pf2eSearchBackendService(
      {} as DatabaseSync,
      catalog as never,
      { embed: vi.fn(async () => new Float32Array()) } as EmbeddingProvider,
      null,
    );
    service.setTraceSink(trace);

    await service.discoverFilterValues(
      { field: "traits", category: "creature" },
      {
        mode: "search",
        search: { query: "ghost", profile: "balanced" },
        filter: {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
      },
    );

    expect(spans.map((span) => span.name)).toEqual(
      expect.arrayContaining(["backend.discoverFilterValues", "backend.resolveDiscoveryRecordKeys"]),
    );
    expect(spans.find((span) => span.name === "backend.discoverFilterValues")).toMatchObject({
      metadata: { field: "traits", requestMode: "search", category: "creature" },
      endMetadata: { values: 1 },
    });
    expect(spans.find((span) => span.name === "backend.resolveDiscoveryRecordKeys")).toMatchObject({
      metadata: { cache: "miss", mode: "search", profile: "balanced" },
      endMetadata: { recordKeys: 2 },
    });
  });
});
