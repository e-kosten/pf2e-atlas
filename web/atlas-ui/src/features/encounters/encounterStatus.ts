import type { EncounterStatusView } from "../../generated/atlas";

export const encounterStatusLabels: Record<EncounterStatusView, string> = {
  draft: "Draft",
  running: "Running",
  complete: "Complete",
  archived: "Archived",
};
