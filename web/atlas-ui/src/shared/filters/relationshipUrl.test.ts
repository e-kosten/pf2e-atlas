import type { RelationshipConstraintView } from "../../generated/atlas";
import {
  decodeRelationships,
  encodeRelationships,
  MAX_RELATIONSHIP_FILTERS,
} from "./relationshipUrl";
import {
  buildSearchRequest,
  DEFAULT_SEARCH_STATE,
  decodeSearchStateFromParams,
  encodeSearchExecutionState,
  hasExecutableSearch,
  searchStateQueryString,
} from "./searchState";

const constraint: RelationshipConstraintView = {
  direction: "incoming",
  selection: {
    record_key: "actors:exact",
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 4 } } }],
    field: null,
    passage: null,
    source_fingerprint: "source-current",
  },
  include_contents: true,
};

it("round trips the exact relationship selection and per-constraint source scope", () => {
  const search = { ...DEFAULT_SEARCH_STATE, relationships: [constraint] };
  const restored = decodeSearchStateFromParams(
    new URLSearchParams(searchStateQueryString(search)),
  );
  expect(restored.relationships).toEqual([constraint]);
  expect(hasExecutableSearch(restored)).toBe(true);
  expect(buildSearchRequest(restored, 3).relationships).toEqual([constraint]);
  expect(encodeSearchExecutionState(restored)).not.toBe(
    encodeSearchExecutionState({
      ...restored,
      relationships: [{ ...constraint, include_contents: false }],
    }),
  );
});

it("keeps relationship constraints with both text and Similar search requests", () => {
  const base = { ...DEFAULT_SEARCH_STATE, relationships: [constraint] };
  expect(buildSearchRequest({ ...base, query: "armor" }, 1)).toMatchObject({
    relationships: [constraint],
    criteria: { kind: "text_search", query: "armor" },
  });
  expect(buildSearchRequest({ ...base, similarSeed: "items:seed" }, 1)).toMatchObject({
    relationships: [constraint],
    criteria: { kind: "similar_records", seed_record_key: "items:seed" },
  });
});

it("rejects malformed relationship links without displaying their raw contents", () => {
  expect(() => decodeRelationships("2.[]")).toThrow("Invalid relationship filter URL");
  const state = decodeSearchStateFromParams(
    new URLSearchParams({ relationships: '1.[{"direction":"secret:record-key"}]' }),
  );
  expect(state.filterError).toContain("Invalid relationship filter URL");
  expect(state.filterError).not.toContain("secret:record-key");
  expect(hasExecutableSearch(state)).toBe(false);
  expect(decodeRelationships(encodeRelationships([constraint]))).toEqual([constraint]);
});

it("round trips multiple ordered constraints with exact embedded scope and explicit Random order", () => {
  const constraints = [
    constraint,
    {
      ...constraint,
      direction: "outgoing" as const,
      include_contents: false,
      selection: { ...constraint.selection, embedded_path: [] },
    },
  ];
  const search = {
    ...DEFAULT_SEARCH_STATE,
    relationships: constraints,
    sort: { kind: "random" as const, seed: 0 },
  };
  const restored = decodeSearchStateFromParams(
    new URLSearchParams(searchStateQueryString(search)),
  );
  expect(restored.relationships).toEqual(constraints);
  expect(restored.sort).toEqual(search.sort);
  expect(buildSearchRequest(restored, 2).relationships).toEqual(constraints);
  expect(encodeSearchExecutionState(restored)).not.toBe(
    encodeSearchExecutionState({
      ...restored,
      relationships: [...constraints].reverse(),
    }),
  );
});

it("uses the same collection limit for admitted relationship links", () => {
  const full = Array.from({ length: MAX_RELATIONSHIP_FILTERS }, () => constraint);
  expect(decodeRelationships(encodeRelationships(full))).toHaveLength(
    MAX_RELATIONSHIP_FILTERS,
  );
  expect(() => decodeRelationships(encodeRelationships([...full, constraint]))).toThrow(
    "Invalid relationship filter URL",
  );
});
