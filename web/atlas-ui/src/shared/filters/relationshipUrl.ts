import type { RelationshipConstraintView } from "../../generated/atlas";

const VERSION = "1.";
const INVALID = "Invalid relationship filter URL. Clear filters to continue.";

export function encodeRelationships(
  relationships: RelationshipConstraintView[],
): string {
  return VERSION + JSON.stringify(relationships);
}

export function decodeRelationships(encoded: string): RelationshipConstraintView[] {
  if (!encoded.startsWith(VERSION) || encoded.length > 32_768) throw new Error(INVALID);
  try {
    const constraints: unknown = JSON.parse(encoded.slice(VERSION.length));
    if (!Array.isArray(constraints) || constraints.length > 16)
      throw new Error(INVALID);
    for (const constraint of constraints) {
      const selection = constraint?.selection;
      if (
        !["incoming", "outgoing"].includes(constraint?.direction) ||
        typeof constraint.include_contents !== "boolean" ||
        !selection ||
        typeof selection.record_key !== "string" ||
        !Array.isArray(selection.embedded_path) ||
        !selection.embedded_path.every(
          (entry: unknown) => typeof entry === "object" && entry !== null,
        ) ||
        ![selection.field, selection.source_fingerprint].every(
          (value) => value == null || typeof value === "string",
        ) ||
        !(selection.passage == null || typeof selection.passage === "object")
      )
        throw new Error(INVALID);
    }
    return constraints as RelationshipConstraintView[];
  } catch {
    throw new Error(INVALID);
  }
}
