import { useEffect, useRef, type ReactNode } from "react";
import { Typography } from "antd";
import type {
  ActorActivityView,
  ActorPresentationView,
  FactView,
} from "../../generated/atlas";
import { atlasRoutePath, shouldHandleAtlasRouteClick } from "../../app/routes";
import {
  ActorActivities,
  ActorCombatBudget,
  ActorDefenses,
  ActorSkills,
  ActorSecondaryFacts,
  ActorSupportingFacts,
  type CastingAnnotations,
} from "./ActorReference";
import type { EmbeddedReadingContext } from "./EmbeddedAbilityReading";
import { FactLine, hasReadingFact } from "./presentationFacts";
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
  const activities = actor.activities.value ?? [];
  const hasStatistics = [
    actor.size,
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
  ].some((fact) => hasReadingFact(fact as FactView<unknown>));
  const hasRules =
    hasStatistics ||
    hasReadingFact(actor.activities) ||
    hasReadingFact(actor.ritual_difficulty_class);
  const sections = [
    ...(!encounter && prose ? [["description", "Description"]] : []),
    ...(!encounter ? [["defenses", "Defenses"]] : []),
    ...(activities.some((a) => a.kind === "strike") ? [["strike", "Strikes"]] : []),
    ...(activities.some((a) => a.ability_group === "primary")
      ? [["ability", "Special abilities"]]
      : []),
    ...(activities.some((a) => ["casting_entry", "spell"].includes(a.kind)) ||
    !["missing", "not_applicable"].includes(actor.ritual_difficulty_class.state)
      ? [["casting", "Spellcasting"]]
      : []),
    ...(activities.some((a) => a.ability_group === "reaction")
      ? [["reaction", "Reactions"]]
      : []),
    ...(activities.some((a) => a.ability_group === "supporting")
      ? [["passive", "Passive abilities"]]
      : []),
    ...(activities.some((a) => a.ability_group === "unclassified")
      ? [["unclassified", "Other abilities"]]
      : []),
    ...(hasStatistics ? [["facts", "Statistics"]] : []),
    ...(activities.some((a) => a.kind === "gear") ? [["gear", "Equipment"]] : []),
    ...(activities.some((a) => a.kind === "other") ? [["other", "Other entries"]] : []),
    ...(encounterContent ? [["conditions", "Conditions"]] : []),
    ...(encounter && prose ? [["description", "Description"]] : []),
  ];
  const sectionHref = (name: string) =>
    `${encounter ? location.pathname + location.search : atlasRoutePath({ kind: "record", recordKey })}#${encodeURIComponent(id(name))}`;
  return (
    <div
      ref={container}
      className={`creature-reference${activities.some((a) => ["casting_entry", "spell"].includes(a.kind)) ? " creature-reference--caster" : ""}${encounter ? " creature-reference--encounter" : ""}${encounter && !hasRules && !prose ? " creature-reference--runtime-only" : ""}`}
    >
      <div className="creature-reference__layout">
        {(!encounter || sections.length > 1) && (
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
                      if (!shouldHandleAtlasRouteClick(event)) return;
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
            durability={encounterContent?.durability}
            runtime={
              encounter && (actor.runtime?.action_budget || encounterContent) ? (
                <>
                  <ActorCombatBudget actor={actor} />
                  {!hasStatistics && <ActorSecondaryFacts actor={actor} />}
                  {encounterContent && (
                    <section id={id("conditions")} tabIndex={-1}>
                      {encounterContent.conditions}
                    </section>
                  )}
                </>
              ) : null
            }
          >
            {(!encounter || hasRules || prose) && (
              <div className="creature-reference__rules">
                {!encounter && (
                  <section
                    className="creature-reference__facts"
                    id={id("defenses")}
                    tabIndex={-1}
                  >
                    <FactLine label="Size" fact={actor.size} />
                    <ActorDefenses actor={actor} />
                  </section>
                )}
                {encounter && !encounterContent && (
                  <ActorDefenses actor={actor} hideHitPoints />
                )}
                <ActorActivities
                  actor={actor}
                  supportingFacts={
                    hasStatistics ? (
                      <section
                        className="creature-reference__facts"
                        id={id("facts")}
                        tabIndex={-1}
                      >
                        <h3>Statistics</h3>
                        {encounter && <FactLine label="Size" fact={actor.size} />}
                        <ActorSkills actor={actor} />
                        <ActorSupportingFacts actor={actor} encounter />
                        <FactLine
                          label="Perception details"
                          fact={actor.perception_details}
                        />
                        <FactLine
                          label="Languages"
                          fact={actor.languages}
                          format={(values) => values.join(", ")}
                        />
                        <FactLine label="Communication" fact={actor.language_details} />
                        <ActorSecondaryFacts actor={actor} runtimeOnly />
                      </section>
                    ) : undefined
                  }
                  encounter={encounter}
                  essential
                  reading={reading}
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
