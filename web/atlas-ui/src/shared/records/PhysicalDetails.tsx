import type {
  FactView,
  NumberFactView,
  PhysicalDetailsView,
  WeaponDamageReferenceView,
  KitEntryReferenceView,
} from "../../generated/atlas";
import { FactLine, FactRow, NumberFact, factText } from "./presentationFacts";

const optional = (fact: FactView<unknown>) =>
  !["missing", "null", "not_applicable"].includes(fact.state);
const nonzero = (fact: NumberFactView) =>
  fact.state === "value"
    ? fact.value !== 0
    : !["missing", "not_applicable"].includes(fact.state);
const secondaryFact = (fact: FactView<unknown>) =>
  fact.state === "value"
    ? fact.value !== 0 &&
      fact.value !== "" &&
      (!Array.isArray(fact.value) || fact.value.length > 0)
    : !["missing", "not_applicable"].includes(fact.state);
const words = (value: string) =>
  value.replace(/([a-z])([A-Z])/g, "$1 $2").replace(/[-_]/g, " ");
const bulk = (value: number) => (value === 0.1 ? "L" : String(value));

function BaseDamage({ fact }: { fact: FactView<WeaponDamageReferenceView> }) {
  if (!fact.value) return <FactLine label="Base damage" fact={fact} required />;
  const d = fact.value;
  const dieKnown = d.die.state === "value" && Boolean(d.die.value);
  return (
    <>
      {dieKnown ? (
        <FactRow label="Base damage">
          <NumberFact fact={d.dice} />
          {d.die.value} {factText(d.damage_type)}
          {d.modifier.state === "value" && d.modifier.value !== 0 && (
            <>
              {" "}
              <NumberFact fact={d.modifier} signed />
            </>
          )}
        </FactRow>
      ) : (
        <>
          <FactLine label="Damage dice" fact={d.dice} />
          {optional(d.die) && <FactLine label="Damage die" fact={d.die} />}
          <FactLine label="Damage type" fact={d.damage_type} required />
        </>
      )}
      {d.modifier.state === "invalid" && (
        <FactLine label="Damage modifier" fact={d.modifier} />
      )}
      {d.persistent.value ? (
        <FactRow label="Persistent damage">
          <NumberFact fact={d.persistent.value.number} />
          {d.persistent.value.faces.state === "value" &&
            d.persistent.value.faces.value !== null && (
              <>
                d<NumberFact fact={d.persistent.value.faces} />
              </>
            )}
          {!["value", "null"].includes(d.persistent.value.faces.state) && (
            <> · Die faces {factText(d.persistent.value.faces)}</>
          )}{" "}
          {factText(d.persistent.value.damage_type)}
        </FactRow>
      ) : (
        optional(d.persistent) && (
          <FactLine label="Persistent damage" fact={d.persistent} />
        )
      )}
    </>
  );
}

function KitEntries({
  items,
  required = false,
}: {
  items: FactView<KitEntryReferenceView[]>;
  required?: boolean;
}) {
  if (!required && ["null", "missing", "not_applicable"].includes(items.state))
    return null;
  if (!items.value) return <FactLine label="Contents" fact={items} required />;
  if (items.value.length === 0) return required ? <p>No items listed.</p> : null;
  return (
    <ul className="physical-reference__kit-items">
      {items.value.map((item, index) => (
        <li key={index}>
          {factText(item.name, String, { required: true })} ×{" "}
          <NumberFact fact={item.quantity} />
          <KitEntries items={item.items} />
        </li>
      ))}
    </ul>
  );
}

export function PhysicalDetails({ details }: { details: PhysicalDetailsView }) {
  switch (details.kind) {
    case "weapon": {
      const w = details.value;
      return (
        <div className="physical-reference__facts">
          <div className="record-fact-list">
            <BaseDamage fact={w.damage} />
            {optional(w.splash_damage) && w.splash_damage.value !== "0" && (
              <FactLine label="Splash damage" fact={w.splash_damage} />
            )}
          </div>
          <div className="record-fact-list">
            <FactLine label="Category" fact={w.category} format={words} />
            <FactLine label="Group" fact={w.group} format={words} />
            {optional(w.range_increment) && (
              <FactLine
                label="Range increment"
                fact={w.range_increment}
                format={(n) => `${n} feet`}
              />
            )}
            {optional(w.maximum_range) && (
              <FactLine
                label="Maximum range"
                fact={w.maximum_range}
                format={(n) => `${n} feet`}
              />
            )}
            {optional(w.reload) && <FactLine label="Reload" fact={w.reload} />}
          </div>
          {w.ammunition.value &&
            (optional(w.ammunition.value.base_type) ||
              optional(w.ammunition.value.capacity) ||
              w.ammunition.value.built_in.value === true ||
              w.ammunition.value.built_in.state === "invalid") && (
              <div className="record-fact-list">
                {optional(w.ammunition.value.base_type) && (
                  <FactLine
                    label="Ammunition"
                    fact={w.ammunition.value.base_type}
                    format={words}
                  />
                )}
                {optional(w.ammunition.value.capacity) && (
                  <FactLine label="Capacity" fact={w.ammunition.value.capacity} />
                )}
                {w.ammunition.value.built_in.value === true && (
                  <FactRow label="Ammunition">Built in</FactRow>
                )}
                {w.ammunition.value.built_in.state === "invalid" && (
                  <FactLine
                    label="Built-in ammunition"
                    fact={w.ammunition.value.built_in}
                  />
                )}
              </div>
            )}
          {w.ammunition.state === "invalid" && (
            <FactLine label="Ammunition" fact={w.ammunition} />
          )}
          {w.runes.value &&
          [w.runes.value.potency, w.runes.value.striking, w.runes.value.property].some(
            secondaryFact,
          ) ? (
            <div className="record-fact-list">
              {nonzero(w.runes.value.potency) && (
                <FactRow label="Potency rune">
                  <NumberFact fact={w.runes.value.potency} signed />
                </FactRow>
              )}
              {secondaryFact(w.runes.value.striking) && (
                <FactLine label="Striking rune" fact={w.runes.value.striking} />
              )}
              {secondaryFact(w.runes.value.property) && (
                <FactLine
                  label="Property runes"
                  fact={w.runes.value.property}
                  format={(v) => v.map(words).join(", ")}
                />
              )}
            </div>
          ) : (
            ["invalid", "null"].includes(w.runes.state) && (
              <FactLine label="Runes" fact={w.runes} />
            )
          )}
          {w.melee_usage.value && (
            <section>
              <h3>Melee usage</h3>
              <div className="record-fact-list">
                <FactLine label="Die" fact={w.melee_usage.value.die} />
                <FactLine label="Damage type" fact={w.melee_usage.value.damage_type} />
                <FactLine
                  label="Group"
                  fact={w.melee_usage.value.group}
                  format={words}
                />
                <FactLine
                  label="Traits"
                  fact={w.melee_usage.value.traits}
                  format={(v) => v.map(words).join(", ")}
                />
              </div>
            </section>
          )}
          {w.melee_usage.state === "invalid" && (
            <FactLine label="Melee usage" fact={w.melee_usage} />
          )}
        </div>
      );
    }
    case "armor": {
      const a = details.value;
      return (
        <div className="physical-reference__facts">
          <div className="record-fact-list">
            <FactRow label="Base AC bonus">
              <NumberFact fact={a.ac_bonus} signed />
            </FactRow>
            <FactLine label="Dex cap" fact={a.dex_cap} />
            <FactLine label="Check penalty" fact={a.check_penalty} />
            <FactLine
              label="Speed penalty"
              fact={a.speed_penalty}
              format={(v) => `${v} feet`}
            />
            <FactRow label="Strength">
              {a.strength.state === "null" ? (
                "Not specified"
              ) : (
                <NumberFact fact={a.strength} signed />
              )}
            </FactRow>
          </div>
          <div className="record-fact-list">
            <FactLine label="Category" fact={a.category} format={words} />
            <FactLine label="Group" fact={a.group} format={words} />
            {nonzero(a.potency) && (
              <FactRow label="Potency rune">
                <NumberFact fact={a.potency} signed />
              </FactRow>
            )}
            {secondaryFact(a.resilient) && (
              <FactLine label="Resilient rune" fact={a.resilient} />
            )}
            {secondaryFact(a.property_runes) && (
              <FactLine
                label="Property runes"
                fact={a.property_runes}
                format={(v) => v.map(words).join(", ")}
              />
            )}
          </div>
        </div>
      );
    }
    case "shield": {
      const s = details.value;
      return (
        <div className="record-fact-list">
          <FactRow label="AC bonus">
            <NumberFact fact={s.ac_bonus} signed />
          </FactRow>
          {nonzero(s.speed_penalty) && (
            <FactLine
              label="Speed penalty"
              fact={s.speed_penalty}
              format={(v) => `${v} feet`}
            />
          )}
          {!s.durability_in_table && (
            <>
              <FactLine label="Hardness" fact={s.hardness} />
              <FactLine label="HP" fact={s.hit_points} />
            </>
          )}
          {nonzero(s.reinforcing) && (
            <FactLine label="Reinforcing rune" fact={s.reinforcing} />
          )}
        </div>
      );
    }
    case "container": {
      const c = details.value;
      return (
        <div className="record-fact-list">
          <FactLine label="Capacity" fact={c.capacity} format={(v) => `${v} Bulk`} />
          <FactLine label="Ignored Bulk" fact={c.ignored_bulk} format={bulk} />
          <FactLine
            label="Held or stowed Bulk"
            fact={c.held_or_stowed_bulk}
            format={bulk}
          />
          <FactLine
            label="Stowing"
            fact={c.stowing}
            format={(v) => (v ? "Yes" : "No")}
          />
        </div>
      );
    }
    case "ammunition": {
      const a = details.value;
      const hasUses =
        (a.maximum_uses.value ?? 0) > 1 ||
        a.source_uses.value !== a.maximum_uses.value ||
        a.maximum_uses.state === "invalid" ||
        a.source_uses.state === "invalid";
      if (!hasUses) return null;
      return (
        <div className="record-fact-list">
          {((a.maximum_uses.value ?? 0) > 1 ||
            a.source_uses.value !== a.maximum_uses.value ||
            a.maximum_uses.state === "invalid" ||
            a.source_uses.state === "invalid") && (
            <FactRow label="Uses">
              {a.source_uses.value !== a.maximum_uses.value && (
                <>{factText(a.source_uses)} / </>
              )}
              {factText(a.maximum_uses)}
            </FactRow>
          )}
        </div>
      );
    }
    case "kit":
      return (
        <section>
          <h3>Contents</h3>
          <KitEntries items={details.value.items} required />
        </section>
      );
  }
}
