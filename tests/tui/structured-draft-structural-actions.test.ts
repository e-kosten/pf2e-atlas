import React from "react";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { describe, expect, it, vi } from "vitest";

import {
  buildInsertionActionEntries,
  buildRootActionEntries,
  getStructuredDraftGroupActionEntries,
  getStructuredDraftLeafActionEntries,
  getStructuredDraftNotActionEntries,
  useStructuredDraftStructuralActions,
} from "../../src/tui/search-screen/structured-draft/structured-draft-structural-actions.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../src/tui/search/service.js";
import {
  allOfFilter,
  browseQuery,
  metadataPredicateFilter,
  notFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

type StructuralActions = ReturnType<typeof useStructuredDraftStructuralActions>;

describe("structured draft structural actions", () => {
  const traitsFieldOption = {
    value: "traits",
    label: "Traits",
    description: "Browse traits.",
    fieldType: "set",
    editor: "sharedExplorer",
  } satisfies Pf2eTerminalQueryFieldOption;

  const getScopedFieldOptions = () => [traitsFieldOption];

  function renderStructuralActions({
    moveSourcePath = null,
    promptForClauseKind = vi.fn(async () => ({ kind: "cancel" })),
    promptForClauseNode = vi.fn(async () => ({ kind: "cancel" })),
    promptForSharedExplorerFieldOption = vi.fn(async () => ({ kind: "apply", value: null })),
  }: {
    moveSourcePath?: number[] | null;
    promptForClauseKind?: ReturnType<typeof vi.fn>;
    promptForClauseNode?: ReturnType<typeof vi.fn>;
    promptForSharedExplorerFieldOption?: ReturnType<typeof vi.fn>;
  } = {}): {
    clearStructuredDraftMoveSource: ReturnType<typeof vi.fn>;
    editFieldClause: ReturnType<typeof vi.fn>;
    enterStructuredDraftMoveMode: ReturnType<typeof vi.fn>;
    getActions: () => StructuralActions;
    openLiveExplorerExactNodeFieldClauseFallback: ReturnType<typeof vi.fn>;
    openLiveExplorerGroupedField: ReturnType<typeof vi.fn>;
    openLiveExplorerGroupFieldByName: ReturnType<typeof vi.fn>;
    renderer: ReactTestRenderer;
    replacements: { query: Pf2eTerminalSearchQuery; options?: unknown }[];
    setStructuredDraftResumeTarget: ReturnType<typeof vi.fn>;
  } {
    let actions: StructuralActions | null = null;
    const replacements: { query: Pf2eTerminalSearchQuery; options?: unknown }[] = [];
    const clearStructuredDraftMoveSource = vi.fn();
    const editFieldClause = vi.fn();
    const enterStructuredDraftMoveMode = vi.fn();
    const openLiveExplorerCanonicalFieldMember = vi.fn();
    const openLiveExplorerExactNodeFieldClauseFallback = vi.fn();
    const openLiveExplorerGroupedField = vi.fn();
    const openLiveExplorerGroupFieldByName = vi.fn();
    const setStructuredDraftResumeTarget = vi.fn();
    const terminal = {
      pauseForAnyKey: vi.fn(),
      runPromptSession: vi.fn((callback) =>
        callback({
          promptMultiSelectOption: vi.fn(),
          promptOptionalSelectOption: vi.fn(),
          promptSelectOption: vi.fn(),
          promptTextInput: vi.fn(),
        }),
      ),
    };

    function Harness(): null {
      actions = useStructuredDraftStructuralActions({
        clearStructuredDraftMoveSource,
        editFieldClause,
        enterStructuredDraftMoveMode,
        getScopedFieldOptions,
        moveSourcePath,
        openLiveExplorerCanonicalFieldMember,
        openLiveExplorerExactNodeFieldClauseFallback,
        openLiveExplorerGroupedField,
        openLiveExplorerGroupFieldByName,
        promptForClauseKind,
        promptForClauseNode,
        promptForSharedExplorerFieldOption,
        prompts: {
          promptMultiSelectOption: vi.fn(),
          promptOptionalSelectOption: vi.fn(),
          promptSelectOption: vi.fn(),
          promptTextInput: vi.fn(),
        },
        replaceStructuredDraftProjection: (update, options) => {
          replacements.push({
            query: update(browseQuery("Browse creatures", { limit: 20 }).request),
            options,
          });
        },
        setStructuredDraftResumeTarget,
        terminal,
      });
      return null;
    }

    let renderer: ReactTestRenderer | null = null;
    act(() => {
      renderer = create(React.createElement(Harness));
    });

    return {
      clearStructuredDraftMoveSource,
      editFieldClause,
      enterStructuredDraftMoveMode,
      getActions: () => {
        if (!actions) {
          throw new Error("Structural actions did not render.");
        }
        return actions;
      },
      openLiveExplorerExactNodeFieldClauseFallback,
      openLiveExplorerGroupedField,
      openLiveExplorerGroupFieldByName,
      renderer: renderer!,
      replacements,
      setStructuredDraftResumeTarget,
    };
  }

  it("builds insertion actions from move mode instead of prompt or explorer state", () => {
    expect(buildInsertionActionEntries(false).map((entry) => entry.id)).toEqual([
      "addClause",
      "addAndGroup",
      "addOrGroup",
      "addNotGroup",
    ]);

    expect(buildInsertionActionEntries(true)).toEqual([
      {
        id: "moveHere",
        label: "Move Here",
        description: "Append the anchored node into this visible insertion slot.",
      },
    ]);
  });

  it("exposes root toggle only when the query has a root filter", () => {
    const emptyQuery = browseQuery("Browse creatures", { limit: 20 }).request;
    const filteredQuery = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature")]),
      limit: 20,
    }).request;

    expect(buildRootActionEntries(emptyQuery).map((entry) => entry.id)).toEqual([
      "addClause",
      "addAndGroup",
      "addOrGroup",
      "addNotGroup",
    ]);
    expect(buildRootActionEntries(filteredQuery).map((entry) => entry.id)).toContain("toggleRoot");
  });

  it("builds leaf structural actions without owning prompt value collection", () => {
    const traitsNode = metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" });
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature"), traitsNode]),
      limit: 20,
    }).request;

    expect(getStructuredDraftLeafActionEntries(query, [1], traitsNode, getScopedFieldOptions).map((entry) => entry.id))
      .toEqual(["edit", "wrapNot", "wrapAnd", "wrapOr", "move", "remove"]);
  });

  it("builds not and group actions around canonical tree capabilities", () => {
    const excludedTraits = notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }));
    const groupNode = allOfFilter([scopeFilter("creature"), excludedTraits]);
    const query = browseQuery("Browse creatures", {
      filter: groupNode,
      limit: 20,
    }).request;

    expect(getStructuredDraftNotActionEntries(query, [1]).map((entry) => entry.id)).toEqual([
      "unwrap",
      "move",
      "remove",
    ]);
    expect(getStructuredDraftGroupActionEntries(query, [], groupNode).map((entry) => entry.id)).toEqual([
      "addClause",
      "addAndGroup",
      "addOrGroup",
      "addNotGroup",
      "toggleGroup",
      "wrapNot",
      "move",
      "unwrap",
      "remove",
    ]);
  });

  it("routes add-clause shared-explorer field selection through an injected prompt callback", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature")]),
      limit: 20,
    }).request;
    const promptForClauseKind = vi.fn(async () => ({ kind: "apply", value: "field" }));
    const promptForSharedExplorerFieldOption = vi.fn(async () => ({ kind: "apply", value: traitsFieldOption }));
    const { getActions, openLiveExplorerGroupFieldByName, renderer } = renderStructuralActions({
      promptForClauseKind,
      promptForSharedExplorerFieldOption,
    });

    await getActions().addQueryClauseAtPath(query, []);

    expect(promptForSharedExplorerFieldOption).toHaveBeenCalled();
    expect(openLiveExplorerGroupFieldByName).toHaveBeenCalledWith(query, [], traitsFieldOption);
    renderer.unmount();
  });

  it("appends prompt-built clauses through bounded host mutation application", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature")]),
      limit: 20,
    }).request;
    const promptForClauseKind = vi.fn(async () => ({ kind: "apply", value: "level" }));
    const promptForClauseNode = vi.fn(async () => ({ kind: "apply", value: { kind: "level", match: { kind: "gte", value: 5 } } }));
    const { getActions, replacements, renderer } = renderStructuralActions({
      promptForClauseKind,
      promptForClauseNode,
    });

    await getActions().addQueryClauseAtPath(query, []);

    expect(replacements.at(-1)?.query.filter).toEqual(
      allOfFilter([scopeFilter("creature"), { kind: "level", match: { kind: "gte", value: 5 } }]),
    );
    renderer.unmount();
  });

  it("flattens prompt-built rarity selection groups when adding a plain clause", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      ]),
      limit: 20,
    }).request;
    const promptForClauseKind = vi.fn(async () => ({ kind: "apply", value: "rarity" }));
    const raritySelection = allOfFilter([
      { kind: "rarity", match: { kind: "eq", value: "common" } },
      notFilter({ kind: "rarity", match: { kind: "eq", value: "uncommon" } }),
    ]);
    const promptForClauseNode = vi.fn(async () => ({ kind: "apply", value: raritySelection }));
    const { getActions, replacements, renderer } = renderStructuralActions({
      promptForClauseKind,
      promptForClauseNode,
    });

    await getActions().addQueryClauseAtPath(query, []);

    expect(replacements.at(-1)?.query.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
        { kind: "rarity", match: { kind: "eq", value: "common" } },
        notFilter({ kind: "rarity", match: { kind: "eq", value: "uncommon" } }),
      ]),
    );
    renderer.unmount();
  });

  it("keeps multi-value rarity selections as rarity leaves instead of boolean branches", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
      ]),
      limit: 20,
    }).request;
    const promptForClauseKind = vi.fn(async () => ({ kind: "apply", value: "rarity" }));
    const raritySelection = allOfFilter([
      { kind: "rarity", match: { kind: "in", values: ["common", "uncommon"] } },
      { kind: "rarity", match: { kind: "notIn", values: ["rare", "unique"] } },
    ]);
    const promptForClauseNode = vi.fn(async () => ({ kind: "apply", value: raritySelection }));
    const { getActions, replacements, renderer } = renderStructuralActions({
      promptForClauseKind,
      promptForClauseNode,
    });

    await getActions().addQueryClauseAtPath(query, []);

    expect(replacements.at(-1)?.query.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" }),
        { kind: "rarity", match: { kind: "in", values: ["common", "uncommon"] } },
        { kind: "rarity", match: { kind: "notIn", values: ["rare", "unique"] } },
      ]),
    );
    renderer.unmount();
  });

  it("preserves explicit add-group wrappers around prompt-built clauses", async () => {
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature")]),
      limit: 20,
    }).request;
    const promptForClauseKind = vi.fn(async () => ({ kind: "apply", value: "rarity" }));
    const raritySelection = allOfFilter([
      { kind: "rarity", match: { kind: "eq", value: "common" } },
      notFilter({ kind: "rarity", match: { kind: "eq", value: "uncommon" } }),
    ]);
    const promptForClauseNode = vi.fn(async () => ({ kind: "apply", value: raritySelection }));
    const { getActions, replacements, renderer } = renderStructuralActions({
      promptForClauseKind,
      promptForClauseNode,
    });

    await getActions().addQueryClauseAtPath(query, [], "allOf");

    expect(replacements.at(-1)?.query.filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        allOfFilter([
          { kind: "rarity", match: { kind: "eq", value: "common" } },
          notFilter({ kind: "rarity", match: { kind: "eq", value: "uncommon" } }),
        ]),
      ]),
    );
    renderer.unmount();
  });

  it("routes leaf shared-explorer edits through injected explorer callbacks", async () => {
    const traitsNode = metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" });
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature"), traitsNode]),
      limit: 20,
    }).request;
    const {
      getActions,
      openLiveExplorerExactNodeFieldClauseFallback,
      openLiveExplorerGroupedField,
      renderer,
    } = renderStructuralActions();

    await getActions().runLeafAction(query, [1], traitsNode, "edit");

    expect(openLiveExplorerGroupedField.mock.calls.length + openLiveExplorerExactNodeFieldClauseFallback.mock.calls.length)
      .toBe(1);
    renderer.unmount();
  });

  it("applies canonical wrap, remove, toggle, unwrap, and move routing", async () => {
    const traitsNode = metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" });
    const query = browseQuery("Browse creatures", {
      filter: allOfFilter([scopeFilter("creature"), traitsNode]),
      limit: 20,
    }).request;
    const { enterStructuredDraftMoveMode, getActions, replacements, renderer } = renderStructuralActions();

    await getActions().runLeafAction(query, [1], traitsNode, "wrapNot");
    await getActions().runLeafAction(query, [1], traitsNode, "remove");
    await getActions().runGroupAction(query, [], query.filter as Extract<typeof query.filter, { kind: "allOf" }>, "toggleGroup");
    getActions().runNotAction(query, [1], notFilter(traitsNode), "unwrap");
    await getActions().runLeafAction(query, [1], traitsNode, "move");

    expect(replacements[0]?.query.filter).toEqual(allOfFilter([scopeFilter("creature"), notFilter(traitsNode)]));
    expect(replacements[1]?.query.filter).toEqual({ kind: "allOf", children: [scopeFilter("creature")] });
    expect(replacements[2]?.query.filter).toEqual({ kind: "anyOf", children: [scopeFilter("creature"), traitsNode] });
    expect(replacements[3]?.query.filter).toEqual(allOfFilter([scopeFilter("creature"), traitsNode]));
    expect(enterStructuredDraftMoveMode).toHaveBeenCalledWith([1]);
    renderer.unmount();
  });
});
