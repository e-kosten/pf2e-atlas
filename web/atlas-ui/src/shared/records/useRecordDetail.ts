import { useQuery } from "@tanstack/react-query";
import { recordDetailQueryOptions } from "./recordDetailQuery";

export function useRecordDetail(
  recordKey: string | null,
  selection?: import("../../generated/atlas").RecordNavigationView,
) {
  return useQuery(recordDetailQueryOptions(recordKey, selection));
}
