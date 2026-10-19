import type { EncounterParticipantConditionView } from "../../generated/atlas";

export function conditionDurationLabel(
  duration: NonNullable<EncounterParticipantConditionView["duration"]>,
): string {
  const unit = duration.amount === 1n ? duration.unit.replace(/s$/, "") : duration.unit;
  return `${duration.amount} ${unit}`;
}
