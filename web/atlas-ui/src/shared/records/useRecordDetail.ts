import { useQuery } from "@tanstack/react-query";
import { getRecordDetail } from "../../api/atlasApi";
import type { SpellFormSelection } from "./SpellRecordSurface";

export function useRecordDetail(
  recordKey: string | null,
  spellSelection?: SpellFormSelection,
) {
  return useQuery({
    queryKey: [
      "record-detail",
      recordKey,
      spellSelection?.formId ?? null,
      spellSelection?.castRank ?? null,
    ],
    queryFn: ({ signal }) =>
      spellSelection
        ? getRecordDetail(
            recordKey!,
            {
              spell_form_id: spellSelection.formId,
              spell_cast_rank: spellSelection.castRank,
            },
            signal,
          )
        : getRecordDetail(recordKey!),
    enabled: recordKey !== null,
  });
}
