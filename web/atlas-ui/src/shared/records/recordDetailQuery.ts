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
  includeSectionRendering = false,
) {
  const request =
    recordKey !== null &&
    (includeSectionRendering ||
      generation != null ||
      fields?.length ||
      selection?.embedded_path.length ||
      selection?.field ||
      selection?.passage ||
      selection?.spell_rank !== undefined ||
      selection?.source_fingerprint)
      ? recordDetailRequest(
          {
            embedded_path: [],
            field: null,
            passage: null,
            source_fingerprint: null,
            ...selection,
            record_key: recordKey,
          },
          fields,
          generation,
          includeSectionRendering,
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
  includeSectionRendering = false,
): RecordDetailRequest {
  const { field, ...address } = selection;
  return {
    ...address,
    ...(includeSectionRendering ? { include_section_rendering: true } : {}),
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
