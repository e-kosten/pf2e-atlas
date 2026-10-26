import type {
  SpellEffectPreviewView,
  SpellEffectFactView,
} from "../../generated/atlas";
import { ValueExplanation } from "../ui/ValueExplanation";
import { FactLine, FactRow } from "./presentationFacts";
import { TraitBadge } from "./TraitBadge";

function EffectFact({ fact }: { fact: SpellEffectFactView }) {
  return (
    <div>
      {fact.value.state === "value" && fact.value.value !== null ? (
        <FactLine
          label={fact.label}
          fact={fact.value}
          renderValue={
            fact.base_value == null
              ? undefined
              : (value) => (
                  <ValueExplanation
                    label={`Heightening details for ${fact.label}`}
                    content={<>Base rank: {fact.base_value}</>}
                  >
                    {value}
                  </ValueExplanation>
                )
          }
        />
      ) : fact.value.state !== "not_applicable" ? (
        <FactRow label={fact.label}>
          {fact.value.state === "null" ? "Not specified" : "Unavailable"}
        </FactRow>
      ) : null}
      {fact.qualifier && (
        <p className="spell-effect-preview__qualifier">{fact.qualifier}</p>
      )}
    </div>
  );
}
export function SpellEffectPreview({ view }: { view: SpellEffectPreviewView }) {
  return (
    <section
      className="spell-effect-preview"
      aria-label={`Rank ${view.rank} statistics`}
    >
      {view.unavailable ? (
        <p role="status">
          Rank {view.rank} statistics unavailable. {view.unavailable}
        </p>
      ) : (
        <>
          {view.facts.length > 0 && (
            <div className="spell-effect-preview__facts">
              {view.facts.map((fact) => (
                <EffectFact key={fact.label} fact={fact} />
              ))}
            </div>
          )}
          {view.choices.length > 0 && (
            <section>
              <h3>Available sizes</h3>
              <div className="spell-effect-preview__choices">
                {view.choices.map((choice) => (
                  <section key={choice.label}>
                    <h4>{choice.label}</h4>
                    {choice.facts
                      .filter((fact) => fact.label !== "Size")
                      .map((fact) => (
                        <EffectFact key={fact.label} fact={fact} />
                      ))}
                  </section>
                ))}
              </div>
            </section>
          )}
          {view.strikes.length > 0 && (
            <section>
              <h3>Strikes</h3>
              {view.strikes.map((strike) => (
                <div className="spell-effect-preview__strike" key={strike.id}>
                  <div className="spell-effect-preview__strike-title">
                    <h4>{strike.title}</h4>
                    {strike.traits.map((trait) => (
                      <TraitBadge key={trait.value} trait={trait} />
                    ))}
                  </div>
                  <div className="spell-effect-preview__facts">
                    <EffectFact fact={strike.attack} />
                    <EffectFact fact={strike.damage} />
                  </div>
                </div>
              ))}
            </section>
          )}
        </>
      )}
    </section>
  );
}
