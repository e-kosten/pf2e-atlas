import React from "react";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { describe, expect, it, vi } from "vitest";

import { useStructuredDraftEditRouteActions } from "../../src/tui/search-screen/structured-draft/structured-draft-edit-route-actions.js";
import type { StructuredDraftEditRoute } from "../../src/tui/search-screen/structured-draft/structured-draft-edit-routes.js";
import type { Pf2eTerminalSearchQuery } from "../../src/tui/search/service.js";
import {
  allOfFilter,
  browseQuery,
  linkedFromFilter,
  linksToFilter,
  metadataPredicateFilter,
  metricFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

type RouteActions = ReturnType<typeof useStructuredDraftEditRouteActions>;

function renderRouteActions({
  promptForClauseNode = vi.fn(async () => ({ kind: "cancel" })),
  promptTextInput = vi.fn(),
}: {
  promptForClauseNode?: ReturnType<typeof vi.fn>;
  promptTextInput?: ReturnType<typeof vi.fn>;
} = {}): {
  editFieldClause: ReturnType<typeof vi.fn>;
  getActions: () => RouteActions;
  openLiveExplorerGroupedField: ReturnType<typeof vi.fn>;
  renderer: ReactTestRenderer;
  replacements: Pf2eTerminalSearchQuery[];
} {
  let actions: RouteActions | null = null;
  const replacements: Pf2eTerminalSearchQuery[] = [];
  const editFieldClause = vi.fn();
  const openLiveExplorerGroupedField = vi.fn(async () => undefined);
  const terminal = {
    pauseForAnyKey: vi.fn(),
    runPromptSession: vi.fn((callback) =>
      callback({
        promptMultiSelectOption: vi.fn(),
        promptOptionalSelectOption: vi.fn(),
        promptSelectOption: vi.fn(),
        promptTextInput,
      }),
    ),
  };

  function Harness(): null {
    actions = useStructuredDraftEditRouteActions({
      editFieldClause,
      openLiveExplorerGroupedField,
      promptForClauseNode,
      prompts: {
        promptMultiSelectOption: vi.fn(),
        promptOptionalSelectOption: vi.fn(),
        promptSelectOption: vi.fn(),
        promptTextInput: vi.fn(),
      },
      replaceStructuredDraftProjection: (update) => {
        replacements.push(update(browseQuery("Browse", { limit: 20 }).request));
      },
      terminal,
    });
    return null;
  }

  let renderer: ReactTestRenderer | null = null;
  act(() => {
    renderer = create(<Harness />);
  });

  return {
    editFieldClause,
    getActions: () => {
      if (!actions) {
        throw new Error("Route actions did not render.");
      }
      return actions;
    },
    openLiveExplorerGroupedField,
    renderer: renderer!,
    replacements,
  };
}

describe("structured draft edit route actions", () => {
  it("executes grouped routes through the grouped field explorer entry point", async () => {
    const { getActions, openLiveExplorerGroupedField, renderer } = renderRouteActions();
    const query = browseQuery("Browse creatures", { filter: scopeFilter("creature"), limit: 20 }).request;
    const route = {
      kind: "groupField",
      field: "traits",
      fieldOption: {
        value: "traits",
        label: "Traits",
        description: "Browse traits.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
      groupPath: [],
      memberPaths: [],
      fieldMemberPaths: [],
      source: "add",
    } satisfies StructuredDraftEditRoute;

    await getActions().executeStructuredDraftEditRoute(query, route);

    expect(openLiveExplorerGroupedField).toHaveBeenCalledWith(
      query,
      expect.objectContaining({
        kind: "queryFieldBucket",
        field: "traits",
        groupPath: [],
        fieldMemberPaths: [],
      }),
    );
    renderer.unmount();
  });

  it("normalizes scope leaf replacement to a singleton root scope clause", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
        scopeFilter("creature"),
      ]),
      limit: 20,
    }).request;
    const promptForClauseNode = vi.fn(async () => ({
      kind: "apply",
      value: scopeFilter("equipment", "armor"),
    }));
    const { getActions, replacements, renderer } = renderRouteActions({ promptForClauseNode });

    await getActions().executeStructuredDraftEditRoute(query, {
      kind: "leaf",
      leafKind: "scope",
      path: [1],
      placement: "rootSingleton",
    });

    expect(replacements.at(-1)?.filter).toEqual(
      scopeFilter("equipment", "armor"),
    );
    renderer.unmount();
  });

  it("edits canonical linksTo and linkedFrom leaf routes through record-key prompts", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature"), linksToFilter("spells:old"), linkedFromFilter("actions:old")]),
      limit: 20,
    }).request;
    const promptTextInput = vi.fn().mockResolvedValueOnce("spells:new").mockResolvedValueOnce("actions:new");
    const { getActions, replacements, renderer } = renderRouteActions({ promptTextInput });

    await getActions().executeStructuredDraftEditRoute(query, {
      kind: "leaf",
      leafKind: "linksTo",
      path: [1],
      placement: "inGroup",
    });

    expect(promptTextInput).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Links To",
        defaultValue: "spells:old",
      }),
    );
    expect(replacements.at(-1)?.filter).toEqual(
      allOfFilter([scopeFilter("creature"), linksToFilter("spells:new"), linkedFromFilter("actions:old")]),
    );

    await getActions().executeStructuredDraftEditRoute(query, {
      kind: "leaf",
      leafKind: "linkedFrom",
      path: [2],
      placement: "inGroup",
    });

    expect(promptTextInput).toHaveBeenLastCalledWith(
      expect.objectContaining({
        title: "Linked From",
        defaultValue: "actions:old",
      }),
    );
    expect(replacements.at(-1)?.filter).toEqual(
      allOfFilter([scopeFilter("creature"), linksToFilter("spells:old"), linkedFromFilter("actions:new")]),
    );
    renderer.unmount();
  });

  it("executes metric edits as ordinary leaf replacement routes", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature"), metricFilter("hp.value", "gte", 20)]),
      limit: 20,
    }).request;
    const promptForClauseNode = vi.fn(async () => ({
      kind: "apply",
      value: metricFilter("hp.value", "gte", 40),
    }));
    const { getActions, replacements, renderer } = renderRouteActions({ promptForClauseNode });

    await getActions().executeStructuredDraftEditRoute(query, {
      kind: "leaf",
      leafKind: "metric",
      path: [1],
      placement: "inGroup",
      fieldOption: {
        value: "actorMetric",
        label: "Actor Metric",
        description: "Browse metrics.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    });

    expect(promptForClauseNode).toHaveBeenCalledWith(expect.anything(), query, "metric", metricFilter("hp.value", "gte", 20));
    expect(replacements.at(-1)?.filter).toEqual(
      allOfFilter([scopeFilter("creature"), metricFilter("hp.value", "gte", 40)]),
    );
    renderer.unmount();
  });
});
