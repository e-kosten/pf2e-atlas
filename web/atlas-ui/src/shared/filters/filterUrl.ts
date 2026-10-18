import LZString from "lz-string";
import type { QueryPredicate } from "../../generated/atlas";

// Version 2 uses the product field vocabulary introduced by catalog v4.
// Old queries must not be reinterpreted or silently lose clauses.
const VERSION = "2.";
export const UNSUPPORTED_FILTER_URL =
  "This search link uses an unsupported filter version. Reset the filters to continue.";
const { compressToEncodedURIComponent, decompressFromEncodedURIComponent } = LZString;

/** The URL carries query meaning; clause/scope identities belong to the editor. */
export function encodeFilterUrl(predicate: QueryPredicate): string {
  const json = JSON.stringify(predicate, (key, value) =>
    key === "clause_id" || key === "scope_id" ? undefined : value,
  );
  return VERSION + compressToEncodedURIComponent(json);
}

export function decodeFilterUrl(encoded: string): QueryPredicate {
  if (encoded.length > 16_384) throw new Error("Invalid filter URL");
  if (!encoded.startsWith(VERSION)) throw new Error(UNSUPPORTED_FILTER_URL);
  const json = decompressFromEncodedURIComponent(encoded.slice(VERSION.length));
  if (!json) throw new Error("Invalid filter URL");
  let ordinal = 0;
  // Check transport structure only. The catalog/backend retain ownership of
  // field availability, operators, scope support and executable query limits.
  function restore(value: unknown, depth = 0): QueryPredicate {
    if (!value || typeof value !== "object" || depth > 64 || ++ordinal > 1024)
      throw new Error("Invalid filter URL");
    const node = value as Record<string, unknown>;
    const clause_id = "url-clause-" + ordinal;
    const literal = (entry: unknown) =>
      typeof entry === "string" ||
      typeof entry === "boolean" ||
      (typeof entry === "number" && Number.isFinite(entry));
    let valid = false;
    switch (node.kind) {
      case "all_of":
      case "any_of":
        if (!Array.isArray(node.children)) throw new Error("Invalid filter URL");
        return {
          kind: node.kind,
          clause_id,
          children: node.children.map((child) => restore(child, depth + 1)),
        };
      case "not":
        return {
          kind: "not",
          clause_id,
          predicate: restore(node.predicate, depth + 1),
        };
      case "exists":
        if (typeof node.collection !== "string") throw new Error("Invalid filter URL");
        return {
          kind: "exists",
          clause_id,
          scope_id: "url-scope-" + ordinal,
          collection: node.collection,
          predicate: restore(node.predicate, depth + 1),
        };
      case "boolean_constant":
        valid = typeof node.value === "boolean";
        break;
      case "compare":
        valid =
          typeof node.field === "string" &&
          typeof node.op === "string" &&
          literal(node.value);
        break;
      case "in":
        valid =
          typeof node.field === "string" &&
          Array.isArray(node.values) &&
          node.values.every(literal);
        break;
      case "set_match":
        valid =
          typeof node.field === "string" &&
          typeof node.op === "string" &&
          Array.isArray(node.values) &&
          node.values.every((entry) => typeof entry === "string");
        break;
      case "state_match":
        valid = typeof node.field === "string" && typeof node.state === "string";
        break;
    }
    if (!valid) throw new Error("Invalid filter URL");
    return { ...node, clause_id } as QueryPredicate;
  }
  return restore(JSON.parse(json));
}
