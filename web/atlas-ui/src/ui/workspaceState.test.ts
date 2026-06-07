import { DEFAULT_SEARCH_STATE } from "../state/searchState";
import {
  recordKeyFromPath,
  workspaceInteractionReducer,
  type WorkspaceInteractionState,
} from "./workspaceState";

describe("workspaceInteractionReducer", () => {
  it("changes search immediately without resetting execution state", () => {
    const state = baseState({
      pageNumber: 3,
      focusedResultKey: "spell:heal",
    });
    const nextSearch = {
      ...DEFAULT_SEARCH_STATE,
      query: "heal",
      mode: "text_search" as const,
    };

    const next = workspaceInteractionReducer(state, {
      type: "search.changed",
      search: nextSearch,
    });

    expect(next.search).toBe(nextSearch);
    expect(next.activeSearch).toBe(state.activeSearch);
    expect(next.pageNumber).toBe(3);
    expect(next.focusedResultKey).toBe("spell:heal");
  });

  it("resets page and focus when search execution commits", () => {
    const nextSearch = {
      ...DEFAULT_SEARCH_STATE,
      query: "fireball",
      mode: "text_search" as const,
    };

    const next = workspaceInteractionReducer(
      baseState({ pageNumber: 4, focusedResultKey: "spell:heal" }),
      { type: "search.executionCommitted", search: nextSearch },
    );

    expect(next.activeSearch).toBe(nextSearch);
    expect(next.pageNumber).toBe(1);
    expect(next.focusedResultKey).toBeNull();
  });

  it("does not reset state when the committed search is already active", () => {
    const state = baseState({
      pageNumber: 4,
      focusedResultKey: "spell:heal",
    });

    const next = workspaceInteractionReducer(state, {
      type: "search.executionCommitted",
      search: state.activeSearch,
    });

    expect(next).toBe(state);
  });

  it("restores URL state as one transition", () => {
    const restoredSearch = {
      ...DEFAULT_SEARCH_STATE,
      kinds: ["creature"],
    };

    const next = workspaceInteractionReducer(
      baseState({ pageNumber: 3, focusedResultKey: "spell:heal" }),
      {
        type: "url.restored",
        search: restoredSearch,
        selectedRecordKey: "spell:heal",
      },
    );

    expect(next.search).toBe(restoredSearch);
    expect(next.activeSearch).toBe(restoredSearch);
    expect(next.selectedRecordKey).toBe("spell:heal");
    expect(next.pageNumber).toBe(1);
    expect(next.focusedResultKey).toBeNull();
  });

  it("tracks result and record interactions explicitly", () => {
    const focused = workspaceInteractionReducer(baseState(), {
      type: "result.focused",
      recordKey: "spell:heal",
    });
    const selected = workspaceInteractionReducer(focused, {
      type: "record.selected",
      recordKey: "spell:heal",
    });

    expect(selected.focusedResultKey).toBe("spell:heal");
    expect(selected.selectedRecordKey).toBe("spell:heal");
  });

  it("clears focused result when the page changes", () => {
    const next = workspaceInteractionReducer(
      baseState({ pageNumber: 1, focusedResultKey: "spell:heal" }),
      { type: "resultPage.changed", pageNumber: 2 },
    );

    expect(next.pageNumber).toBe(2);
    expect(next.focusedResultKey).toBeNull();
  });
});

describe("recordKeyFromPath", () => {
  it("decodes first-class record routes", () => {
    expect(recordKeyFromPath("/records/spell%3Aheal")).toBe("spell:heal");
    expect(recordKeyFromPath("/")).toBeNull();
  });
});

function baseState(
  overrides: Partial<WorkspaceInteractionState> = {},
): WorkspaceInteractionState {
  return {
    search: DEFAULT_SEARCH_STATE,
    selectedRecordKey: null,
    focusedResultKey: null,
    pageNumber: 1,
    activeSearch: DEFAULT_SEARCH_STATE,
    ...overrides,
  };
}
