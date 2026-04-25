import { describe, expect, it, vi } from "vitest";

import {
  createPf2eApplicationSearchDiscoveryService,
  createScopedSearchDiscoveryApplicability,
} from "../../src/app/search-discovery-service.js";
import { createSearchFilterDiscoveryContext } from "../../src/domain/search-field-domains.js";
import { buildScopeFilter } from "../../src/domain/search-request-types.js";

describe("application search discovery service", () => {
  it("orders and caches scoped filter-value discovery at the app boundary", () => {
    const listFilterValues = vi.fn(({ field, category }: { field: string; category?: string }) => ({
      field,
      values:
        field === "rarity" && category === "spell"
          ? [
              { value: "rare", count: 2 },
              { value: "common", count: 5 },
            ]
          : [],
    }));
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues: vi.fn(async () => ({ field: "rarity", values: [] })),
      getPack: vi.fn(() => undefined),
      listFilterValues,
    });
    const applicability = createScopedSearchDiscoveryApplicability("browse", "spell", null);

    const first = service.discoverCatalogFilterValues({
      applicability,
      target: { field: "rarity" },
    });
    const second = service.discoverCatalogFilterValues({
      applicability,
      target: { field: "rarity" },
    });

    expect(first.options.map((entry) => entry.value)).toEqual(["common", "rare"]);
    expect(second).toBe(first);
    expect(listFilterValues).toHaveBeenCalledTimes(1);
    expect(listFilterValues).toHaveBeenCalledWith({
      field: "rarity",
      category: "spell",
    });
  });

  it("orders promoted numeric fields through the shared promoted-field domain owner", () => {
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues: vi.fn(async () => ({ field: "actionCost", values: [] })),
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(({ field }: { field: string }) => ({
        field,
        values:
          field === "actionCost"
            ? [
                { value: "3", count: 1 },
                { value: "1", count: 3 },
                { value: "2", count: 2 },
              ]
            : [],
      })),
    });

    const result = service.discoverCatalogFilterValues({
      applicability: createScopedSearchDiscoveryApplicability("browse", "spell", null),
      target: { field: "actionCost" },
    });

    expect(result.options.map((entry) => entry.value)).toEqual(["1", "2", "3"]);
  });

  it("exposes scoped metadata fields and metric discovery groups from one service", () => {
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues: vi.fn(async () => ({ field: "traits", values: [] })),
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });

    const creatureFields = service.getScopedMetadataFields({ category: "creature", subcategory: null });
    const creatureMetricGroups = service.getMetricDiscoveryGroups({ category: "creature", subcategory: null });

    expect(creatureFields.some((field) => field.field === "traits")).toBe(true);
    expect(creatureMetricGroups).toEqual([
      expect.objectContaining({
        metricField: "actorMetrics",
        metadataField: "actorMetric",
      }),
    ]);
  });

  it("routes metric key and scalar discovery through the shared boundary", () => {
    const listFilterValues = vi.fn(
      ({
        field,
        category,
        metric,
        metricPrefix,
      }: {
        field: string;
        category?: string;
        metric?: string;
        metricPrefix?: string;
      }) => ({
        field,
        values:
          field === "actorMetrics" && category === "creature" && metricPrefix === "save."
            ? [{ value: "save.best", count: 3 }]
            : field === "actorMetrics" && category === "creature" && metric === "save.best"
              ? [{ value: "fort", count: 3 }]
              : [],
      }),
    );
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues: vi.fn(async () => ({ field: "actorMetrics", values: [] })),
      getPack: vi.fn(() => undefined),
      listFilterValues,
    });
    const applicability = createScopedSearchDiscoveryApplicability("browse", "creature", null);

    expect(
      service.discoverMetricKeys({
        applicability,
        metricField: "actorMetrics",
        metricPrefix: "save.",
      }),
    ).toEqual([{ id: "save.best", value: "save.best", count: 3 }]);
    expect(
      service.discoverMetricValues({
        applicability,
        metricField: "actorMetrics",
        metricKey: "save.best",
      }),
    ).toEqual([{ id: "fort", value: "fort", count: 3 }]);
  });

  it("resolves pack aliases through the shared service", () => {
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues: vi.fn(async () => ({ field: "traits", values: [] })),
      getPack: vi.fn((packValue: string) => (packValue === "Actions" ? { name: "actions" } : undefined)),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });

    expect(service.resolvePackName("Actions")).toBe("actions");
    expect(service.resolvePackName("missing-pack")).toBeUndefined();
  });

  it("uses the full canonical request for matching discovery", async () => {
    const discoverFilterValues = vi.fn(async ({ field }: { field: string }) => ({
      field,
      values: [{ value: "illusion", count: 2 }],
    }));
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });
    const context = createSearchFilterDiscoveryContext({
      mode: "search",
      search: { query: "phantom", profile: "balanced" },
      filter: buildScopeFilter("spell"),
    });

    const result = await service.discoverFilterValues({
      mode: "matching",
      context,
      target: { field: "traits" },
    });

    expect(result.options).toEqual([{ id: "illusion", value: "illusion", count: 2 }]);
    expect(discoverFilterValues).toHaveBeenCalledWith(
      {
        field: "traits",
        category: "spell",
      },
      context.request,
    );
  });

  it("sanitizes empty ranked-search matching discovery back to the applicability slice", async () => {
    const discoverFilterValues = vi.fn(async ({ field }: { field: string }) => ({
      field,
      values: [{ value: "undead", count: 3 }],
    }));
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });
    const context = createSearchFilterDiscoveryContext({
      mode: "search",
      search: { query: "" },
    });

    await service.discoverFilterValues({
      mode: "matching",
      context,
      target: { field: "traits" },
    });

    expect(discoverFilterValues).toHaveBeenCalledWith(
      {
        field: "traits",
      },
      {
        mode: "browse",
      },
    );
  });

  it("uses only the applicability slice for catalog discovery", async () => {
    const discoverFilterValues = vi.fn(async ({ field }: { field: string }) => ({
      field,
      values: [{ value: "rare", count: 4 }],
    }));
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "rarity", values: [] })),
    });
    const context = createSearchFilterDiscoveryContext({
      mode: "search",
      search: { query: "dragon", profile: "balanced" },
      filter: {
        kind: "allOf",
        children: [
          { kind: "pack", value: "bestiary" },
          buildScopeFilter("creature"),
          {
            kind: "metadataPredicate",
            predicate: { field: "rarity", op: "eq", value: "rare" },
          },
        ],
      },
    });

    await service.discoverFilterValues({
      mode: "catalog",
      context,
      target: { field: "rarity" },
    });

    expect(discoverFilterValues).toHaveBeenCalledWith(
      {
        field: "rarity",
        category: "creature",
      },
      {
        mode: "browse",
        filter: {
          kind: "allOf",
          children: [
            { kind: "pack", value: "bestiary" },
            {
              kind: "scope",
              category: "creature",
              subcategory: { kind: "any" },
            },
          ],
        },
      },
    );
  });

  it("retries matching discovery after a transient failure instead of caching the rejection", async () => {
    const discoverFilterValues = vi
      .fn<
        (
          query: { field: string; category?: string },
          request: import("../../src/domain/search-request-types.js").SearchRequest,
        ) => Promise<{ field: string; values: Array<{ value: string; count: number }> }>
      >()
      .mockRejectedValueOnce(new Error("temporary failure"))
      .mockResolvedValueOnce({
        field: "traits",
        values: [{ value: "undead", count: 2 }],
      });
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });
    const context = createSearchFilterDiscoveryContext({
      mode: "search",
      search: { query: "ghost", profile: "balanced" },
      filter: buildScopeFilter("creature"),
    });

    await expect(
      service.discoverFilterValues({
        mode: "matching",
        context,
        target: { field: "traits" },
      }),
    ).rejects.toThrow("temporary failure");

    await expect(
      service.discoverFilterValues({
        mode: "matching",
        context,
        target: { field: "traits" },
      }),
    ).resolves.toEqual({
      mode: "matching",
      target: { field: "traits" },
      options: [{ id: "undead", value: "undead", count: 2 }],
    });
    expect(discoverFilterValues).toHaveBeenCalledTimes(2);
  });

  it("prepares a matching search-semantics reader from the active canonical request", async () => {
    const request = {
      mode: "search",
      search: { query: "ghost", profile: "balanced" as const },
      filter: buildScopeFilter("creature"),
      limit: 20,
    } satisfies import("../../src/domain/search-request-types.js").SearchRequest;
    const discoverFilterValues = vi.fn(
      async (
        {
          field,
          metric,
          metricPrefix,
        }: {
          field: string;
          metric?: string;
          metricPrefix?: string;
        },
        currentRequest: import("../../src/domain/search-request-types.js").SearchRequest,
      ) => ({
        field,
        values:
          field === "traits"
            ? [{ value: "undead", count: 2 }]
            : field === "actorMetrics" && metricPrefix === "save."
              ? [{ value: "save.best", count: 2 }]
              : field === "actorMetrics" && metric === "save.best"
                ? [{ value: "fort", count: 2 }]
                : [],
        requestMode: currentRequest.mode,
      }),
    );
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });

    const reader = await service.prepareSearchSemanticsReader(request, "matching");

    expect(reader.scope).toEqual({ category: "creature", subcategory: null });
    expect(reader.discoverFieldValues({ category: "creature", subcategory: null, field: "traits" })).toEqual([
      { id: "undead", value: "undead", count: 2 },
    ]);
    expect(
      reader.discoverMetricKeys({
        category: "creature",
        subcategory: null,
        metricField: "actorMetrics",
        metricPrefix: "save.",
      }),
    ).toEqual([{ id: "save.best", value: "save.best", count: 2 }]);
    expect(
      reader.discoverMetricValues({
        category: "creature",
        subcategory: null,
        metricField: "actorMetrics",
        metricKey: "save.best",
      }),
    ).toEqual([{ id: "fort", value: "fort", count: 2 }]);
    expect(discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field: "traits", category: "creature" }),
      request,
    );
  });

  it("keeps broad scope-only derived-tag discovery populated in matching mode", async () => {
    const request = {
      mode: "search",
      search: { query: "", profile: "balanced" as const },
      filter: buildScopeFilter("creature"),
      limit: 20,
    } satisfies import("../../src/domain/search-request-types.js").SearchRequest;
    const discoverFilterValues = vi.fn(
      async (
        { field }: { field: string },
        currentRequest: import("../../src/domain/search-request-types.js").SearchRequest,
      ) => ({
        field,
        values:
          field === "derivedTags"
            ? [{ value: "undead_adjacent", count: 3 }]
            : field === "traits"
              ? [{ value: "undead", count: 2 }]
              : [],
        requestMode: currentRequest.mode,
      }),
    );
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });

    const reader = await service.prepareSearchSemanticsReader(request, "matching");

    expect(reader.scope).toEqual({ category: "creature", subcategory: null });
    expect(reader.discoverFieldValues({ category: "creature", subcategory: null, field: "derivedTags" })).toEqual([
      { id: "undead_adjacent", value: "undead_adjacent", count: 3 },
    ]);
    expect(discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field: "derivedTags", category: "creature" }),
      request,
    );
  });

  it("prepares a catalog search-semantics reader from the applicability slice only", async () => {
    const request = {
      mode: "search",
      search: { query: "dragon", profile: "balanced" as const },
      filter: {
        kind: "allOf",
        children: [
          { kind: "pack", value: "bestiary" },
          buildScopeFilter("creature"),
          {
            kind: "metadataPredicate",
            predicate: { field: "rarity", op: "eq", value: "rare" },
          },
        ],
      },
      limit: 20,
    } satisfies import("../../src/domain/search-request-types.js").SearchRequest;
    const discoverFilterValues = vi.fn(async ({ field }: { field: string }) => ({
      field,
      values: [{ value: "rare", count: 4 }],
    }));
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "rarity", values: [] })),
    });

    const reader = await service.prepareSearchSemanticsReader(request, "catalog");

    expect(reader.discoverFieldValues({ category: "creature", subcategory: null, field: "rarity" })).toEqual([
      { id: "rare", value: "rare", count: 4 },
    ]);
    expect(discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field: "rarity", category: "creature" }),
      {
        mode: "browse",
        filter: {
          kind: "allOf",
          children: [
            { kind: "pack", value: "bestiary" },
            {
              kind: "scope",
              category: "creature",
              subcategory: { kind: "any" },
            },
          ],
        },
      },
    );
  });

  it("keeps broad scope-only derived-tag discovery populated in catalog mode", async () => {
    const request = {
      mode: "search",
      search: { query: "", profile: "balanced" as const },
      filter: buildScopeFilter("creature"),
      limit: 20,
    } satisfies import("../../src/domain/search-request-types.js").SearchRequest;
    const discoverFilterValues = vi.fn(async ({ field }: { field: string }) => ({
      field,
      values: field === "derivedTags" ? [{ value: "undead_adjacent", count: 4 }] : [],
    }));
    const service = createPf2eApplicationSearchDiscoveryService({
      discoverFilterValues,
      getPack: vi.fn(() => undefined),
      listFilterValues: vi.fn(() => ({ field: "traits", values: [] })),
    });

    const reader = await service.prepareSearchSemanticsReader(request, "catalog");

    expect(reader.discoverFieldValues({ category: "creature", subcategory: null, field: "derivedTags" })).toEqual([
      { id: "undead_adjacent", value: "undead_adjacent", count: 4 },
    ]);
    expect(discoverFilterValues).toHaveBeenCalledWith(
      expect.objectContaining({ field: "derivedTags", category: "creature" }),
      {
        mode: "browse",
        filter: buildScopeFilter("creature"),
      },
    );
  });
});
