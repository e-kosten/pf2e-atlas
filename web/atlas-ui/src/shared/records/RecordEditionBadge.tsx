import { Tag } from "antd";
import type { RecordSummaryView } from "../../generated/atlas";

export function RecordEditionBadge({
  edition,
}: {
  edition: RecordSummaryView["edition"];
}) {
  return edition ? <Tag>{edition === "legacy" ? "Legacy" : "Remaster"}</Tag> : null;
}
