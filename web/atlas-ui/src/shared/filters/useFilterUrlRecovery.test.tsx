import { atlasRouter } from "../../app/router";
import { createBrowserHistory } from "@tanstack/react-router";
import { StrictMode, useState } from "react";
import { renderHook, waitFor } from "@testing-library/react";
import {
  buildSearchRequest,
  decodeSearchStateFromParams,
  encodeSearchExecutionState,
  type SearchFormState,
} from "./searchState";
import { encodeFilterUrl } from "./filterUrl";
import { FILTER_URL_RESET_NOTICE, useFilterUrlRecovery } from "./useFilterUrlRecovery";

const info = vi.hoisted(() => vi.fn());
vi.mock("antd", async (original) => {
  const actual = await original<typeof import("antd")>();
  return {
    ...actual,
    App: Object.assign(actual.App, { useApp: () => ({ message: { info } }) }),
  };
});

beforeEach(() => {
  vi.clearAllMocks();
  history.replaceState(null, "", "/search");
});

it.each(["2.obsolete", "3.broken", "", "4.future"])(
  "recovers unreadable filter %s while retaining independent criteria",
  (filter) => {
    const state = decodeSearchStateFromParams(
      new URLSearchParams({
        q: "armor",
        filter,
        ranking: "rrf",
        sort: "name_asc",
        limit: "50",
      }),
    );
    expect(state).toMatchObject({
      query: "armor",
      ranking: "rrf",
      filter: null,
      filterError: null,
      filterWasReset: true,
      pageSize: 50,
      sort: { kind: "attributes", rules: [{ field: "name", direction: "asc" }] },
    });
    const { filterWasReset: _reset, ...clean } = state;
    expect(buildSearchRequest(state, 1)).toEqual(buildSearchRequest(clean, 1));
    expect(encodeSearchExecutionState(state)).toBe(encodeSearchExecutionState(clean));
  },
);

it("retains Similar, independent validation failures, and saved-list scope", () => {
  const state = decodeSearchStateFromParams(
    new URLSearchParams({
      similar: "actors:seed",
      filter: "2.old",
      ranking: "unknown",
      sort: "saved_order",
    }),
    "saved_list",
  );
  expect(state).toMatchObject({
    similarSeed: "actors:seed",
    filter: null,
    filterWasReset: true,
    filterError: "Unknown search ranking",
    sort: { kind: "saved_order" },
  });
});

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

it("clears the whole ambiguous legacy expression instead of salvaging a current clause", () => {
  const state = decodeSearchStateFromParams(
    new URLSearchParams({
      filter: encodeFilterUrl({ kind: "boolean_constant", value: false }),
      relationships: "old-secret-reference",
    }),
  );
  expect(state).toMatchObject({
    filter: null,
    filterWasReset: true,
    filterError: null,
  });
  expect(JSON.stringify(state)).not.toContain("old-secret-reference");
});

it.each(["/search/records/items:chosen", "/lists/my-list/records/items:chosen"])(
  "cleans only filter transport at %s, retains reading/history, and notifies once in StrictMode",
  async (path) => {
    const params = new URLSearchParams({
      filter: "2.obsolete",
      relationships: "old-secret",
      q: "armor",
      selection: '{"record_key":"items:chosen","embedded_path":[]}',
      landing: '{"field":"description"}',
      search_profile: "chosen-profile",
      sort: "random",
      seed: "0",
      limit: "50",
      ranking: "rrf",
      other: "retained",
    });
    atlasRouter.update({ history: createBrowserHistory() });
    history.replaceState(
      { atlasSearchPage: 9, atlasOwnerOrigin: { recordKey: "items:chosen" } },
      "",
      path + "?" + params + "#section",
    );
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
    await waitFor(() => expect(result.current.filterWasReset).toBeUndefined());
    await waitFor(() =>
      expect(new URLSearchParams(location.search).has("filter")).toBe(false),
    );
    expect(location.pathname).toBe(path);
    expect(location.hash).toBe("#section");
    const current = new URLSearchParams(location.search);
    params.delete("filter");
    params.delete("relationships");
    expect([...current]).toEqual([...params]);
    expect(history.length).toBe(initialLength);
    expect(history.state).toMatchObject({
      atlasSearchPage: 1,
      atlasOwnerOrigin: { recordKey: "items:chosen" },
    });
    expect(info).toHaveBeenCalledExactlyOnceWith({
      key: "filter-url-reset",
      content: FILTER_URL_RESET_NOTICE,
    });
    rerender();
    expect(info).toHaveBeenCalledTimes(1);
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
  expect(info).not.toHaveBeenCalled();
  expect(location.search).toBe("?" + params);
});
