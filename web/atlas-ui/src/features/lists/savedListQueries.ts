import { useQuery, type QueryClient } from "@tanstack/react-query";
import { getSavedLists } from "../../api/atlasApi";

export function invalidateSavedListQueries(
  client: QueryClient,
  listRefs: readonly string[],
) {
  const refs = new Set(listRefs);
  return client.invalidateQueries({
    predicate: ({ queryKey }) => {
      const [kind, scope] = queryKey;
      if (kind === "saved-lists") return true;
      if (
        kind === "saved-list" ||
        kind === "saved-list-item-note" ||
        kind === "saved-list-filter-discovery"
      )
        return typeof scope === "string" && refs.has(scope);
      return (
        kind === "filter-vocabulary" &&
        typeof scope === "object" &&
        scope !== null &&
        "kind" in scope &&
        scope.kind === "saved_list" &&
        "list_ref" in scope &&
        typeof scope.list_ref === "string" &&
        refs.has(scope.list_ref)
      );
    },
  });
}

export function useSavedLists(options: { enabled?: boolean } = {}) {
  return useQuery({
    queryKey: ["saved-lists"],
    queryFn: getSavedLists,
    enabled: options.enabled ?? true,
  });
}
