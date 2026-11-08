import type {
  EncounterParticipantView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";

export function participantUpdate(
  participant: EncounterParticipantView,
  changes: Partial<
    Omit<UpdateEncounterParticipantRequest, "participant_key" | "expected_revision">
  >,
): UpdateEncounterParticipantRequest {
  return {
    hp_edit: changes.hp_edit ?? "current_hp" in changes,
    max_hp_edit: changes.max_hp_edit ?? "max_hp" in changes,
    variant_edit: changes.variant_edit ?? "participant_variant" in changes,
    use_derived_max: changes.use_derived_max ?? false,
    display_name: participant.display_name,
    side: participant.side,
    participant_variant: participant.participant_variant,
    initiative: participant.initiative,
    max_hp: participant.max_hp,
    current_hp: participant.current_hp,
    temporary_hp: participant.temporary_hp,
    defeated: participant.defeated,
    hazard_disabled: participant.hazard_disabled,
    hidden: participant.hidden,
    note: participant.note,
    ...changes,
    participant_key: participant.participant_key,
    expected_revision: participant.revision,
  };
}

export function optionalIntegerInput(value: string): number | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  if (!/^-?\d+$/.test(trimmed)) {
    return null;
  }
  const parsed = Number(trimmed);
  return Number.isSafeInteger(parsed) ? parsed : null;
}

export function asNumber(value: number | undefined): number {
  return value === undefined ? 0 : value;
}

export function displayNumber(value: number | undefined): string {
  return value === undefined ? "--" : value.toString();
}
