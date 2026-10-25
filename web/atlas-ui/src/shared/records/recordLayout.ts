import type { RecordPresentationView } from "../../generated/atlas";

/** Creature task composition earns width independently of prose-only documents. */
export function recordLayout(
  presentation?: RecordPresentationView,
): "document" | "creature" {
  return presentation?.body.kind === "creature" ? "creature" : "document";
}
