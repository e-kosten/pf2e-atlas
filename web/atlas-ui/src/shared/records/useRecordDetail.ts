import { useQuery } from "@tanstack/react-query";
import { getRecordDetail } from "../../api/atlasApi";

export function useRecordDetail(recordKey: string | null) {
  return useQuery({
    queryKey: ["record-detail", recordKey],
    queryFn: () => getRecordDetail(recordKey!),
    enabled: recordKey !== null,
  });
}
