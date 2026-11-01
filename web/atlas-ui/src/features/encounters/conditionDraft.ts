import type {
  ConditionActionView,
  ConditionParametersInputView,
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
  UpdateEncounterParticipantConditionRequest,
} from "../../generated/atlas";
import type { ConditionEventInput } from "./useConditionEventPreview";

type ConditionExtraActionView = EncounterParticipantConditionView["extra_action"];
type ConditionParametersView = EncounterParticipantConditionView["parameters"];
type Duration = NonNullable<EncounterParticipantConditionView["duration"]>;
export type ConditionFormValues = {
  conditionRef?: string;
  stunMode?: "value" | "duration";
  value?: number | null;
  durationAmount?: number | null;
  durationUnit: Duration["unit"] | "none";
  durationEvent?: "start" | "end";
  durationParticipant?: string;
  sourceParticipantKey?: string;
  note?: string;
  againstKeys?: string[];
  extraActionGuidance?: string;
  extraActionKeys?: Array<
    ConditionActionView | "unspecified" | "unrestricted" | "custom"
  >;
  immunityOverride?: boolean;
  persistentFormula?: string;
  persistentDamageType?: string;
  persistentMultiplier?: "normal" | "double" | "half";
  persistentRecoveryDcEdited?: boolean;
  persistentRecoveryDc?: number | null;
  sickenedRecoveryDc?: number | null;
  wakingRule?: "normal" | "source_controlled";
  conditionSubject?: string;
  conditionSubjectDescription?: string;
  frightenedRecovery?: "end_turn" | "held" | "source_dependent";
  frightenedMinimum?: number | null;
  brokenThreshold?: number | null;
};
export function conditionDuration(values: ConditionFormValues): Duration | undefined {
  if (
    values.durationUnit === "none" ||
    values.durationAmount === null ||
    values.durationAmount === undefined
  )
    return undefined;
  return {
    amount: BigInt(values.durationAmount),
    unit: values.durationUnit,
    timing:
      values.durationEvent && values.durationParticipant
        ? {
            participant_key: values.durationParticipant,
            event: values.durationEvent,
          }
        : null,
  };
}

export function editableConditionParameters(
  parameters: ConditionParametersView,
): ConditionParametersInputView {
  if (
    parameters.kind === "persistent_damage" ||
    parameters.kind === "persistent_damage_reminder"
  )
    return {
      kind: "persistent_damage",
      formula: parameters.formula,
      damage_type: parameters.damage_type,
      multiplier: parameters.multiplier,
      recovery: parameters.recovery,
    };
  return parameters;
}

export function conditionUpdate(
  condition: EncounterParticipantConditionView,
  revision: bigint,
  changes: Partial<Omit<UpdateEncounterParticipantConditionRequest, "request_id">>,
): Omit<UpdateEncounterParticipantConditionRequest, "request_id"> {
  return {
    expected_encounter_revision: revision,
    event_intent: "apply",
    decisions: { keep_upright: false },
    condition_id: condition.condition_id,
    name: condition.name,
    value: condition.value,
    duration: condition.duration,
    source_participant_key: condition.source_participant_key,
    note: condition.note,
    scope: condition.scope,
    extra_action: condition.extra_action,
    immunity_override: condition.immunity_override,
    parameters: editableConditionParameters(condition.parameters),
    origin: condition.origin,
    ...changes,
  };
}

export function conditionDraft(
  values: ConditionFormValues,
  durationEdited: boolean,
  revision: bigint,
  participantKey: string,
  definition: EncounterConditionDefinitionView | undefined,
  condition?: EncounterParticipantConditionView,
): ConditionEventInput {
  const previous = condition?.parameters;
  let parameters: ConditionParametersInputView;
  const subject =
    values.conditionSubject === "__other"
      ? {
          kind: "other" as const,
          description: values.conditionSubjectDescription!.trim(),
        }
      : values.conditionSubject && values.conditionSubject !== "__unknown"
        ? { kind: "participant" as const, participant_key: values.conditionSubject }
        : null;
  switch (
    definition?.editor_kind === "ordinary"
      ? previous?.kind
      : (definition?.editor_kind ?? previous?.kind)
  ) {
    case "persistent_damage":
    case "persistent_damage_reminder":
      parameters = {
        kind: "persistent_damage",
        formula: values.persistentFormula?.trim() || null,
        damage_type: values.persistentDamageType || null,
        multiplier: values.persistentMultiplier ?? "normal",
        recovery:
          (previous?.kind === "persistent_damage" ||
            previous?.kind === "persistent_damage_reminder") &&
          !values.persistentRecoveryDcEdited
            ? previous.recovery
            : values.persistentRecoveryDc == null
              ? { kind: "source_dependent" }
              : { kind: "specified", dc: BigInt(values.persistentRecoveryDc) },
      };
      break;
    case "sickened":
      parameters = {
        kind: "sickened",
        recovery_dc:
          values.sickenedRecoveryDc == null
            ? undefined
            : BigInt(values.sickenedRecoveryDc),
      };
      break;
    case "unconscious":
      parameters = { kind: "unconscious", waking: values.wakingRule ?? "normal" };
      break;
    case "fascinated":
      parameters = { kind: "fascinated", subject };
      break;
    case "fleeing":
      parameters = { kind: "fleeing", subject };
      break;
    case "attitude":
      parameters = { kind: "attitude" };
      break;
    case "frightened":
      parameters = {
        kind: "frightened",
        recovery:
          values.frightenedRecovery === "held"
            ? { kind: "held" }
            : values.frightenedRecovery === "source_dependent"
              ? { kind: "source_dependent" }
              : { kind: "end_turn", minimum: BigInt(values.frightenedMinimum ?? 0) },
      };
      break;
    case "broken":
      parameters = {
        kind: "broken",
        threshold:
          values.brokenThreshold == null ? null : BigInt(values.brokenThreshold),
      };
      break;
    default:
      parameters = previous ? editableConditionParameters(previous) : { kind: "none" };
  }
  const keys = values.extraActionKeys ?? ["unspecified"];
  const extra_action: ConditionExtraActionView = keys.includes("unspecified")
    ? { kind: "unspecified" }
    : keys.includes("unrestricted")
      ? { kind: "unrestricted" }
      : {
          kind: "restricted",
          actions: keys.filter(
            (key): key is ConditionActionView =>
              key !== "unspecified" && key !== "unrestricted" && key !== "custom",
          ),
          guidance: keys.includes("custom")
            ? values.extraActionGuidance?.trim() || null
            : null,
        };
  const common = {
    parameters,
    extra_action,
    immunity_override: values.immunityOverride ?? false,
    scope:
      values.againstKeys && !values.againstKeys.includes("everyone")
        ? {
            kind: "selected_participants" as const,
            participant_keys: values.againstKeys,
          }
        : { kind: "everyone" as const },
    value: values.value == null ? undefined : BigInt(values.value),
    duration:
      condition && !durationEdited ? condition.duration : conditionDuration(values),
    source_participant_key: values.sourceParticipantKey,
    note: values.note || undefined,
  };
  if (condition) return conditionUpdate(condition, revision, common);
  if (!definition) throw new Error("Choose a condition.");
  return {
    participant_key: participantKey,
    expected_encounter_revision: revision,
    event_intent: "apply",
    decisions: { keep_upright: false },
    condition_ref: definition.condition_ref,
    ...common,
    value: definition.has_value ? common.value : undefined,
  };
}
