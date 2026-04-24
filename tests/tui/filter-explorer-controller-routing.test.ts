import { describe, expect, it, vi } from "vitest";

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
    mode: {
      kind: "compose",
      resolveSelectionTarget: () => undefined,
    },
    onExit: vi.fn(),
    ...overrides,
  };
}

function createDraft(): FilterExplorerComposeDraft {
  return {
    selection: {},
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
      showNotification: vi.fn(),
    });

    expect(layoutDispatch).toHaveBeenCalledWith({ type: "toggle_layout" });
  });

  it("routes discovery-mode switching through the shared command palette path", async () => {
    const onModeChange = vi.fn();
    const promptCommandPalette = vi.fn(() => Promise.resolve("switchToMatching"));

    handleFilterExplorerInteractionRoute({
      route: {
        event: createEvent(),
        interactionAction: { id: "commands" },
      },
      adapters: {
        promptCommandPalette,
      } as never,
      browserContext: createBrowserContext(),
      options: createOptions({
        discovery: {
          mode: "catalog",
          onModeChange,
        },
      }),
      draft: createDraft(),
      updateDraft: () => {},
      dispatch: vi.fn(),
      showNotification: vi.fn(),
    });

    await Promise.resolve();

    expect(promptCommandPalette).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Filter Explorer Commands",
        entries: expect.arrayContaining([
          expect.objectContaining({
            value: "switchToMatching",
            label: "Use Matching Counts",
          }),
        ]),
      }),
    );
    expect(onModeChange).toHaveBeenCalledWith("matching");
  });
});
