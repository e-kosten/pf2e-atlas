import { atlasRouter } from "../../app/router";
import { createBrowserHistory } from "@tanstack/react-router";
import { StrictMode, useState } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import {
  DEFAULT_SEARCH_STATE,
  buildSearchRequest,
  decodeSearchStateFromParams,
  hasExecutableSearch,
  type SearchFormState,
} from "./searchState";
import { encodeFilterUrl } from "./filterUrl";
import { useFilterUrlRecovery } from "./useFilterUrlRecovery";

beforeEach(() => {
  history.replaceState(null, "", "/search");
});

it.each(["2.obsolete", "3.broken", "", "4.future"])(
  "blocks all execution and discards criteria for unreadable filter %s",
  (filter) => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams({
        q: "armor",
        similar: "actors:seed",
        filter,
        ranking: "rrf",
        sort: "name_asc",
        limit: "50",
      }),
    );
    expect(state).toEqual({ ...DEFAULT_SEARCH_STATE, filterWasReset: true });
    expect(hasExecutableSearch(state)).toBe(false);
    expect(() => buildSearchRequest(state, 1)).toThrow("must be reset");
  },
);

it("keeps decoded predicates for backend validation instead of dropping unavailable fields", () => {
  const filter = {
    kind: "compare" as const,
    field: "removed.field",
    op: "eq" as const,
    value: 0,
  };
  const state = decodeSearchStateFromParams(
    new URLSearchParams({ filter: encodeFilterUrl(filter) }),
  );
  expect(state.filter).toMatchObject(filter);
  expect(state.filterWasReset).toBeUndefined();
});

it("treats conflicting filter duplicates as unreadable but accepts equal duplicates", () => {
  const filter = encodeFilterUrl({ kind: "boolean_constant", value: false });
  expect(
    decodeSearchStateFromParams(new URLSearchParams({ filter })).filterWasReset,
  ).toBeUndefined();
  expect(
    decodeSearchStateFromParams(
      new URLSearchParams([
        ["filter", filter],
        ["filter", filter],
      ]),
    ).filterWasReset,
  ).toBeUndefined();
  expect(
    decodeSearchStateFromParams(
      new URLSearchParams([
        ["filter", filter],
        ["filter", "2.old"],
      ]),
    ),
  ).toEqual({ ...DEFAULT_SEARCH_STATE, filterWasReset: true });
});

it("rejects the whole ambiguous retired expression", () => {
  const state = decodeSearchStateFromParams(
    new URLSearchParams({
      filter: encodeFilterUrl({ kind: "boolean_constant", value: false }),
      relationships: "old-secret-reference",
    }),
  );
  expect(state).toEqual({ ...DEFAULT_SEARCH_STATE, filterWasReset: true });
});

it.each(["/search/records/items:chosen", "/lists/my-list/records/items:chosen"])(
  "replaces %s with empty Search and one transient notice in StrictMode",
  async (path) => {
    const params = new URLSearchParams({
      filter: "2.obsolete",
      q: "armor",
      rank: "4",
      search_profile: "chosen-profile",
      sort: "random",
      seed: "0",
      limit: "50",
      ranking: "rrf",
      page: "9",
      record_view: "explore",
    });
    atlasRouter.update({ history: createBrowserHistory() });
    history.replaceState(null, "", path + "?" + params + "#section");
    atlasRouter.updateLatestLocation();
    const initialLength = history.length;
    const restored = decodeSearchStateFromParams(params);
    const { result, rerender } = renderHook(
      () => {
        const [search, setSearch] = useState(restored);
        useFilterUrlRecovery(search, setSearch);
        return search;
      },
      { wrapper: StrictMode },
    );
    await waitFor(() => expect(result.current).toEqual(DEFAULT_SEARCH_STATE));
    expect(location.pathname).toBe("/search");
    expect(location.search).toBe("");
    expect(location.hash).toBe("");
    expect(history.length).toBe(initialLength);
    expect(history.state).toMatchObject({
      atlasSearchReset: expect.any(String),
      atlasFilterResetNotice: true,
    });
    const token = history.state.atlasSearchReset;
    rerender();
    expect(history.state.atlasSearchReset).toBe(token);
  },
);

it("never clears a decoded backend or validation error", () => {
  const params = new URLSearchParams({
    filter: encodeFilterUrl({ kind: "boolean_constant", value: false }),
  });
  history.replaceState(null, "", "/search?" + params);
  const search: SearchFormState = {
    ...decodeSearchStateFromParams(params),
    filterError: "Record unavailable",
  };
  const recover = vi.fn();
  renderHook(() => useFilterUrlRecovery(search, recover));
  expect(recover).not.toHaveBeenCalled();
  expect(location.search).toBe("?" + params);
});
