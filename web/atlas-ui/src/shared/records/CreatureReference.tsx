import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { useEffect, useRef, type ReactNode } from "react";
import { activityAnchor, landOnActivity } from "./embeddedNavigation";
import { Typography } from "antd";
import type {
  ActorActivityView,
  ActorEmbeddedConditionView,
  ActorPresentationView,
  FactView,
} from "../../generated/atlas";
import {
  ActorActivities,
  ActorDefenses,
  hasActorDefenses,
  hasActorActivities,
  hasCastingActivities,
  ActorSkills,
  ActorAwareness,
  ActorMovement,
  ActorLanguages,
  type CastingAnnotations,
} from "./ActorReference";
import type { EmbeddedReadingContext } from "./EmbeddedAbilityReading";
import { FactLine, FactRow, hasReadingFact } from "./presentationFacts";
import type { RecordReferenceHandler } from "./PreparedContent";
import "../../styles/ui/creature-reference.css";
import {
  ActorEncounterLayout,
  type ActorEncounterContent,
} from "./ActorEncounterLayout";
/** Shared source reading with host-owned encounter controls. No rules are inferred here. */
export function CreatureReference({
  actor,
  recordKey,
  prose,
  encounter = false,
  encounterContent,
  reading,
  onReference,
  activityRolls,
  castingAnnotations,
  sectionLanding,
}: {
  actor: ActorPresentationView;
  recordKey: string;
  prose: ReactNode;
  encounter?: boolean;
  encounterContent?: ActorEncounterContent;
  reading?: EmbeddedReadingContext;
  onReference: RecordReferenceHandler;
  activityRolls?: (activity: ActorActivityView) => ReactNode;
  castingAnnotations?: CastingAnnotations;
  sectionLanding?: string;
}) {
  const container = useRef<HTMLDivElement>(null);
  const prefix = `creature-${encounterContent?.participantKey ?? recordKey}`;
  const id = (name: string) => `${prefix}-${name}`;
  const landed = useRef<string | undefined>(undefined);
  useEffect(() => {
    const identity = `${prefix}:${sectionLanding}`;
    if (!sectionLanding || landed.current === identity) return;
    const target = Array.from(
      container.current?.querySelectorAll<HTMLElement>("[id]") ?? [],
    ).find((node) => node.id === sectionLanding);
    if (target) {
      landed.current = identity;
      target.scrollIntoView?.({ block: "start" });
      target.focus({ preventScroll: true });
    }
  }, [prefix, sectionLanding]);
  const conditions = encounter && encounterContent?.conditions;
  const authoredConditions = encounter ? [] : (actor.embedded_conditions ?? []);
  const conditionAnchors = new Set(
    authoredConditions.map(({ navigation }) => activityAnchor(navigation)),
  );
  const landing = reading?.landing;
  const onLandingReady = reading?.onLandingReady;
  const conditionLanding =
    landing && conditionAnchors.has(activityAnchor(landing)) ? landing : undefined;
  useEffect(() => {
    if (conditionLanding) {
      landOnActivity(container.current, conditionLanding, onLandingReady);
    }
  }, [conditionLanding, onLandingReady]);
  const overviewActor = conditionAnchors.size
    ? {
        ...actor,
        activities: {
          ...actor.activities,
          value:
            actor.activities.value?.filter(
              ({ navigation }) => !conditionAnchors.has(activityAnchor(navigation)),
            ) ?? null,
        },
      }
    : actor;
  const activities = overviewActor.activities.value ?? [];
  const hasStatistics =
    [
      actor.perception,
      actor.senses,
      actor.land_speed,
      actor.movement,
      actor.movement_details,
      actor.perception_details,
      actor.languages,
      actor.language_details,
      actor.abilities,
      actor.skills,
    ].some((fact) => hasReadingFact(fact as FactView<unknown>)) ||
    activities.some((activity) => activity.kind === "lore");
  const hasRules =
    hasStatistics ||
    hasActorActivities(overviewActor) ||
    Boolean(landing && !conditionLanding);
  const runtimeOnly = encounter && !hasRules && !prose;
  const hasDefenses =
    authoredConditions.length > 0 ||
    hasReadingFact(actor.size) ||
    Boolean(encounterContent?.durability) ||
    hasActorDefenses(actor, { hideHitPoints: encounter });
  const sections = [
    ...(!encounter && prose ? [["description", "Description"]] : []),
    ...(hasDefenses ? [["defenses", "Defenses"]] : []),
    ...(conditions ? [["conditions", "Actions & conditions"]] : []),
    ...(hasStatistics ? [["facts", "Statistics"]] : []),
    ...(activities.some((a) => a.kind === "strike") ? [["strike", "Strikes"]] : []),
    ...(activities.some((a) => a.kind === "ability" && a.ability_group === "primary")
      ? [["ability", "Special abilities"]]
      : []),
    ...(hasCastingActivities(actor) ? [["casting", "Spellcasting"]] : []),
    ...(activities.some((a) => a.kind === "ability" && a.ability_group === "reaction")
      ? [["reaction", "Reactions"]]
      : []),
    ...(activities.some((a) => a.kind === "ability" && a.ability_group === "supporting")
      ? [["passive", "Passive abilities"]]
      : []),
    ...(activities.some(
      (a) => a.kind === "ability" && a.ability_group === "unclassified",
    )
      ? [["unclassified", "Other abilities"]]
      : []),
    ...(activities.some((a) => a.kind === "gear") ? [["gear", "Equipment"]] : []),
    ...(activities.some((a) => a.kind === "other") ? [["other", "Other entries"]] : []),
    ...(encounter && prose ? [["description", "Description"]] : []),
  ];
  const showOutline = !runtimeOnly && sections.length > (encounter ? 1 : 0);
  const sectionHref = (name: string) =>
    `${
      encounter
        ? location.pathname + location.search
        : destinationHref(
            recordDestination({
              recordKey,
            }),
          )
    }#${encodeURIComponent(id(name))}`;
  return (
    <div
      ref={container}
      className={`creature-reference${encounter ? " creature-reference--encounter" : ""}${runtimeOnly ? " creature-reference--runtime-only" : ""}`}
    >
      <div className="creature-reference__layout">
        {showOutline && (
          <nav
            className={`document-outline creature-reference__outline`}
            aria-label="Creature sections"
          >
            {!encounter && <strong>On this page</strong>}
            <ul>
              {sections.map(([name, label]) => (
                <li key={name}>
                  <Typography.Link
                    href={sectionHref(name)}
                    onClick={(event) => {
                      if (!isPrimaryActivation(event)) return;
                      event.preventDefault();
                      const section = Array.from(
                        container.current?.querySelectorAll<HTMLElement>("[id]") ?? [],
                      ).find((node) => node.id === id(name));
                      section?.scrollIntoView?.({ block: "start" });
                      section?.focus({ preventScroll: true });
                    }}
                  >
                    {label}
                  </Typography.Link>
                </li>
              ))}
            </ul>
          </nav>
        )}
        <div className="creature-reference__body">
          {!encounter && prose && (
            <section
              id={id("description")}
              tabIndex={-1}
              className="creature-reference__prose"
            >
              {prose}
            </section>
          )}
          <ActorEncounterLayout
            durability={
              hasDefenses ? (
                <div className="creature-reference__essentials">
                  <section
                    className={
                      !encounter ? "creature-reference__source-defenses" : undefined
                    }
                    id={id("defenses")}
                    tabIndex={-1}
                  >
                    <FactLine label="Size" fact={actor.size} />
                    {encounterContent?.durability ?? (
                      <ActorDefenses
                        actor={actor}
                        hideHitPoints={encounter}
                        conditions={
                          authoredConditions.length > 0 && (
                            <CreatureConditions
                              conditions={authoredConditions}
                              onReference={onReference}
                            />
                          )
                        }
                      />
                    )}
                  </section>
                </div>
              ) : null
            }
            runtime={
              conditions ? (
                <section id={id("conditions")} tabIndex={-1}>
                  {conditions}
                </section>
              ) : null
            }
          >
            {(hasRules || (encounter && prose)) && (
              <div className="creature-reference__rules">
                {hasStatistics && (
                  <section
                    className="creature-reference__facts"
                    id={id("facts")}
                    tabIndex={-1}
                  >
                    <h3>Statistics</h3>
                    <div className="creature-reference__fact-groups">
                      <ActorAwareness actor={actor} encounter={encounter} />
                      <ActorSkills
                        actor={actor}
                        headingLevel={4}
                        encounter={encounter}
                      />
                      <ActorMovement actor={actor} />
                      <ActorLanguages actor={actor} />
                    </div>
                  </section>
                )}
                <ActorActivities
                  actor={overviewActor}
                  encounter={encounter}
                  essential
                  reading={
                    reading && conditionLanding
                      ? { ...reading, landing: undefined }
                      : reading
                  }
                  onReference={onReference}
                  castingAnnotations={castingAnnotations}
                  activityRolls={activityRolls}
                  sectionIds={{
                    strike: id("strike"),
                    ability: id("ability"),
                    reaction: id("reaction"),
                    passive: id("passive"),
                    unclassified: id("unclassified"),
                    gear: id("gear"),
                    other: id("other"),
                    casting: id("casting"),
                  }}
                />
                {encounter && prose && (
                  <section
                    id={id("description")}
                    tabIndex={-1}
                    className="creature-reference__prose"
                  >
                    {prose}
                  </section>
                )}
              </div>
            )}
          </ActorEncounterLayout>
        </div>
      </div>
    </div>
  );
}

function CreatureConditions({
  conditions,
  onReference,
}: {
  conditions: ActorEmbeddedConditionView[];
  onReference: RecordReferenceHandler;
}) {
  return (
    <FactRow label="Conditions">
      {conditions.map((condition, index) => {
        const { navigation } = condition;
        const title =
          condition.value === null
            ? condition.name
            : `${condition.name} ${condition.value}`;
        return (
          <span
            key={activityAnchor(navigation)}
            data-atlas-activity-anchor={activityAnchor(navigation)}
          >
            {index > 0 && ", "}
            <Typography.Link
              href={destinationHref(
                recordDestination({
                  recordKey: navigation.record_key,
                  selection: navigation,
                }),
              )}
              onClick={(event) => {
                if (!isPrimaryActivation(event)) return;
                event.preventDefault();
                onReference(navigation.record_key, event.currentTarget, navigation);
              }}
            >
              {title}
            </Typography.Link>
          </span>
        );
      })}
    </FactRow>
  );
}
