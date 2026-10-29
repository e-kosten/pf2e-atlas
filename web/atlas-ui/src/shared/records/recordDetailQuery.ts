import { queryOptions } from "@tanstack/react-query";
import { AtlasApiError, getRecordDetail } from "../../api/atlasApi";
import { referenceQueryMeta } from "../queries/referenceQueries";
import type { RecordDetailRequest, RecordNavigationView } from "../../generated/atlas";

export function recordDetailQueryOptions(
  recordKey: string | null,
  selection?: RecordNavigationView,
  generation?: string | null,
) {
  const request =
    selection &&
    recordKey !== null &&
    (selection.embedded_path.length ||
      selection.field ||
      selection.passage ||
      selection.spell_rank !== undefined ||
      selection.source_fingerprint)
      ? recordDetailRequest({ ...selection, record_key: recordKey })
      : undefined;
  return queryOptions({
    queryKey: [
      "record-detail",
      recordKey,
      request ?? null,
      ...(generation == null ? [] : [generation]),
    ],
    meta: referenceQueryMeta,
    queryFn: async ({ signal }) => {
      const detail = await getRecordDetail(recordKey!, request, signal);
      signal.throwIfAborted();
      if (generation != null && detail.artifact_generation !== generation)
        throw new AtlasApiError(409, "The reference collection changed.", {
          code: "artifact_generation_changed",
          message: "The reference collection changed.",
          recoverable_action: "refresh_results",
        });
      return detail;
    },
    enabled: recordKey !== null,
  });
}

export function recordDetailRequest(
  selection: RecordNavigationView,
): RecordDetailRequest {
  const { field, ...address } = selection;
  return { ...address, fields: field ? [field] : [] };
}
