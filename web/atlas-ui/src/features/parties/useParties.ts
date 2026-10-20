import { useQuery } from "@tanstack/react-query";
import { getParties } from "../../api/atlasApi";
export function useParties() {
  return useQuery({ queryKey: ["parties"], queryFn: getParties, retry: false });
}
