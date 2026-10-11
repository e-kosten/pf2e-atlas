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
import { FactLine, FactRow, NumberFact, factText } from "./presentationFacts";

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
      {[factText(fact.value.size), factText(fact.value.shape), fact.value.details.value]
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
      {fact.value.passive.value ? `; ${fact.value.passive.value}` : ""}
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
  return (
    <>
      {fact.value.map((component) => (
        <FactLine
          key={component.id}
          label={`Damage ${component.id}`}
          fact={component.formula}
          patch={patch}
        />
      ))}
    </>
  );
}
function FixedLevels({ fact }: { fact: FactView<SpellFixedLevelView[]> }) {
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Fixed ranks" fact={fact} patch />;
  if (!fact.value.length) return <FactLine label="Fixed ranks" fact={fact} patch />;
  return (
    <>
      {fact.value.map((level) => (
        <section key={level.rank} className="spell-reference__change">
          <h4>Heightened (rank {level.rank})</h4>
          {level.changes.value ? (
            <SpellChanges changes={level.changes.value} />
          ) : (
            <FactLine label="Authored changes" fact={level.changes} patch />
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
      <FixedLevels fact={value.levels} />
    </div>
  );
}
export function SpellChanges({ changes }: { changes: SpellChangeView }) {
  return (
    <>
      <FactLine label="Cast" fact={changes.cast} patch />
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
    return ["invalid", "null"].includes(fact.state) ? (
      <FactLine label="Authored heightening" fact={fact} />
    ) : null;
  const heightening = fact.value;
  return (
    <section className="spell-reference__heightening">
      <h3>Authored heightening</h3>
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
        <FactLine label="Cast" fact={spell.cast} />
        <FactLine label="Requirements" fact={spell.requirements} />
        <FactLine label="Cost" fact={spell.cost} />
        <FactLine label="Range" fact={spell.range} />
        <FactLine label="Target" fact={spell.target} />
        <Area fact={spell.area} />
        <Defense fact={spell.defense} />
        <FactLine label="Duration" fact={spell.duration} />
        {spell.sustained.value && <FactRow label="Sustained">Yes</FactRow>}
        <FactLine label="Casting entry" fact={spell.casting_entry} />
        {spell.authored_cast_rank.state === "value" && (
          <FactRow label="Authored cast rank">
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
              <h3>Authored forms</h3>
              <p className="muted">Changes to the base spell.</p>
              {spell.forms.value.map((form) => (
                <section className="spell-reference__change" key={form.id}>
                  <h4>{form.name.value || form.id}</h4>
                  {form.changes.value ? (
                    <SpellChanges changes={form.changes.value} />
                  ) : (
                    <FactLine label="Authored changes" fact={form.changes} patch />
                  )}
                </section>
              ))}
            </section>
          ) : ["invalid", "null"].includes(spell.forms.state) ? (
            <FactLine label="Authored forms" fact={spell.forms} />
          ) : null}
        </>
      )}
    </div>
  );
}
