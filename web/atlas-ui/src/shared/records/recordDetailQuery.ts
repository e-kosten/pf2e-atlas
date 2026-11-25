import { queryOptions } from "@tanstack/react-query";
import { AtlasApiError, getRecordDetail } from "../../api/atlasApi";
import { referenceQueryMeta } from "../queries/referenceQueries";
import type {
  RecordDetailRequest,
  RecordNavigationView,
  RecordSelection,
} from "../../generated/atlas";

export function recordDetailQueryOptions(
  recordKey: string | null,
  selection?: RecordNavigationView,
  generation?: string | null,
  fields?: string[],
) {
  const request =
    selection &&
    recordKey !== null &&
    (fields?.length ||
      selection.embedded_path.length ||
      selection.field ||
      selection.passage ||
      selection.spell_rank !== undefined ||
      selection.source_fingerprint)
      ? recordDetailRequest(
          { ...selection, record_key: recordKey },
          fields,
          fields?.length ? generation : undefined,
        )
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
  fields?: string[],
  generation?: string | null,
): RecordDetailRequest {
  const { field, ...address } = selection;
  return {
    ...address,
    fields: fields ?? (field ? [field] : []),
    ...(generation ? { expected_artifact_generation: generation } : {}),
  };
}

export function recordSelection(navigation: RecordNavigationView): RecordSelection {
  return {
    address: {
      record_key: navigation.record_key,
      embedded_path: navigation.embedded_path,
      source_fingerprint: navigation.source_fingerprint,
    },
    fields: navigation.field ? [navigation.field] : [],
    passage: navigation.passage,
    spell_rank: navigation.spell_rank,
  };
}
