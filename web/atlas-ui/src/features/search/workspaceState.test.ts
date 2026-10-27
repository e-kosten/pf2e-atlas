import { atlasRouter } from "../../app/router";
import { encodeFilterUrl } from "../../shared/filters/filterUrl";
import { fieldFixture } from "../../test/fixtures";
import { DEFAULT_SEARCH_STATE } from "../../shared/filters/searchState";
import {
  initialWorkspaceInteractionState as readInitialState,
  rememberKindPreference,
  KIND_PREFERENCE_KEY,
  workspaceInteractionReducer,
  type WorkspaceInteractionState,
} from "./workspaceState";

describe("workspaceInteractionReducer", () => {
  it("changes search immediately without resetting execution state", () => {
    const state = baseState();
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
  });

  it("commits execution without owning URL pagination", () => {
    const nextSearch = {
      ...DEFAULT_SEARCH_STATE,
      query: "fireball",
      mode: "text_search" as const,
    };

    const next = workspaceInteractionReducer(baseState(), {
      type: "search.executionCommitted",
      search: nextSearch,
    });

    expect(next.activeSearch).toBe(nextSearch);
  });

  it("does not reset state when the committed search is already active", () => {
    const state = baseState();

    const next = workspaceInteractionReducer(state, {
      type: "search.executionCommitted",
      search: state.activeSearch,
    });

    expect(next).toBe(state);
  });

  it("restores URL state as one transition", () => {
    const restoredSearch = {
      ...DEFAULT_SEARCH_STATE,
      filter: {
        kind: "in" as const,
        clause_id: "kind",
        field: "kind",
        values: ["creature"],
      },
    };

    const next = workspaceInteractionReducer(baseState(), {
      type: "url.restored",
      search: restoredSearch,
      selectedRecordKey: "spell:heal",
    });

    expect(next.search).toBe(restoredSearch);
    expect(next.activeSearch).toBe(restoredSearch);
    expect(next.selectedRecordKey).toBe("spell:heal");
  });

  it("tracks result and record interactions explicitly", () => {
    const selected = workspaceInteractionReducer(baseState(), {
      type: "record.selected",
      recordKey: "spell:heal",
    });
    expect(selected.selectedRecordKey).toBe("spell:heal");
  });
});

function baseState(
  overrides: Partial<WorkspaceInteractionState> = {},
): WorkspaceInteractionState {
  return {
    search: DEFAULT_SEARCH_STATE,
    selectedRecordKey: null,
    activeSearch: DEFAULT_SEARCH_STATE,
    ...overrides,
  };
}

describe("remembered Kind default precedence", () => {
  const field = fieldFixture("kind");
  const predicate = {
    kind: "all_of" as const,
    children: [
      { kind: "in" as const, field: field.path, values: ["creature"] },
      {
        kind: "not" as const,
        predicate: { kind: "in" as const, field: field.path, values: ["hazard"] },
      },
    ],
  };
  beforeEach(() => localStorage.clear());
  afterEach(() => localStorage.clear());
  it("remembers only bounded literal preferences and restores them on a fresh bare search", () => {
    rememberKindPreference(predicate, field);
    expect(JSON.parse(localStorage.getItem(KIND_PREFERENCE_KEY)!)).toEqual({
      include: ["creature"],
      exclude: ["hazard"],
    });
    history.replaceState(null, "", "/search");
    expect(initialWorkspaceInteractionState().search.filter).toMatchObject({
      kind: "all_of",
      children: [
        { kind: "in", values: ["creature"] },
        { kind: "not", predicate: { values: ["hazard"] } },
      ],
    });
  });
  it.each(["?q=heal", "?filter=null", "?filter=bad", "?pageSize=50", "?mode=browse"])(
    "keeps explicit incoming state authoritative: %s",
    (query) => {
      rememberKindPreference(predicate, field);
      history.replaceState(null, "", "/search" + query);
      expect(initialWorkspaceInteractionState().search.filter).toBeNull();
    },
  );
  it("uses a deep linked predicate exactly and clears the preference when Kind is removed", () => {
    rememberKindPreference(predicate, field);
    const incoming = {
      kind: "in" as const,
      clause_id: "url",
      field: field.path,
      values: ["spell"],
    };
    history.replaceState(
      null,
      "",
      "/search?filter=" + encodeURIComponent(encodeFilterUrl(incoming)),
    );
    expect(initialWorkspaceInteractionState().search.filter).toMatchObject({
      ...incoming,
      clause_id: expect.any(String),
    });
    rememberKindPreference(null, field);
    expect(localStorage.getItem(KIND_PREFERENCE_KEY)).toBeNull();
    history.replaceState(null, "", "/search");
    expect(initialWorkspaceInteractionState().search.filter).toBeNull();
  });
  it.each([
    { include: [1], exclude: [] },
    { include: ["creature"], exclude: ["creature"] },
    { include: [""], exclude: [] },
    { include: ["spell", "spell"], exclude: [] },
  ])("ignores malformed remembered literals: %j", (stored) => {
    localStorage.setItem(KIND_PREFERENCE_KEY, JSON.stringify(stored));
    history.replaceState(null, "", "/search");
    expect(initialWorkspaceInteractionState().search.filter).toBeNull();
  });
});

it("resets restored paging before executing a recovered URL without adding a remembered Kind", () => {
  localStorage.setItem(
    KIND_PREFERENCE_KEY,
    JSON.stringify({ kind: "in", field: "kind", values: ["creature"] }),
  );
  history.replaceState(null, "", "/search?filter=2.obsolete&page=9");
  const state = initialWorkspaceInteractionState();
  expect(state).not.toHaveProperty("pageNumber");
  expect(state.search.filter).toBeNull();
  expect(state.search.filterWasReset).toBe(true);
  localStorage.clear();
});

function initialWorkspaceInteractionState() {
  atlasRouter.updateLatestLocation();
  return readInitialState();
}

it("suppresses remembered Kind defaults for an empty recovery entry", () => {
  localStorage.setItem(
    KIND_PREFERENCE_KEY,
    JSON.stringify({ include: ["creature"], exclude: [] }),
  );
  history.replaceState({ atlasSearchReset: "recovery-entry" }, "", "/search");
  const state = initialWorkspaceInteractionState();
  expect(state.search.filter).toBeNull();
  expect(state.activeSearch.filter).toBeNull();
  localStorage.clear();
});
