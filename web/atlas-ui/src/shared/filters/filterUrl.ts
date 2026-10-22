import LZString from "lz-string";
import type {
  QueryCompare,
  QueryFieldState,
  QueryPredicate,
  QuerySetMatch,
} from "../../generated/atlas";
import { packSelection, unpackSelection } from "./filterUrlSelection";
import { invalidFilterUrl, urlLiteral, urlString, urlTuple } from "./filterUrlValues";

// Version 3 is a compact transport for the existing query tree, not a query language.
const VERSION = "3.";
export const UNSUPPORTED_FILTER_URL =
  "This search link uses an unsupported filter version.";
const { compressToEncodedURIComponent, decompressFromEncodedURIComponent } = LZString;

/** The URL carries query meaning; clause/scope identities belong to the editor. */
export function encodeFilterUrl(predicate: QueryPredicate): string {
  // '_' replaces lz-string's '+' so URLSearchParams need not percent-escape it.
  return (
    VERSION +
    compressToEncodedURIComponent(JSON.stringify(pack(predicate))).replace(/\+/g, "_")
  );
}

function pack(node: QueryPredicate): unknown[] {
  switch (node.kind) {
    case "all_of":
      return ["a", ...node.children.map(pack)];
    case "any_of":
      return ["o", ...node.children.map(pack)];
    case "not":
      return ["n", pack(node.predicate)];
    case "exists":
      return ["e", node.collection, pack(node.predicate)];
    case "boolean_constant":
      return ["b", node.value];
    case "compare":
      return ["c", node.field, node.op, node.value];
    case "in":
      return ["i", node.field, node.values];
    case "set_match":
      return ["s", node.field, node.op, node.values];
    case "state_match":
      return ["m", node.field, node.state];
    case "relationship": {
      const link = node.relationship;
      const packed: unknown[] = [
        link.direction === "incoming" ? "t" : "f",
        packSelection(link.selection),
      ];
      // Whole entry is the wire default; an explicit 0 means main entry only.
      if (!link.include_contents) packed.push(0);
      return packed;
    }
  }
}

export function decodeFilterUrl(encoded: string): QueryPredicate {
  if (encoded.length > 16_384) invalidFilterUrl();
  if (!encoded.startsWith(VERSION)) throw new Error(UNSUPPORTED_FILTER_URL);
  const payload = encoded.slice(VERSION.length);
  if (!/^[A-Za-z0-9_-]+$/.test(payload)) invalidFilterUrl();
  const json = decompressFromEncodedURIComponent(payload.replace(/_/g, "+"));
  if (!json) invalidFilterUrl();
  let ordinal = 0;
  // Check transport structure only. The backend owns field/operator eligibility
  // and executable limits; unknown field/operator strings must reach its errors.
  function restore(value: unknown, depth = 0): QueryPredicate {
    if (depth > 64 || ++ordinal > 1024) invalidFilterUrl();
    const node = urlTuple(value, 1, Infinity);
    const clause_id = "url-clause-" + ordinal;
    switch (node[0]) {
      case "a":
      case "o":
        return {
          kind: node[0] === "a" ? "all_of" : "any_of",
          clause_id,
          children: node.slice(1).map((child) => restore(child, depth + 1)),
        };
      case "n":
        urlTuple(node, 2);
        return { kind: "not", clause_id, predicate: restore(node[1], depth + 1) };
      case "e":
        urlTuple(node, 3);
        return {
          kind: "exists",
          clause_id,
          scope_id: "url-scope-" + ordinal,
          collection: urlString(node[1]),
          predicate: restore(node[2], depth + 1),
        };
      case "b":
        urlTuple(node, 2);
        if (typeof node[1] !== "boolean") invalidFilterUrl();
        return { kind: "boolean_constant", clause_id, value: node[1] };
      case "c":
        urlTuple(node, 4);
        return {
          kind: "compare",
          clause_id,
          field: urlString(node[1]),
          op: urlString(node[2]) as QueryCompare,
          value: urlLiteral(node[3]),
        };
      case "i":
        urlTuple(node, 3);
        return {
          kind: "in",
          clause_id,
          field: urlString(node[1]),
          values: urlTuple(node[2], 0, Infinity).map(urlLiteral),
        };
      case "s":
        urlTuple(node, 4);
        return {
          kind: "set_match",
          clause_id,
          field: urlString(node[1]),
          op: urlString(node[2]) as QuerySetMatch,
          values: urlTuple(node[3], 0, Infinity).map(urlString),
        };
      case "m":
        urlTuple(node, 3);
        return {
          kind: "state_match",
          clause_id,
          field: urlString(node[1]),
          state: urlString(node[2]) as QueryFieldState,
        };
      case "t":
      case "f":
        urlTuple(node, 2, 3);
        if (node.length === 3 && node[2] !== 0) invalidFilterUrl();
        return {
          kind: "relationship",
          clause_id,
          relationship: {
            direction: node[0] === "t" ? "incoming" : "outgoing",
            include_contents: node.length === 2,
            selection: unpackSelection(node[1]),
          },
        };
      default:
        return invalidFilterUrl();
    }
  }
  return restore(JSON.parse(json));
}
