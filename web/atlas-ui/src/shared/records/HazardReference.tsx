import type { ReactNode } from "react";
import type { HazardPresentationView } from "../../generated/atlas";
import {
  ActorActivities,
  ActorSecondaryFacts,
  type CastingAnnotations,
} from "./ActorReference";
import {
  ActorEncounterLayout,
  type ActorEncounterContent,
} from "./ActorEncounterLayout";
import { HazardDefenses } from "./HazardDefenses";
import { ReferenceFact } from "./ReferenceProfiles";
import type { EmbeddedReadingContext } from "./EmbeddedAbilityReading";
import type { RecordReferenceHandler } from "./PreparedContent";
import type { ActorActivityView } from "../../generated/atlas";

/** Hazard reading follows detection, defenses, disable, activation, routine and reset. */
export function HazardReference({
  hazard,
  content,
  encounter = false,
  encounterContent,
  reading,
  onReference,
  activityRolls,
  castingAnnotations,
}: {
  hazard: HazardPresentationView;
  content: {
    description: ReactNode;
    detection: ReactNode;
    disable: ReactNode;
    routine: ReactNode;
    reset: ReactNode;
    other: ReactNode;
  };
  encounter?: boolean;
  encounterContent?: ActorEncounterContent;
  reading?: EmbeddedReadingContext;
  onReference: RecordReferenceHandler;
  activityRolls?: (activity: ActorActivityView) => ReactNode;
  castingAnnotations?: CastingAnnotations;
}) {
  return (
    <div className="hazard-reference__rules">
      {content.description}
      <ActorEncounterLayout
        durability={
          <div className="hazard-reference__rules">
            <section className="hazard-reference__detection">
              <h3>Detection</h3>
              <div className="reference-profile__facts">
                {hazard.complex.state !== "value" && (
                  <ReferenceFact
                    label="Complexity"
                    fact={hazard.complex}
                    format={(value) => (value ? "Complex" : "Simple")}
                  />
                )}
                <ReferenceFact label="Size" fact={hazard.actor.size} optional />
                <ReferenceFact
                  label="Stealth"
                  fact={hazard.stealth}
                  format={(value) => `${Number(value) >= 0 ? "+" : ""}${value}`}
                />
              </div>
              {content.detection}
            </section>
            {encounterContent?.durability ?? <HazardDefenses hazard={hazard} />}
            {content.disable}
          </div>
        }
        runtime={
          encounter && (hazard.actor.runtime || encounterContent) ? (
            <>
              <ActorSecondaryFacts actor={hazard.actor} runtimeOnly />
              {encounterContent?.conditions}
            </>
          ) : null
        }
      >
        <ActorActivities
          actor={hazard.actor}
          hazardContent={{ routine: content.routine, reset: content.reset }}
          castingAnnotations={castingAnnotations}
          essential
          activityRolls={activityRolls}
          onReference={onReference}
          encounter={encounter}
          reading={reading}
        />
        {content.other}
      </ActorEncounterLayout>
    </div>
  );
}
