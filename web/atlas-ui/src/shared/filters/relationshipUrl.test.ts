import { encodeFilterUrl, decodeFilterUrl } from "./filterUrl";
import { linkLeaf } from "./linkPredicates";
import { rootNavigation } from "../records/useReferenceReading";
import {
  DEFAULT_SEARCH_STATE,
  buildSearchRequest,
  decodeSearchStateFromParams,
  encodeSearchExecutionState,
  hasExecutableSearch,
  searchStateQueryString,
} from "./searchState";
const leaf = linkLeaf(
  {
    direction: "incoming",
    include_contents: true,
    selection: {
      ...rootNavigation("actors:exact"),
      embedded_path: [
        { collection: "items", identity: { SnapshotLocal: { index: 4 } } },
      ],
      source_fingerprint: "checked-source",
    },
  },
  "initial",
);
it("round trips exact link selections inside mixed AND/OR/NOT with independent sort", () => {
  const filter = {
    kind: "any_of" as const,
    children: [
      leaf,
      {
        kind: "not" as const,
        predicate: {
          ...leaf,
          clause_id: "other",
          relationship: {
            ...leaf.relationship,
            direction: "outgoing" as const,
            include_contents: false,
          },
        },
      },
    ],
  };
  const state = {
    ...DEFAULT_SEARCH_STATE,
    filter,
    sort: { kind: "random" as const, seed: 0 },
  };
  const restored = decodeSearchStateFromParams(
    new URLSearchParams(searchStateQueryString(state)),
  );
  expect(restored.filter).toEqual(decodeFilterUrl(encodeFilterUrl(filter)));
  expect(restored.sort).toEqual(state.sort);
  expect(buildSearchRequest(restored, 2).criteria.filter).toEqual(restored.filter);
  expect(encodeSearchExecutionState(restored)).not.toBe(
    encodeSearchExecutionState({ ...restored, filter: leaf }),
  );
});
it("preserves the unified tree with text and Similar requests", () => {
  const base = { ...DEFAULT_SEARCH_STATE, filter: leaf };
  expect(buildSearchRequest({ ...base, query: "armor" }, 1).criteria).toMatchObject({
    kind: "text_search",
    filter: leaf,
  });
  expect(
    buildSearchRequest({ ...base, similarSeed: "items:seed" }, 1).criteria,
  ).toMatchObject({ kind: "similar_records", filter: leaf });
});
it("marks the retired array for automatic recovery without leaking its raw IDs", () => {
  const state = decodeSearchStateFromParams(
    new URLSearchParams({
      relationships: '1.[{"selection":{"record_key":"secret:id"}}]',
    }),
  );
  expect(state.filterError).toBeNull();
  expect(state.filter).toBeNull();
  expect(state.filterWasReset).toBe(true);
  expect(JSON.stringify(state)).not.toContain("secret:id");
  expect(hasExecutableSearch(state)).toBe(false);
});
