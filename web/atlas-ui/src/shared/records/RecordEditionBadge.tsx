import { DisplayChip } from "../ui/DisplayChip";
import type { RecordSummaryView } from "../../generated/atlas";

export function RecordEditionBadge({
  edition,
}: {
  edition: RecordSummaryView["edition"];
}) {
  return edition ? (
    <DisplayChip>{edition === "legacy" ? "Legacy" : "Remaster"}</DisplayChip>
  ) : null;
}
