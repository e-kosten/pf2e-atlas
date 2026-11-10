import { useQueryClient } from "@tanstack/react-query";
import type { EncounterDetailView } from "../../generated/atlas";
import { getEncounter } from "../../api/atlasApi";
import { useSettingsOpening } from "../../shared/ui/overlays/useSettingsOpening";

export function useEncounterSettingsOpening() {
  const client = useQueryClient();
  return useSettingsOpening(async (ref, signal) => {
    const key = ["encounter", ref];
    const before = client.getQueryData<EncounterDetailView>(key);
    const view = await getEncounter(ref);
    signal.throwIfAborted();
    const current = client.getQueryData<EncounterDetailView>(key);
    if (current && current !== before && current.revision > view.revision)
      return current;
    client.setQueryData(["encounter", view.encounter.encounter_key], view);
    return view;
  });
}
