import { useQuery } from "@tanstack/react-query";
import { getParties } from "../../api/atlasApi";
export function useParties(enabled = true) {
  return useQuery({
    queryKey: ["parties"],
    queryFn: getParties,
    enabled,
    retry: false,
  });
}
