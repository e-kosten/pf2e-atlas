import type { RecordPresentationView } from "../../generated/atlas";

/** Keep wide actor layouts until their separate presentation review. */
export function recordLayout(
  presentation?: RecordPresentationView,
): "document" | "workspace" {
  return presentation?.body.kind === "creature" || presentation?.body.kind === "hazard"
    ? "workspace"
    : "document";
}
