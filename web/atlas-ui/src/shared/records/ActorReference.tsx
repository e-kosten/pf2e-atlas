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
import { OwnedAbilityReading, type OwnedReadingContext } from "./OwnedAbilityReading";
import { ActionCost } from "./ActionCost";
import { activityAnchor } from "./ownedNavigation";
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
    <div
      className={`actor-reference__facts${encounter ? " actor-reference__facts--combat" : ""}`}
    >
      {defenseFacts.some((fact) => hasReadingFact(fact)) && (
        <section className="actor-reference__defenses">
          <h3>Defenses</h3>
          {!hidePrimaryDefenses && actor.armor_class.state !== "not_applicable" && (
            <FactRow label="AC">
              <NumberFact fact={actor.armor_class} />
            </FactRow>
          )}
          {!hideSaves && (
            <div className="record-fact-list">
              {Object.entries(actor.saves).map(([save, fact]) =>
                fact.state === "not_applicable" ? null : (
                  <FactRow key={save} label={save[0].toUpperCase() + save.slice(1)}>
                    <NumberFact fact={fact} signed />
                  </FactRow>
                ),
              )}
            </div>
          )}
          {!hidePrimaryDefenses &&
            !hideHitPoints &&
            actor.maximum_hp.state !== "not_applicable" && (
              <FactRow label="HP">
                <NumberFact fact={actor.maximum_hp} />
              </FactRow>
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
      {encounter ? <ActorCombatBudget actor={actor} /> : <ActorRuntime actor={actor} />}
    </div>
  );
}

function ActorSkills({ actor }: { actor: ActorPresentationView }) {
  if (!hasReadingFact(actor.abilities) && !hasReadingFact(actor.skills)) return null;
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
      {hasReadingFact(actor.skills) && (
        <section>
          <h3>Skills</h3>
          {actor.skills.state !== "value" && (
            <FactLine label="Skills" fact={actor.skills} />
          )}
          <div className="record-fact-list">
            {(actor.skills.value ?? []).map((skill) => (
              <div key={skill.key}>
                <FactRow label={skill.label}>
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
          </div>
        </section>
      )}
    </div>
  );
}
function hasRuntimeDetails(actor: ActorPresentationView) {
  return (
    (actor.runtime?.action_budget?.notes.length ?? 0) > 0 ||
    (actor.runtime?.unapplied_effects.length ?? 0) > 0
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
            {actor.runtime.unapplied_effects.map((effect, index) => (
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
      ![
        actor.perception_details,
        actor.languages,
        actor.language_details,
        actor.abilities,
        actor.skills,
      ].some((fact) => hasReadingFact(fact as FactView<unknown>)))
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

export function ActorActivity({
  activity,
  onReference,
  selected = false,
  hideCost = false,
  inline = false,
  permalink = true,
}: {
  activity: ActorActivityView;
  onReference: RecordReferenceHandler;
  selected?: boolean;
  hideCost?: boolean;
  inline?: boolean;
  permalink?: boolean;
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
          {activity.title} <ActionCost fact={activity.action_cost} />
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
      {!selected && !inline && (
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
      )}
      {!hideCost && !inline && <ActionCost fact={activity.action_cost} />}
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
      {activity.attack.state !== "not_applicable" && (
        <FactRow label="Attack">
          <NumberFact fact={activity.attack} signed />
        </FactRow>
      )}
      <DamageComponents damage={activity.damage} />
      <FactLine label="Tradition" fact={activity.casting_tradition} />
      <FactLine label="Preparation" fact={activity.preparation} />
      {activity.difficulty_class.state !== "not_applicable" && (
        <FactRow label="DC">
          <NumberFact fact={activity.difficulty_class} />
        </FactRow>
      )}
      {activity.lore_modifier.state !== "not_applicable" && (
        <FactRow label="Lore">
          <NumberFact fact={activity.lore_modifier} signed />
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
}: {
  actor: ActorPresentationView;
  onReference: RecordReferenceHandler;
  encounter?: boolean;
  reading?: OwnedReadingContext;
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
        // OwnedAbilityReading normalizes it for its bounded target request.
        landing: targetActivity?.kind === "ability" ? target : undefined,
      }
    : undefined;
  const groups: Array<[ActorActivityView["kind"][], string]> = [
    [["strike"], "Strikes"],
    [["ability"], "Abilities"],
    [["lore"], "Lore"],
    [["gear"], "Equipment"],
    [["other"], "Other owned records"],
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
            <FactRow label="Owned activities">{factText(actor.activities)}</FactRow>
          )}
        {groups.map(([kinds, title]) => {
          const activities = (actor.activities.value ?? []).filter((activity) =>
            kinds.includes(activity.kind),
          );
          return activities.length ? (
            title === "Abilities" && !encounter ? (
              <OwnedAbilityReading
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
                    onReference={onReference}
                  />
                ))}
              </section>
            )
          ) : null;
        })}
      </div>
      <CastingRoster
        activities={actor.activities.value ?? []}
        onReference={onReference}
      />
    </div>
  );
}

function RuntimeCount({ count }: { count: RuntimeCountView }) {
  const explanation =
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
              <p>Baseline: {String(count.base_value)}</p>
              <p>Effective: {String(count.adjusted_value)}</p>
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
            {String(count.adjusted_value)}
          </Button>
        </Popover>
      ) : (
        String(count.adjusted_value)
      )}
    </FactRow>
  );
}

function CastingRoster({
  activities,
  onReference,
}: {
  activities: ActorActivityView[];
  onReference: RecordReferenceHandler;
}) {
  const entries = activities.filter((activity) => activity.kind === "casting_entry");
  const spells = activities.filter((activity) => activity.kind === "spell");
  if (!entries.length && !spells.length) return null;
  const belongs = (spell: ActorActivityView, entry: ActorActivityView) =>
    spell.casting_entry !== null &&
    JSON.stringify(spell.casting_entry) === JSON.stringify(entry.navigation);
  const unassociated = spells.filter(
    (spell) => !entries.some((entry) => belongs(spell, entry)),
  );
  return (
    <section className="actor-reference__activities actor-casting">
      <h3>Spellcasting</h3>
      {entries.map((entry, index) => (
        <div key={index}>
          <ActorActivity activity={entry} onReference={onReference} />
          <div className="actor-casting__spells">
            <CastingSpells
              spells={spells.filter((spell) => belongs(spell, entry))}
              onReference={onReference}
            />
          </div>
        </div>
      ))}
      {unassociated.length ? (
        <div>
          <h4>Other spells</h4>
          <CastingSpells spells={unassociated} onReference={onReference} />
        </div>
      ) : null}
    </section>
  );
}

function CastingSpells({
  spells,
  onReference,
}: {
  spells: ActorActivityView[];
  onReference: RecordReferenceHandler;
}) {
  return (
    <div className="actor-casting__rows">
      {spells.length ? (
        spells.map((spell) => (
          <div
            className="actor-casting__row"
            key={JSON.stringify(spell.navigation)}
            data-atlas-activity-anchor={activityAnchor(spell.navigation)}
          >
            <Button
              type="link"
              size="small"
              style={{
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
            <span className="muted">Base rank {factText(spell.base_rank)}</span>
            {spell.authored_cast_rank.state !== "missing" &&
              spell.authored_cast_rank.state !== "not_applicable" && (
                <span>Cast rank {factText(spell.authored_cast_rank)}</span>
              )}
            {spell.casting_entry === null && (
              <span className="muted">Casting entry unavailable</span>
            )}
          </div>
        ))
      ) : (
        <p className="muted">No spells could be matched to this entry.</p>
      )}
    </div>
  );
}
