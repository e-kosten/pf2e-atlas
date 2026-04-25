import { describe, expect, it, vi } from "vitest";

import { HashEmbeddingProvider } from "../../src/embeddings.js";
import { buildSearchWindowSnapshot } from "../../src/search/runtime-search.js";
import { DEFAULT_RANKING_CONFIG } from "../../src/search/ranking-config.js";
import type { RuntimeSearchDependencies } from "../../src/search/contracts.js";

function createDependencies(overrides: Partial<RuntimeSearchDependencies> = {}): RuntimeSearchDependencies {
  return {
    embeddingProvider: new HashEmbeddingProvider(),
    rankingConfig: DEFAULT_RANKING_CONFIG,
    rankingConfigStatus: {
      path: "default",
      source: "default",
      revision: 1,
      loadedAt: "2026-04-24T00:00:00.000Z",
      lastError: null,
    },
    fetchCandidateCount: () => 0,
    fetchPagedCandidates: () => [],
    getAliases: () => [],
    fetchCandidates: () => [],
    fetchLexicalRetrievalRows: () => [],
    fetchSemanticRetrievalRows: () => [],
    ...overrides,
  };
}

describe("runtime search", () => {
  it("clamps broad semantic retrieval below the sqlite vector k ceiling", async () => {
    const fetchSemanticRetrievalRows = vi.fn(() => []);
    const deps = createDependencies({
      fetchCandidateCount: () => 26_377,
      fetchSemanticRetrievalRows,
    });

    await buildSearchWindowSnapshot(
      {
        query: "ghost",
      },
      deps,
    );

    expect(fetchSemanticRetrievalRows).toHaveBeenCalledTimes(1);
    expect(fetchSemanticRetrievalRows.mock.calls[0]?.[2]).toBe(4096);
  });
});
