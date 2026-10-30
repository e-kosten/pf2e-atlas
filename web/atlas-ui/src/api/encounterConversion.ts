import type {
  EncounterDetailView,
  EncounterParticipantConditionView,
  EncounterParticipantView,
  PersistentDamageGroupView,
} from "../generated/atlas";
import { AtlasApiError } from "./apiError";
import { toBigInt, type Wire } from "./wire";
import {
  convertNumberFact,
  convertRecordPresentation,
  convertTurnResources,
} from "./presentationConversion";

type ConditionParametersView = EncounterParticipantConditionView["parameters"];
type PersistentRecoveryView = Extract<
  ConditionParametersView,
  { kind: "persistent_damage" }
>["recovery"];

function mapArray<T, U>(value: T[], convert: (entry: T) => U): U[] {
  if (!Array.isArray(value)) return value;
  return value.map(convert);
}

type Spellcasting = NonNullable<EncounterParticipantView["spellcasting"]>;
function spellcastingView(value: Wire<Spellcasting>): Spellcasting {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    resources: mapArray(value.resources, (resource) => ({
      ...resource,
      capacity:
        resource.capacity == null ? resource.capacity : toBigInt(resource.capacity),
      remaining:
        resource.remaining == null ? resource.remaining : toBigInt(resource.remaining),
    })),
  };
}

type HpContext = EncounterParticipantView["knockout_options"][number]["context"];
function hpContext(value: Wire<HpContext>): HpContext {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    base_death_threshold:
      value.base_death_threshold == null
        ? value.base_death_threshold
        : toBigInt(value.base_death_threshold),
  };
}

function recovery(value: Wire<PersistentRecoveryView>): PersistentRecoveryView {
  if (!value) return value;
  return value.kind === "specified"
    ? { ...value, dc: value.dc == null ? value.dc : toBigInt(value.dc) }
    : value;
}

export function convertConditionParameters(
  value: Wire<ConditionParametersView>,
): ConditionParametersView {
  if (!value) return value;
  switch (value.kind) {
    case "broken":
      return {
        ...value,
        threshold:
          value.threshold == null ? value.threshold : toBigInt(value.threshold),
      };
    case "frightened":
      return {
        ...value,
        recovery:
          value.recovery?.kind === "end_turn"
            ? {
                ...value.recovery,
                minimum:
                  value.recovery.minimum == null
                    ? value.recovery.minimum
                    : toBigInt(value.recovery.minimum),
              }
            : value.recovery,
      };
    case "persistent_damage":
      if (!value.recovery || typeof value.recovery !== "object")
        throw new AtlasApiError(200, "Invalid persistent recovery");
      return {
        ...value,
        recovery:
          "dc" in value.recovery
            ? { ...value.recovery, dc: toBigInt(value.recovery.dc) }
            : value.recovery,
      };
    case "persistent_damage_reminder":
      return { ...value, recovery: recovery(value.recovery) };
    case "sickened": {
      const { recovery_dc, ...rest } = value;
      return {
        ...rest,
        ...(recovery_dc === undefined ? {} : { recovery_dc: toBigInt(recovery_dc) }),
      };
    }
    default:
      return value;
  }
}

export function convertCondition(
  value: Wire<EncounterParticipantConditionView>,
): EncounterParticipantConditionView {
  if (!value || typeof value !== "object")
    throw new AtlasApiError(200, "Invalid encounter condition");
  const { value: magnitude, duration, parameters, ...rest } = value;
  if (duration !== undefined && (!duration || typeof duration !== "object"))
    throw new AtlasApiError(200, "Invalid condition duration");
  return {
    ...rest,
    condition_id: toBigInt(value.condition_id),
    parameters: convertConditionParameters(parameters),
    ...(magnitude === undefined ? {} : { value: toBigInt(magnitude) }),
    ...(duration === undefined
      ? {}
      : { duration: { ...duration, amount: toBigInt(duration.amount) } }),
  };
}

export function convertPersistentGroup(
  value: Wire<PersistentDamageGroupView>,
): PersistentDamageGroupView {
  if (!value || !Array.isArray(value.sources))
    throw new AtlasApiError(200, "Invalid persistent damage group");
  return {
    ...value,
    selected_source_id: toBigInt(value.selected_source_id),
    sources: value.sources.map((source) => {
      if (!source || typeof source !== "object")
        throw new AtlasApiError(200, "Invalid persistent damage source");
      return {
        ...source,
        condition_id:
          source.condition_id === null
            ? source.condition_id
            : toBigInt(source.condition_id),
        recovery_dc: source.recovery_dc === null ? null : toBigInt(source.recovery_dc),
      };
    }),
  };
}

function participant(value: Wire<EncounterParticipantView>): EncounterParticipantView {
  if (!value || !Array.isArray(value.conditions))
    throw new AtlasApiError(200, "Invalid encounter participant");
  if (
    typeof value.revision !== "number" ||
    !Number.isSafeInteger(value.revision) ||
    value.revision < 0
  )
    throw new AtlasApiError(200, "Invalid participant revision in API response");
  const {
    initiative,
    max_hp,
    current_hp,
    level_override,
    ability_states,
    presentation,
    turn_resources,
    spellcasting,
    ...rest
  } = value;
  return {
    ...rest,
    revision: BigInt(value.revision),
    position: toBigInt(value.position),
    initiative_order: toBigInt(value.initiative_order),
    temporary_hp: toBigInt(value.temporary_hp),
    base_death_threshold:
      value.base_death_threshold === undefined
        ? value.base_death_threshold
        : toBigInt(value.base_death_threshold),
    ...(initiative === undefined ? {} : { initiative: toBigInt(initiative) }),
    ...(max_hp === undefined ? {} : { max_hp: toBigInt(max_hp) }),
    ...(current_hp === undefined ? {} : { current_hp: toBigInt(current_hp) }),
    ...(level_override === undefined
      ? {}
      : { level_override: toBigInt(level_override) }),
    conditions: value.conditions.map(convertCondition),
    effective_max_hp: convertNumberFact(value.effective_max_hp),
    knockout_options: Array.isArray(value.knockout_options)
      ? value.knockout_options.map((option) => ({
          ...option,
          context: hpContext(option.context),
        }))
      : value.knockout_options,
    persistent_damage_groups: Array.isArray(value.persistent_damage_groups)
      ? value.persistent_damage_groups.map(convertPersistentGroup)
      : value.persistent_damage_groups,
    ...(ability_states === undefined
      ? {}
      : {
          ability_states: mapArray(ability_states, (ability) => {
            if (
              !ability ||
              !ability.state ||
              typeof ability !== "object" ||
              typeof ability.state !== "object"
            )
              throw new AtlasApiError(200, "Invalid ability state");
            return {
              ...ability,
              state:
                ability.state.kind === "uses"
                  ? {
                      ...ability.state,
                      remaining: toBigInt(ability.state.remaining),
                      maximum: toBigInt(ability.state.maximum),
                    }
                  : ability.state,
            };
          }),
        }),
    ...(presentation === undefined
      ? {}
      : { presentation: convertRecordPresentation(presentation) }),
    ...(turn_resources === undefined
      ? {}
      : { turn_resources: convertTurnResources(turn_resources) }),
    ...(spellcasting === undefined
      ? {}
      : {
          spellcasting: spellcastingView(spellcasting),
        }),
  };
}

export function convertEncounterDetail(
  value: Wire<EncounterDetailView>,
): EncounterDetailView {
  if (!value || typeof value !== "object")
    throw new AtlasApiError(200, "Invalid encounter response");
  if (!value.encounter || typeof value.encounter !== "object")
    throw new AtlasApiError(200, "Invalid encounter summary");
  return {
    ...value,
    revision: toBigInt(value.revision),
    encounter: {
      ...value.encounter,
      round_number: toBigInt(value.encounter.round_number),
    },
    participants: value.participants.map(participant),
  };
}
