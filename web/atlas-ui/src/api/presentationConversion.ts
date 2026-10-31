import type * as Atlas from "../generated/atlas";
import { toBigInt, type Wire } from "./wire";

type InherentActionConditionView = NonNullable<
  Atlas.TurnResourcesView["inherent_conditions"]
>["entries"][number];
type PreparedInclusionView = Extract<
  Atlas.PreparedFieldBodyView,
  { kind: "html" }
>["inclusions"][number];
type ActivityFrequencyView = NonNullable<Atlas.ActorActivityView["frequency"]["value"]>;
type CastingEntryContextView = NonNullable<Atlas.ActorActivityView["casting_context"]>;
type RitualCastingView = NonNullable<Atlas.SpellPresentationView["ritual"]["value"]>;
type ConsumablePresentationView = NonNullable<
  Atlas.PhysicalPresentationView["consumable"]
>;
type ContentRollEncounterContext = NonNullable<Atlas.RollContentRequest["encounter"]>;

function mapArray<T, U>(value: T[], convert: (entry: T) => U): U[] {
  if (!Array.isArray(value)) return value;
  return value.map(convert);
}
function mapFact<T, U>(
  value: Atlas.FactView<T>,
  convert: (entry: T) => U,
): Atlas.FactView<U> {
  if (!value || typeof value !== "object") return value;
  return { ...value, value: value.value === null ? null : convert(value.value) };
}
function mapNullable<T extends object, U>(
  value: T | null,
  convert: (entry: T) => U,
): U | null {
  return value ? convert(value) : value;
}
function integer(value: number): bigint;
function integer(value: number | null): bigint | null;
function integer(value: number | null): bigint | null {
  return value == null ? value : toBigInt(value);
}
function runtimeAdjustment(
  value: Wire<Atlas.RuntimeAdjustmentView>,
): Atlas.RuntimeAdjustmentView {
  if (!value || typeof value !== "object") return value;
  return { ...value, value: integer(value.value) };
}
export function convertRuntimeCount(
  value: Wire<Atlas.RuntimeCountView>,
): Atlas.RuntimeCountView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    base_value: integer(value.base_value),
    adjusted_value: integer(value.adjusted_value),
    segments: mapArray(value.segments, (segment): Atlas.RuntimeCountSegmentView => {
      if (!segment || typeof segment !== "object") return segment;
      return { ...segment, value: integer(segment.value) };
    }),
    adjustments: mapArray(value.adjustments, runtimeAdjustment),
    suppressed_adjustments: mapArray(value.suppressed_adjustments, runtimeAdjustment),
  };
}
function actionBudget(value: Wire<Atlas.ActionBudgetView>): Atlas.ActionBudgetView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    actions: convertRuntimeCount(value.actions),
    reactions: convertRuntimeCount(value.reactions),
  };
}
function inherentConditions(
  value: Wire<NonNullable<Atlas.TurnResourcesView["inherent_conditions"]>>,
): NonNullable<Atlas.TurnResourcesView["inherent_conditions"]> {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    entries: mapArray(value.entries, (entry): InherentActionConditionView => {
      if (!entry || typeof entry !== "object") return entry;
      return { ...entry, value: integer(entry.value) };
    }),
  };
}
export function convertTurnResources(
  value: Wire<Atlas.TurnResourcesView>,
): Atlas.TurnResourcesView {
  if (!value || typeof value !== "object") return value;
  const { inherent_conditions, ...rest } = value;
  const calculation = value.action_calculation;
  return {
    ...rest,
    action_calculation:
      !calculation || typeof calculation !== "object"
        ? calculation
        : calculation.kind === "this_turn"
          ? {
              ...calculation,
              count: mapNullable(calculation.count, convertRuntimeCount),
              changes: mapArray(calculation.changes, runtimeAdjustment),
            }
          : { ...calculation, count: convertRuntimeCount(calculation.count) },
    ...(inherent_conditions === undefined
      ? {}
      : {
          inherent_conditions: inherentConditions(inherent_conditions),
        }),
    round_number: integer(value.round_number),
    regular_total: integer(value.regular_total),
    regular_remaining: integer(value.regular_remaining),
    extra_total: integer(value.extra_total),
    extra_remaining: integer(value.extra_remaining),
    reactions_total: integer(value.reactions_total),
    reactions_remaining: integer(value.reactions_remaining),
    forecast: convertRuntimeCount(value.forecast),
  };
}
function statModifier(value: Wire<Atlas.StatModifierView>): Atlas.StatModifierView {
  if (!value || typeof value !== "object") return value;
  return { ...value, value: integer(value.value) };
}
function numberAdjustment(
  value: Wire<Atlas.NumberAdjustmentView>,
): Atlas.NumberAdjustmentView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    applied: mapArray(value.applied, statModifier),
    suppressed: mapArray(value.suppressed, statModifier),
  };
}
export function convertNumberFact(
  value: Wire<Atlas.NumberFactView>,
): Atlas.NumberFactView {
  if (!value || typeof value !== "object") return value;
  return { ...value, adjustment: mapNullable(value.adjustment, numberAdjustment) };
}
function actorRuntime(value: Wire<Atlas.ActorRuntimeView>): Atlas.ActorRuntimeView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    action_budget: mapNullable(value.action_budget, actionBudget),
    unapplied_effects: mapArray(
      value.unapplied_effects,
      (note): Atlas.UnappliedEffectView => {
        const { condition_id, ...rest } = note;
        return condition_id === undefined
          ? rest
          : { ...rest, condition_id: toBigInt(condition_id) };
      },
    ),
  };
}
function contentRoll(value: Wire<Atlas.RollContentRequest>): Atlas.RollContentRequest {
  if (!value || typeof value !== "object") return value;
  const { encounter, ...rest } = value;
  return encounter === undefined
    ? rest
    : { ...rest, encounter: convertContentRollEncounterContext(encounter) };
}
export function convertContentRollEncounterContext(
  value: Wire<ContentRollEncounterContext>,
): ContentRollEncounterContext {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    participant_revision: integer(value.participant_revision),
    condition_id: integer(value.condition_id),
  };
}
export function convertContentControl(
  value: Wire<Atlas.ContentControlView>,
): Atlas.ContentControlView {
  if (!value || typeof value !== "object") return value;
  const control = value.control;
  if (!control) return { ...value, control };
  return {
    ...value,
    control:
      control.kind === "damage" || control.kind === "command"
        ? { ...control, roll: mapNullable(control.roll, contentRoll) }
        : control,
  };
}
export function convertPreparedFieldBody(
  value: Wire<Atlas.PreparedFieldBodyView>,
): Atlas.PreparedFieldBodyView {
  if (!value || typeof value !== "object" || value.kind !== "html") return value;
  return {
    ...value,
    controls: mapArray(value.controls, convertContentControl),
    inclusions: mapArray(value.inclusions, (inclusion): PreparedInclusionView => {
      if (!inclusion || typeof inclusion !== "object") return inclusion;
      return {
        ...inclusion,
        controls: mapArray(inclusion.controls, convertContentControl),
      };
    }),
  };
}
export function convertPreparedContentField(
  value: Wire<Atlas.PreparedContentFieldView>,
): Atlas.PreparedContentFieldView {
  if (!value || typeof value !== "object") return value;
  return { ...value, body: convertPreparedFieldBody(value.body) };
}
function strikeRange(value: Wire<Atlas.StrikeRangeView>): Atlas.StrikeRangeView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    increment: convertNumberFact(value.increment),
    maximum: convertNumberFact(value.maximum),
  };
}
function activityFrequency(value: Wire<ActivityFrequencyView>): ActivityFrequencyView {
  if (!value || typeof value !== "object") return value;
  return { ...value, maximum: convertNumberFact(value.maximum) };
}
function castingEntry(value: Wire<CastingEntryContextView>): CastingEntryContextView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    spells: mapFact(value.spells, (entries) => mapArray(entries, actorActivity)),
  };
}
function actorActivity(value: Wire<Atlas.ActorActivityView>): Atlas.ActorActivityView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    strike_range: mapFact(value.strike_range, strikeRange),
    focus_capacity: convertNumberFact(value.focus_capacity),
    frequency: mapFact(value.frequency, activityFrequency),
    attack: convertNumberFact(value.attack),
    lore_modifier: convertNumberFact(value.lore_modifier),
    difficulty_class: convertNumberFact(value.difficulty_class),
    base_rank: convertNumberFact(value.base_rank),
    authored_cast_rank: convertNumberFact(value.authored_cast_rank),
    casting_context: mapNullable(value.casting_context, castingEntry),
  };
}
function actorModifier(value: Wire<Atlas.ActorModifierView>): Atlas.ActorModifierView {
  if (!value || typeof value !== "object") return value;
  return { ...value, modifier: convertNumberFact(value.modifier) };
}
function actorSkill(value: Wire<Atlas.ActorSkillView>): Atlas.ActorSkillView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    modifier: convertNumberFact(value.modifier),
    conditional: mapFact(value.conditional, (entries) =>
      mapArray(entries, (entry): Atlas.ConditionalSkillView => {
        if (!entry || typeof entry !== "object") return entry;
        return { ...entry, modifier: convertNumberFact(entry.modifier) };
      }),
    ),
  };
}
function actorMovement(value: Wire<Atlas.ActorMovementView>): Atlas.ActorMovementView {
  if (!value || typeof value !== "object") return value;
  return { ...value, feet: convertNumberFact(value.feet) };
}
function actorSense(value: Wire<Atlas.ActorSenseView>): Atlas.ActorSenseView {
  if (!value || typeof value !== "object") return value;
  return { ...value, range_feet: convertNumberFact(value.range_feet) };
}
function iwr(value: Wire<Atlas.IwrEntryView>): Atlas.IwrEntryView {
  if (!value || typeof value !== "object") return value;
  return { ...value, magnitude: convertNumberFact(value.magnitude) };
}
function actorSaves(value: Wire<Atlas.ActorSavesView>): Atlas.ActorSavesView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    fortitude: convertNumberFact(value.fortitude),
    reflex: convertNumberFact(value.reflex),
    will: convertNumberFact(value.will),
  };
}
function actor(value: Wire<Atlas.ActorPresentationView>): Atlas.ActorPresentationView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    hardness: convertNumberFact(value.hardness),
    ritual_difficulty_class: convertNumberFact(value.ritual_difficulty_class),
    level: convertNumberFact(value.level),
    armor_class: convertNumberFact(value.armor_class),
    maximum_hp: convertNumberFact(value.maximum_hp),
    perception: convertNumberFact(value.perception),
    saves: actorSaves(value.saves),
    abilities: mapFact(value.abilities, (entries) => mapArray(entries, actorModifier)),
    skills: mapFact(value.skills, (entries) => mapArray(entries, actorSkill)),
    movement: mapFact(value.movement, (entries) => mapArray(entries, actorMovement)),
    land_speed: convertNumberFact(value.land_speed),
    immunities: mapFact(value.immunities, (entries) => mapArray(entries, iwr)),
    weaknesses: mapFact(value.weaknesses, (entries) => mapArray(entries, iwr)),
    resistances: mapFact(value.resistances, (entries) => mapArray(entries, iwr)),
    senses: mapFact(value.senses, (entries) => mapArray(entries, actorSense)),
    activities: mapFact(value.activities, (entries) =>
      mapArray(entries, actorActivity),
    ),
    runtime: mapNullable(value.runtime, actorRuntime),
  };
}
function hazard(
  value: Wire<Atlas.HazardPresentationView>,
): Atlas.HazardPresentationView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    detection_dc: convertNumberFact(value.detection_dc),
    broken_threshold: convertNumberFact(value.broken_threshold),
    actor: actor(value.actor),
    stealth: convertNumberFact(value.stealth),
    hardness: convertNumberFact(value.hardness),
  };
}
function ritual(value: Wire<RitualCastingView>): RitualCastingView {
  if (!value || typeof value !== "object") return value;
  return { ...value, secondary_casters: convertNumberFact(value.secondary_casters) };
}
function spellFixedLevel(
  value: Wire<Atlas.SpellFixedLevelView>,
): Atlas.SpellFixedLevelView {
  if (!value || typeof value !== "object") return value;
  return { ...value, changes: mapFact(value.changes, spellChange) };
}
function spellHeighteningChange(
  value: Wire<Atlas.SpellHeighteningChangeView>,
): Atlas.SpellHeighteningChangeView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    interval: convertNumberFact(value.interval),
    area: convertNumberFact(value.area),
    levels: mapFact(value.levels, (entries) => mapArray(entries, spellFixedLevel)),
  };
}
function spellChange(value: Wire<Atlas.SpellChangeView>): Atlas.SpellChangeView {
  if (!value || typeof value !== "object") return value;
  return { ...value, heightening: mapFact(value.heightening, spellHeighteningChange) };
}
function spellHeightening(
  value: Wire<Atlas.SpellHeighteningView>,
): Atlas.SpellHeighteningView {
  if (!value || typeof value !== "object") return value;
  return value.kind === "interval"
    ? {
        ...value,
        interval: convertNumberFact(value.interval),
        area: convertNumberFact(value.area),
      }
    : {
        ...value,
        levels: mapFact(value.levels, (entries) => mapArray(entries, spellFixedLevel)),
      };
}
function spellForm(value: Wire<Atlas.SpellFormView>): Atlas.SpellFormView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    sort: convertNumberFact(value.sort),
    changes: mapFact(value.changes, spellChange),
  };
}
function spell(value: Wire<Atlas.SpellPresentationView>): Atlas.SpellPresentationView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    runtime_attack: mapNullable(value.runtime_attack, convertNumberFact),
    runtime_difficulty_class: mapNullable(
      value.runtime_difficulty_class,
      convertNumberFact,
    ),
    ritual: mapFact(value.ritual, ritual),
    base_rank: convertNumberFact(value.base_rank),
    rank: convertNumberFact(value.rank),
    heightening: mapFact(value.heightening, spellHeightening),
    forms: mapFact(value.forms, (entries) => mapArray(entries, spellForm)),
    authored_cast_rank: convertNumberFact(value.authored_cast_rank),
  };
}
function weaponPersistent(
  value: Wire<Atlas.WeaponPersistentReferenceView>,
): Atlas.WeaponPersistentReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    number: convertNumberFact(value.number),
    faces: convertNumberFact(value.faces),
  };
}
function weaponDamage(
  value: Wire<Atlas.WeaponDamageReferenceView>,
): Atlas.WeaponDamageReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    dice: convertNumberFact(value.dice),
    modifier: convertNumberFact(value.modifier),
    persistent: mapFact(value.persistent, weaponPersistent),
  };
}
function weaponAmmunition(
  value: Wire<Atlas.WeaponAmmunitionReferenceView>,
): Atlas.WeaponAmmunitionReferenceView {
  if (!value || typeof value !== "object") return value;
  return { ...value, capacity: convertNumberFact(value.capacity) };
}
function weaponRunes(
  value: Wire<Atlas.WeaponRuneReferenceView>,
): Atlas.WeaponRuneReferenceView {
  if (!value || typeof value !== "object") return value;
  return { ...value, potency: convertNumberFact(value.potency) };
}
function weapon(value: Wire<Atlas.WeaponReferenceView>): Atlas.WeaponReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    damage: mapFact(value.damage, weaponDamage),
    range_increment: convertNumberFact(value.range_increment),
    maximum_range: convertNumberFact(value.maximum_range),
    ammunition: mapFact(value.ammunition, weaponAmmunition),
    runes: mapFact(value.runes, weaponRunes),
  };
}
function armor(value: Wire<Atlas.ArmorReferenceView>): Atlas.ArmorReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    ac_bonus: convertNumberFact(value.ac_bonus),
    dex_cap: convertNumberFact(value.dex_cap),
    check_penalty: convertNumberFact(value.check_penalty),
    speed_penalty: convertNumberFact(value.speed_penalty),
    strength: convertNumberFact(value.strength),
    potency: convertNumberFact(value.potency),
  };
}
function shield(value: Wire<Atlas.ShieldReferenceView>): Atlas.ShieldReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    ac_bonus: convertNumberFact(value.ac_bonus),
    speed_penalty: convertNumberFact(value.speed_penalty),
    hardness: convertNumberFact(value.hardness),
    hit_points: convertNumberFact(value.hit_points),
    reinforcing: convertNumberFact(value.reinforcing),
  };
}
function container(
  value: Wire<Atlas.ContainerReferenceView>,
): Atlas.ContainerReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    capacity: convertNumberFact(value.capacity),
    ignored_bulk: convertNumberFact(value.ignored_bulk),
    held_or_stowed_bulk: convertNumberFact(value.held_or_stowed_bulk),
  };
}
function ammunition(
  value: Wire<Atlas.AmmunitionReferenceView>,
): Atlas.AmmunitionReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    maximum_uses: convertNumberFact(value.maximum_uses),
    source_uses: convertNumberFact(value.source_uses),
  };
}
function kitEntry(
  value: Wire<Atlas.KitEntryReferenceView>,
): Atlas.KitEntryReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    quantity: convertNumberFact(value.quantity),
    items: mapFact(value.items, (entries) => mapArray(entries, kitEntry)),
  };
}
function kit(value: Wire<Atlas.KitReferenceView>): Atlas.KitReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    items: mapFact(value.items, (entries) => mapArray(entries, kitEntry)),
  };
}
function physicalDetails(
  value: Wire<Atlas.PhysicalDetailsView>,
): Atlas.PhysicalDetailsView {
  if (!value || typeof value !== "object") return value;
  switch (value.kind) {
    case "weapon":
      return { ...value, value: weapon(value.value) };
    case "armor":
      return { ...value, value: armor(value.value) };
    case "shield":
      return { ...value, value: shield(value.value) };
    case "container":
      return { ...value, value: container(value.value) };
    case "ammunition":
      return { ...value, value: ammunition(value.value) };
    case "kit":
      return { ...value, value: kit(value.value) };
    default:
      return value;
  }
}
function physicalPrice(value: Wire<Atlas.PhysicalPriceView>): Atlas.PhysicalPriceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    pp: convertNumberFact(value.pp),
    gp: convertNumberFact(value.gp),
    sp: convertNumberFact(value.sp),
    cp: convertNumberFact(value.cp),
  };
}
function consumable(
  value: Wire<ConsumablePresentationView>,
): ConsumablePresentationView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    maximum_hp: convertNumberFact(value.maximum_hp),
    hardness: convertNumberFact(value.hardness),
    maximum_uses: convertNumberFact(value.maximum_uses),
    source_uses: convertNumberFact(value.source_uses),
  };
}
function physical(
  value: Wire<Atlas.PhysicalPresentationView>,
): Atlas.PhysicalPresentationView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    details: mapNullable(value.details, physicalDetails),
    quantity: convertNumberFact(value.quantity),
    consumable: mapNullable(value.consumable, consumable),
    bulk: convertNumberFact(value.bulk),
    price: mapFact(value.price, physicalPrice),
    price_per: convertNumberFact(value.price_per),
  };
}
function referenceGrant(
  value: Wire<Atlas.ReferenceGrantView>,
): Atlas.ReferenceGrantView {
  if (!value || typeof value !== "object") return value;
  return { ...value, level: convertNumberFact(value.level) };
}
function ancestry(
  value: Wire<Atlas.AncestryReferenceView>,
): Atlas.AncestryReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    hit_points: convertNumberFact(value.hit_points),
    speed: convertNumberFact(value.speed),
    hands: convertNumberFact(value.hands),
    reach: convertNumberFact(value.reach),
    additional_language_count: convertNumberFact(value.additional_language_count),
    grants: mapFact(value.grants, (entries) => mapArray(entries, referenceGrant)),
  };
}
function classReference(
  value: Wire<Atlas.ClassReferenceView>,
): Atlas.ClassReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    hit_points: convertNumberFact(value.hit_points),
    additional_skills: convertNumberFact(value.additional_skills),
    grants: mapFact(value.grants, (entries) => mapArray(entries, referenceGrant)),
  };
}
function deity(value: Wire<Atlas.DeityReferenceView>): Atlas.DeityReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    spells: mapFact(value.spells, (entries) => mapArray(entries, referenceGrant)),
  };
}
function objectDefenses(
  value: Wire<Atlas.ObjectDefensesReferenceView>,
): Atlas.ObjectDefensesReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    armor_class: convertNumberFact(value.armor_class),
    hit_points: convertNumberFact(value.hit_points),
    hardness: convertNumberFact(value.hardness),
    immunities: mapFact(value.immunities, (entries) => mapArray(entries, iwr)),
    weaknesses: mapFact(value.weaknesses, (entries) => mapArray(entries, iwr)),
    resistances: mapFact(value.resistances, (entries) => mapArray(entries, iwr)),
  };
}
function vehicle(value: Wire<Atlas.VehicleReferenceView>): Atlas.VehicleReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    price_gp: convertNumberFact(value.price_gp),
    length: convertNumberFact(value.length),
    width: convertNumberFact(value.width),
    height: convertNumberFact(value.height),
    collision_dc: convertNumberFact(value.collision_dc),
    fortitude: convertNumberFact(value.fortitude),
    defenses: objectDefenses(value.defenses),
  };
}
function army(value: Wire<Atlas.ArmyReferenceView>): Atlas.ArmyReferenceView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    armor_class: convertNumberFact(value.armor_class),
    hit_points: convertNumberFact(value.hit_points),
    rout_threshold: convertNumberFact(value.rout_threshold),
    maneuver: convertNumberFact(value.maneuver),
    morale: convertNumberFact(value.morale),
    scouting: convertNumberFact(value.scouting),
    consumption: convertNumberFact(value.consumption),
    recruitment_dc: convertNumberFact(value.recruitment_dc),
    ammunition: convertNumberFact(value.ammunition),
    potions: convertNumberFact(value.potions),
    immunities: mapFact(value.immunities, (entries) => mapArray(entries, iwr)),
    weaknesses: mapFact(value.weaknesses, (entries) => mapArray(entries, iwr)),
    resistances: mapFact(value.resistances, (entries) => mapArray(entries, iwr)),
  };
}
function tableResult(
  value: Wire<Atlas.TableResultPresentationView>,
): Atlas.TableResultPresentationView {
  if (!value || typeof value !== "object") return value;
  return { ...value, weight: convertNumberFact(value.weight) };
}
function recordBody(value: Wire<Atlas.RecordBodyView>): Atlas.RecordBodyView {
  if (!value || typeof value !== "object") return value;
  switch (value.kind) {
    case "creature":
      return { ...value, value: actor(value.value) };
    case "hazard":
      return { ...value, value: hazard(value.value) };
    case "spell":
      return { ...value, value: spell(value.value) };
    case "activity":
      return { ...value, value: actorActivity(value.value) };
    case "physical_reference":
      return { ...value, value: physical(value.value) };
    case "ancestry":
      return { ...value, value: ancestry(value.value) };
    case "class":
      return { ...value, value: classReference(value.value) };
    case "deity":
      return { ...value, value: deity(value.value) };
    case "vehicle":
      return { ...value, value: vehicle(value.value) };
    case "army":
      return { ...value, value: army(value.value) };
    case "table_result":
      return { ...value, value: tableResult(value.value) };
    case "heritage":
    case "roll_table":
    case "journal":
    case "journal_page":
    case "content":
      return value;
    default:
      return value;
  }
}
export function convertRecordPresentation(
  value: Wire<Atlas.RecordPresentationView>,
): Atlas.RecordPresentationView {
  if (!value || typeof value !== "object") return value;
  return {
    ...value,
    content: mapArray(value.content, convertPreparedContentField),
    body: recordBody(value.body),
  };
}
export function convertRecordDetail(
  value: Wire<Atlas.RecordDetailView>,
): Atlas.RecordDetailView {
  if (!value || typeof value !== "object") return value;
  return { ...value, presentation: convertRecordPresentation(value.presentation) };
}
