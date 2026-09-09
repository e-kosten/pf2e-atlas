import { useQuery } from "@tanstack/react-query";
import { getRecordDetail } from "../../api/atlasApi";
import type { RecordDetailRequest } from "../../generated/atlas";
import type { SpellFormSelection } from "./SpellRecordSurface";

type RecordReferenceRequest = Pick<
  RecordDetailRequest,
  "reference_backlink_limit" | "reference_outgoing_limit"
>;

type RecordChildRequest = Pick<RecordDetailRequest, "child_locator">;

export function useRecordDetail(
  recordKey: string | null,
  spellSelection?: SpellFormSelection,
  referenceRequest?: RecordReferenceRequest,
  childRequest?: RecordChildRequest,
) {
  return useQuery({
    queryKey: [
      "record-detail",
      recordKey,
      spellSelection?.formId ?? null,
      spellSelection?.castRank ?? null,
      referenceRequest?.reference_outgoing_limit ?? null,
      referenceRequest?.reference_backlink_limit ?? null,
      ...(childRequest?.child_locator === undefined
        ? []
        : [childRequest.child_locator]),
    ],
    queryFn: ({ signal }) => {
      const request: RecordDetailRequest = {
        ...(spellSelection
          ? {
              spell_form_id: spellSelection.formId,
              spell_cast_rank: spellSelection.castRank,
            }
          : {}),
        ...referenceRequest,
        ...childRequest,
      };
      return getRecordDetail(
        recordKey!,
        Object.keys(request).length ? request : undefined,
        signal,
      );
    },
    enabled: recordKey !== null,
  });
}
