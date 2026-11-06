import { ReferenceValue } from "./ReferenceHelp";
import { Children, type ReactNode } from "react";
import type { HazardPresentationView } from "../../generated/atlas";
import {
  ActorActivities,
  hasActorActivities,
  hasRuntimeDetails,
  ActorSecondaryFacts,
  type CastingAnnotations,
} from "./ActorReference";
import {
  ActorEncounterLayout,
  type ActorEncounterContent,
} from "./ActorEncounterLayout";
import { HazardDefenses, hasHazardDefenses } from "./HazardDefenses";
import { ReferenceFact, referenceRows, renderReferenceFact } from "./ReferenceProfiles";
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
  const hasRules =
    hasActorActivities(hazard.actor, true) ||
    content.routine ||
    content.reset ||
    content.other ||
    reading?.landing;
  const hasRuntime = hasRuntimeDetails(hazard.actor, true);
  const detection = referenceRows([
    hazard.complex.state !== "value"
      ? renderReferenceFact({
          label: "Complexity",
          fact: hazard.complex,
          format: (value) => (value ? "Complex" : "Simple"),
        })
      : null,
    renderReferenceFact({
      label: "Size",
      fact: hazard.actor.size,
      optional: true,
      renderValue: (value) => <ReferenceValue value={value} />,
    }),
    renderReferenceFact({
      label: "Detection DC",
      fact: hazard.detection_dc,
      optional: true,
    }),
    hazard.stealth.state !== "not_applicable" ||
    Children.toArray(content.detection).length > 0 ? (
      <div key="stealth" className="hazard-reference__stealth">
        <ReferenceFact
          label="Stealth"
          fact={hazard.stealth}
          format={(value) => `${Number(value) >= 0 ? "+" : ""}${value}`}
        />{" "}
        {content.detection}
      </div>
    ) : null,
  ]);
  const defenses =
    encounterContent?.durability ??
    (hasHazardDefenses(hazard) ? <HazardDefenses hazard={hazard} /> : null);
  return (
    <div className="hazard-reference__rules">
      {content.description}
      <ActorEncounterLayout
        durability={
          detection.length || defenses || content.disable ? (
            <div className="hazard-reference__rules">
              {detection.length > 0 && (
                <section className="hazard-reference__detection">
                  <h3>Detection</h3>
                  <div className="reference-profile__facts">{detection}</div>
                </section>
              )}
              {defenses}
              {content.disable}
            </div>
          ) : null
        }
        runtime={
          encounter && (hasRuntime || encounterContent?.conditions) ? (
            <>
              <ActorSecondaryFacts actor={hazard.actor} runtimeOnly />
              {encounterContent?.conditions}
            </>
          ) : null
        }
      >
        {hasRules ? (
          <>
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
          </>
        ) : null}
      </ActorEncounterLayout>
    </div>
  );
}
