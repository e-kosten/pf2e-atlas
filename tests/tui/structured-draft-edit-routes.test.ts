import { describe, expect, it } from "vitest";

import {
  buildStructuredDraftExplorerOnlyFieldOption,
  classifyStructuredDraftAddFieldRoute,
  classifyStructuredDraftBucketEditRoute,
  classifyStructuredDraftNodeEditRoute,
  classifyStructuredDraftPromptLeafAddRoute,
  getStructuredDraftSyntheticFieldOption,
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
    value: "description",
    label: "Description",
    description: "Description text.",
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

  it("classifies add intents into group-field and leaf routes", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature")]),
      limit: 20,
    }).request;

    expect(
      classifyStructuredDraftAddFieldRoute({
        fieldOption: traitsField,
        groupPath: [2],
        query,
      }),
    ).toEqual({
      kind: "groupField",
      field: "traits",
      fieldOption: traitsField,
      groupPath: [2],
      memberPaths: [],
      fieldMemberPaths: [],
      source: "add",
    });

    expect(
      classifyStructuredDraftAddFieldRoute({
        fieldOption: booleanField,
        groupPath: [],
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
    expect(classifyStructuredDraftPromptLeafAddRoute({ clauseKind: "scope", groupPath: [] })).toEqual({
      kind: "leaf",
      leafKind: "scope",
      path: null,
      groupPath: [],
      placement: "rootSingleton",
    });

    expect(classifyStructuredDraftPromptLeafAddRoute({ clauseKind: "metricCompare", groupPath: [1] })).toEqual({
      kind: "leaf",
      leafKind: "metricCompare",
      path: null,
      groupPath: [1],
      placement: "inGroup",
    });

    expect(classifyStructuredDraftPromptLeafAddRoute({ clauseKind: "pack", groupPath: [] })).toMatchObject({
      kind: "unsupported",
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
      classifyStructuredDraftAddFieldRoute({
        fieldOption: traitsField,
        groupPath: [],
        query,
      }),
    ).toEqual({
      kind: "groupField",
      field: "traits",
      fieldOption: traitsField,
      groupPath: [],
      memberPaths: [[1], [2]],
      fieldMemberPaths: [[1], [2]],
      source: "add",
    });
  });

  it("classifies synthetic grouped-field add intents with existing cohort paths", () => {
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
      classifyStructuredDraftAddFieldRoute({
        fieldOption: getStructuredDraftSyntheticFieldOption("pack")!,
        groupPath: [],
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
      classifyStructuredDraftAddFieldRoute({
        fieldOption: getStructuredDraftSyntheticFieldOption("rarity")!,
        groupPath: [],
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
      classifyStructuredDraftAddFieldRoute({
        fieldOption: getStructuredDraftSyntheticFieldOption("actionCost")!,
        groupPath: [],
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
        fieldOptions,
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
        fieldOptions,
      }),
    ).toEqual({
      kind: "groupField",
      field: "traits",
      fieldOption: traitsField,
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
        metadataPredicateFilter({ field: "description", op: "contains", value: "fire" }),
        metadataPredicateFilter({ field: "publicationRemaster", op: "eq", value: true }),
        metadataPredicateFilter({ field: "areaValue", op: "gte", value: 20 }),
        linksToFilter("spells:fireball"),
        linkedFromFilter("actions:activate"),
        metricFilter("hp.value", "gte", 20),
        metricCompareFilter("hp.value", "gte", "hardness"),
      ]),
      limit: 20,
    }).request;

    expect(classifyStructuredDraftNodeEditRoute({ query, path: [0], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "scope",
      placement: "rootSingleton",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [1], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "level",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [2], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "price",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [3], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "metadataText",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [4], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "metadataBoolean",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [5], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "metadataScalar",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [6], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "linksTo",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [7], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "linkedFrom",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [8], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "metric",
    });
    expect(classifyStructuredDraftNodeEditRoute({ query, path: [9], fieldOptions })).toMatchObject({
      kind: "leaf",
      leafKind: "metricCompare",
    });
  });

  it("centralizes synthetic shared-explorer field options for grouped query fields", () => {
    expect(getStructuredDraftSyntheticFieldOption("pack")).toMatchObject({ value: "pack", editor: "sharedExplorer" });
    expect(getStructuredDraftSyntheticFieldOption("rarity")).toMatchObject({
      value: "rarity",
      editor: "sharedExplorer",
    });
    expect(getStructuredDraftSyntheticFieldOption("actionCost")).toMatchObject({
      value: "actionCost",
      editor: "sharedExplorer",
    });
  });
});
