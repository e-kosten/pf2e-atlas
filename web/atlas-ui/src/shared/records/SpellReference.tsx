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
} from "../../generated/atlas";
import { DamageComponents } from "./ActorReference";
import { ActionCost } from "./ActionCost";
import {
  FactLine,
  FactRow,
  NumberFact,
  factText,
  hasReadingFact,
} from "./presentationFacts";

function Area({
  fact,
  patch = false,
}: {
  fact: FactView<SpellAreaView>;
  patch?: boolean;
}) {
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Area" fact={fact} patch={patch} />;
  if (patch)
    return (
      <>
        <FactLine label="Area size" fact={fact.value.size} patch />
        <FactLine label="Area shape" fact={fact.value.shape} patch />
        <FactLine label="Area details" fact={fact.value.details} patch />
      </>
    );
  return (
    <FactRow label="Area">
      {[
        factText(fact.value.size, String, { required: true }),
        factText(fact.value.shape, String, { required: true }),
        fact.value.details.state === "missing" || !hasReadingFact(fact.value.details)
          ? ""
          : factText(fact.value.details),
      ]
        .filter(Boolean)
        .join(" ")}
    </FactRow>
  );
}
function Defense({
  fact,
  patch = false,
}: {
  fact: FactView<SpellDefenseView>;
  patch?: boolean;
}) {
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Defense" fact={fact} patch={patch} />;
  if (patch)
    return (
      <>
        <FactLine label="Save statistic" fact={fact.value.statistic} patch />
        <FactLine
          label="Basic save"
          fact={fact.value.basic}
          patch
          format={(basic) => (basic ? "Yes" : "No")}
        />
        <FactLine label="Passive defense" fact={fact.value.passive} patch />
      </>
    );
  return (
    <FactRow label="Defense">
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
    </FactRow>
  );
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
}: {
  fact: FactView<SpellFixedLevelView[]>;
  patch?: boolean;
}) {
  if (!hasReadingFact(fact, { patch })) return null;
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Fixed ranks" fact={fact} patch={patch} />;
  if (!fact.value.length)
    return <FactLine label="Fixed ranks" fact={fact} patch={patch} />;
  return (
    <>
      {fact.value.map((level) => (
        <section key={level.rank} className="spell-reference__change">
          <h4>Heightened (rank {level.rank})</h4>
          {level.changes.value ? (
            <SpellChanges changes={level.changes.value} />
          ) : (
            <FactLine label="Other changes" fact={level.changes} patch />
          )}
        </section>
      ))}
    </>
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
export function SpellChanges({ changes }: { changes: SpellChangeView }) {
  return (
    <>
      <ActionCost label="Cast" fact={changes.cast} patch />
      <FactLine label="Range" fact={changes.range} patch />
      <FactLine label="Target" fact={changes.target} patch />
      <Area fact={changes.area} patch />
      <Defense fact={changes.defense} patch />
      <FactLine label="Duration" fact={changes.duration} patch />
      <FactLine
        label="Sustained"
        fact={changes.sustained}
        patch
        format={(value) => (value ? "Yes" : "No")}
      />
      <FactLine
        label="Traits"
        fact={changes.traits}
        patch
        format={(value) => value.join(", ")}
      />
      <FactLine
        label="Traditions"
        fact={changes.traditions}
        patch
        format={(value) => value.join(", ")}
      />
      <DamageComponents damage={changes.damage} patch />
      <HeighteningChanges fact={changes.heightening} />
    </>
  );
}
function Heightening({ fact }: { fact: FactView<SpellHeighteningView> }) {
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Heightening" fact={fact} />;
  const heightening = fact.value;
  if (heightening.kind === "fixed" && !hasReadingFact(heightening.levels)) return null;
  return (
    <section className="spell-reference__heightening">
      <h3>Heightening</h3>
      {heightening.kind === "interval" ? (
        <>
          <FactRow label="Heightened">
            (+
            <NumberFact fact={heightening.interval} />)
          </FactRow>
          {!["missing", "not_applicable"].includes(heightening.area.state) && (
            <FactRow label="Area increase">
              <NumberFact fact={heightening.area} unit=" feet" />
            </FactRow>
          )}
          <HeighteningDamage fact={heightening.damage} />
        </>
      ) : (
        <FixedLevels fact={heightening.levels} />
      )}
    </section>
  );
}
export function SpellReference({
  spell,
  compact = false,
}: {
  spell: SpellPresentationView;
  compact?: boolean;
}) {
  return (
    <div className="spell-reference">
      <div className="spell-reference__casting">
        <FactLine
          label="Traditions"
          fact={spell.traditions}
          format={(value) => value.join(", ")}
        />
        <ActionCost label="Cast" fact={spell.cast} />
        {spell.requirements && (
          <FactLine label="Requirements" fact={spell.requirements} />
        )}
        {spell.cost && <FactLine label="Cost" fact={spell.cost} />}
        <FactLine label="Range" fact={spell.range} />
        <FactLine label="Target" fact={spell.target} />
        {spell.area && <Area fact={spell.area} />}
        {spell.defense && <Defense fact={spell.defense} />}
        <FactLine label="Duration" fact={spell.duration} />
        {spell.sustained.state === "value" ? (
          spell.sustained.value && <FactRow label="Sustained">Yes</FactRow>
        ) : (
          <FactLine label="Sustained" fact={spell.sustained} />
        )}
        {!["missing", "not_applicable"].includes(spell.authored_cast_rank.state) && (
          <FactRow label="Cast rank">
            <NumberFact fact={spell.authored_cast_rank} />
          </FactRow>
        )}
      </div>
      <DamageComponents damage={spell.damage} />
      {!compact && (
        <>
          <Heightening fact={spell.heightening} />
          {spell.forms.value?.length ? (
            <section className="spell-reference__forms">
              <h3>Spell forms</h3>
              <p className="muted">Changes to the base spell.</p>
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
                  {form.changes.value ? (
                    <SpellChanges changes={form.changes.value} />
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
      )}
    </div>
  );
}
