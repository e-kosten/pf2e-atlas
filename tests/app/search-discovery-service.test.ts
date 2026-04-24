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
});
