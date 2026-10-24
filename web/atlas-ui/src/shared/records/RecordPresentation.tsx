import { ReferenceProfiles, ReferenceFact } from "./ReferenceProfiles";
import { JournalContents } from "./JournalContents";
import { DocumentIdentityHeader } from "./DocumentIdentityHeader";
import { TraitBadge } from "./TraitBadge";
import { RollTableReading } from "./RollTableReading";
import { RecordRarity } from "./RecordRarity";
import { SpellRankControl } from "./SpellRankControl";
import { recordReadingIdentity } from "./useRetainedRecord";
import type { ReactNode, Ref } from "react";
import { Button, Collapse, Typography } from "antd";
import { BookOpen } from "lucide-react";
import type {
  PreparedContentFieldView,
  RecordNavigationView,
  RecordPresentationView,
  RecordRelationshipView,
  RecordSummaryView,
  ActionCostView,
  FactView,
} from "../../generated/atlas";
import {
  ActorActivities,
  ActorActivity,
  ActorCombatBudget,
  ActorSecondaryFacts,
} from "./ActorReference";
import { FactLine, factText, isEmptyFactValue } from "./presentationFacts";
import { PreparedContent, type RecordReferenceHandler } from "./PreparedContent";
import { SpellReference } from "./SpellReference";
import { PhysicalReference } from "./PhysicalReference";
import { ActionCost } from "./ActionCost";
import { CreatureReference } from "./CreatureReference";
import {
  ActorEncounterLayout,
  type ActorEncounterContent,
} from "./ActorEncounterLayout";
import { HazardDefenses } from "./HazardDefenses";
import type { EmbeddedReadingContext } from "./EmbeddedAbilityReading";
import { activityAddress } from "./embeddedNavigation";
import {
  ownerReturnPath,
  atlasRoutePath,
  navigateToAtlasPath,
  shouldHandleAtlasRouteClick,
} from "../../app/routes";
import { ChevronLeft } from "lucide-react";

const contentRoleLabels: Record<string, string> = {
  name: "Name",
  description: "Description",
  notes: "Notes",
  routine: "Routine",
  disable: "Disable",
  reset: "Reset",
  stealth: "Detection details",
  biography: "Biography",
  caption: "Caption",
  tableresult: "Table result",
};

export function ContentFields({
  fields,
  relationships = [],
  onReference,
  inlineAbility = false,
  hideDescriptionHeading = false,
  documentNavigation = false,
  sectionLanding,
}: {
  documentNavigation?: boolean;
  sectionLanding?: string;
  fields: PreparedContentFieldView[];
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
  inlineAbility?: boolean;
  hideDescriptionHeading?: boolean;
}) {
  const visibleFields = fields.filter(contentFieldVisible);
  const prerequisites = visibleFields.filter((field) => field.role === "prerequisite");
  return (
    <>
      {prerequisites.length > 0 && (
        <div className="record-fact record-reference__prerequisites">
          <strong>Prerequisites</strong>
          <div>
            {prerequisites.map((field) => (
              <div key={JSON.stringify(field.locator)}>
                <PreparedContent
                  content={field}
                  documentNavigation={documentNavigation}
                  sectionLanding={sectionLanding}
                  relationships={relationships}
                  onReference={onReference}
                />
              </div>
            ))}
          </div>
        </div>
      )}
      {visibleFields
        .filter((field) => field.role !== "prerequisite")
        .map((field) => (
          <section
            className="record-reference__content"
            key={JSON.stringify(field.locator)}
          >
            {hideDescriptionHeading &&
            field.role === "description" ? null : inlineAbility ? (
              field.role !== "description" && (
                <h5>{contentRoleLabels[field.role] ?? field.role}</h5>
              )
            ) : (
              <h3>{contentRoleLabels[field.role] ?? field.role}</h3>
            )}
            <PreparedContent
              content={field}
              documentNavigation={documentNavigation}
              sectionLanding={sectionLanding}
              relationships={relationships}
              onReference={onReference}
            />
          </section>
        ))}
    </>
  );
}

export function contentFieldVisible(field: PreparedContentFieldView) {
  if (field.body.kind === "unavailable" && field.body.state === "not_applicable")
    return false;
  if (field.body.kind === "plain") return field.body.text.trim().length > 0;
  if (field.body.kind === "html")
    return field.body.html.trim().length > 0 || field.body.controls.length > 0;
  return true;
}

export function RecordIdentity({
  identity,
  classification,
  rankControl,
  actionCost,
  root,
  selection,
  onOwnerReturn,
  visibleOwnerRecordKey,
  referenceRecord,
  referenceLabel,
  onReference,
  stacked = true,
  headingRef,
  headingId,
  actions,
  showOwnerReturn = true,
}: {
  identity?: RecordSummaryView;
  stacked?: boolean;
  headingRef?: Ref<HTMLHeadingElement>;
  headingId?: string;
  actions?: ReactNode;
  showOwnerReturn?: boolean;
  classification: string;
  rankControl?: ReactNode;
  actionCost?: FactView<ActionCostView>;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  referenceRecord?: RecordNavigationView;
  referenceLabel?: string;
  onReference: RecordReferenceHandler;
  visibleOwnerRecordKey?: string;
  onOwnerReturn?: (
    parent: string,
    landing: RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
}) {
  const traits = identity?.traits.filter((trait) => trait.kind !== "rarity") ?? [];
  const hasRarity = Boolean(identity?.rarity?.trim() && identity.rarity !== "common");
  const textCost =
    stacked && actionCost?.state === "value" && !actionCost.value?.glyph ? (
      <ActionCost fact={actionCost} />
    ) : null;
  return (
    <DocumentIdentityHeader
      title={
        <>
          {identity?.title ?? "Reference"}{" "}
          {actionCost?.state === "value" && (!stacked || actionCost.value?.glyph) && (
            <ActionCost fact={actionCost} />
          )}
        </>
      }
      classification={classification}
      classificationControl={
        rankControl || textCost ? (
          <>
            {rankControl}
            {textCost}
          </>
        ) : undefined
      }
      stacked={stacked}
      headingRef={headingRef}
      headingId={headingId}
      actions={actions}
      attribution={
        identity?.publication?.trim() && (
          <p className="muted record-reference__source">
            Source: <span>{identity.publication}</span>
          </p>
        )
      }
      navigation={
        showOwnerReturn &&
        root &&
        root.record_key !== visibleOwnerRecordKey &&
        selection?.embedded_path.length ? (
          <Button
            type="link"
            size="small"
            className="record-reference__owner"
            icon={<ChevronLeft size={14} aria-hidden="true" />}
            aria-label={`Return to ${root.title}`}
            href={ownerReturnPath(root.record_key, activityAddress(selection))}
            onClick={(event) => {
              if (!shouldHandleAtlasRouteClick(event)) return;
              event.preventDefault();
              if (onOwnerReturn)
                onOwnerReturn(
                  root.record_key,
                  activityAddress(selection),
                  event.currentTarget,
                );
              else
                navigateToAtlasPath(
                  ownerReturnPath(root.record_key, activityAddress(selection)),
                );
            }}
          >
            {root.title}
          </Button>
        ) : null
      }
    >
      {(hasRarity || traits.length > 0) && (
        <div className="badge-row">
          <RecordRarity rarity={identity?.rarity ?? null} />
          {traits.map((trait) => (
            <TraitBadge key={`${trait.kind}-${trait.value}`} trait={trait} />
          ))}
        </div>
      )}
      {referenceRecord && (
        <Typography.Link
          className="record-reference__reference-link"
          href={atlasRoutePath({
            kind: "record",
            recordKey: referenceRecord.record_key,
            selection: referenceRecord,
          })}
          onClick={(event) => {
            if (!shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            onReference(
              referenceRecord.record_key,
              event.currentTarget,
              referenceRecord,
            );
          }}
        >
          <BookOpen size={14} aria-hidden="true" />
          {referenceLabel ?? "View reference entry"}
        </Typography.Link>
      )}
    </DocumentIdentityHeader>
  );
}

export function RecordPresentationIdentity({
  presentation,
  root,
  selection,
  relationships = [],
  onReference,
  onRankChange,
  onOwnerReturn,
  visibleOwnerRecordKey,
  showOwnerReturn = true,
  stacked = true,
  headingRef,
  headingId,
  actions,
}: {
  presentation?: RecordPresentationView;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
  onRankChange?: (rank: number | undefined) => void;
  onOwnerReturn?: (
    parent: string,
    landing: RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
  visibleOwnerRecordKey?: string;
  showOwnerReturn?: boolean;
  stacked?: boolean;
  headingRef?: Ref<HTMLHeadingElement>;
  headingId?: string;
  actions?: ReactNode;
}) {
  const body = presentation?.body;
  const documentIdentity = stacked;
  return (
    <RecordIdentity
      stacked={documentIdentity}
      headingRef={headingRef}
      headingId={headingId}
      actions={actions}
      showOwnerReturn={showOwnerReturn}
      identity={presentation?.identity}
      classification={presentation?.classification ?? ""}
      rankControl={
        body?.kind === "spell" && onRankChange && body.value.base_rank.value != null ? (
          <SpellRankControl
            key={recordReadingIdentity(selection?.record_key ?? null, selection)}
            spell={body.value}
            embedded={Boolean(selection?.embedded_path.length)}
            onChange={onRankChange}
          />
        ) : undefined
      }
      actionCost={body?.kind === "activity" ? body.value.action_cost : undefined}
      visibleOwnerRecordKey={visibleOwnerRecordKey}
      root={root}
      selection={selection}
      onOwnerReturn={onOwnerReturn}
      onReference={onReference}
      referenceLabel={
        body?.kind === "spell"
          ? "View reference spell"
          : body?.kind === "physical_reference"
            ? "View reference item"
            : body?.kind === "activity"
              ? "View reference ability"
              : "View reference entry"
      }
      referenceRecord={
        selection?.embedded_path.length
          ? (relationships.find(
              (relationship) =>
                relationship.origin === "structured" &&
                relationship.kind === "CompendiumProvenance" &&
                relationship.status === "resolved_record",
            )?.target ?? undefined)
          : undefined
      }
    />
  );
}

export function RecordPresentation({
  presentation,
  root,
  selection,
  relationships = [],
  onReference,
  onRankChange,
  compact = false,
  encounter = false,
  castingAnnotations,
  activityRolls,
  hideIdentity = false,
  identityInHost = false,
  showOwnerReturn = true,
  reading,
  onOwnerReturn,
  visibleOwnerRecordKey,
  sectionLanding,
  actorEncounter,
}: {
  actorEncounter?: ActorEncounterContent;
  sectionLanding?: string;
  presentation: RecordPresentationView;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
  onRankChange?: (rank: number | undefined) => void;
  compact?: boolean;
  encounter?: boolean;
  castingAnnotations?: import("./ActorReference").CastingAnnotations;
  activityRolls?: (
    activity: import("../../generated/atlas").ActorActivityView,
  ) => ReactNode;
  hideIdentity?: boolean;
  identityInHost?: boolean;
  showOwnerReturn?: boolean;
  reading?: EmbeddedReadingContext;
  visibleOwnerRecordKey?: string;
  onOwnerReturn?: (
    parent: string,
    landing: RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
}) {
  const body = presentation.body;
  const descriptions = presentation.content.filter(
    (field) => field.role === "description" && contentFieldVisible(field),
  );
  const actor =
    body.kind === "creature"
      ? body.value
      : body.kind === "hazard"
        ? body.value.actor
        : null;
  const content = (
    <ContentFields
      fields={
        body.kind === "hazard"
          ? presentation.content.filter((field) => field.role !== "stealth")
          : presentation.content
      }
      relationships={relationships}
      onReference={onReference}
    />
  );
  const usedOwned =
    actor?.activities.value?.map((activity) => JSON.stringify(activity.navigation)) ??
    [];
  const embedded = (
    body.kind === "journal" ? (body.value.pages.value ?? []) : presentation.embedded
  ).filter((link) => !usedOwned.includes(JSON.stringify(link.navigation)));
  const flavorFields = presentation.content.filter(
    (field) =>
      contentFieldVisible(field) &&
      (body.kind !== "hazard" || field.role === "description"),
  );
  const flavor =
    flavorFields.length > 0 ? (
      <div className="record-reference__flavor">
        <ContentFields
          fields={flavorFields}
          relationships={relationships}
          onReference={onReference}
          documentNavigation={body.kind === "journal_page"}
          sectionLanding={sectionLanding}
          hideDescriptionHeading={
            (body.kind === "journal_page" &&
              body.value.show_body_heading.value === false) ||
            (body.kind !== "creature" &&
              descriptions.length === 1 &&
              descriptions[0].body.kind !== "unavailable")
          }
        />
      </div>
    ) : null;
  return (
    <article
      className={`record-reference${compact ? " record-reference--compact" : ""}${encounter ? " record-reference--encounter" : ""}`}
    >
      {!hideIdentity && !identityInHost && (
        <RecordPresentationIdentity
          presentation={presentation}
          root={root}
          selection={selection}
          relationships={relationships}
          onReference={onReference}
          onRankChange={onRankChange}
          onOwnerReturn={onOwnerReturn}
          visibleOwnerRecordKey={visibleOwnerRecordKey}
          showOwnerReturn={showOwnerReturn}
        />
      )}
      {body.kind === "spell" ? (
        <SpellReference spell={body.value}>{!encounter && flavor}</SpellReference>
      ) : body.kind === "physical_reference" ? (
        <PhysicalReference
          item={body.value}
          identity={presentation.identity}
          onReference={onReference}
        >
          {!encounter && flavor}
        </PhysicalReference>
      ) : (
        body.kind !== "activity" &&
        body.kind !== "creature" && (
          <ReferenceProfiles body={body} onReference={onReference}>
            {!encounter && flavor}
          </ReferenceProfiles>
        )
      )}
      {body.kind === "creature" && (
        <CreatureReference
          actor={body.value}
          recordKey={presentation.identity.record_key}
          prose={flavor}
          encounter={encounter}
          encounterContent={actorEncounter}
          reading={reading}
          onReference={onReference}
          activityRolls={activityRolls}
          castingAnnotations={castingAnnotations}
          sectionLanding={sectionLanding}
        />
      )}
      {body.kind === "hazard" && (
        <ActorEncounterLayout
          durability={actorEncounter?.durability}
          runtime={
            actorEncounter ? (
              <>
                <ActorCombatBudget actor={body.value.actor} />
                {actorEncounter.conditions}
              </>
            ) : null
          }
        >
          <div className="hazard-reference__rules">
            <section className="hazard-reference__detection">
              <h3>Detection</h3>
              <div className="reference-profile__facts">
                <ReferenceFact
                  label="Complexity"
                  fact={body.value.complex}
                  format={(complex) => (complex ? "Complex" : "Simple")}
                />
                <ReferenceFact label="Size" fact={body.value.actor.size} optional />
                <ReferenceFact
                  label="Stealth"
                  fact={body.value.stealth}
                  format={(value) => `${Number(value) >= 0 ? "+" : ""}${value}`}
                />
              </div>
              {presentation.content
                .filter(
                  (field) => field.role === "stealth" && contentFieldVisible(field),
                )
                .map((field) => (
                  <PreparedContent
                    key={JSON.stringify(field.locator)}
                    content={field}
                    relationships={relationships}
                    onReference={onReference}
                  />
                ))}
            </section>
            {
              <ContentFields
                fields={presentation.content.filter(
                  (field) => field.role === "disable",
                )}
                relationships={relationships}
                onReference={onReference}
              />
            }
            {!actorEncounter && <HazardDefenses hazard={body.value} />}
            {encounter && !actorEncounter && (
              <ActorCombatBudget actor={body.value.actor} />
            )}
            {encounter && <ActorSecondaryFacts actor={body.value.actor} hazard />}
            <ActorActivities
              castingAnnotations={castingAnnotations}
              essential
              actor={body.value.actor}
              activityRolls={activityRolls}
              onReference={onReference}
              encounter={encounter}
              reading={reading}
            />
            <ContentFields
              fields={presentation.content.filter(
                (field) =>
                  ![
                    "disable",
                    "stealth",
                    ...(!encounter ? ["description"] : []),
                  ].includes(field.role),
              )}
              relationships={relationships}
              onReference={onReference}
            />
          </div>
        </ActorEncounterLayout>
      )}
      {body.kind === "activity" && (
        <>
          {" "}
          <ActorActivity
            activity={body.value}
            onReference={onReference}
            selected
            hideCost={
              (!hideIdentity || identityInHost) &&
              body.value.action_cost.state === "value"
            }
          />
          {!encounter && flavor}
        </>
      )}
      {body.kind === "roll_table" &&
        (selection && reading?.generation ? (
          <RollTableReading
            key={JSON.stringify([selection, reading.generation])}
            table={body.value}
            selection={selection}
            context={reading}
            onReference={onReference}
          />
        ) : (
          <FactLine label="Formula" fact={body.value.formula} required />
        ))}
      {body.kind === "journal" &&
        (body.value.pages.value ? (
          body.value.pages.value.length === 0 && <p>No pages.</p>
        ) : (
          <p>Pages unavailable {factText(body.value.pages)}</p>
        ))}
      {body.kind === "journal_page" && body.value.media_notice && (
        <p>{body.value.media_notice}</p>
      )}
      {body.kind === "table_result" && (
        <div className="record-fact-list">
          <FactLine label="Range" fact={body.value.range} required />
          <FactLine label="Weight" fact={body.value.weight} />
          <FactLine label="Result type" fact={body.value.result_type} required />
        </div>
      )}
      {encounter &&
        body.kind !== "creature" &&
        body.kind !== "hazard" &&
        presentation.content.some(contentFieldVisible) && (
          <Collapse
            size="small"
            ghost
            items={[{ key: "content", label: "Source description", children: content }]}
          />
        )}
      {body.kind === "roll_table" && body.value.results.state !== "value" && (
        <FactLine label="Results" fact={body.value.results} />
      )}
      {body.kind === "journal" && (
        <JournalContents
          key={recordReadingIdentity(presentation.identity.record_key, selection)}
          pages={body.value.pages}
          onReference={onReference}
        />
      )}
      {body.kind !== "journal" &&
        !(body.kind === "roll_table" && selection && reading?.generation) &&
        (body.kind === "roll_table"
          ? (body.value.results.value ?? []).map((row) => row.link)
          : embedded
        ).length > 0 && (
          <section className="record-reference__embedded">
            <h3>
              {body.kind === "roll_table"
                ? "Results"
                : embedded.every(
                      (link) =>
                        link.navigation.embedded_path[
                          link.navigation.embedded_path.length - 1
                        ]?.collection === "/pages",
                    )
                  ? "Pages"
                  : "Contents"}
            </h3>
            {(body.kind === "roll_table"
              ? (body.value.results.value ?? []).map((row) => row.link)
              : embedded
            ).map((link, index) => (
              <div key={index}>
                <Button
                  type="link"
                  href={atlasRoutePath({
                    kind: "record",
                    recordKey: link.navigation.record_key,
                    selection: link.navigation,
                  })}
                  onClick={(event) => {
                    if (!shouldHandleAtlasRouteClick(event)) return;
                    event.preventDefault();
                    onReference(
                      link.navigation.record_key,
                      event.currentTarget,
                      link.navigation,
                    );
                  }}
                >
                  {link.title}
                </Button>
                <ActionCost fact={link.action_cost} />
                {link.usage && !isEmptyFactValue(link.usage) && (
                  <span className="muted"> {link.usage}</span>
                )}
              </div>
            ))}
          </section>
        )}
    </article>
  );
}
