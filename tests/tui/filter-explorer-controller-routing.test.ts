import { describe, expect, it, vi } from "vitest";

import { createDerivedTagTerminalActionTargetState } from "../../src/tui/action-target.js";
import { createComposeFilterExplorerHostAdapter } from "../../src/tui/filter-explorer/host-adapter.js";
import { handleFilterExplorerInteractionRoute } from "../../src/tui/filter-explorer/controller-routing.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerComposeDraft,
  FilterExplorerModel,
  FilterExplorerOptions,
} from "../../src/tui/filter-explorer/types.js";

function createModel(): FilterExplorerModel {
  return {
    id: "search-semantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [],
  };
}

function createBrowserContext(overrides: Partial<FilterExplorerBrowserContext> = {}): FilterExplorerBrowserContext {
  const state = {
    activePane: "list" as const,
    browserState: { depth: 0, selectedNodeIds: ["spell:field:traits"], filter: "", detailScroll: 0 },
    layoutMode: "split" as const,
    searchInput: "",
    searchMode: false,
  };

  return {
    state,
    effectiveState: state.browserState,
    selection: { ancestors: [], currentNodes: [], currentNode: undefined, currentParent: undefined },
    currentNode: {
      id: "spell:field:traits",
      kind: "field",
      label: "Traits",
      filterText: "traits",
      detailLines: [{ text: "Traits" }],
    },
    currentNodeHasChildren: false,
    breadcrumb: "Search Semantics > Spell > Traits",
    bodyHeight: 20,
    detailWidth: 40,
    detailLines: [],
    visibleDetailLines: [],
    detailTitle: "Detail",
    layoutMode: "split",
    maxDetailScroll: 0,
    detailJumpSize: 5,
    detailPageSize: 10,
    selectionJumpSize: 5,
    searchIndicator: "",
    ...overrides,
  };
}

function createOptions(overrides: Partial<FilterExplorerOptions> = {}): FilterExplorerOptions {
  return {
    model: createModel(),
    host: createComposeFilterExplorerHostAdapter({
      resolveTarget: () => undefined,
    }),
    mode: {
      kind: "compose",
    },
    onOutcome: vi.fn(),
    ...overrides,
  };
}

function createDraft(): FilterExplorerComposeDraft {
  return {
    discreteClauses: [],
    scalarClauses: {},
  };
}

function createEvent() {
  return {
    input: "\r",
    key: { return: true },
    isBackNavigationKey: () => false,
    isCommandPaletteKey: () => false,
    isConfirmKey: () => true,
    isConfirmOrToggleKey: () => true,
    isExactPrintableKey: () => false,
    isExecuteKey: () => true,
    isFocusToggleKey: () => false,
    isHelpKey: () => false,
    isLayoutToggleKey: () => false,
    isMoveDownKey: () => false,
    isMoveLeftKey: () => false,
    isMoveRightKey: () => true,
    isMoveUpKey: () => false,
    isPageDownKey: () => false,
    isPageUpKey: () => false,
    isSearchKey: () => false,
    isTerminalBoundaryEndKey: () => false,
    isTerminalBoundaryStartKey: () => false,
    isTerminalJumpBackwardKey: () => false,
    isTerminalJumpForwardKey: () => false,
    isTerminalQuitKey: () => false,
    getCycleDirection: () => undefined,
  };
}

describe("filter explorer controller routing", () => {
  it("emits an explicit back outcome when back leaves the root level", () => {
    const onOutcome = vi.fn();

    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        interactionAction: { id: "back" },
      },
      adapters: {} as never,
      browserContext: createBrowserContext(),
      options: createOptions({ onOutcome }),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch: vi.fn(),
      actionEntries: [],
      actionTargetState: createDerivedTagTerminalActionTargetState(),
      dispatchActionTarget: vi.fn(),
      showNotification: vi.fn(),
    });

    expect(onOutcome).toHaveBeenCalledWith(
      { kind: "back" },
      expect.objectContaining({
        activePane: "list",
        browserState: { depth: 0, selectedNodeIds: ["spell:field:traits"], filter: "", detailScroll: 0 },
      }),
    );
  });

  it("emits an explicit exitRoot outcome for return-based root exit", () => {
    const onOutcome = vi.fn();

    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        interactionAction: { id: "return" },
      },
      adapters: {} as never,
      browserContext: createBrowserContext(),
      options: createOptions({ onOutcome }),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch: vi.fn(),
      actionEntries: [],
      actionTargetState: createDerivedTagTerminalActionTargetState(),
      dispatchActionTarget: vi.fn(),
      showNotification: vi.fn(),
    });

    expect(onOutcome).toHaveBeenCalledWith(
      { kind: "exitRoot" },
      expect.objectContaining({
        activePane: "list",
        browserState: { depth: 0, selectedNodeIds: ["spell:field:traits"], filter: "", detailScroll: 0 },
      }),
    );
  });

  it("emits an explicit cancel outcome for shared quit handling", () => {
    const onOutcome = vi.fn();

    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        interactionAction: { id: "quit" },
      },
      adapters: {} as never,
      browserContext: createBrowserContext(),
      options: createOptions({ onOutcome }),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch: vi.fn(),
      actionEntries: [],
      actionTargetState: createDerivedTagTerminalActionTargetState(),
      dispatchActionTarget: vi.fn(),
      showNotification: vi.fn(),
    });

    expect(onOutcome).toHaveBeenCalledWith(
      { kind: "cancel" },
      expect.objectContaining({
        activePane: "list",
        browserState: { depth: 0, selectedNodeIds: ["spell:field:traits"], filter: "", detailScroll: 0 },
      }),
    );
  });

  it("shows a transient notification instead of toggling focus when drill-in hits a dead end", () => {
    const dispatch = vi.fn();
    const showNotification = vi.fn();

    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        listNavigationAction: { kind: "confirm" },
      },
      adapters: {} as never,
      browserContext: createBrowserContext(),
      options: createOptions(),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch,
      actionEntries: [],
      actionTargetState: createDerivedTagTerminalActionTargetState(),
      dispatchActionTarget: vi.fn(),
      showNotification,
    });

    expect(dispatch).not.toHaveBeenCalledWith({ type: "toggle_focus" });
    expect(showNotification).toHaveBeenCalledWith({
      message: "No rightward action is available for the focused entry.",
      tone: "warning",
    });
  });

  it("keeps explicit focus and layout actions working normally", () => {
    const focusDispatch = vi.fn();
    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        interactionAction: { id: "focus" },
      },
      adapters: {} as never,
      browserContext: createBrowserContext(),
      options: createOptions(),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch: focusDispatch,
      actionEntries: [],
      actionTargetState: createDerivedTagTerminalActionTargetState(),
      dispatchActionTarget: vi.fn(),
      showNotification: vi.fn(),
    });

    expect(focusDispatch).toHaveBeenCalledWith({ type: "toggle_focus" });

    const layoutDispatch = vi.fn();
    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        interactionAction: { id: "layout" },
      },
      adapters: {} as never,
      browserContext: createBrowserContext(),
      options: createOptions(),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch: layoutDispatch,
      actionEntries: [],
      actionTargetState: createDerivedTagTerminalActionTargetState(),
      dispatchActionTarget: vi.fn(),
      showNotification: vi.fn(),
    });

    expect(layoutDispatch).toHaveBeenCalledWith({ type: "toggle_layout" });
  });

  it("routes discovery-mode switching through the shared action rail path", async () => {
    const onModeChange = vi.fn();
    const dispatchActionTarget = vi.fn();

    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        actionTargetIntent: { kind: "apply_action" },
      } as never,
      adapters: {} as never,
      browserContext: createBrowserContext(),
      options: createOptions({
        discovery: {
          mode: "catalog",
          modes: [
            {
              value: "matching",
              label: "Matching Counts",
              description: "Show matching counts.",
            },
            {
              value: "catalog",
              label: "Catalog Counts",
              description: "Show catalog counts.",
            },
          ],
          onModeChange,
        },
      }),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch: vi.fn(),
      actionEntries: [
        {
          id: "setMode:matching",
          label: "Use Matching Counts",
          description: "Show matching counts.",
          action: {
            kind: "setMode",
            mode: "matching",
          },
        },
      ],
      actionTargetState: { activeTarget: "actions", selectedActionIndex: 0 },
      dispatchActionTarget,
      showNotification: vi.fn(),
    });

    expect(onModeChange).toHaveBeenCalledWith("matching");
    expect(dispatchActionTarget).not.toHaveBeenCalled();
  });
});
