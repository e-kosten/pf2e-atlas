import type { RelationshipConstraintView } from "../../generated/atlas";
import { relationshipSearchRoute } from "./relationshipSearch";
import {
  decodeSearchStateFromParams,
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
} from "../filters/searchState";

it("replaces one record-origin relationship while retaining text, filters and exact checked selection", () => {
  const constraint: RelationshipConstraintView = {
    direction: "outgoing",
    selection: {
      record_key: "actors:selected",
      embedded_path: [{ collection: "items", identity: { Stable: "ability" } }],
      field: null,
      passage: null,
      source_fingerprint: "current-fingerprint",
    },
    include_contents: false,
  };
  const filter = { kind: "in" as const, field: "kind", values: ["spell"] };
  history.replaceState(
    null,
    "",
    "/search" +
      searchStateQueryString(
        {
          ...DEFAULT_SEARCH_STATE,
          query: "cold",
          filter,
          relationships: [{ ...constraint, direction: "incoming" }],
        },
        "bge-base",
      ),
  );
  const route = relationshipSearchRoute(constraint);
  if (route.kind !== "search") throw new Error("Expected Search");
  const search = decodeSearchStateFromParams(new URLSearchParams(route.query));
  expect(search.relationships).toEqual([constraint]);
  expect(search.query).toBe("cold");
  expect(search.filter).toMatchObject(filter);
  expect(new URLSearchParams(route.query).get("search_profile")).toBe("bge-base");
});
