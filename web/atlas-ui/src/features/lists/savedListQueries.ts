import { useQuery, type QueryClient } from "@tanstack/react-query";
import type { SavedListIndexView, SavedListSummaryView } from "../../generated/atlas";
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
        kind === "saved-list-header" ||
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

function savedListsKey(includeArchived: boolean) {
  return includeArchived ? ["saved-lists", "all"] : ["saved-lists"];
}

export function useSavedLists({
  enabled = true,
  includeArchived = false,
}: { enabled?: boolean; includeArchived?: boolean } = {}) {
  return useQuery({
    queryKey: savedListsKey(includeArchived),
    queryFn: () => getSavedLists(includeArchived),
    enabled,
  });
}

export function updateSavedListIndexes(
  client: QueryClient,
  saved: SavedListSummaryView,
) {
  for (const includeArchived of [false, true]) {
    client.setQueryData<SavedListIndexView>(
      savedListsKey(includeArchived),
      (current) => {
        if (!current) return current;

        const lists = current.lists.filter((list) => list.list_key !== saved.list_key);
        if (includeArchived || !saved.archived) {
          const before = lists.findIndex(
            (list) =>
              list.updated_at < saved.updated_at ||
              (list.updated_at === saved.updated_at && list.slug > saved.slug),
          );
          lists.splice(before === -1 ? lists.length : before, 0, saved);
        }
        return { ...current, lists };
      },
    );
  }
}
