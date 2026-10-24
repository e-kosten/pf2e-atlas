import { useEffect, useRef, type ReactNode } from "react";
import { Typography } from "antd";
import type { ActorActivityView, ActorPresentationView } from "../../generated/atlas";
import { atlasRoutePath, shouldHandleAtlasRouteClick } from "../../app/routes";
import {
  ActorActivities,
  ActorCombatBudget,
  ActorFacts,
  ActorSecondaryFacts,
  ActorSupportingFacts,
  type CastingAnnotations,
} from "./ActorReference";
import type { EmbeddedReadingContext } from "./EmbeddedAbilityReading";
import { FactLine } from "./presentationFacts";
import type { RecordReferenceHandler } from "./PreparedContent";
import "../../styles/ui/creature-reference.css";

export type CreatureEncounterContent = {
  durability: ReactNode;
  conditions: ReactNode;
  participantKey: string;
};

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
  encounterContent?: CreatureEncounterContent;
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
  const sections = [
    ...(!encounter && prose ? [["description", "Description"]] : []),
    ["facts", "Statistics"],
    ...(activities.some((a) => a.kind === "strike") ? [["strike", "Strikes"]] : []),
    ...(activities.some((a) => ["casting_entry", "spell"].includes(a.kind)) ||
    !["missing", "not_applicable"].includes(actor.ritual_difficulty_class.state)
      ? [["casting", "Spellcasting"]]
      : []),
    ...(activities.some((a) => a.kind === "ability") ? [["ability", "Abilities"]] : []),
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
      className={`creature-reference${activities.some((a) => ["casting_entry", "spell"].includes(a.kind)) ? " creature-reference--caster" : ""}${encounter ? " creature-reference--encounter" : ""}`}
    >
      <div className="creature-reference__layout">
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
          {encounterContent?.durability}
          <div className="creature-reference__tasks">
            {encounter && (
              <aside className="creature-reference__runtime">
                <ActorCombatBudget actor={actor} />
                {encounterContent && (
                  <section id={id("conditions")} tabIndex={-1}>
                    {encounterContent.conditions}
                  </section>
                )}
              </aside>
            )}
            <div className="creature-reference__rules">
              <section
                className="creature-reference__facts"
                id={id("facts")}
                tabIndex={-1}
              >
                {encounter ? (
                  <>
                    <FactLine label="Size" fact={actor.size} />
                    <ActorSupportingFacts actor={actor} encounter />
                    <ActorSecondaryFacts actor={actor} />
                  </>
                ) : (
                  <ActorFacts actor={actor} />
                )}
              </section>
              <ActorActivities
                actor={actor}
                encounter={encounter}
                essential
                reading={reading}
                onReference={onReference}
                castingAnnotations={castingAnnotations}
                activityRolls={activityRolls}
                sectionIds={{
                  strike: id("strike"),
                  ability: id("ability"),
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
          </div>
        </div>
      </div>
    </div>
  );
}
