import type {
  EncounterParticipantView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";

export function participantUpdate(
  participant: EncounterParticipantView,
  changes: Partial<UpdateEncounterParticipantRequest>,
): UpdateEncounterParticipantRequest {
  const vitals = requireParticipantVitals(participant);
  return {
    participant_key: participant.participant_key,
    display_name: participant.display_name,
    side: participant.side,
    participant_variant: participant.participant_variant,
    initiative: participant.initiative,
    max_hp: vitals.maximum_hp?.adjusted_value,
    current_hp: vitals.current_hp,
    temporary_hp: vitals.temporary_hp,
    defeated: participant.defeated,
    hidden: participant.hidden,
    note: participant.note,
    ...changes,
  };
}

export function applyParticipantUpdate(
  participant: EncounterParticipantView,
  request: UpdateEncounterParticipantRequest,
): EncounterParticipantView {
  const encounter = participant.record_view.encounter;
  const vitals = encounter?.vitals;
  return {
    ...participant,
    display_name: request.display_name,
    side: request.side,
    participant_variant: request.participant_variant,
    initiative: request.initiative,
    defeated: request.defeated,
    hidden: request.hidden,
    note: request.note,
    record_view:
      encounter && vitals
        ? {
            ...participant.record_view,
            encounter: {
              ...encounter,
              vitals: {
                ...vitals,
                current_hp: request.current_hp,
                temporary_hp: request.temporary_hp,
                maximum_hp:
                  vitals.maximum_hp && request.max_hp !== undefined
                    ? {
                        ...vitals.maximum_hp,
                        adjusted_value: request.max_hp,
                      }
                    : vitals.maximum_hp,
              },
            },
          }
        : participant.record_view,
  };
}

export function damageChanges(
  participant: EncounterParticipantView,
  amount: number,
): Partial<UpdateEncounterParticipantRequest> {
  const vitals = requireParticipantVitals(participant);
  const temporaryHp = asNumber(vitals.temporary_hp);
  const currentHp = asNumber(vitals.current_hp);
  const tempDamage = Math.min(temporaryHp, amount);
  const remaining = amount - tempDamage;
  const current_hp = Math.max(0, currentHp - remaining);
  return {
    temporary_hp: temporaryHp - tempDamage,
    current_hp,
    defeated: current_hp === 0 ? true : participant.defeated,
  };
}

export function healChanges(
  participant: EncounterParticipantView,
  amount: number,
): Partial<UpdateEncounterParticipantRequest> {
  return {
    current_hp: clampCurrentHp(
      participant,
      asNumber(requireParticipantVitals(participant).current_hp) + amount,
    ),
  };
}

export function clampCurrentHp(
  participant: EncounterParticipantView,
  hp: number,
): number {
  const lowerBounded = Math.max(0, hp);
  const maximumHp = requireParticipantVitals(participant).maximum_hp?.adjusted_value;
  if (maximumHp === undefined) {
    return lowerBounded;
  }
  return Math.min(lowerBounded, Number(maximumHp));
}

export function participantCurrentHp(
  participant: EncounterParticipantView,
): number | undefined {
  return participant.record_view.encounter?.vitals?.current_hp;
}

export function participantMaximumHp(
  participant: EncounterParticipantView,
): number | undefined {
  return participant.record_view.encounter?.vitals?.maximum_hp?.adjusted_value;
}

export function participantTemporaryHp(
  participant: EncounterParticipantView,
): number | undefined {
  return participant.record_view.encounter?.vitals?.temporary_hp;
}

function requireParticipantVitals(participant: EncounterParticipantView) {
  const vitals = participant.record_view.encounter?.vitals;
  if (!vitals) {
    throw new Error("Encounter participant is missing its typed runtime vitals.");
  }
  return vitals;
}

export function evaluateHpFormula(value: string): number | null {
  const trimmed = value.trim();
  if (!/^\d+(\s*[+-]\s*\d+)*$/.test(trimmed)) {
    return null;
  }
  const tokens = trimmed.match(/\d+|[+-]/g);
  if (!tokens || tokens.length === 0) {
    return null;
  }
  let result = Number(tokens[0]);
  for (let index = 1; index < tokens.length; index += 2) {
    const operator = tokens[index];
    const next = Number(tokens[index + 1]);
    result = operator === "-" ? result - next : result + next;
  }
  return Math.max(0, result);
}

export function optionalIntegerInput(value: string): number | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  if (!/^-?\d+$/.test(trimmed)) {
    return null;
  }
  return Number(trimmed);
}

export function optionalHpFormulaInput(value: string): number | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  const hp = evaluateHpFormula(trimmed);
  return hp;
}

export function asNumber(value: number | undefined): number {
  return value ?? 0;
}

export function optionalNumber(value: number | undefined): number | undefined {
  return value;
}

export function displayNumber(value: number | undefined): string {
  return value === undefined ? "--" : value.toString();
}
