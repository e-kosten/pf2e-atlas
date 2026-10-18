import { queryOptions } from "@tanstack/react-query";
import { getRecordDetail } from "../../api/atlasApi";
import type { RecordNavigationView } from "../../generated/atlas";

export function recordDetailQueryOptions(
  recordKey: string | null,
  selection?: RecordNavigationView,
  epoch?: number,
) {
  const request =
    selection &&
    (selection.owners.length ||
      selection.field ||
      selection.passage ||
      selection.spell_rank ||
      selection.source_fingerprint)
      ? { ...selection, fields: selection.field ? [selection.field] : [] }
      : undefined;
  return queryOptions({
    queryKey: [
      "record-detail",
      recordKey,
      request ?? null,
      ...(epoch === undefined ? [] : [epoch]),
    ],
    queryFn: ({ signal }) => getRecordDetail(recordKey!, request, signal),
    enabled: recordKey !== null,
  });
}
