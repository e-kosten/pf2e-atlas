import { Alert, Button, Collapse, Popover, Tag } from "antd";
import { Link2 } from "lucide-react";
import { useEffect, useId, useRef } from "react";
import type {
  ActorActivityView,
  ActorPresentationView,
  RuntimeCountView,
  DamageComponentView,
  FactView,
  IwrEntryView,
} from "../../generated/atlas";
import {
  FactLine,
  FactRow,
  NumberFact,
  factText,
  hasReadingFact,
  isEmptyFactValue,
} from "./presentationFacts";
import type { RecordReferenceHandler } from "./PreparedContent";
import {
  EmbeddedAbilityReading,
  type EmbeddedReadingContext,
} from "./EmbeddedAbilityReading";
import { EffectReferenceLink } from "./EffectReferenceLink";
import { ActionCost } from "./ActionCost";
import { sentenceCase } from "../ui/labels";
import { activityAnchor } from "./embeddedNavigation";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";

function qualifierText<T extends string | string[]>(
  fact: FactView<T>,
  patch = false,
): string {
  if (!hasReadingFact(fact, { patch })) return "";
  return patch && fact.state === "null"
    ? "cleared"
    : factText(
        fact,
        (value) => (Array.isArray(value) ? value.join(", ") : String(value)),
        { patch },
      );
}

export function DamageComponents({
  damage,
  patch = false,
}: {
  damage: FactView<DamageComponentView[]>;
  patch?: boolean;
}) {
  if (!hasReadingFact(damage, { patch })) return null;
  if (damage.state !== "value" || damage.value === null)
    return (
      <FactRow label="Damage">
        {patch && damage.state === "null" ? "Cleared" : factText(damage)}
      </FactRow>
    );
  if (!damage.value.length)
    return <FactLine label="Damage" fact={damage} patch={patch} />;
  const components = damage.value;
  return (
    <div className="record-damage">
      {components.map((component, index) => {
        const kinds = qualifierText(component.kinds, patch);
        const category =
          !patch && ["missing", "null"].includes(component.category.state)
            ? ""
            : qualifierText(component.category, patch);
        const materials = qualifierText(component.materials, patch);
        return (
          <p key={component.id}>
            {patch || components.length > 1 ? (
              <strong>
                {components.length === 1 ? "Damage" : `Damage component ${index + 1}`}
                :{" "}
              </strong>
            ) : null}
            <strong>
              {patch && component.formula.state === "missing"
                ? ""
                : patch && component.formula.state === "null"
                  ? "Formula cleared"
                  : factText(component.formula, String, { required: true, patch })}
            </strong>{" "}
            {patch && component.damage_type.state === "missing"
              ? ""
              : patch && component.damage_type.state === "null"
                ? "Type cleared"
                : factText(component.damage_type, String, { required: true, patch })}
            {kinds
              ? component.kinds.state === "value" &&
                !(patch && isEmptyFactValue(component.kinds.value))
                ? ` (${kinds})`
                : `; kinds ${kinds}`
              : ""}
            {category
              ? component.category.state === "value" &&
                !(patch && isEmptyFactValue(component.category.value))
                ? `; ${category}`
                : `; category ${category}`
              : ""}
            {materials
              ? component.materials.state === "value" &&
                !(patch && isEmptyFactValue(component.materials.value))
                ? `; ${materials}`
                : `; materials ${materials}`
              : ""}
            {component.apply_modifier.state === "value" &&
            component.apply_modifier.value
              ? "; applies modifier"
              : ""}
            {patch &&
            component.apply_modifier.state === "value" &&
            component.apply_modifier.value === false
              ? "; does not apply modifier"
              : ""}
            {["null", "invalid"].includes(component.apply_modifier.state) ||
            (!patch && component.apply_modifier.state === "missing")
              ? `; applies modifier ${factText(component.apply_modifier, String, { patch })}`
              : ""}
          </p>
        );
      })}
    </div>
  );
}

function Iwr({ label, fact }: { label: string; fact: FactView<IwrEntryView[]> }) {
  if (
    fact.state === "not_applicable" ||
    (fact.state === "value" && !fact.value?.length)
  )
    return null;
  return (
    <FactRow label={label}>
      {fact.state !== "value"
        ? factText(fact)
        : fact.value?.map((entry, index) => {
            const exceptions =
              entry.exceptions.state === "missing"
                ? ""
                : qualifierText(entry.exceptions);
            const doubleAgainst =
              entry.double_against.state === "missing"
                ? ""
                : qualifierText(entry.double_against);
            return (
              <span key={index}>
                {index ? "; " : ""}
                {factText(entry.damage_type)}
                {entry.magnitude.state !== "not_applicable" ? (
                  <>
                    {" "}
                    <NumberFact fact={entry.magnitude} />
                  </>
                ) : null}
                {exceptions
                  ? ` (${entry.exceptions.state === "value" ? "except" : "exceptions"} ${exceptions})`
                  : ""}
                {doubleAgainst ? ` (double against ${doubleAgainst})` : ""}
              </span>
            );
          })}
    </FactRow>
  );
}

function DefenseNote({ label, fact }: { label: string; fact: FactView<string> }) {
  if (fact.state === "missing" || fact.state === "not_applicable") return null;
  return <FactLine label={label} fact={fact} />;
}

function DefenseQualifier({ fact }: { fact: FactView<string> }) {
  if (fact.state === "missing" || !hasReadingFact(fact)) return null;
  return <> {factText(fact)}</>;
}

export function ActorFacts({
  actor,
  hazard = false,
  hideHitPoints = false,
  encounter = false,
  hidePrimaryDefenses = false,
  hideSaves = false,
}: {
  actor: ActorPresentationView;
  hazard?: boolean;
  hideHitPoints?: boolean;
  encounter?: boolean;
  hidePrimaryDefenses?: boolean;
  hideSaves?: boolean;
}) {
  const defenseFacts: FactView<unknown>[] = [
    ...(!hidePrimaryDefenses ? [actor.armor_class] : []),
    ...(!hidePrimaryDefenses && !hideHitPoints ? [actor.maximum_hp] : []),
    ...(!hideSaves ? Object.values(actor.saves) : []),
    actor.hardness,
    ...Object.values(actor.defense_notes),
    actor.immunities,
    actor.weaknesses,
    actor.resistances,
  ];
  const awarenessFacts: FactView<unknown>[] = [
    actor.perception,
    actor.senses,
    ...(!encounter ? [actor.perception_details] : []),
    ...(!encounter ? [actor.languages, actor.language_details] : []),
  ];
  return (
    <>
      <DefenseNote label="Size" fact={actor.size} />
      <div
        className={`actor-reference__facts${encounter ? " actor-reference__facts--combat" : ""}`}
      >
        {defenseFacts.some((fact) => hasReadingFact(fact)) && (
          <section className="actor-reference__defenses">
            <h3>Defenses</h3>
            {actor.hardness.state !== "missing" &&
              actor.hardness.state !== "not_applicable" && (
                <FactRow label="Hardness">
                  <NumberFact fact={actor.hardness} />
                </FactRow>
              )}
            {!hidePrimaryDefenses && actor.armor_class.state !== "not_applicable" && (
              <FactRow label="AC">
                <NumberFact fact={actor.armor_class} />
                <DefenseQualifier fact={actor.defense_notes.armor_class} />
              </FactRow>
            )}
            {!hideSaves && (
              <div>
                <div className="record-fact-list">
                  {Object.entries(actor.saves).map(([save, fact]) =>
                    fact.state === "not_applicable" ? null : (
                      <FactRow key={save} label={save[0].toUpperCase() + save.slice(1)}>
                        <NumberFact fact={fact} signed />
                        <DefenseQualifier
                          fact={actor.defense_notes[save as keyof typeof actor.saves]}
                        />
                      </FactRow>
                    ),
                  )}
                </div>
                {actor.defense_notes.all_saves.state !== "missing" &&
                  hasReadingFact(actor.defense_notes.all_saves) && (
                    <div className="record-fact">
                      {factText(actor.defense_notes.all_saves)}
                    </div>
                  )}
              </div>
            )}
            {!hidePrimaryDefenses &&
              !hideHitPoints &&
              actor.maximum_hp.state !== "not_applicable" && (
                <FactRow label="HP">
                  <NumberFact fact={actor.maximum_hp} />
                  <DefenseQualifier fact={actor.defense_notes.hit_points} />
                </FactRow>
              )}
            {(hideHitPoints || hidePrimaryDefenses) && (
              <DefenseNote label="HP" fact={actor.defense_notes.hit_points} />
            )}
            <Iwr label="Immunities" fact={actor.immunities} />
            <Iwr label="Weaknesses" fact={actor.weaknesses} />
            <Iwr label="Resistances" fact={actor.resistances} />
          </section>
        )}
        {!hazard && awarenessFacts.some((fact) => hasReadingFact(fact)) && (
          <div className="actor-reference__awareness">
            {!encounter && <h3>Awareness and communication</h3>}
            {actor.perception.state !== "not_applicable" && (
              <FactRow label="Perception">
                <NumberFact fact={actor.perception} signed />
              </FactRow>
            )}
            <FactLine
              label="Senses"
              fact={actor.senses}
              format={(senses) =>
                senses
                  .map((sense) =>
                    [
                      factText(sense.sense),
                      sense.acuity.state === "missing" ? "" : factText(sense.acuity),
                      sense.range_feet.state === "missing"
                        ? ""
                        : factText(sense.range_feet, (value) => `${value} feet`),
                    ]
                      .filter(Boolean)
                      .join(" "),
                  )
                  .join(", ")
              }
            />
            {!encounter && (
              <FactLine label="Perception details" fact={actor.perception_details} />
            )}
            {!encounter && (
              <>
                <FactLine
                  label="Languages"
                  fact={actor.languages}
                  format={(value) => value.join(", ")}
                />
                <FactLine label="Communication" fact={actor.language_details} />
              </>
            )}
          </div>
        )}
        {!hazard && !encounter && <ActorSkills actor={actor} />}
        {!hazard &&
          (hasReadingFact(actor.land_speed) || hasReadingFact(actor.movement)) && (
            <section className="actor-reference__movement">
              {!encounter && <h3>Movement</h3>}
              <div className="record-fact-list">
                {actor.land_speed.state !== "not_applicable" && (
                  <FactRow label="Speed">
                    <NumberFact fact={actor.land_speed} unit=" feet" label="Speed" />
                  </FactRow>
                )}
                {actor.movement.state !== "value" && (
                  <FactLine label="Other movement" fact={actor.movement} />
                )}
                {(actor.movement.value ?? []).map((movement, index) => (
                  <FactRow key={index} label={factText(movement.movement_type)}>
                    <NumberFact fact={movement.feet} unit=" feet" />
                  </FactRow>
                ))}
              </div>
            </section>
          )}
        {encounter ? (
          <ActorCombatBudget actor={actor} />
        ) : (
          <ActorRuntime actor={actor} />
        )}
      </div>
    </>
  );
}

function ActorSkills({ actor }: { actor: ActorPresentationView }) {
  const lore = (actor.activities.value ?? []).filter(
    (activity) => activity.kind === "lore",
  );
  if (!hasReadingFact(actor.abilities) && !hasReadingFact(actor.skills) && !lore.length)
    return null;
  return (
    <div className="actor-reference__skills">
      {hasReadingFact(actor.abilities) && (
        <section>
          <h3>Ability modifiers</h3>
          {actor.abilities.state !== "value" && (
            <FactLine label="Ability modifiers" fact={actor.abilities} />
          )}
          <div className="record-fact-list">
            {(actor.abilities.value ?? []).map((ability) => (
              <FactRow key={ability.key} label={ability.label}>
                <NumberFact fact={ability.modifier} signed />
              </FactRow>
            ))}
          </div>
        </section>
      )}
      {(hasReadingFact(actor.skills) || lore.length > 0) && (
        <section>
          <h3>Skills</h3>
          {hasReadingFact(actor.skills) && actor.skills.state !== "value" && (
            <FactLine label="Skills" fact={actor.skills} />
          )}
          <div className="record-fact-list">
            {(actor.skills.value ?? []).map((skill) => (
              <div key={skill.key}>
                <FactRow label={sentenceCase(skill.label)}>
                  <NumberFact fact={skill.modifier} signed />
                  {skill.note.state !== "missing" && hasReadingFact(skill.note)
                    ? `; ${factText(skill.note)}`
                    : ""}
                  {!["missing", "value", "not_applicable"].includes(
                    skill.conditional.state,
                  )
                    ? `; conditional modifiers ${factText(skill.conditional)}`
                    : ""}
                  {skill.conditional.value?.map((conditional, index) => (
                    <span key={index}>
                      ; {factText(conditional.label)}{" "}
                      <NumberFact fact={conditional.modifier} signed />
                    </span>
                  ))}
                </FactRow>
              </div>
            ))}
            {lore.map((activity) => (
              <FactRow key={JSON.stringify(activity.navigation)} label={activity.title}>
                <NumberFact fact={activity.lore_modifier} signed />
              </FactRow>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
function hasRuntimeDetails(actor: ActorPresentationView) {
  return (
    (actor.runtime?.action_budget?.notes.length ?? 0) > 0 ||
    (actor.runtime?.unapplied_effects.some(
      (effect) => effect.condition_id === undefined,
    ) ??
      false)
  );
}
function ActorRuntime({
  actor,
  detailsOnly = false,
}: {
  actor: ActorPresentationView;
  detailsOnly?: boolean;
}) {
  return (
    <>
      {actor.runtime &&
        (detailsOnly
          ? hasRuntimeDetails(actor)
          : actor.runtime.action_budget !== null ||
            actor.runtime.unapplied_effects.length > 0) && (
          <div className="actor-reference__runtime">
            <h3>Runtime</h3>
            {actor.runtime.action_budget && (
              <>
                {!detailsOnly && (
                  <RuntimeCount count={actor.runtime.action_budget.actions} />
                )}
                {!detailsOnly && (
                  <RuntimeCount count={actor.runtime.action_budget.reactions} />
                )}
                {actor.runtime.action_budget.notes.map((note, index) => (
                  <p key={index}>
                    {note.label}: {note.reason}
                  </p>
                ))}
              </>
            )}
            {!detailsOnly &&
              actor.runtime.action_budget &&
              !actor.runtime.action_budget.can_act.available && (
                <FactRow label="Can act">
                  No; {actor.runtime.action_budget.can_act.reason}
                </FactRow>
              )}
            {!detailsOnly &&
              actor.runtime.action_budget &&
              !actor.runtime.action_budget.can_react.available && (
                <FactRow label="Can react">
                  No; {actor.runtime.action_budget.can_react.reason}
                </FactRow>
              )}
            {actor.runtime.unapplied_effects
              .filter((effect) => !detailsOnly || effect.condition_id === undefined)
              .map((effect, index) => (
                <p key={index}>
                  {effect.label}: {effect.reason}
                </p>
              ))}
          </div>
        )}
    </>
  );
}
function ActorCombatBudget({ actor }: { actor: ActorPresentationView }) {
  const budget = actor.runtime?.action_budget;
  return budget ? (
    <div className="actor-reference__combat-budget record-fact-list">
      <RuntimeCount count={budget.actions} />
      <RuntimeCount count={budget.reactions} />
      {!budget.can_act.available && (
        <FactRow label="Can act">No; {budget.can_act.reason}</FactRow>
      )}
      {!budget.can_react.available && (
        <FactRow label="Can react">No; {budget.can_react.reason}</FactRow>
      )}
    </div>
  ) : null;
}
export function ActorSecondaryFacts({
  actor,
  hazard = false,
}: {
  actor: ActorPresentationView;
  hazard?: boolean;
}) {
  if (
    !hasRuntimeDetails(actor) &&
    (hazard ||
      (![
        actor.perception_details,
        actor.languages,
        actor.language_details,
        actor.abilities,
        actor.skills,
      ].some((fact) => hasReadingFact(fact as FactView<unknown>)) &&
        !(actor.activities.value ?? []).some((activity) => activity.kind === "lore")))
  )
    return null;
  if (hazard)
    return (
      <Collapse
        size="small"
        ghost
        items={[
          {
            key: "runtime",
            label: "Runtime details",
            children: <ActorRuntime actor={actor} detailsOnly />,
          },
        ]}
      />
    );
  return (
    <section className="actor-reference__secondary">
      {!hazard && (
        <>
          <FactLine label="Perception details" fact={actor.perception_details} />
          <FactLine
            label="Languages"
            fact={actor.languages}
            format={(value) => value.join(", ")}
          />
          <FactLine label="Communication" fact={actor.language_details} />
          <ActorSkills actor={actor} />
        </>
      )}
      <ActorRuntime actor={actor} detailsOnly />
    </section>
  );
}

export type CastingAnnotations = {
  entry: (entry: ActorActivityView) => React.ReactNode;
  rank: (entry: ActorActivityView, rank: number) => React.ReactNode;
  spell: (spell: ActorActivityView) => {
    muted: boolean;
    status?: string;
    suffix?: React.ReactNode;
  };
};

export function ActorActivity({
  activity,
  onReference,
  selected = false,
  headingActions,
  rollControls,
  hideCost = false,
  inline = false,
  permalink = true,
  interactive = true,
}: {
  activity: ActorActivityView;
  onReference: RecordReferenceHandler;
  selected?: boolean;
  headingActions?: React.ReactNode;
  rollControls?: React.ReactNode;
  hideCost?: boolean;
  inline?: boolean;
  permalink?: boolean;
  interactive?: boolean;
}) {
  const id = useId();
  const route = {
    kind: "record" as const,
    recordKey: activity.navigation.record_key,
    selection: activity.navigation,
  };
  return (
    <div
      className="actor-activity"
      data-atlas-activity-anchor={
        !inline ? activityAnchor(activity.navigation) : undefined
      }
    >
      {inline && (
        <h4
          id={`${id}-${activityAnchor(activity.navigation)}`}
          data-atlas-activity-anchor={activityAnchor(activity.navigation)}
          tabIndex={-1}
          className="actor-activity__title"
        >
          {activity.title}{" "}
          {activity.action_cost.state === "value" && (
            <ActionCost fact={activity.action_cost} />
          )}
          {permalink && (
            <Button
              type="link"
              size="small"
              icon={<Link2 size={14} aria-hidden="true" />}
              aria-label={`Open ${activity.title} full reference`}
              href={atlasRoutePath(route)}
              onClick={(event) => {
                if (!shouldHandleAtlasRouteClick(event)) return;
                event.preventDefault();
                navigateToAtlasRoute(route);
              }}
            />
          )}
        </h4>
      )}
      {!selected &&
        !inline &&
        (interactive ? (
          <Button
            type="link"
            style={{ paddingInline: 0, height: "auto", fontWeight: 700 }}
            href={atlasRoutePath(route)}
            onClick={(event) => {
              if (!shouldHandleAtlasRouteClick(event)) return;
              event.preventDefault();
              onReference(
                activity.navigation.record_key,
                event.currentTarget,
                activity.navigation,
              );
            }}
          >
            {activity.title}
          </Button>
        ) : (
          <strong>{activity.title}</strong>
        ))}
      {headingActions}
      {activity.lore_modifier.state !== "not_applicable" && (
        <span className="actor-activity__modifier">
          {" "}
          <NumberFact fact={activity.lore_modifier} signed />
        </span>
      )}
      {activity.effect_duration.value ? (
        <>
          <FactLine
            label="Duration"
            fact={activity.effect_duration.value.duration}
            required
          />
          {activity.effect_duration.value.sustained.value !== false && (
            <FactLine
              label="Sustained"
              fact={activity.effect_duration.value.sustained}
              format={() => "Yes"}
            />
          )}
          <FactLine label="Expires" fact={activity.effect_duration.value.expiry} />
        </>
      ) : (
        <FactLine label="Duration" fact={activity.effect_duration} />
      )}
      {activity.frequency.state !== "missing" &&
        activity.frequency.state !== "not_applicable" && (
          <FactLine
            label="Frequency"
            fact={activity.frequency}
            format={(frequency) =>
              `${factText(frequency.maximum)} per ${factText(frequency.interval)}`
            }
          />
        )}
      {!hideCost && (!inline || activity.action_cost.state !== "value") && (
        <ActionCost fact={activity.action_cost} label="Action cost" />
      )}
      {activity.usage && !isEmptyFactValue(activity.usage) && (
        <Tag>{activity.usage}</Tag>
      )}
      {!selected && (
        <div className="badge-row">
          {activity.traits.map((trait) => (
            <Tag key={`${trait.kind}-${trait.value}`}>{trait.label}</Tag>
          ))}
        </div>
      )}
      {rollControls ?? (
        <>
          {activity.attack.state !== "not_applicable" && (
            <FactRow label="Attack">
              <NumberFact fact={activity.attack} signed />
            </FactRow>
          )}
          <DamageComponents damage={activity.damage} />
        </>
      )}
      {activity.strike_effects.state !== "missing" &&
        activity.strike_effects.state !== "not_applicable" && (
          <FactLine
            label="Attack effects"
            fact={activity.strike_effects}
            format={(effects) => effects.join(", ")}
          />
        )}
      {activity.self_effect.state !== "missing" &&
        activity.self_effect.state !== "not_applicable" && (
          <FactRow label="Self effect">
            {activity.self_effect.value ? (
              activity.self_effect.value.navigation ? (
                <EffectReferenceLink
                  label={{
                    text: factText(activity.self_effect.value.name),
                    title: factText(activity.self_effect.value.name),
                  }}
                  target={activity.self_effect.value.navigation}
                  onReference={onReference}
                />
              ) : (
                <>
                  {factText(activity.self_effect.value.name)}
                  {!activity.self_effect.value.navigation && " (reference unavailable)"}
                </>
              )
            ) : (
              factText(activity.self_effect)
            )}
          </FactRow>
        )}
      <FactLine label="Tradition" fact={activity.casting_tradition} />
      <FactLine label="Preparation" fact={activity.preparation} />
      {activity.focus_capacity.state !== "missing" &&
        activity.focus_capacity.state !== "not_applicable" && (
          <FactRow label="Focus capacity">
            <NumberFact fact={activity.focus_capacity} />
          </FactRow>
        )}
      {activity.difficulty_class.state !== "not_applicable" && (
        <FactRow label="DC">
          <NumberFact fact={activity.difficulty_class} />
        </FactRow>
      )}

      {activity.kind === "spell" &&
        activity.casting_entry === null &&
        activity.association.state !== "not_applicable" && (
          <p className="muted">Spellcasting group unavailable.</p>
        )}
      {activity.base_rank.state !== "not_applicable" && (
        <FactRow label="Base rank">
          <NumberFact fact={activity.base_rank} />
        </FactRow>
      )}
      {activity.authored_cast_rank.state !== "not_applicable" &&
        activity.authored_cast_rank.state !== "missing" && (
          <FactRow label="Cast rank">
            <NumberFact fact={activity.authored_cast_rank} />
          </FactRow>
        )}
      {activity.casting_context && hasReadingFact(activity.casting_context.spells) && (
        <section className="actor-casting__context">
          <h3>Spells for {activity.casting_context.parent_title}</h3>
          {activity.casting_context.spells.state === "value" ? (
            <CastingSpells
              spells={activity.casting_context.spells.value ?? []}
              onReference={onReference}
            />
          ) : (
            <FactRow label="Associated spells">
              {factText(activity.casting_context.spells)}
            </FactRow>
          )}
        </section>
      )}
      {activity.notes.map((note, index) => (
        <p key={index}>
          {note.label}: {note.reason}
        </p>
      ))}
    </div>
  );
}

export function ActorActivities({
  actor,
  onReference,
  encounter = false,
  reading,
  castingAnnotations,
  activityRolls,
}: {
  actor: ActorPresentationView;
  onReference: RecordReferenceHandler;
  encounter?: boolean;
  reading?: EmbeddedReadingContext;
  castingAnnotations?: CastingAnnotations;
  activityRolls?: (activity: ActorActivityView) => React.ReactNode;
}) {
  const container = useRef<HTMLDivElement>(null);
  const target = reading?.landing;
  const onLandingReady = reading?.onLandingReady;
  const targetActivity = target
    ? actor.activities.value?.find(
        (activity) => activityAnchor(activity.navigation) === activityAnchor(target),
      )
    : undefined;
  useEffect(() => {
    if (!target || targetActivity?.kind === "ability") return;
    if (!targetActivity) {
      onLandingReady?.();
      return;
    }
    const element = Array.from(
      container.current?.querySelectorAll<HTMLElement>(
        "[data-atlas-activity-anchor]",
      ) ?? [],
    ).find(
      (candidate) => candidate.dataset.atlasActivityAnchor === activityAnchor(target),
    );
    if (element) {
      if (!onLandingReady) element.scrollIntoView?.({ block: "start" });
      (element.querySelector<HTMLElement>("button,a[href]") ?? element).focus({
        preventScroll: true,
      });
      onLandingReady?.(element);
    }
  }, [target, targetActivity, onLandingReady]);
  const abilityReading = reading
    ? {
        ...reading,
        // Keep this visit's target stable through preview positioning renders.
        // EmbeddedAbilityReading normalizes it for its bounded target request.
        landing: targetActivity?.kind === "ability" ? target : undefined,
      }
    : undefined;
  const groups: Array<[ActorActivityView["kind"][], string]> = [
    [["strike"], "Strikes"],
    [["ability"], "Abilities"],
    [["gear"], "Equipment"],
    [["other"], "Other entries"],
  ];
  return (
    <div ref={container} className="actor-reference__mechanics">
      {target && !targetActivity && (
        <Alert
          type="info"
          title="This entry has changed"
          description="Reload the record to find its current entry."
        />
      )}
      <div className="actor-reference__actions">
        {actor.activities.state !== "value" &&
          actor.activities.state !== "not_applicable" && (
            <FactRow label="Activities">{factText(actor.activities)}</FactRow>
          )}
        {groups.map(([kinds, title]) => {
          const activities = (actor.activities.value ?? []).filter((activity) =>
            kinds.includes(activity.kind),
          );
          return activities.length ? (
            title === "Abilities" && !encounter ? (
              <EmbeddedAbilityReading
                key={title}
                abilities={activities}
                context={abilityReading}
                onReference={onReference}
              />
            ) : (
              <section className="actor-reference__activities" key={title}>
                <h3>{title}</h3>
                {activities.map((activity, index) => (
                  <ActorActivity
                    key={index}
                    activity={activity}
                    rollControls={activityRolls?.(activity)}
                    onReference={onReference}
                    interactive={activity.has_reference_details}
                  />
                ))}
              </section>
            )
          ) : null;
        })}
      </div>
      <CastingRoster
        ritualDifficultyClass={actor.ritual_difficulty_class}
        annotations={castingAnnotations}
        activities={actor.activities.value ?? []}
        onReference={onReference}
      />
    </div>
  );
}

function RuntimeCount({ count }: { count: RuntimeCountView }) {
  const explanation =
    count.explanation ||
    count.adjustments.length ||
    count.suppressed_adjustments.length ||
    count.base_value !== count.adjusted_value ||
    count.segments.some((segment) => segment.restricted);
  return (
    <FactRow label={count.label}>
      {explanation ? (
        <Popover
          title={count.label}
          content={
            <div>
              {count.explanation && <p>{count.explanation}</p>}
              <p>Baseline: {String(count.base_value)}</p>
              <p>
                Effective:{" "}
                {count.adjusted_value === null
                  ? "Needs review"
                  : String(count.adjusted_value)}
              </p>
              {count.adjustments.map((modifier, index) => (
                <p key={index}>
                  {modifier.source}: {String(modifier.value)}; {modifier.reason}
                </p>
              ))}
              {count.suppressed_adjustments.map((modifier, index) => (
                <p key={index}>
                  {modifier.source}: {modifier.reason} (suppressed)
                </p>
              ))}
              {count.segments.map((segment, index) => (
                <p key={index}>
                  {segment.label}: {String(segment.value)}
                  {segment.restricted ? " (restricted)" : ""}
                </p>
              ))}
            </div>
          }
        >
          <Button
            className="record-number"
            type="link"
            size="small"
            aria-label={`Show explanation for ${count.label}`}
          >
            {count.adjusted_value === null
              ? "Needs review"
              : String(count.adjusted_value)}
          </Button>
        </Popover>
      ) : count.adjusted_value === null ? (
        "Needs review"
      ) : (
        String(count.adjusted_value)
      )}
    </FactRow>
  );
}

function CastingRoster({
  ritualDifficultyClass,
  activities,
  annotations,
  onReference,
}: {
  ritualDifficultyClass: ActorPresentationView["ritual_difficulty_class"];
  activities: ActorActivityView[];
  annotations?: CastingAnnotations;
  onReference: RecordReferenceHandler;
}) {
  const entries = activities.filter((activity) => activity.kind === "casting_entry");
  const spells = activities.filter((activity) => activity.kind === "spell");
  const showRitualDc =
    ritualDifficultyClass.state !== "missing" &&
    ritualDifficultyClass.state !== "not_applicable";
  if (!entries.length && !spells.length && !showRitualDc) return null;
  const belongs = (spell: ActorActivityView, entry: ActorActivityView) =>
    spell.casting_entry !== null &&
    JSON.stringify(spell.casting_entry) === JSON.stringify(entry.navigation);
  const unassociated = spells.filter(
    (spell) => !entries.some((entry) => belongs(spell, entry)),
  );
  return (
    <section className="actor-reference__activities actor-casting">
      <h3>Spellcasting</h3>
      {showRitualDc && (
        <FactRow label="Ritual DC">
          <NumberFact fact={ritualDifficultyClass} />
        </FactRow>
      )}
      {entries.map((entry, index) => (
        <div key={index}>
          <ActorActivity
            activity={entry}
            onReference={onReference}
            headingActions={annotations?.entry(entry)}
          />
          <div className="actor-casting__spells">
            <CastingSpells
              annotations={annotations}
              entry={entry}
              spells={spells.filter((spell) => belongs(spell, entry))}
              onReference={onReference}
            />
          </div>
        </div>
      ))}
      {unassociated.length ? (
        <div>
          <h4>Other spells</h4>
          <CastingSpells
            annotations={annotations}
            spells={unassociated}
            onReference={onReference}
          />
        </div>
      ) : null}
    </section>
  );
}

function CastingSpells({
  annotations,
  entry,
  spells,
  onReference,
}: {
  spells: ActorActivityView[];
  entry?: ActorActivityView;
  annotations?: CastingAnnotations;
  onReference: RecordReferenceHandler;
}) {
  const groups = new Map<string, { rank: number; spells: ActorActivityView[] }>();
  for (const spell of spells) {
    const ranks: (number | undefined)[] = spell.casting_ranks.length
      ? spell.casting_ranks
      : [undefined];
    for (const rank of ranks) {
      const cantrip = spell.traits.some((trait) => trait.value === "cantrip");
      const label =
        rank === undefined
          ? "Rank unavailable"
          : cantrip
            ? `Cantrips (rank ${rank})`
            : `Rank ${rank}`;
      const group = groups.get(label) ?? {
        rank: cantrip ? 0 : (rank ?? 11),
        spells: [],
      };
      group.spells.push({
        ...spell,
        navigation: { ...spell.navigation, spell_rank: rank },
      });
      groups.set(label, group);
    }
  }
  return (
    <div className="actor-casting__rows">
      {[...groups]
        .sort((a, b) => a[1].rank - b[1].rank)
        .map(([label, group]) => (
          <div className="actor-casting__row" key={label}>
            <strong>
              {label} {entry && annotations?.rank(entry, group.rank)}
            </strong>
            <span>
              {group.spells.map((spell, index) => {
                const annotation = annotations?.spell(spell);
                return (
                  <span key={JSON.stringify(spell.navigation)}>
                    {index > 0 && ", "}
                    <Button
                      type="link"
                      className={
                        annotation?.muted ? "actor-casting__spell--expended" : undefined
                      }
                      aria-label={
                        annotation?.status
                          ? `${spell.title}: ${annotation.status}`
                          : undefined
                      }
                      size="small"
                      style={{
                        color: annotation?.muted ? "var(--muted)" : undefined,
                        paddingInline: 0,
                        height: "auto",
                        whiteSpace: "normal",
                        textAlign: "left",
                      }}
                      href={atlasRoutePath({
                        kind: "record",
                        recordKey: spell.navigation.record_key,
                        selection: spell.navigation,
                      })}
                      onClick={(event) => {
                        if (!shouldHandleAtlasRouteClick(event)) return;
                        event.preventDefault();
                        onReference(
                          spell.navigation.record_key,
                          event.currentTarget,
                          spell.navigation,
                        );
                      }}
                    >
                      {spell.title}
                    </Button>
                    {annotation?.suffix && <> {annotation.suffix}</>}
                  </span>
                );
              })}
            </span>
          </div>
        ))}
      {!spells.length && (
        <p className="muted">No spells could be matched to this entry.</p>
      )}
    </div>
  );
}
