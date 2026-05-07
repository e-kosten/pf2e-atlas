import { describe, expect, it, vi } from "vitest";

import type { SearchFilterNode } from "../../src/domain/search-request-types.js";
import type { Pf2eTerminalQueryFieldOption } from "../../src/tui/search/service.js";
import type { FilterExplorerSelectTargetOutcome } from "../../src/tui/filter-explorer/types.js";
import type { SearchFilterExplorerFieldState } from "../../src/tui/search-screen/filter-explorer-field-state.js";
import {
  runStructuredDraftExplorerChildSurface,
  runStructuredDraftExplorerContinuation,
  structuredDraftPromptApply,
  structuredDraftPromptBack,
  structuredDraftPromptCancel,
} from "../../src/tui/search-screen/structured-draft/structured-draft-continuation.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspaceUser,
} from "../../src/tui/search-screen/workspace/workspace-action-types.js";
import {
  browseQuery,
  metadataPredicateFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

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

function createUser(
  searchResult: { kind: "insert"; nodes: SearchFilterNode[] } | { kind: "replace"; node: SearchFilterNode },
  preservedFilter: SearchFilterNode | null,
) {
  return {
    search: {
      prepareFilterExplorerDraft: vi.fn(() => ({
        draft: {
          discreteClauses: [],
          scalarClauses: {},
        },
        preservedFilter,
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
        fieldOption: promptField,
        buildHostMutation: () => ({ kind: "appendNodes", nodes: [] }),
        openFilterExplorer,
        query,
        user,
      }),
    ).resolves.toEqual({ kind: "notOpened" });

    expect(openFilterExplorer).not.toHaveBeenCalled();
  });

  it("turns shared explorer live changes and back into one host resume result", async () => {
    const insertionResult = {
      kind: "insert",
      nodes: [],
    } as const;
    const user = createUser(insertionResult, null);
    const onHostChange = vi.fn();
    let explorerOptions: Parameters<OpenSearchFilterExplorer>[0] | undefined;
    const openFilterExplorer: OpenSearchFilterExplorer = vi.fn(async (options) => {
      explorerOptions = options;
      return true;
    });
    const nextState = fieldState(["amphibious"]);
    const continuation = runStructuredDraftExplorerContinuation({
      fieldOption: sharedExplorerField,
      buildHostMutation: (fieldStateValue) => ({
        kind: "replaceGroupedField",
        field: sharedExplorerField.value,
        fieldOption: sharedExplorerField,
        fieldState: fieldStateValue,
      }),
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

    explorerOptions?.onEvent?.({ kind: "change", query, fieldState: nextState });
    explorerOptions?.onEvent?.({ kind: "back", query, fieldState: nextState });

    await expect(continuation).resolves.toEqual({
      kind: "resumeHost",
      change: {
        mutation: {
          kind: "replaceGroupedField",
          field: "traits",
          fieldOption: sharedExplorerField,
          fieldState: nextState,
        },
        query,
        fieldState: nextState,
      },
    });
    expect(onHostChange).toHaveBeenCalledWith({
      mutation: {
        kind: "replaceGroupedField",
        field: "traits",
        fieldOption: sharedExplorerField,
        fieldState: nextState,
      },
      query,
      fieldState: nextState,
    });
    expect(user.search.buildFilterExplorerInsertionResult).not.toHaveBeenCalled();
  });

  it("preserves the latest live explorer change when canceling a node edit", async () => {
    const preservedFilter = metadataPredicateFilter({ field: "rarity", op: "is", value: "common" });
    const replacementResult = {
      kind: "replace",
      node: metadataPredicateFilter({ field: "traits", op: "includes", value: "amphibious" }),
    } as const;
    const user = createUser(replacementResult, preservedFilter);
    let explorerOptions: Parameters<OpenSearchFilterExplorer>[0] | undefined;
    const openFilterExplorer: OpenSearchFilterExplorer = vi.fn(async (options) => {
      explorerOptions = options;
      return true;
    });
    const latestState = fieldState(["amphibious"]);
    const cancelState = fieldState(["aquatic"]);
    const continuation = runStructuredDraftExplorerContinuation({
      fieldOption: sharedExplorerField,
      buildHostMutation: (fieldStateValue) => ({
        kind: "replaceGroupedField",
        field: sharedExplorerField.value,
        fieldOption: sharedExplorerField,
        fieldState: fieldStateValue,
      }),
      openFilterExplorer,
      query,
      user,
    });

    explorerOptions?.onEvent?.({ kind: "change", query, fieldState: latestState });
    explorerOptions?.onEvent?.({ kind: "cancel", query, fieldState: cancelState });

    await expect(continuation).resolves.toEqual({
      kind: "cancel",
      change: {
        mutation: {
          kind: "replaceGroupedField",
          field: "traits",
          fieldOption: sharedExplorerField,
          fieldState: latestState,
        },
        query,
        fieldState: latestState,
      },
    });
    expect(user.search.prepareFilterExplorerDraft).toHaveBeenCalledWith(query, ["traits"]);
    expect(user.search.buildFilterExplorerInsertionResult).not.toHaveBeenCalled();
  });

  it("only resolves select-target outcomes for coordinator callers that request target selection", async () => {
    const insertionResult = {
      kind: "insert",
      nodes: [],
    } as const;
    const user = createUser(insertionResult, null);
    let explorerOptions: Parameters<OpenSearchFilterExplorer>[0] | undefined;
    const openFilterExplorer: OpenSearchFilterExplorer = vi.fn(async (options) => {
      explorerOptions = options;
      return true;
    });
    const targetOutcome = {
      result: {
        target: {
          kind: "scalar",
          key: "actorMetric:ac.value",
          fieldLabel: "Creature Statistics",
          subjectLabel: "Armor Class",
          valueType: "number",
          editorLabel: "Creature Statistics / Armor Class",
        },
      },
    } as FilterExplorerSelectTargetOutcome;
    const latestState = fieldState([]);
    const continuation = runStructuredDraftExplorerChildSurface({
      fieldOption: sharedExplorerField,
      openFilterExplorer,
      query,
      resolveSelectionTarget: () => targetOutcome.result.target,
      title: "Creature Statistics",
      user,
    });

    explorerOptions?.onEvent?.({
      kind: "selectTarget",
      outcome: targetOutcome,
      query,
      fieldState: latestState,
      discoveryMode: "catalog",
    });

    await expect(continuation).resolves.toMatchObject({
      kind: "selectTarget",
      outcome: targetOutcome,
      discoveryMode: "catalog",
    });
    expect(openFilterExplorer).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Creature Statistics",
        resolveSelectionTarget: expect.any(Function),
      }),
    );
  });

  it("lets the host translate shared explorer field changes into grouped-field mutations", async () => {
    const insertionResult = {
      kind: "insert",
      nodes: [],
    } as const;
    const user = createUser(insertionResult, null);
    let explorerOptions: Parameters<OpenSearchFilterExplorer>[0] | undefined;
    const openFilterExplorer: OpenSearchFilterExplorer = vi.fn(async (options) => {
      explorerOptions = options;
      return true;
    });
    const nextState = fieldState(["amphibious"]);
    const continuation = runStructuredDraftExplorerContinuation({
      fieldOption: sharedExplorerField,
      buildHostMutation: (fieldStateValue) => ({
        kind: "replaceGroupedField",
        field: sharedExplorerField.value,
        fieldOption: sharedExplorerField,
        fieldState: fieldStateValue,
      }),
      openFilterExplorer,
      query,
      user,
    });

    explorerOptions?.onEvent?.({ kind: "back", query, fieldState: nextState });

    await expect(continuation).resolves.toMatchObject({
      kind: "resumeHost",
      change: {
        mutation: {
          kind: "replaceGroupedField",
          field: "traits",
          fieldState: nextState,
        },
      },
    });
  });

  it("exposes prompt child outcomes as explicit coordinator-consumable results", () => {
    expect(structuredDraftPromptApply("field")).toEqual({ kind: "apply", value: "field" });
    expect(structuredDraftPromptBack()).toEqual({ kind: "back" });
    expect(structuredDraftPromptCancel()).toEqual({ kind: "cancel" });
  });
});
