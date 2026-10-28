import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { ValueExplanation } from "../ui/ValueExplanation";
import { RuntimeCountCalculation } from "./RuntimeCountCalculation";
import { statusReasonLabel } from "./statusReasonLabel";
import { TraitBadge } from "./TraitBadge";
import { Alert, Button, Collapse, Popover, Typography } from "antd";
import { InfoCircleOutlined } from "@ant-design/icons";
import { useEffect, useId, useRef, type ReactNode } from "react";
import type {
  ActorActivityView,
  ActorAbilityGroupView,
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
  renderComponent,
}: {
  damage: FactView<DamageComponentView[]>;
  patch?: boolean;
  renderComponent?: (component: DamageComponentView, value: ReactNode) => ReactNode;
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
      {components.map((component) => {
        const kinds =
          !patch &&
          component.kinds.state === "value" &&
          component.kinds.value?.length === 1 &&
          component.kinds.value[0] === "damage"
            ? ""
            : qualifierText(component.kinds, patch);
        const category =
          !patch && ["missing", "null"].includes(component.category.state)
            ? ""
            : qualifierText(component.category, patch);
        const materials =
          !patch && ["missing", "null"].includes(component.materials.state)
            ? ""
            : qualifierText(component.materials, patch);
        const value = (
          <>
            {patch ? <strong>Damage : </strong> : null}
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
          </>
        );
        return (
          <p key={component.id}>
            {renderComponent ? renderComponent(component, value) : value}
          </p>
        );
      })}
    </div>
  );
}
export function Iwr({
  label,
  fact,
}: {
  label: string;
  fact: FactView<IwrEntryView[]>;
}) {
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
export function ActorDefenses({
  actor,
  hideHitPoints = false,
  hidePrimaryDefenses = false,
  hideSaves = false,
}: {
  actor: ActorPresentationView;
  hideHitPoints?: boolean;
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
  return (
    <>
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
    </>
  );
}
export function ActorAwareness({
  actor,
  encounter = false,
}: {
  actor: ActorPresentationView;
  encounter?: boolean;
}) {
  if (
    ![actor.perception, actor.senses, actor.perception_details].some((fact) =>
      hasReadingFact(fact as FactView<unknown>),
    )
  )
    return null;
  return (
    <div className="actor-reference__awareness">
      {actor.perception.state !== "not_applicable" && (
        <FactRow label="Perception">
          <NumberFact fact={actor.perception} signed label="Perception" />
        </FactRow>
      )}
      {encounter && actor.runtime ? (
        <div className="record-fact-list">
          {actor.runtime.senses.map((sense, index) => (
            <FactRow
              label={sentenceCase(sense.sense_type.replace(/-/g, " "))}
              key={index}
            >
              {[
                sense.precision,
                sense.range_feet === null ? null : `${sense.range_feet} feet`,
                sense.available === false
                  ? `Unavailable${sense.reason ? ` (${sense.reason})` : ""}`
                  : null,
                sense.perception === null || sense.perception === actor.perception.value
                  ? null
                  : `Perception ${sense.perception >= 0 ? "+" : ""}${sense.perception}`,
              ]
                .filter(Boolean)
                .join(" · ")}
            </FactRow>
          ))}
        </div>
      ) : (
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
      )}
      <FactLine label="Perception details" fact={actor.perception_details} />
    </div>
  );
}
export function ActorMovement({ actor }: { actor: ActorPresentationView }) {
  if (
    ![actor.land_speed, actor.movement, actor.movement_details].some((fact) =>
      hasReadingFact(fact as FactView<unknown>),
    )
  )
    return null;
  return (
    <section className="actor-reference__movement">
      <div className="record-fact-list">
        {actor.land_speed.state !== "not_applicable" && (
          <FactRow label="Speed">
            <NumberFact fact={actor.land_speed} unit=" feet" label="Speed" />
          </FactRow>
        )}
        {actor.movement.state !== "value" && (
          <FactLine label="Other movement" fact={actor.movement} />
        )}
        {!["missing", "null", "not_applicable"].includes(
          actor.movement_details.state,
        ) && <FactLine label="Movement details" fact={actor.movement_details} />}
        {(actor.movement.value ?? []).map((movement, index) => (
          <FactRow key={index} label={factText(movement.movement_type)}>
            <NumberFact fact={movement.feet} unit=" feet" />
          </FactRow>
        ))}
      </div>
    </section>
  );
}
export function ActorLanguages({ actor }: { actor: ActorPresentationView }) {
  if (!hasReadingFact(actor.languages) && !hasReadingFact(actor.language_details))
    return null;
  return (
    <div className="actor-reference__languages">
      <FactLine
        label="Languages"
        fact={actor.languages}
        format={(values) => values.join(", ")}
      />
      <FactLine label="Communication" fact={actor.language_details} />
    </div>
  );
}
export function ActorSupportingFacts({
  actor,
  encounter = false,
  hazard = false,
}: {
  actor: ActorPresentationView;
  encounter?: boolean;
  hazard?: boolean;
}) {
  if (hazard) return null;
  return (
    <>
      <div className="actor-reference__awareness-movement">
        <ActorAwareness actor={actor} encounter={encounter} />
        <ActorMovement actor={actor} />
      </div>
      {!encounter && (
        <>
          <ActorLanguages actor={actor} />
          <ActorSkills actor={actor} />
        </>
      )}
    </>
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
  return (
    <>
      <DefenseNote label="Size" fact={actor.size} />
      <ActorDefenses
        actor={actor}
        hideHitPoints={hideHitPoints}
        hidePrimaryDefenses={hidePrimaryDefenses}
        hideSaves={hideSaves}
      />
      <div
        className={`actor-reference__facts${encounter ? " actor-reference__facts--combat" : ""}`}
      >
        <ActorSupportingFacts actor={actor} encounter={encounter} hazard={hazard} />
        {encounter ? (
          <ActorCombatBudget actor={actor} />
        ) : (
          <ActorRuntime actor={actor} />
        )}
      </div>
    </>
  );
}
export function ActorSkills({ actor }: { actor: ActorPresentationView }) {
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
                    ? ` · ${factText(skill.note)}`
                    : ""}
                  {!["missing", "value", "not_applicable"].includes(
                    skill.conditional.state,
                  )
                    ? ` · conditional modifiers ${factText(skill.conditional)}`
                    : ""}
                  {skill.conditional.value?.map((conditional, index) => (
                    <span key={index}>
                      {" · "}
                      <NumberFact fact={conditional.modifier} signed />{" "}
                      {conditional.label.state === "value" &&
                      !conditional.label.value?.trim()
                        ? "Unspecified condition"
                        : factText(conditional.label)}
                      {!["missing", "not_applicable"].includes(
                        conditional.qualification.state,
                      ) &&
                        hasReadingFact(conditional.qualification) && (
                          <Popover
                            title="Condition details"
                            content={factText(conditional.qualification)}
                            trigger="click"
                          >
                            <Button
                              type="text"
                              size="small"
                              icon={<InfoCircleOutlined />}
                              aria-label={`${sentenceCase(skill.label)}: condition details`}
                            />
                          </Popover>
                        )}
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
function castingHint(actor: ActorPresentationView) {
  if (
    !actor.activities.value?.some(
      (activity) => activity.kind === "spell" || activity.kind === "casting_entry",
    )
  )
    return undefined;
  return actor.runtime?.unapplied_effects.find(
    (effect) => effect.source === "condition:stupefied",
  );
}
function hasRuntimeDetails(actor: ActorPresentationView, omitBudgetNotes = false) {
  return (
    (!omitBudgetNotes && (actor.runtime?.action_budget?.notes.length ?? 0) > 0) ||
    (actor.runtime?.unapplied_effects.some(
      (effect) => effect.condition_id === undefined && effect !== castingHint(actor),
    ) ??
      false)
  );
}
function ActorRuntime({
  actor,
  detailsOnly = false,
  omitBudgetNotes = false,
}: {
  actor: ActorPresentationView;
  detailsOnly?: boolean;
  omitBudgetNotes?: boolean;
}) {
  return (
    <>
      {actor.runtime &&
        (detailsOnly
          ? hasRuntimeDetails(actor, omitBudgetNotes)
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
                {!omitBudgetNotes &&
                  actor.runtime.action_budget.notes.map((note, index) => (
                    <p key={index}>
                      {note.label}: {note.reason}
                    </p>
                  ))}
              </>
            )}
            {!detailsOnly &&
              actor.runtime.action_budget &&
              !actor.runtime.action_budget.can_act.available &&
              actor.runtime.action_budget.can_act.source !== "participant_status" && (
                <FactRow label="Can act">
                  {statusReasonLabel("No", actor.runtime.action_budget.can_act.reason)}
                </FactRow>
              )}
            {!detailsOnly &&
              actor.runtime.action_budget &&
              !actor.runtime.action_budget.can_react.available &&
              actor.runtime.action_budget.can_react.source !== "participant_status" && (
                <FactRow label="Can react">
                  {statusReasonLabel(
                    "No",
                    actor.runtime.action_budget.can_react.reason,
                  )}
                </FactRow>
              )}
            {actor.runtime.unapplied_effects
              .filter(
                (effect) =>
                  (!detailsOnly || effect.condition_id === undefined) &&
                  effect !== castingHint(actor),
              )
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
export function ActorCombatBudget({ actor }: { actor: ActorPresentationView }) {
  const budget = actor.runtime?.action_budget;
  if (!budget) return null;
  const cannotAct =
    !budget.can_act.available && budget.can_act.source !== "participant_status";
  const cannotReact =
    !budget.can_react.available && budget.can_react.source !== "participant_status";
  const sharedRestriction =
    cannotAct &&
    cannotReact &&
    budget.can_act.source === budget.can_react.source &&
    budget.can_act.reason === budget.can_react.reason;
  return (
    <section className="actor-reference__combat-budget" aria-label="Turn">
      <h3>Turn</h3>
      {sharedRestriction ? (
        <p className="actor-reference__restriction">
          {statusReasonLabel("Cannot act or react", budget.can_act.reason)}
        </p>
      ) : (
        <>
          {cannotAct && (
            <p className="actor-reference__restriction">
              {statusReasonLabel("Cannot act", budget.can_act.reason)}
            </p>
          )}
          {cannotReact && (
            <p className="actor-reference__restriction">
              {statusReasonLabel("Cannot react", budget.can_react.reason)}
            </p>
          )}
        </>
      )}
      <RuntimeCount
        count={budget.actions}
        label="At turn start"
        unit={Number(budget.actions.adjusted_value) === 1 ? " action" : " actions"}
      />
      {(!cannotReact ||
        budget.reactions.adjusted_value === null ||
        Number(budget.reactions.adjusted_value) !== 0) && (
        <RuntimeCount count={budget.reactions} label="Reactions now" />
      )}
      {budget.extra_action_unavailable_reason ? (
        <FactRow label="Extra action">
          <span className="muted">{budget.extra_action_unavailable_reason}</span>
        </FactRow>
      ) : (
        budget.quickened_sources.map((source, index) => (
          <FactRow label="Extra action" key={index}>
            {source.kind === "unspecified"
              ? "Unspecified"
              : source.kind === "unrestricted"
                ? "Unrestricted"
                : [
                    source.actions
                      .map((action) => sentenceCase(action.replace(/_/g, " ")))
                      .join(", "),
                    source.guidance,
                  ]
                    .filter(Boolean)
                    .join(" · ")}
          </FactRow>
        ))
      )}
      {budget.notes.map((note, index) => (
        <p key={index}>
          {note.label}: {note.reason}
        </p>
      ))}
    </section>
  );
}
export function ActorSecondaryFacts({
  actor,
  runtimeOnly = false,
  hazard = false,
}: {
  runtimeOnly?: boolean;
  actor: ActorPresentationView;
  hazard?: boolean;
}) {
  if (runtimeOnly) return <ActorRuntime actor={actor} detailsOnly omitBudgetNotes />;
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
  activationControls,
  hideFrequency = false,
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
  activationControls?: React.ReactNode;
  hideFrequency?: boolean;
  rollControls?: React.ReactNode;
  hideCost?: boolean;
  inline?: boolean;
  permalink?: boolean;
  interactive?: boolean;
}) {
  const id = useId();
  const route = recordDestination({
    recordKey: activity.navigation.record_key,
    selection: activity.navigation,
  });
  return (
    <div
      className={`actor-activity actor-activity--${activity.kind}${selected ? " actor-activity--selected" : ""}`}
      data-atlas-activity-anchor={
        !inline ? activityAnchor(activity.navigation) : undefined
      }
    >
      {(!selected ||
        headingActions ||
        (activity.kind === "strike" && activity.is_ranged.state === "value") ||
        (activity.usage && !isEmptyFactValue(activity.usage))) && (
        <div className="actor-activity__identity">
          <div className="actor-activity__name">
            {inline && (
              <h4
                id={`${id}-${activityAnchor(activity.navigation)}`}
                data-atlas-activity-anchor={activityAnchor(activity.navigation)}
                tabIndex={-1}
                className="actor-activity__title"
              >
                {permalink ? (
                  <Typography.Link
                    className="actor-activity__heading-link"
                    href={destinationHref(route)}
                    onClick={(event) => {
                      if (!isPrimaryActivation(event)) return;
                      event.preventDefault();
                      onReference(
                        activity.navigation.record_key,
                        event.currentTarget,
                        activity.navigation,
                      );
                    }}
                  >
                    {activity.title}
                  </Typography.Link>
                ) : (
                  activity.title
                )}{" "}
                {!activationControls && activity.action_cost.state === "value" && (
                  <ActionCost fact={activity.action_cost} />
                )}
              </h4>
            )}
            {!selected &&
              !inline &&
              (interactive ? (
                <Button
                  type="link"
                  style={{ paddingInline: 0, height: "auto", fontWeight: 700 }}
                  href={destinationHref(route)}
                  onClick={(event) => {
                    if (!isPrimaryActivation(event)) return;
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
            {inline && activationControls && (
              <div className="actor-activity__activation">
                {activity.action_cost.state === "value" && (
                  <ActionCost fact={activity.action_cost} />
                )}
                {activationControls}
              </div>
            )}
            {headingActions}
            {!inline && activationControls}
            {activity.kind === "strike" && activity.is_ranged.state === "value" && (
              <span className="actor-activity__usage">
                {activity.is_ranged.value ? "Ranged" : "Melee"}
              </span>
            )}
            {activity.usage && !isEmptyFactValue(activity.usage) && (
              <span className="actor-activity__usage">{activity.usage}</span>
            )}
          </div>
          {!selected && (
            <div className="badge-row">
              {activity.traits.map((trait) => (
                <TraitBadge key={`${trait.kind}-${trait.value}`} trait={trait} />
              ))}
            </div>
          )}
        </div>
      )}
      <div className="actor-activity__facts">
        {activity.first_level_only?.value === true && (
          <FactRow label="Selection">At 1st level only</FactRow>
        )}
        {activity.first_level_only?.state === "invalid" && (
          <FactRow label="Selection">Unavailable</FactRow>
        )}
        {activity.strike_range.value ? (
          <>
            {activity.strike_range.value.increment.state !== "not_applicable" && (
              <FactRow label="Range increment">
                <NumberFact fact={activity.strike_range.value.increment} unit=" feet" />
              </FactRow>
            )}
            {activity.strike_range.value.maximum.state !== "not_applicable" && (
              <FactRow label="Maximum range">
                <NumberFact fact={activity.strike_range.value.maximum} unit=" feet" />
              </FactRow>
            )}
          </>
        ) : activity.strike_range.state === "invalid" ? (
          <FactRow label="Range">{factText(activity.strike_range)}</FactRow>
        ) : null}
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
        {!hideFrequency &&
          activity.frequency.state !== "missing" &&
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
          <ActionCost fact={activity.action_cost} label="Activation" />
        )}
        {rollControls ?? (
          <>
            {activity.attack.state !== "not_applicable" && (
              <FactRow label="Attack">
                <NumberFact fact={activity.attack} signed />
                {activity.attack_options
                  .filter((option) => option.attack_number > 1)
                  .map((option) => (
                    <span key={option.attack_number}>
                      {" "}
                      / {option.modifier >= 0 ? "+" : ""}
                      {option.modifier}
                    </span>
                  ))}
              </FactRow>
            )}
            <DamageComponents damage={activity.damage} />
          </>
        )}
        {hasReadingFact(activity.strike_effects) && (
          <FactRow label="Attack effects">
            {activity.strike_effects.value
              ? activity.strike_effects.value.map((effect, index) => (
                  <span key={index}>
                    {index ? ", " : ""}
                    {effect.navigation ? (
                      <Typography.Link
                        href={destinationHref(
                          recordDestination({
                            recordKey: effect.navigation.record_key,
                            selection: effect.navigation,
                          }),
                        )}
                        onClick={(event) => {
                          if (!isPrimaryActivation(event)) return;
                          event.preventDefault();
                          onReference(
                            effect.navigation!.record_key,
                            event.currentTarget,
                            effect.navigation!,
                          );
                        }}
                      >
                        {factText(effect.name)}
                      </Typography.Link>
                    ) : (
                      factText(effect.name)
                    )}
                  </span>
                ))
              : factText(activity.strike_effects)}
          </FactRow>
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
                    {!activity.self_effect.value.navigation &&
                      " (reference unavailable)"}
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
        {activity.casting_context &&
          hasReadingFact(activity.casting_context.spells) && (
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
  essential = false,
  sectionIds,
  hazardContent,
}: {
  hazardContent?: {
    routine: React.ReactNode;
    reset: React.ReactNode;
  };
  essential?: boolean;
  sectionIds?: Partial<
    Record<
      | "strike"
      | "ability"
      | "reaction"
      | "passive"
      | "unclassified"
      | "gear"
      | "other"
      | "casting",
      string
    >
  >;
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
  const renderGroup = (
    kind: "strike" | "ability" | "gear" | "other",
    title: string,
    group?: ActorAbilityGroupView,
  ) => {
    const activities = (actor.activities.value ?? [])
      .filter(
        (activity) =>
          activity.kind === kind && (!group || activity.ability_group === group),
      )
      .sort((a, b) => a.reading_order - b.reading_order);
    if (!activities.length) return null;
    return (
      <section
        className={`actor-reference__activities actor-reference__activities--${kind}`}
        key={group ?? kind}
        id={
          sectionIds?.[
            group === "reaction"
              ? "reaction"
              : group === "supporting"
                ? "passive"
                : group === "unclassified"
                  ? "unclassified"
                  : kind
          ]
        }
        tabIndex={-1}
      >
        {kind === "ability" && (!encounter || reading) ? (
          <EmbeddedAbilityReading
            abilities={activities}
            essential={essential}
            group={group}
            title={title}
            context={
              abilityReading
                ? {
                    ...abilityReading,
                    landing:
                      targetActivity?.ability_group === group
                        ? abilityReading.landing
                        : undefined,
                    onLandingReady:
                      targetActivity?.ability_group === group
                        ? abilityReading.onLandingReady
                        : undefined,
                  }
                : undefined
            }
            onReference={onReference}
          />
        ) : (
          <>
            <h3>{title}</h3>
            {activities.map((activity) => (
              <ActorActivity
                key={activityAnchor(activity.navigation)}
                activity={activity}
                rollControls={activityRolls?.(activity)}
                onReference={onReference}
                interactive={activity.has_reference_details}
              />
            ))}
          </>
        )}
      </section>
    );
  };
  const hasPrimary = actor.activities.value?.some(
    (activity) => activity.ability_group === "primary",
  );
  const hasCasting =
    actor.activities.value?.some(
      (activity) => activity.kind === "spell" || activity.kind === "casting_entry",
    ) || !["missing", "not_applicable"].includes(actor.ritual_difficulty_class.state);
  return (
    <div ref={container} className="actor-reference__mechanics">
      {target && !targetActivity && (
        <Alert
          type="info"
          title="This entry has changed"
          description="Reload the record to find its current entry."
        />
      )}
      {actor.activities.state !== "value" &&
        actor.activities.state !== "not_applicable" && (
          <FactRow label="Activities">{factText(actor.activities)}</FactRow>
        )}
      {hazardContent && (
        <>
          {renderGroup("ability", "Initial activation", "activation")}
          {hazardContent.routine}
        </>
      )}
      {renderGroup("strike", "Strikes")}
      {(hasCasting || hasPrimary) && (
        <div
          className={`actor-reference__rule-grid${hasCasting && hasPrimary ? " actor-reference__rule-grid--caster" : ""}`}
        >
          {hasPrimary && (
            <div className="actor-reference__actions">
              {renderGroup("ability", "Special abilities", "primary")}
            </div>
          )}
          {hasCasting && (
            <div
              className="actor-reference__casting"
              id={sectionIds?.casting}
              tabIndex={-1}
            >
              <CastingRoster
                hint={castingHint(actor)?.reason}
                ritualDifficultyClass={actor.ritual_difficulty_class}
                annotations={castingAnnotations}
                activities={actor.activities.value ?? []}
                onReference={onReference}
              />
            </div>
          )}
        </div>
      )}
      {!hazardContent && renderGroup("ability", "Reactions", "reaction")}
      {renderGroup("ability", "Passive abilities", "supporting")}
      {renderGroup("ability", "Other abilities", "unclassified")}
      {hazardContent?.reset}
      {renderGroup("gear", "Equipment")}
      {renderGroup("other", "Other entries")}
    </div>
  );
}
export function RuntimeCount({
  count,
  label = count.label,
  unit = "",
}: {
  count: RuntimeCountView;
  label?: string;
  unit?: string;
}) {
  const explanation =
    count.explanation ||
    count.adjustments.length ||
    count.suppressed_adjustments.length ||
    count.base_value !== count.adjusted_value ||
    count.segments.some((segment) => segment.restricted);
  return (
    <FactRow label={label}>
      {explanation ? (
        <ValueExplanation
          title={count.label}
          label={`Show explanation for ${count.label}`}
          content={<RuntimeCountCalculation count={count} />}
        >
          {count.adjusted_value === null
            ? "Needs review"
            : `${count.adjusted_value}${unit}`}
        </ValueExplanation>
      ) : count.adjusted_value === null ? (
        "Needs review"
      ) : (
        `${count.adjusted_value}${unit}`
      )}
    </FactRow>
  );
}
function CastingRoster({
  hint,
  ritualDifficultyClass,
  activities,
  annotations,
  onReference,
}: {
  hint?: string;
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
      {hint && <p>{hint}</p>}
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
  const groups = new Map<
    string,
    {
      rank: number;
      spells: ActorActivityView[];
    }
  >();
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
                      href={destinationHref(
                        recordDestination({
                          recordKey: spell.navigation.record_key,
                          selection: spell.navigation,
                        }),
                      )}
                      onClick={(event) => {
                        if (!isPrimaryActivation(event)) return;
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
