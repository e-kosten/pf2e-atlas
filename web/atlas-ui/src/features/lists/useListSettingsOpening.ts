import { useQueryClient } from "@tanstack/react-query";
import type { SavedListSummaryView } from "../../generated/atlas";
import { getSavedListHeader } from "../../api/atlasApi";
import { useSettingsOpening } from "../../shared/ui/overlays/useSettingsOpening";

export function useListSettingsOpening(initialRef?: string) {
  const client = useQueryClient();
  return useSettingsOpening(async (ref, signal) => {
    const started = Date.now();
    const view = await getSavedListHeader(ref, signal);
    signal.throwIfAborted();
    const key = ["saved-list-header", view.list_key];
    const current = client.getQueryState<SavedListSummaryView>(key);
    if (
      current?.data &&
      current.dataUpdatedAt >= started &&
      Date.parse(current.data.updated_at) > Date.parse(view.updated_at)
    )
      return current.data;
    client.setQueryData(key, view);
    return view;
  }, initialRef);
}
