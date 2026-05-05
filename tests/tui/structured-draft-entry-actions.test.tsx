import React from "react";
import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { describe, expect, it, vi } from "vitest";

import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalSearchQuery } from "../../src/tui/search/service.js";
import type { SearchStructuredDraftEntry } from "../../src/tui/search/structured-draft-session.js";
import { useSearchStructuredDraftEntryActions } from "../../src/tui/search-screen/structured-draft/structured-draft-entry-actions.js";
import type { SearchWorkspaceUser } from "../../src/tui/search-screen/workspace/workspace-action-types.js";
import { allOfFilter, browseQuery, metadataPredicateFilter, scopeFilter } from "../helpers/search-request-fixture.js";

const hookMocks = vi.hoisted(() => ({
  editStructuredDraftStructuralEntry: vi.fn(async () => true),
  executeStructuredDraftEditRoute: vi.fn(async () => "applied"),
  getStructuredDraftStructuralEntryActions: vi.fn(() => []),
  openLiveExplorerGroupedField: vi.fn(async () => undefined),
  runStructuredDraftStructuralEntryAction: vi.fn(async () => true),
}));

vi.mock("../../src/tui/search-screen/structured-draft/structured-draft-explorer-actions.js", () => ({
  useStructuredDraftExplorerActions: () => ({
    openLiveExplorerGroupedField: hookMocks.openLiveExplorerGroupedField,
    openPromptFieldClause: vi.fn(),
    selectPromptMetricKey: vi.fn(),
  }),
}));

vi.mock("../../src/tui/search-screen/structured-draft/structured-draft-prompt-actions.js", () => ({
  useStructuredDraftPromptActions: () => ({
    promptForClauseKind: vi.fn(),
    promptForClauseNode: vi.fn(),
    promptForSharedExplorerFieldOption: vi.fn(),
  }),
}));

vi.mock("../../src/tui/search-screen/structured-draft/structured-draft-edit-route-actions.js", () => ({
  useStructuredDraftEditRouteActions: () => ({
    executeStructuredDraftEditRoute: hookMocks.executeStructuredDraftEditRoute,
  }),
}));

vi.mock("../../src/tui/search-screen/structured-draft/structured-draft-structural-actions.js", () => ({
  useStructuredDraftStructuralActions: () => ({
    editStructuredDraftStructuralEntry: hookMocks.editStructuredDraftStructuralEntry,
    getStructuredDraftStructuralEntryActions: hookMocks.getStructuredDraftStructuralEntryActions,
    runStructuredDraftStructuralEntryAction: hookMocks.runStructuredDraftStructuralEntryAction,
  }),
}));

type EntryActions = ReturnType<typeof useSearchStructuredDraftEntryActions>;

const traitsField = {
  value: "traits",
  label: "Traits",
  description: "Browse traits.",
  fieldType: "set",
  editor: "sharedExplorer",
} satisfies Pf2eTerminalQueryFieldOption;

const query = browseQuery("Browse creatures", {
  filter: allOfFilter([
    scopeFilter("creature"),
    metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
  ]),
  limit: 20,
}).request;

const bucketEntry = {
  kind: "queryFieldBucket",
  key: "bucket:traits",
  label: "Traits",
  description: "Trait clauses in this group.",
  groupPath: [],
  field: "traits",
  fieldOperator: "include",
  memberPaths: [[1]],
  fieldMemberPaths: [[1]],
  indent: 1,
  menuLabel: "Traits",
} satisfies SearchStructuredDraftEntry;

function renderEntryActions(structuredDraftQuery: Pf2eTerminalSearchQuery = query): {
  getActions: () => EntryActions;
  renderer: ReactTestRenderer;
} {
  let actions: EntryActions | null = null;
  const terminal = {
    pauseForAnyKey: vi.fn(),
    runPromptSession: vi.fn(),
  };

  function Harness(): null {
    actions = useSearchStructuredDraftEntryActions({
      clearStructuredDraftMoveSource: vi.fn(),
      editFieldClause: vi.fn(),
      enterStructuredDraftMoveMode: vi.fn(),
      getScopedFieldOptions: () => [traitsField],
      moveSourcePath: null,
      openFilterExplorer: vi.fn(),
      prompts: {
        promptMultiSelectOption: vi.fn(),
        promptOptionalSelectOption: vi.fn(),
        promptSelectOption: vi.fn(),
        promptTextInput: vi.fn(),
      },
      replaceStructuredDraftProjection: vi.fn(),
      setStructuredDraftResumeTarget: vi.fn(),
      structuredDraftQuery,
      terminal,
      user: {
        search: {},
      } as SearchWorkspaceUser,
    });
    return null;
  }

  let renderer: ReactTestRenderer | null = null;
  act(() => {
    renderer = create(<Harness />);
  });

  return {
    getActions: () => {
      if (!actions) {
        throw new Error("Entry actions did not render.");
      }
      return actions;
    },
    renderer: renderer!,
  };
}

describe("structured draft entry actions", () => {
  it("routes direct projected bucket edits through the edit route executor", async () => {
    hookMocks.executeStructuredDraftEditRoute.mockClear();
    hookMocks.openLiveExplorerGroupedField.mockClear();
    const { getActions, renderer } = renderEntryActions();

    await act(async () => {
      await getActions().editStructuredDraftMetadata(bucketEntry);
    });

    expect(hookMocks.openLiveExplorerGroupedField).not.toHaveBeenCalled();
    expect(hookMocks.executeStructuredDraftEditRoute).toHaveBeenCalledWith(
      query,
      expect.objectContaining({
        kind: "groupField",
        field: "traits",
        groupPath: [],
        memberPaths: [[1]],
        fieldMemberPaths: [[1]],
        source: "bucket",
      }),
    );
    renderer.unmount();
  });

  it("routes projected bucket edit actions through the edit route executor", async () => {
    hookMocks.executeStructuredDraftEditRoute.mockClear();
    hookMocks.openLiveExplorerGroupedField.mockClear();
    const { getActions, renderer } = renderEntryActions();

    await act(async () => {
      await getActions().runStructuredDraftEntryAction(bucketEntry, "edit");
    });

    expect(hookMocks.openLiveExplorerGroupedField).not.toHaveBeenCalled();
    expect(hookMocks.executeStructuredDraftEditRoute).toHaveBeenCalledWith(
      query,
      expect.objectContaining({
        kind: "groupField",
        field: "traits",
        groupPath: [],
        memberPaths: [[1]],
        fieldMemberPaths: [[1]],
        source: "bucket",
      }),
    );
    renderer.unmount();
  });
});
