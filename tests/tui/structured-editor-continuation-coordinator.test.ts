import { describe, expect, it, vi } from "vitest";

import type { MetadataFilterNode } from "../../src/tui/search/metadata-filter-draft.js";
import type {
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalQueryFieldOption,
} from "../../src/tui/search/service.js";
import type { SearchFilterExplorerFieldState } from "../../src/tui/search-screen/filter-explorer-field-state.js";
import { runStructuredDraftExplorerContinuation } from "../../src/tui/search-screen/structured-draft/structured-draft-continuation.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspaceUser,
} from "../../src/tui/search-screen/workspace/workspace-action-types.js";
import { browseQuery, metadataPredicateFilter, scopeFilter } from "../helpers/search-request-fixture.js";

const query = browseQuery("Browse creatures", {
  filter: scopeFilter("creature"),
}).request;

const sharedExplorerField: Pf2eTerminalQueryFieldOption = {
  value: "traits",
  label: "Traits",
  description: "Trait field.",
  fieldType: "set",
  editor: "sharedExplorer",
};

const promptField: Pf2eTerminalQueryFieldOption = {
  ...sharedExplorerField,
  value: "level",
  label: "Level",
  fieldType: "number",
  editor: "structuredForm",
};

function fieldState(include: string[]): SearchFilterExplorerFieldState {
  return {
    discreteSelections: {
      traits: {
        include,
        exclude: [],
      },
    },
    scalarClauses: {},
  };
}

function createUser(searchResult: Pf2eTerminalFilterExplorerInsertionResult, preservedMetadata: MetadataFilterNode | null) {
  return {
    search: {
      prepareFilterExplorerDraft: vi.fn(() => ({
        draft: {
          discreteClauses: [],
          scalarClauses: {},
        },
        preservedMetadata,
        scopedFields: ["traits"],
      })),
      buildFilterExplorerInsertionResult: vi.fn(() => searchResult),
    },
  } as unknown as SearchWorkspaceUser;
}

describe("structured editor continuation coordinator", () => {
  it("ignores non-explorer prompt fields so prompt-local builders stay outside the explorer path", async () => {
    const openFilterExplorer = vi.fn() as unknown as OpenSearchFilterExplorer;
    const user = createUser({ kind: "insert", nodes: [] }, null);

    await expect(
      runStructuredDraftExplorerContinuation({
        currentNode: null,
        fieldOption: promptField,
        openFilterExplorer,
        query,
        user,
      }),
    ).resolves.toEqual({ kind: "notOpened" });

    expect(openFilterExplorer).not.toHaveBeenCalled();
  });

  it("turns shared explorer live changes and back into one host resume result", async () => {
    const nextNode = metadataPredicateFilter({ field: "traits", op: "includes", value: "amphibious" });
    const insertionResult: Pf2eTerminalFilterExplorerInsertionResult = {
      kind: "insert",
      nodes: [nextNode],
    };
    const user = createUser(insertionResult, null);
    const onHostChange = vi.fn();
    let explorerOptions: Parameters<OpenSearchFilterExplorer>[0] | undefined;
    const openFilterExplorer: OpenSearchFilterExplorer = vi.fn(async (options) => {
      explorerOptions = options;
      return true;
    });
    const nextState = fieldState(["amphibious"]);
    const continuation = runStructuredDraftExplorerContinuation({
      currentNode: null,
      fieldOption: sharedExplorerField,
      onHostChange,
      openFilterExplorer,
      query,
      user,
    });

    expect(openFilterExplorer).toHaveBeenCalledWith(
      expect.objectContaining({
        queryOverride: query,
        fieldOptions: [sharedExplorerField],
        singleFieldBehavior: "directValues",
      }),
    );

    explorerOptions?.onQueryChange?.(query, nextState);
    explorerOptions?.onBack?.(query, nextState);

    await expect(continuation).resolves.toEqual({
      kind: "resumeHost",
      change: {
        result: insertionResult,
        query,
        fieldState: nextState,
      },
    });
    expect(onHostChange).toHaveBeenCalledWith({
      result: insertionResult,
      query,
      fieldState: nextState,
    });
    expect(user.search.buildFilterExplorerInsertionResult).toHaveBeenCalledWith(
      {
        discreteClauses: [{ field: "traits", value: "amphibious", operator: "include" }],
        scalarClauses: {},
      },
      {
        preservedMetadata: null,
        preferReplace: false,
      },
    );
  });

  it("preserves the latest live explorer change when canceling a node edit", async () => {
    const currentNode = metadataPredicateFilter({ field: "traits", op: "includes", value: "aquatic" });
    const preservedMetadata = metadataPredicateFilter({ field: "rarity", op: "is", value: "common" });
    const replacementResult: Pf2eTerminalFilterExplorerInsertionResult = {
      kind: "replace",
      node: metadataPredicateFilter({ field: "traits", op: "includes", value: "amphibious" }),
    };
    const user = createUser(replacementResult, preservedMetadata);
    let explorerOptions: Parameters<OpenSearchFilterExplorer>[0] | undefined;
    const openFilterExplorer: OpenSearchFilterExplorer = vi.fn(async (options) => {
      explorerOptions = options;
      return true;
    });
    const latestState = fieldState(["amphibious"]);
    const cancelState = fieldState(["aquatic"]);
    const continuation = runStructuredDraftExplorerContinuation({
      currentNode,
      fieldOption: sharedExplorerField,
      openFilterExplorer,
      query,
      user,
    });

    explorerOptions?.onQueryChange?.(query, latestState);
    explorerOptions?.onCancel?.(query, cancelState);

    await expect(continuation).resolves.toEqual({
      kind: "cancel",
      change: {
        result: replacementResult,
        query,
        fieldState: latestState,
      },
    });
    expect(user.search.prepareFilterExplorerDraft).toHaveBeenCalledWith(query, ["traits"]);
    expect(user.search.buildFilterExplorerInsertionResult).toHaveBeenCalledWith(
      expect.any(Object),
      {
        preservedMetadata,
        preferReplace: true,
      },
    );
  });
});
