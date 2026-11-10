import { useQueryClient } from "@tanstack/react-query";
import type { SavedListDetailView } from "../../generated/atlas";
import { getSavedList } from "../../api/atlasApi";
import { useSettingsOpening } from "../../shared/ui/overlays/useSettingsOpening";

export function useListSettingsOpening(initialRef?: string) {
  const client = useQueryClient();
  return useSettingsOpening(async (ref, signal) => {
    const started = Date.now();
    const view = await getSavedList(ref);
    signal.throwIfAborted();
    const key = ["saved-list", view.list.list_key];
    const current = client.getQueryState<SavedListDetailView>(key);
    if (
      current?.data &&
      current.dataUpdatedAt >= started &&
      Date.parse(current.data.list.updated_at) > Date.parse(view.list.updated_at)
    )
      return current.data;
    client.setQueryData(key, view);
    return view;
  }, initialRef);
}
