import { describe, expect, it } from "vitest";

import {
  buildStructuredDraftExplorerOnlyFieldOption,
  classifyStructuredDraftAddIntentRoute,
  classifyStructuredDraftBucketEditRoute,
  classifyStructuredDraftNodeEditRoute,
  createStructuredDraftRouteCatalog,
  getStructuredDraftAddIntentForClauseKind,
  getStructuredDraftPromotedFieldOption,
  isStructuredDraftGroupFieldRoute,
} from "../../src/tui/search-screen/structured-draft/structured-draft-edit-routes.js";
import type { Pf2eTerminalQueryFieldOption } from "../../src/tui/search/service.js";
import {
  actionCostFilter,
  allOfFilter,
  browseQuery,
  linkedFromFilter,
  linksToFilter,
  levelFilter,
  metadataPredicateFilter,
  metricCompareFilter,
  metricFilter,
  packFilter,
  priceFilter,
  rarityFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

describe("structured draft edit routes", () => {
  const traitsField = buildStructuredDraftExplorerOnlyFieldOption(
    "traits",
    "Traits",
    "Browse traits.",
    "set",
  );
  const descriptionField = {
    value: "durationText",
    label: "Duration",
    description: "Duration text.",
    fieldType: "text",
    editor: "structuredForm",
  } satisfies Pf2eTerminalQueryFieldOption;
  const booleanField = {
    value: "publicationRemaster",
    label: "Remaster",
    description: "Remaster flag.",
    fieldType: "boolean",
    editor: "sharedExplorer",
  } satisfies Pf2eTerminalQueryFieldOption;
  const numericField = {
    value: "areaValue",
    label: "Area Value",
    description: "Area size.",
    fieldType: "number",
    editor: "structuredForm",
  } satisfies Pf2eTerminalQueryFieldOption;
  const actorMetricField = buildStructuredDraftExplorerOnlyFieldOption(
    "actorMetric",
    "Actor Metric",
    "Browse actor metrics.",
    "enumString",
  );

  const fieldOptions = [traitsField, descriptionField, booleanField, numericField, actorMetricField];
  const catalog = createStructuredDraftRouteCatalog(fieldOptions.map((fieldOption) => fieldOption.value));

  it("classifies add intents into group-field and leaf routes", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature")]),
      limit: 20,
    }).request;

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: { kind: "field", field: { kind: "metadata", field: "traits" }, groupPath: [2] },
        query,
      }),
    ).toMatchObject({
      kind: "groupField",
      field: "traits",
      groupPath: [2],
      memberPaths: [],
      fieldMemberPaths: [],
      source: "add",
    });

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: { kind: "field", field: { kind: "metadata", field: "publicationRemaster" }, groupPath: [] },
        query,
      }),
    ).toMatchObject({
      kind: "leaf",
      leafKind: "metadataBoolean",
      groupPath: [],
      path: null,
      placement: "inGroup",
    });
  });

  it("classifies prompt-built add clauses in the route owner", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature")]),
      limit: 20,
    }).request;

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: getStructuredDraftAddIntentForClauseKind("scope", [])!,
        query,
      }),
    ).toEqual({
      kind: "leaf",
      leafKind: "scope",
      path: null,
      groupPath: [],
      placement: "rootSingleton",
    });

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: getStructuredDraftAddIntentForClauseKind("metricCompare", [1])!,
        query,
      }),
    ).toEqual({
      kind: "leaf",
      leafKind: "metricCompare",
      path: null,
      groupPath: [1],
      placement: "inGroup",
    });

    expect(getStructuredDraftAddIntentForClauseKind("pack", [])).toEqual({
      kind: "field",
      field: { kind: "pack" },
      groupPath: [],
    });
  });

  it("classifies add intents with existing grouped-field cohort paths", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "undead" }),
      ]),
      limit: 20,
    }).request;

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: { kind: "field", field: { kind: "metadata", field: "traits" }, groupPath: [] },
        query,
      }),
    ).toMatchObject({
      kind: "groupField",
      field: "traits",
      groupPath: [],
      memberPaths: [[1], [2]],
      fieldMemberPaths: [[1], [2]],
      source: "add",
    });
  });

  it("classifies promoted grouped-field add intents with existing cohort paths", () => {
    const query = browseQuery("Browse actions", {
      filter: allOfFilter([
        scopeFilter("rule", "action"),
        packFilter("pathfinder-actions-core"),
        rarityFilter({ kind: "eq", value: "common" }),
        actionCostFilter({ kind: "eq", value: 1 }),
        actionCostFilter({ kind: "eq", value: 2 }),
      ]),
      limit: 20,
    }).request;

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: { kind: "field", field: { kind: "pack" }, groupPath: [] },
        query,
      }),
    ).toMatchObject({
      kind: "groupField",
      field: "pack",
      memberPaths: [[1]],
      fieldMemberPaths: [[1]],
      source: "add",
    });

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: { kind: "field", field: { kind: "metadata", field: "rarity" }, groupPath: [] },
        query,
      }),
    ).toMatchObject({
      kind: "groupField",
      field: "rarity",
      memberPaths: [[2]],
      fieldMemberPaths: [[2]],
      source: "add",
    });

    expect(
      classifyStructuredDraftAddIntentRoute({
        catalog,
        intent: { kind: "field", field: { kind: "metadata", field: "actionCost" }, groupPath: [] },
        query,
      }),
    ).toMatchObject({
      kind: "groupField",
      field: "actionCost",
      memberPaths: [[3], [4]],
      fieldMemberPaths: [[3], [4]],
      source: "add",
    });
  });

  it("classifies pack, rarity, and action-cost leaves as grouped field-member routes", () => {
    const query = browseQuery("Browse actions", {
      filter: allOfFilter([
        scopeFilter("rule", "action"),
        packFilter("pathfinder-monster-core"),
        rarityFilter({ kind: "eq", value: "common" }),
        actionCostFilter({ kind: "eq", value: 1 }),
      ]),
      limit: 20,
    }).request;

    for (const [path, field] of [
      [[1], "pack"],
      [[2], "rarity"],
      [[3], "actionCost"],
    ] as const) {
      const route = classifyStructuredDraftNodeEditRoute({
        query,
        path,
        catalog,
      });
      expect(isStructuredDraftGroupFieldRoute(route)).toBe(true);
      expect(route).toMatchObject({
        kind: "groupField",
        field,
        groupPath: [],
        memberPaths: [path],
        fieldMemberPaths: [path],
        source: "member",
      });
    }
  });

  it("classifies projected query-field buckets as grouped routes with bucket source", () => {
    expect(
      classifyStructuredDraftBucketEditRoute({
        entry: {
          kind: "queryFieldBucket",
          key: "bucket:traits",
          label: "Traits",
          description: "Existing trait clauses.",
          groupPath: [1],
          field: "traits",
          fieldOperator: "include",
          memberPaths: [[1, 0]],
          fieldMemberPaths: [[1, 0]],
        },
        catalog,
        query: browseQuery("Browse creatures", { filter: allOfFilter([scopeFilter("creature")]), limit: 20 }).request,
      }),
    ).toMatchObject({
      kind: "groupField",
      field: "traits",
      groupPath: [1],
      memberPaths: [[1, 0]],
      fieldMemberPaths: [[1, 0]],
      source: "bucket",
    });
  });

  it("classifies representative leaf routes without inventing a metric fallback route", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        levelFilter({ kind: "gte", value: 5 }),
        priceFilter({ kind: "lte", value: 100 }),
        metadataPredicateFilter({ field: "durationText", op: "contains", value: "fire" }),
        metadataPredicateFilter({ field: "publicationRemaster", op: "eq", value: true }),
        metadataPredicateFilter({ field: "areaValue", op: "gte", value: 20 }),
        linksToFilter("spells:fireball"),
        linkedFromFilter("actions:activate"),
        metricFilter("hp.value", "gte", 20),
        metricCompareFilter("hp.value", "gte", "hardness"),
      ]),
      limit: 20,
    }).request;

    expect(classifyStructuredDraftNodeEditRoute({ query, path: [0], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "scope",
      placement: "rootSingleton",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [1], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "level",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [2], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "price",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [3], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "metadataText",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [4], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "metadataBoolean",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [5], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "metadataScalar",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [6], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "linksTo",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [7], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "linkedFrom",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [8], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "metric",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [9], catalog })).toMatchObject({
      kind: "leaf",
      leafKind: "metricCompare",
    });
  });

  it("centralizes promoted shared-explorer field options for grouped query fields", () => {
    expect(getStructuredDraftPromotedFieldOption("pack")).toMatchObject({ value: "pack", editor: "sharedExplorer" });
    expect(getStructuredDraftPromotedFieldOption("rarity")).toMatchObject({
      value: "rarity",
      editor: "sharedExplorer",
    });
    expect(getStructuredDraftPromotedFieldOption("actionCost")).toMatchObject({
      value: "actionCost",
      editor: "sharedExplorer",
    });
  });
});
