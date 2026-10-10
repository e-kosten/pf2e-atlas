import { useQuery } from "@tanstack/react-query";
import { getRecordDetail } from "../../api/atlasApi";

export function useRecordDetail(
  recordKey: string | null,
  selection?: import("../../generated/atlas").RecordNavigationView,
) {
  return useQuery({
    queryKey: ["record-detail", recordKey, selection],
    queryFn: () =>
      selection
        ? getRecordDetail(recordKey!, {
            ...selection,
            fields: selection.field ? [selection.field] : [],
          })
        : getRecordDetail(recordKey!),
    enabled: recordKey !== null,
  });
}
