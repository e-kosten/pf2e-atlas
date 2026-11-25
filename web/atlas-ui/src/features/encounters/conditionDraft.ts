import { hashKey } from "@tanstack/react-query";
import type { UpdateConditionDraft } from "./useConditionEventPreview";
import type {
  ConditionActionView,
  ConditionParametersInputView,
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
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
    amount: values.durationAmount,
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

type ConditionChanges = Partial<
  Pick<
    UpdateConditionDraft,
    | "value"
    | "scope"
    | "extra_action"
    | "immunity_override"
    | "source_participant_key"
    | "duration"
    | "note"
    | "parameters"
  >
>;

export function conditionUpdate(
  condition: EncounterParticipantConditionView,
  revision: number,
  changes: ConditionChanges,
): UpdateConditionDraft {
  const saved = {
    ...condition,
    parameters: editableConditionParameters(condition.parameters),
  };
  const edits = Object.fromEntries(
    Object.entries(changes).filter(([key, value]) => {
      const previous = saved[key as keyof ConditionChanges];
      return hashKey([value]) !== hashKey([previous]);
    }),
  );
  return {
    expected_encounter_revision: revision,
    event_intent: Object.keys(edits).some(
      (key) => key !== "note" && key !== "source_participant_key",
    )
      ? "apply"
      : "correction",
    condition_id: condition.condition_id,
    ...edits,
  };
}

export function conditionDraft(
  values: ConditionFormValues,
  durationEdited: boolean,
  revision: number,
  participantKey: string,
  definition: EncounterConditionDefinitionView | undefined,
  condition?: EncounterParticipantConditionView,
  isTouched: (key: keyof ConditionFormValues) => boolean = () => true,
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
              : { kind: "specified", dc: values.persistentRecoveryDc },
      };
      break;
    case "sickened":
      parameters = {
        kind: "sickened",
        recovery_dc:
          values.sickenedRecoveryDc == null ? undefined : values.sickenedRecoveryDc,
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
              : { kind: "end_turn", minimum: values.frightenedMinimum ?? 0 },
      };
      break;
    case "broken":
      parameters = {
        kind: "broken",
        threshold: values.brokenThreshold == null ? null : values.brokenThreshold,
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
    value: values.value == null ? undefined : values.value,
    duration:
      condition && !durationEdited ? condition.duration : conditionDuration(values),
    source_participant_key: values.sourceParticipantKey,
    note: values.note || undefined,
  };
  if (condition) {
    const changes: ConditionChanges = {};
    if (isTouched("value") || isTouched("stunMode"))
      changes.value = common.value ?? null;
    if (durationEdited) changes.duration = common.duration ?? null;
    if (isTouched("note")) changes.note = common.note ?? null;
    if (isTouched("sourceParticipantKey"))
      changes.source_participant_key = common.source_participant_key ?? null;
    if (isTouched("againstKeys")) changes.scope = common.scope;
    if (isTouched("extraActionKeys") || isTouched("extraActionGuidance"))
      changes.extra_action = extra_action;
    changes.immunity_override = common.immunity_override;
    const parameterFields: Array<keyof ConditionFormValues> = [
      "persistentFormula",
      "persistentDamageType",
      "persistentMultiplier",
      "persistentRecoveryDc",
      "sickenedRecoveryDc",
      "wakingRule",
      "conditionSubject",
      "conditionSubjectDescription",
      "frightenedRecovery",
      "frightenedMinimum",
      "brokenThreshold",
    ];
    if (parameterFields.some(isTouched)) changes.parameters = parameters;
    return conditionUpdate(condition, revision, changes);
  }
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
