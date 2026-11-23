import type { EncounterStatusView, EncounterSummaryView } from "../../generated/atlas";

export const encounterStatusLabels: Record<EncounterStatusView, string> = {
  draft: "Draft",
  running: "Running",
  complete: "Complete",
  archived: "Archived",
};

export function displayRound(encounter: EncounterSummaryView): string | null {
  return encounter.status === "draft" ? null : encounter.round_number.toLocaleString();
}
