import { useQueryClient } from "@tanstack/react-query";
import type { EncounterDetailView, EncounterSettingsView } from "../../generated/atlas";
import { getEncounter, getEncounterSettings } from "../../api/atlasApi";
import { useSettingsOpening } from "../../shared/ui/overlays/useSettingsOpening";

export function useEncounterSettingsOpening(surface: "runner" | "directory") {
  const client = useQueryClient();
  return useSettingsOpening(async (ref, signal) => {
    const settings = await getEncounterSettings(ref);
    signal.throwIfAborted();
    const key = settings.encounter.encounter_key;
    const latest = client.setQueryData<EncounterSettingsView>(
      ["encounter-settings", key],
      (current) =>
        current && current.revision > settings.revision ? current : settings,
    )!;
    if (surface === "directory") return latest;

    const runnerKey = ["encounter", key];
    const current = client.getQueryData<EncounterDetailView>(runnerKey);
    if (current && current.revision >= latest.revision) return current;
    // The mounted roster also needs refreshing when settings reveal a newer revision.
    const detail = await getEncounter(key);
    signal.throwIfAborted();
    return client.setQueryData<EncounterDetailView>(runnerKey, (current) =>
      current && current.revision > detail.revision ? current : detail,
    )!;
  });
}
