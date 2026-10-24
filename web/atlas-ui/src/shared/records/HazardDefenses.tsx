import type { RecordBodyView } from "../../generated/atlas";
import { ReferenceFact } from "./ReferenceProfiles";
import { Iwr } from "./ActorReference";
import { hasReadingFact } from "./presentationFacts";

export function HazardDefenses({
  hazard,
  hideHitPoints = false,
}: {
  hazard: Extract<RecordBodyView, { kind: "hazard" }>["value"];
  hideHitPoints?: boolean;
}) {
  return (
    <>
      {(hazard.defense_block.value !== false ||
        hazard.show_saves ||
        [
          hazard.actor.immunities,
          hazard.actor.weaknesses,
          hazard.actor.resistances,
          ...Object.values(hazard.actor.defense_notes),
        ].some(
          (fact) => fact.state !== "missing" && hasReadingFact<unknown>(fact),
        )) && (
        <section className="reference-profile__section">
          <h3>Defenses</h3>
          <div className="reference-profile__groups">
            {hazard.defense_block.value !== false && (
              <div className="reference-profile__facts">
                <ReferenceFact label="AC" fact={hazard.actor.armor_class} />
                {!hideHitPoints && (
                  <ReferenceFact label="HP" fact={hazard.actor.maximum_hp} />
                )}
                <ReferenceFact label="Hardness" fact={hazard.hardness} />
              </div>
            )}
            {hazard.show_saves && (
              <div className="reference-profile__facts">
                {(["fortitude", "reflex", "will"] as const).map((save) => (
                  <ReferenceFact
                    key={save}
                    label={save[0].toUpperCase() + save.slice(1)}
                    fact={hazard.actor.saves[save]}
                    format={(value) => `${Number(value) >= 0 ? "+" : ""}${value}`}
                  />
                ))}
              </div>
            )}
            <Iwr label="Immunities" fact={hazard.actor.immunities} />
            <Iwr label="Weaknesses" fact={hazard.actor.weaknesses} />
            <Iwr label="Resistances" fact={hazard.actor.resistances} />
            <ReferenceFact
              label="AC details"
              fact={hazard.actor.defense_notes.armor_class}
              optional
            />
            <ReferenceFact
              label="HP details"
              fact={hazard.actor.defense_notes.hit_points}
              optional
            />
            <ReferenceFact
              label="Save details"
              fact={hazard.actor.defense_notes.all_saves}
              optional
            />
            {(["fortitude", "reflex", "will"] as const).map((save) => (
              <ReferenceFact
                key={save}
                label={`${save} details`}
                fact={hazard.actor.defense_notes[save]}
                optional
              />
            ))}
          </div>
        </section>
      )}
    </>
  );
}
