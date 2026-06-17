import { useQuery } from "@tanstack/react-query";
import { getSavedLists } from "../api/atlasApi";

export function useSavedLists(options: { enabled?: boolean } = {}) {
  return useQuery({
    queryKey: ["saved-lists"],
    queryFn: getSavedLists,
    enabled: options.enabled ?? true,
  });
}
