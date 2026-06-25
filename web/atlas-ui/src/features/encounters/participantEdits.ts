import type {
  EncounterParticipantView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";

export function participantUpdate(
  participant: EncounterParticipantView,
  changes: Partial<UpdateEncounterParticipantRequest>,
): UpdateEncounterParticipantRequest {
  return {
    participant_key: participant.participant_key,
    display_name: participant.display_name,
    side: participant.side,
    participant_variant: participant.participant_variant,
    initiative: participant.initiative,
    max_hp: participant.max_hp,
    current_hp: participant.current_hp,
    temporary_hp: participant.temporary_hp,
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
  return {
    ...participant,
    display_name: request.display_name,
    side: request.side,
    participant_variant: request.participant_variant,
    initiative: request.initiative,
    max_hp: request.max_hp,
    current_hp: request.current_hp,
    temporary_hp: request.temporary_hp,
    defeated: request.defeated,
    hidden: request.hidden,
    note: request.note,
  };
}

export function damageChanges(
  participant: EncounterParticipantView,
  amount: number,
): Partial<UpdateEncounterParticipantRequest> {
  const temporaryHp = asNumber(participant.temporary_hp);
  const currentHp = asNumber(participant.current_hp);
  const tempDamage = Math.min(temporaryHp, amount);
  const remaining = amount - tempDamage;
  const current_hp = BigInt(Math.max(0, currentHp - remaining));
  return {
    temporary_hp: BigInt(temporaryHp - tempDamage),
    current_hp,
    defeated: current_hp === BigInt(0) ? true : participant.defeated,
  };
}

export function healChanges(
  participant: EncounterParticipantView,
  amount: number,
): Partial<UpdateEncounterParticipantRequest> {
  return {
    current_hp: BigInt(clampCurrentHp(participant, asNumber(participant.current_hp) + amount)),
  };
}

export function clampCurrentHp(
  participant: EncounterParticipantView,
  hp: number,
): number {
  const lowerBounded = Math.max(0, hp);
  if (participant.max_hp === undefined) {
    return lowerBounded;
  }
  return Math.min(lowerBounded, Number(participant.max_hp));
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

export function optionalBigIntInput(value: string): bigint | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  if (!/^-?\d+$/.test(trimmed)) {
    return null;
  }
  return BigInt(trimmed);
}

export function optionalHpFormulaInput(value: string): bigint | undefined | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) {
    return undefined;
  }
  const hp = evaluateHpFormula(trimmed);
  return hp === null ? null : BigInt(hp);
}

export function asNumber(value: bigint | undefined): number {
  return value === undefined ? 0 : Number(value);
}

export function optionalNumber(value: bigint | undefined): number | undefined {
  return value === undefined ? undefined : Number(value);
}

export function displayNumber(value: bigint | undefined): string {
  return value === undefined ? "--" : value.toString();
}
