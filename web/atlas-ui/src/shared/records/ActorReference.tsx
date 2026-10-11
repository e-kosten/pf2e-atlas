import { Button, Popover, Tag } from "antd";
import type {
  ActorActivityView,
  ActorPresentationView,
  RuntimeCountView,
  DamageComponentView,
  FactView,
  IwrEntryView,
} from "../../generated/atlas";
import { FactLine, FactRow, NumberFact, factText } from "./presentationFacts";
import type { RecordReferenceHandler } from "./PreparedContent";

function qualifierText<T extends string | string[]>(
  fact: FactView<T>,
  patch = false,
): string {
  if (
    fact.state === "not_applicable" ||
    (patch && fact.state === "missing") ||
    (fact.state === "value" && fact.value?.length === 0)
  )
    return "";
  return patch && fact.state === "null"
    ? "cleared"
    : factText(fact, (value) =>
        Array.isArray(value) ? value.join(", ") : String(value),
      );
}

export function DamageComponents({
  damage,
  patch = false,
}: {
  damage: FactView<DamageComponentView[]>;
  patch?: boolean;
}) {
  if (damage.state === "not_applicable" || (patch && damage.state === "missing"))
    return null;
  if (damage.state !== "value" || damage.value === null)
    return (
      <FactRow label="Damage">
        {patch && damage.state === "null" ? "Cleared" : factText(damage)}
      </FactRow>
    );
  if (!damage.value.length)
    return <FactRow label="Damage">No structured damage</FactRow>;
  const components = damage.value;
  return (
    <div className="record-damage">
      {components.map((component) => {
        const kinds = qualifierText(component.kinds, patch);
        const category = qualifierText(component.category, patch);
        const materials = qualifierText(component.materials, patch);
        return (
          <p key={component.id}>
            {patch || components.length > 1 ? (
              <strong>Damage {component.id}: </strong>
            ) : null}
            <strong>
              {patch && component.formula.state === "missing"
                ? ""
                : patch && component.formula.state === "null"
                  ? "Formula cleared"
                  : factText(component.formula)}
            </strong>{" "}
            {patch && component.damage_type.state === "missing"
              ? ""
              : patch && component.damage_type.state === "null"
                ? "Type cleared"
                : factText(component.damage_type)}
            {kinds
              ? component.kinds.state === "value"
                ? ` (${kinds})`
                : `; kinds ${kinds}`
              : ""}
            {category
              ? component.category.state === "value"
                ? `; ${category}`
                : `; category ${category}`
              : ""}
            {materials
              ? component.materials.state === "value"
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
            const exceptions = qualifierText(entry.exceptions);
            const doubleAgainst = qualifierText(entry.double_against);
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
  compact = false,
  hazard = false,
  hideHitPoints = false,
}: {
  actor: ActorPresentationView;
  compact?: boolean;
  hazard?: boolean;
  hideHitPoints?: boolean;
}) {
  return (
    <div className="actor-reference__facts">
      {!hazard && (
        <div className="actor-reference__awareness">
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
                    sense.acuity.value,
                    sense.range_feet.value === null
                      ? ""
                      : `${sense.range_feet.value} feet`,
                  ]
                    .filter(Boolean)
                    .join(" "),
                )
                .join(", ")
            }
          />
          <FactLine label="Perception details" fact={actor.perception_details} />
          {!compact && (
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
      {!compact && !hazard && (
        <>
          {actor.skills.state !== "value" && (
            <FactLine label="Skills" fact={actor.skills} />
          )}
          {actor.abilities.state !== "value" && (
            <FactLine label="Ability modifiers" fact={actor.abilities} />
          )}
          <div className="record-fact-list">
            {(actor.skills.value ?? []).map((skill) => (
              <div key={skill.key}>
                <FactRow label={skill.label}>
                  <NumberFact fact={skill.modifier} signed />
                  {skill.note.value ? `; ${skill.note.value}` : ""}
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
          <div className="record-fact-list">
            {(actor.abilities.value ?? []).map((ability) => (
              <FactRow key={ability.key} label={ability.label}>
                <NumberFact fact={ability.modifier} signed />
              </FactRow>
            ))}
          </div>
        </>
      )}
      <div className="actor-reference__defenses">
        {actor.armor_class.state !== "not_applicable" && (
          <FactRow label="AC">
            <NumberFact fact={actor.armor_class} />
          </FactRow>
        )}
        <div className="record-fact-list">
          {Object.entries(actor.saves).map(([save, fact]) =>
            fact.state === "not_applicable" ? null : (
              <FactRow key={save} label={save[0].toUpperCase() + save.slice(1)}>
                <NumberFact fact={fact} signed />
              </FactRow>
            ),
          )}
        </div>
        {!hideHitPoints && actor.maximum_hp.state !== "not_applicable" && (
          <FactRow label="HP">
            <NumberFact fact={actor.maximum_hp} />
          </FactRow>
        )}
        <Iwr label="Immunities" fact={actor.immunities} />
        <Iwr label="Weaknesses" fact={actor.weaknesses} />
        <Iwr label="Resistances" fact={actor.resistances} />
      </div>
      {!hazard && (
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
      )}
      {actor.runtime &&
        (actor.runtime.action_budget || actor.runtime.unapplied_effects.length > 0) && (
          <div className="actor-reference__runtime">
            <h3>Runtime</h3>
            {actor.runtime.action_budget && (
              <>
                <RuntimeCount count={actor.runtime.action_budget.actions} />
                <RuntimeCount count={actor.runtime.action_budget.reactions} />
                {actor.runtime.action_budget.notes.map((note, index) => (
                  <p key={index}>
                    {note.label}: {note.reason}
                  </p>
                ))}
              </>
            )}
            {actor.runtime.action_budget &&
              !actor.runtime.action_budget.can_act.available && (
                <FactRow label="Can act">
                  No; {actor.runtime.action_budget.can_act.reason}
                </FactRow>
              )}
            {actor.runtime.action_budget &&
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
    </div>
  );
}

export function ActorActivity({
  activity,
  onReference,
  selected = false,
}: {
  activity: ActorActivityView;
  onReference: RecordReferenceHandler;
  selected?: boolean;
}) {
  return (
    <div className="actor-activity">
      {!selected && (
        <Button
          type="link"
          onClick={(event) =>
            onReference(
              activity.navigation.record_key,
              event.currentTarget,
              activity.navigation,
            )
          }
        >
          {activity.title}
        </Button>
      )}
      {activity.usage && <Tag>{activity.usage}</Tag>}
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
      <FactLine label="Casting association" fact={activity.association} />
      {activity.casting_entry && (
        <Button
          type="link"
          onClick={(event) =>
            onReference(
              activity.casting_entry!.record_key,
              event.currentTarget,
              activity.casting_entry!,
            )
          }
        >
          Casting entry
        </Button>
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
}: {
  actor: ActorPresentationView;
  onReference: RecordReferenceHandler;
}) {
  const groups: Array<[ActorActivityView["kind"][], string]> = [
    [["strike"], "Strikes"],
    [["ability"], "Abilities"],
    [["lore"], "Lore"],
    [["gear"], "Equipment"],
    [["other"], "Other owned records"],
  ];
  return (
    <>
      {actor.activities.state !== "value" &&
        actor.activities.state !== "not_applicable" && (
          <FactRow label="Owned activities">{factText(actor.activities)}</FactRow>
        )}
      {groups.map(([kinds, title]) => {
        const activities = (actor.activities.value ?? []).filter((activity) =>
          kinds.includes(activity.kind),
        );
        return activities.length ? (
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
        ) : null;
      })}
      <CastingRoster
        activities={actor.activities.value ?? []}
        onReference={onReference}
      />
    </>
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
              <p>Authored: {String(count.base_value)}</p>
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
    <section className="actor-reference__activities">
      <h3>Spellcasting</h3>
      {entries.map((entry, index) => (
        <div key={index}>
          <ActorActivity activity={entry} onReference={onReference} />
          <div className="actor-casting__spells">
            {spells
              .filter((spell) => belongs(spell, entry))
              .map((spell, index) => (
                <ActorActivity key={index} activity={spell} onReference={onReference} />
              ))}
          </div>
        </div>
      ))}
      {unassociated.length ? (
        <div>
          <h4>Other authored spells</h4>
          {unassociated.map((spell, index) => (
            <ActorActivity key={index} activity={spell} onReference={onReference} />
          ))}
        </div>
      ) : null}
    </section>
  );
}
