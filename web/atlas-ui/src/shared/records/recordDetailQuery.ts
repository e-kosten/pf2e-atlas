import { queryOptions } from "@tanstack/react-query";
import { getRecordDetail } from "../../api/atlasApi";
import type { RecordDetailRequest, RecordNavigationView } from "../../generated/atlas";

export function recordDetailQueryOptions(
  recordKey: string | null,
  selection?: RecordNavigationView,
  epoch?: number,
) {
  const request =
    selection &&
    (selection.embedded_path.length ||
      selection.field ||
      selection.passage ||
      selection.spell_rank ||
      selection.source_fingerprint)
      ? recordDetailRequest(selection)
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

export function recordDetailRequest(
  selection: RecordNavigationView,
): RecordDetailRequest {
  const { field, ...address } = selection;
  return { ...address, fields: field ? [field] : [] };
}
