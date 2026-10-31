import type { ReactNode } from "react";
import { Collapse } from "antd";
import type {
  FactView,
  HeighteningDamageView,
  SpellAreaView,
  SpellChangeView,
  SpellDefenseView,
  SpellFixedLevelView,
  SpellHeighteningChangeView,
  SpellHeighteningView,
  SpellPresentationView,
  SpellRankFactView,
  SpellRankChangeView,
} from "../../generated/atlas";
import { SpellRankValue } from "./SpellRankValue";
import { DamageComponents } from "./ActorReference";
import { ActionCost } from "./ActionCost";
import {
  FactLine,
  FactRow,
  NumberFact,
  factText,
  hasReadingFact,
  type FactValueRenderer,
} from "./presentationFacts";

function rankValue(
  changes: SpellRankChangeView[] | undefined,
  kind: SpellRankFactView["kind"],
  label: string,
  componentId?: string,
): FactValueRenderer | undefined {
  const change = changes?.find(
    (change) =>
      change.fact.kind === kind &&
      (change.fact.kind !== "damage" || change.fact.component_id === componentId),
  );
  return change
    ? (value) => (
        <SpellRankValue change={change} label={label}>
          {value}
        </SpellRankValue>
      )
    : undefined;
}

function Area({
  fact,
  patch = false,
  renderValue,
}: {
  fact: FactView<SpellAreaView>;
  patch?: boolean;
  renderValue?: FactValueRenderer;
}) {
  if (fact.state !== "value" || !fact.value)
    return (
      <FactLine label="Area" fact={fact} patch={patch} renderValue={renderValue} />
    );
  if (patch)
    return (
      <>
        <FactLine
          label="Area size"
          fact={fact.value.size}
          patch
          renderValue={renderValue}
        />
        <FactLine label="Area shape" fact={fact.value.shape} patch />
        <FactLine label="Area details" fact={fact.value.details} patch />
      </>
    );
  const value = [
    factText(fact.value.size, String, { required: true }),
    factText(fact.value.shape, String, { required: true }),
    fact.value.details.state === "missing" || !hasReadingFact(fact.value.details)
      ? ""
      : factText(fact.value.details),
  ]
    .filter(Boolean)
    .join(" ");
  return <FactRow label="Area">{renderValue ? renderValue(value) : value}</FactRow>;
}
function Defense({
  fact,
  patch = false,
  renderValue,
}: {
  fact: FactView<SpellDefenseView>;
  patch?: boolean;
  renderValue?: FactValueRenderer;
}) {
  if (fact.state !== "value" || !fact.value)
    return (
      <FactLine label="Defense" fact={fact} patch={patch} renderValue={renderValue} />
    );
  if (patch)
    return (
      <>
        <FactLine
          label="Save statistic"
          fact={fact.value.statistic}
          patch
          renderValue={renderValue}
        />
        <FactLine
          label="Basic save"
          fact={fact.value.basic}
          patch
          format={(basic) => (basic ? "Yes" : "No")}
        />
        <FactLine label="Passive defense" fact={fact.value.passive} patch />
      </>
    );
  const value = (
    <>
      {fact.value.basic.value ? "Basic " : ""}
      {factText(fact.value.statistic)}
      {["invalid", "null"].includes(fact.value.basic.state)
        ? `; basic save ${factText(fact.value.basic)}`
        : ""}
      {fact.value.passive.state === "value" && hasReadingFact(fact.value.passive)
        ? `; ${fact.value.passive.value}`
        : ""}
      {["invalid", "null"].includes(fact.value.passive.state)
        ? `; passive defense ${factText(fact.value.passive)}`
        : ""}
    </>
  );
  return <FactRow label="Defense">{renderValue ? renderValue(value) : value}</FactRow>;
}
function HeighteningDamage({
  fact,
  patch = false,
}: {
  fact: FactView<HeighteningDamageView[]>;
  patch?: boolean;
}) {
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Heightening damage" fact={fact} patch={patch} />;
  if (patch && !fact.value.length)
    return <FactLine label="Heightening damage" fact={fact} patch />;
  const components = fact.value.filter((component) =>
    hasReadingFact(component.formula, { patch }),
  );
  return (
    <>
      {components.map((component, index) => (
        <FactLine
          key={component.id}
          label={
            components.length === 1 ? "Damage increase" : `Damage increase ${index + 1}`
          }
          fact={component.formula}
          patch={patch}
        />
      ))}
    </>
  );
}
function FixedLevels({
  fact,
  patch = false,
  selectedRank,
}: {
  fact: FactView<SpellFixedLevelView[]>;
  patch?: boolean;
  selectedRank?: number;
}) {
  if (!hasReadingFact(fact, { patch })) return null;
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Fixed ranks" fact={fact} patch={patch} />;
  if (!fact.value.length)
    return <FactLine label="Fixed ranks" fact={fact} patch={patch} />;
  return (
    <>
      {fact.value.filter(hasLevelChanges).map((level) => (
        <section
          key={level.rank}
          className="spell-reference__rank-change"
          aria-current={level.rank === selectedRank ? "true" : undefined}
        >
          <h4>Rank {level.rank}</h4>
          <div>
            {level.changes.value ? (
              <SpellChanges changes={level.changes.value} />
            ) : (
              <FactLine label="Other changes" fact={level.changes} patch />
            )}
          </div>
        </section>
      ))}
    </>
  );
}

function hasLevelChanges(level: SpellFixedLevelView) {
  if (level.changes.state !== "value" || !level.changes.value) return true;
  // Omit an empty sparse patch, but retain explicit clears, empty values and errors.
  return Object.values(level.changes.value).some(
    (fact) => fact.state !== "missing" && fact.state !== "not_applicable",
  );
}
function HeighteningChanges({ fact }: { fact: FactView<SpellHeighteningChangeView> }) {
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Heightening" fact={fact} patch />;
  const value = fact.value;
  return (
    <div className="spell-reference__change">
      <FactLine label="Heightening kind" fact={value.kind} patch />
      <FactLine label="Interval" fact={value.interval} patch />
      <FactLine label="Area increase" fact={value.area} patch />
      <HeighteningDamage fact={value.damage} patch />
      <FixedLevels fact={value.levels} patch />
    </div>
  );
}
export function SpellChanges({
  changes,
  rankChanges,
  effective = false,
}: {
  changes: SpellChangeView;
  rankChanges?: SpellRankChangeView[];
  effective?: boolean;
}) {
  const explain = (
    kind: SpellRankFactView["kind"],
    label: string,
    componentId?: string,
  ) => rankValue(rankChanges, kind, label, componentId);
  return (
    <>
      <ActionCost
        label="Cast"
        fact={changes.cast}
        patch={!effective}
        renderValue={explain("cast", "Cast")}
      />
      <FactLine
        label="Range"
        fact={changes.range}
        patch={!effective}
        renderValue={explain("range", "Range")}
      />
      <FactLine
        label="Target"
        fact={changes.target}
        patch={!effective}
        renderValue={explain("target", "Target")}
      />
      {effective && changes.area.state === "null" ? (
        <FactRow label="Area">{explain("area", "Area")?.("None") ?? "None"}</FactRow>
      ) : (
        <Area
          fact={changes.area}
          patch={!effective}
          renderValue={explain("area", "Area")}
        />
      )}
      {effective && changes.defense.state === "null" ? (
        <FactRow label="Defense">
          {explain("defense", "Defense")?.("None") ?? "None"}
        </FactRow>
      ) : (
        <Defense
          fact={changes.defense}
          patch={!effective}
          renderValue={explain("defense", "Defense")}
        />
      )}
      <FactLine
        label="Duration"
        fact={changes.duration}
        patch={!effective}
        renderValue={explain("duration", "Duration")}
      />
      <FactLine
        label="Sustained"
        renderValue={explain("sustained", "Sustained")}
        fact={changes.sustained}
        patch={!effective}
        format={(value) => (value ? "Yes" : "No")}
      />
      <FactLine
        label="Traits"
        renderValue={explain("traits", "Traits")}
        fact={changes.traits}
        patch={!effective}
        format={(value) => value.join(", ")}
      />
      <FactLine
        label="Traditions"
        renderValue={explain("traditions", "Traditions")}
        fact={changes.traditions}
        patch={!effective}
        format={(value) => value.join(", ")}
      />
      {effective && hasReadingFact(changes.damage) ? (
        <div className="record-fact spell-reference__damage">
          <strong>Damage</strong>
          <DamageComponents
            damage={changes.damage}
            renderComponent={(component, value) =>
              explain("damage", "Damage", component.id)?.(value) ?? value
            }
          />
        </div>
      ) : (
        <DamageComponents
          damage={changes.damage}
          patch={!effective}
          renderComponent={(component, value) =>
            explain("damage", "Damage", component.id)?.(value) ?? value
          }
        />
      )}
      <HeighteningChanges fact={changes.heightening} />
    </>
  );
}
function Heightening({
  fact,
  selectedRank,
}: {
  fact: FactView<SpellHeighteningView>;
  selectedRank?: number;
}) {
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Heightening" fact={fact} />;
  const heightening = fact.value;
  if (heightening.kind === "fixed" && !hasReadingFact(heightening.levels)) return null;
  if (
    heightening.kind === "fixed" &&
    heightening.levels.state === "value" &&
    heightening.levels.value &&
    !heightening.levels.value.some(hasLevelChanges)
  )
    return null;
  return (
    <Collapse
      className="spell-reference__heightening"
      size="small"
      items={[
        {
          key: "heightening",
          label: "Heightening details",
          children:
            heightening.kind === "interval" ? (
              <>
                <FactRow label="Heightened">
                  (+
                  <NumberFact fact={heightening.interval} />)
                </FactRow>
                {!["missing", "not_applicable"].includes(heightening.area.state) &&
                  !(
                    heightening.area.state === "value" &&
                    Number(heightening.area.value) === 0
                  ) && (
                    <FactRow label="Area increase">
                      <NumberFact fact={heightening.area} unit=" feet" />
                    </FactRow>
                  )}
                <HeighteningDamage fact={heightening.damage} />
              </>
            ) : (
              <FixedLevels fact={heightening.levels} selectedRank={selectedRank} />
            ),
        },
      ]}
    />
  );
}
export function SpellReference({
  spell,
  children,
}: {
  spell: SpellPresentationView;
  children?: ReactNode;
}) {
  const ritual = spell.ritual.state === "value" ? spell.ritual.value : null;
  const explain = (
    kind: SpellRankFactView["kind"],
    label: string,
    componentId?: string,
  ) => rankValue(spell.rank_changes, kind, label, componentId);

  return (
    <div className="spell-reference">
      <div className="spell-reference__casting">
        {spell.rank_note && <p>{spell.rank_note}</p>}
        {spell.runtime_notes?.map((note, index) => (
          <p key={index}>
            {note.label}: {note.reason}
          </p>
        ))}
        {((spell.runtime_attack && hasReadingFact(spell.runtime_attack)) ||
          (spell.runtime_difficulty_class &&
            hasReadingFact(spell.runtime_difficulty_class))) && (
          <div className="record-fact-list">
            {spell.runtime_attack && hasReadingFact(spell.runtime_attack) && (
              <FactRow label="Spell attack">
                <NumberFact fact={spell.runtime_attack} signed />
              </FactRow>
            )}
            {spell.runtime_difficulty_class &&
              hasReadingFact(spell.runtime_difficulty_class) && (
                <FactRow label="Spell DC">
                  <NumberFact fact={spell.runtime_difficulty_class} />
                </FactRow>
              )}
          </div>
        )}
        <FactLine
          label="Traditions"
          fact={spell.traditions}
          format={(value) => value.join(", ")}
          renderValue={explain("traditions", "Traditions")}
        />
        <ActionCost
          label="Cast"
          fact={spell.cast}
          renderValue={explain("cast", "Cast")}
        />
        {spell.requirements && (
          <FactLine
            label="Requirements"
            fact={spell.requirements}
            renderValue={explain("requirements", "Requirements")}
          />
        )}
        {!spell.requirements && explain("requirements", "Requirements") && (
          <FactRow label="Requirements">
            {explain("requirements", "Requirements")?.("Not listed") ?? "Not listed"}
          </FactRow>
        )}
        {spell.cost && (
          <FactLine
            label="Cost"
            fact={spell.cost}
            renderValue={explain("cost", "Cost")}
          />
        )}
        {!spell.cost && explain("cost", "Cost") && (
          <FactRow label="Cost">
            {explain("cost", "Cost")?.("Not listed") ?? "Not listed"}
          </FactRow>
        )}
        {ritual ? (
          <div className="spell-reference__ritual-checks">
            <FactLine label="Primary check" fact={ritual.primary_check} required />
            <FactRow label="Secondary casters">
              {ritual.secondary_casters.state === "null" ? (
                "Not specified"
              ) : (
                <NumberFact fact={ritual.secondary_casters} label="Secondary casters" />
              )}
            </FactRow>
            {ritual.secondary_checks.state === "value" &&
            !ritual.secondary_checks.value?.trim() ? (
              <FactRow label="Secondary checks">None listed</FactRow>
            ) : (
              <FactLine
                label="Secondary checks"
                fact={ritual.secondary_checks}
                required
              />
            )}
          </div>
        ) : (
          spell.ritual.state !== "missing" && (
            <FactLine label="Ritual casting" fact={spell.ritual} />
          )
        )}
        {(spell.counteraction.value !== false ||
          explain("counteraction", "Counteract")) && (
          <FactLine
            label="Counteract"
            fact={spell.counteraction}
            format={(value) => (value ? "Yes" : "No")}
            renderValue={explain("counteraction", "Counteract")}
          />
        )}
        <FactLine
          label="Range"
          fact={spell.range}
          renderValue={explain("range", "Range")}
        />
        <FactLine
          label="Target"
          fact={spell.target}
          renderValue={explain("target", "Target")}
        />
        {spell.area ? (
          <Area fact={spell.area} renderValue={explain("area", "Area")} />
        ) : (
          explain("area", "Area") && (
            <FactRow label="Area">
              {explain("area", "Area")?.("None") ?? "None"}
            </FactRow>
          )
        )}
        {spell.defense ? (
          <Defense fact={spell.defense} renderValue={explain("defense", "Defense")} />
        ) : (
          explain("defense", "Defense") && (
            <FactRow label="Defense">
              {explain("defense", "Defense")?.("None") ?? "None"}
            </FactRow>
          )
        )}
        <FactLine
          label="Duration"
          fact={spell.duration}
          renderValue={explain("duration", "Duration")}
        />
        {spell.sustained.state === "value" ? (
          (spell.sustained.value || explain("sustained", "Sustained")) && (
            <FactRow label="Sustained">
              {explain(
                "sustained",
                "Sustained",
              )?.(spell.sustained.value ? "Yes" : "No") ??
                (spell.sustained.value ? "Yes" : "No")}
            </FactRow>
          )
        ) : (
          <FactLine label="Sustained" fact={spell.sustained} />
        )}
        {spell.damage.state === "value" && spell.damage.value?.length ? (
          <div className="record-fact spell-reference__damage">
            <strong>Damage</strong>
            <DamageComponents
              damage={spell.damage}
              renderComponent={(component, value) =>
                explain("damage", "Damage", component.id)?.(value) ?? value
              }
            />
          </div>
        ) : (
          <DamageComponents
            damage={spell.damage}
            renderComponent={(component, value) =>
              explain("damage", "Damage", component.id)?.(value) ?? value
            }
          />
        )}
      </div>
      {children}
      {
        <>
          <Heightening
            fact={spell.heightening}
            selectedRank={spell.rank.value ?? undefined}
          />
          {spell.forms.value?.length ? (
            <section className="spell-reference__forms">
              <h3>Spell forms</h3>
              {spell.forms.value.map((form, index) => (
                <section className="spell-reference__change" key={form.id}>
                  <h4>
                    {form.name.state === "value" && form.name.value?.trim()
                      ? form.name.value
                      : `Spell form ${index + 1}`}
                  </h4>
                  {["null", "invalid"].includes(form.name.state) && (
                    <FactLine label="Form name" fact={form.name} />
                  )}
                  {form.rank_note && <p>{form.rank_note}</p>}
                  {form.changes.value ? (
                    <SpellChanges
                      changes={form.changes.value}
                      rankChanges={form.rank_changes}
                      effective={form.effective_rank != null}
                    />
                  ) : (
                    <FactLine label="Other changes" fact={form.changes} patch />
                  )}
                </section>
              ))}
            </section>
          ) : (
            <FactLine label="Spell forms" fact={spell.forms} />
          )}
        </>
      }
    </div>
  );
}
