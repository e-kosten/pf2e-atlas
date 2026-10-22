import type { QueryRelationship } from "../../generated/atlas";
import { linkLeaf } from "../filters/linkPredicates";
import { linkClauses } from "../filters/LinkFilterControls";
import { relationshipSearchRoute } from "./relationshipSearch";
import {
  decodeSearchStateFromParams,
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
} from "../filters/searchState";

it("ANDs a record-origin condition with the entire previous tree while retaining text, filters and exact checked selection", () => {
  const constraint: QueryRelationship = {
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
          filter: {
            kind: "any_of",
            children: [
              filter,
              linkLeaf({ ...constraint, direction: "incoming" }, "old"),
            ],
          },
        },
        "bge-base",
      ),
  );
  const route = relationshipSearchRoute(linkLeaf(constraint, "new"));
  if (route.kind !== "search") throw new Error("Expected Search");
  const search = decodeSearchStateFromParams(new URLSearchParams(route.query));
  expect(linkClauses(search.filter).map((leaf) => leaf.relationship)).toEqual([
    { ...constraint, direction: "incoming" },
    constraint,
  ]);
  expect(search.query).toBe("cold");
  expect(search.filter).toMatchObject({
    kind: "all_of",
    children: [
      { kind: "any_of", children: [filter, expect.anything()] },
      expect.objectContaining({ kind: "relationship", relationship: constraint }),
    ],
  });
  expect(new URLSearchParams(route.query).get("search_profile")).toBe("bge-base");
});
