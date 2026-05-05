import { describe, expect, it } from "vitest";

import { buildSearchFilterExplorerFieldState } from "../../src/tui/search-screen/filter-explorer-field-state.js";
import { applyStructuredDraftHostMutationToQuery } from "../../src/tui/search-screen/structured-draft/structured-draft-host-mutations.js";
import type { Pf2eTerminalQueryFieldOption } from "../../src/tui/search/service.js";
import {
  allOfFilter,
  actionCostFilter,
  browseQuery,
  metadataPredicateFilter,
  notFilter,
  packFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

describe("structured draft host mutations", () => {
  const traitsSharedExplorerFieldOption = {
    value: "traits",
    label: "Traits",
    description: "Browse traits.",
    fieldType: "set",
    editor: "sharedExplorer",
  } satisfies Pf2eTerminalQueryFieldOption;

  it("applies prompt-built additions through the bounded host mutation applier", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })]),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "appendNodes",
        nodes: [{ kind: "level", match: { kind: "gte", value: 5 } }],
      },
      {
        kind: "appendNodes",
        groupPath: [1],
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
          { kind: "level", match: { kind: "gte", value: 5 } },
        ]),
      ]),
    );
    expect(application?.resumeTarget).toEqual({ kind: "group", groupPath: [1] });
  });

  it("applies exact leaf edits through the same bounded host mutation applier", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceNode",
        node: metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
      },
      {
        kind: "replaceNode",
        path: [1],
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
      ]),
    );
    expect(application?.resumeTarget).toEqual({ kind: "group", groupPath: [] });
  });

  it("removes exact leaf nodes and resumes at the containing group", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceNode",
        node: null,
      },
      {
        kind: "replaceNode",
        path: [1],
      },
    );

    expect(application?.nextQuery.filter).toEqual({
      kind: "allOf",
      children: [scopeFilter("creature")],
    });
    expect(application?.resumeTarget).toEqual({ kind: "group", groupPath: [] });
  });

  it("replaces the root when a grouped-field mutation targets root replacement", () => {
    const query = browseQuery("Browse creatures", {
      filter: metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceGroupedField",
        field: "traits",
        fieldOption: traitsSharedExplorerFieldOption,
        fieldState: buildSearchFilterExplorerFieldState({ discreteClauses: [], scalarClauses: {} }),
      },
      {
        kind: "replaceGroupedField",
        groupPath: [],
        field: "traits",
        fieldMemberPaths: [],
        replacementNodes: [
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "magical" }),
        ],
        replaceRoot: true,
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([
        metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "magical" }),
      ]),
    );
    expect(application?.resumeTarget).toEqual({ kind: "group", groupPath: [] });
  });

  it("replaces grouped field children directly instead of introducing nested all-of wrappers", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        ]),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceGroupedField",
        field: "traits",
        fieldOption: traitsSharedExplorerFieldOption,
        fieldState: buildSearchFilterExplorerFieldState({ discreteClauses: [], scalarClauses: {} }),
      },
      {
        kind: "replaceGroupedField",
        groupPath: [1],
        field: "traits",
        fieldMemberPaths: [[1, 0], [1, 1]],
        replacementNodes: [
          metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
        ],
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "chaotic" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
        ]),
      ]),
    );
    expect(application?.resumeTarget).toEqual({ kind: "group", groupPath: [1] });
  });

  it("replaces direct action-cost shared-explorer leaf edits through grouped field mutation semantics", () => {
    const query = browseQuery("Browse actions", {
      filter: allOfFilter([scopeFilter("rule", "action"), { kind: "actionCost", match: { kind: "eq", value: 1 } }]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceGroupedField",
        field: "actionCost",
        fieldOption: {
          value: "actionCost",
          label: "Action Cost",
          description: "Browse action costs.",
          fieldType: "number",
          editor: "sharedExplorer",
        },
        fieldState: buildSearchFilterExplorerFieldState({ discreteClauses: [], scalarClauses: {} }),
      },
      {
        kind: "replaceGroupedField",
        groupPath: [],
        field: "actionCost",
        fieldMemberPaths: [[1]],
        replacementNodes: [{ kind: "actionCost", match: { kind: "eq", value: 2 } }],
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([scopeFilter("rule", "action"), { kind: "actionCost", match: { kind: "eq", value: 2 } }]),
    );
    expect(application?.resumeTarget).toEqual({ kind: "group", groupPath: [] });
  });

  it("replaces repeated action-cost grouped edits without stale duplicate peers", () => {
    const query = browseQuery("Browse actions", {
      filter: allOfFilter([
        scopeFilter("rule", "action"),
        actionCostFilter({ kind: "eq", value: 1 }),
        actionCostFilter({ kind: "eq", value: 2 }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" }),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceGroupedField",
        field: "actionCost",
        fieldOption: {
          value: "actionCost",
          label: "Action Cost",
          description: "Browse action costs.",
          fieldType: "number",
          editor: "sharedExplorer",
        },
        fieldState: buildSearchFilterExplorerFieldState({ discreteClauses: [], scalarClauses: {} }),
      },
      {
        kind: "replaceGroupedField",
        groupPath: [],
        field: "actionCost",
        fieldMemberPaths: [[1], [2]],
        replacementNodes: [actionCostFilter({ kind: "eq", value: 3 })],
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("rule", "action"),
        actionCostFilter({ kind: "eq", value: 3 }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" }),
      ]),
    );
  });

  it("replaces repeated pack grouped edits without stale duplicate peers", () => {
    const query = browseQuery("Browse equipment", {
      filter: allOfFilter([
        scopeFilter("equipment"),
        packFilter("equipment"),
        notFilter(packFilter("vehicles")),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "consumable" }),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceGroupedField",
        field: "pack",
        fieldOption: {
          value: "pack",
          label: "Pack",
          description: "Browse packs.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
        fieldState: buildSearchFilterExplorerFieldState({ discreteClauses: [], scalarClauses: {} }),
      },
      {
        kind: "replaceGroupedField",
        groupPath: [],
        field: "pack",
        fieldMemberPaths: [[1], [2]],
        replacementNodes: [packFilter("feats")],
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("equipment"),
        packFilter("feats"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "consumable" }),
      ]),
    );
  });

  it("does not apply non-replace mutations through an exact-node fallback target", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "appendNodes",
        nodes: [metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" })],
      },
      {
        kind: "replaceNode",
        path: [1],
      },
    );

    expect(application).toBeNull();
  });

  it("splits exact-node all-of replacements into the containing group when requested", () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      ]),
      limit: 20,
    }).request;

    const application = applyStructuredDraftHostMutationToQuery(
      query,
      {
        kind: "replaceNode",
        node: allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" }),
        ])!,
      },
      {
        kind: "replaceNode",
        path: [1],
        splitAllOfReplacementIntoContainingGroup: true,
      },
    );

    expect(application?.nextQuery.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" }),
      ]),
    );
    expect(application?.resumeTarget).toEqual({ kind: "group", groupPath: [] });
  });
});
