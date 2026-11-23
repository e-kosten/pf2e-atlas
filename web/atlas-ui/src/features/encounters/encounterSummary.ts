import { useEffect } from "react";
import { isUnconfirmedMutation } from "../../api/apiError";
import { savedEditConflictFields } from "../../shared/feedback/savedEditConflict";
import { updateEncounter } from "../../api/atlasApi";
import type { EncounterSummaryView } from "../../generated/atlas";

export function setEncounterArchived(encounterKey: string, archived: boolean) {
  return updateEncounter(encounterKey, {
    archived: { expected: !archived, value: archived },
  });
}

export function encounterTurnText(encounter: EncounterSummaryView): string | undefined {
  const round = `Round ${encounter.round_number.toLocaleString()}`;
  if (encounter.current_turn_participant_name)
    return `${round} · ${encounter.current_turn_participant_name}’s turn`;
  return encounter.round_number > 1 ? round : undefined;
}

export function archiveNeedsRefresh(error: unknown) {
  return (
    isUnconfirmedMutation(error) ||
    savedEditConflictFields(error)?.includes("archived") === true
  );
}

export function useArchiveError(
  error: Error | null,
  target: boolean | undefined,
  saved: boolean | undefined,
  reset: () => void,
) {
  const confirmed =
    error != null &&
    target !== undefined &&
    saved === target &&
    archiveNeedsRefresh(error);
  useEffect(() => {
    if (confirmed) reset();
  }, [confirmed, reset]);
  return confirmed ? null : error;
}
