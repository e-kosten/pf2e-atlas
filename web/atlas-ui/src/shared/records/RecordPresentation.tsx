import {
  ownerReturnHref,
  isPrimaryActivation,
  navigatePath,
  destinationHref,
} from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { ReferenceProfiles } from "./ReferenceProfiles";
import { JournalContents } from "./JournalContents";
import { DocumentIdentityHeader } from "./DocumentIdentityHeader";
import { TraitBadge } from "./TraitBadge";
import { RollTableReading } from "./RollTableReading";
import { RecordRarity } from "./RecordRarity";
import { SpellRankControl } from "./SpellRankControl";
import { SpellRankValue } from "./SpellRankValue";
import { recordReadingIdentity } from "./useRetainedRecord";
import { RecordProvenanceAction } from "./RecordProvenanceAction";
import type { ReactNode, Ref } from "react";
import { Button, Collapse, Space } from "antd";
import type {
  PreparedContentFieldView,
  RecordNavigationView,
  RecordPresentationView,
  RecordRelationshipView,
  RecordSummaryView,
  ActionCostView,
  FactView,
} from "../../generated/atlas";
import { SpellEffectPreview } from "./SpellEffectPreview";
import { ActorActivity } from "./ActorReference";
import { FactLine, factText, isEmptyFactValue } from "./presentationFacts";
import { PreparedContent, type RecordReferenceHandler } from "./PreparedContent";
import { SpellReference } from "./SpellReference";
import { PhysicalReference } from "./PhysicalReference";
import { ActionCost } from "./ActionCost";
import { CreatureReference } from "./CreatureReference";
import type { ActorEncounterContent } from "./ActorEncounterLayout";
import { HazardReference } from "./HazardReference";
import type { EmbeddedReadingContext } from "./EmbeddedAbilityReading";
import { activityAddress } from "./embeddedNavigation";
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
  introduction,
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
  introduction?: PreparedContentFieldView;
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
            {field === introduction ||
            (hideDescriptionHeading &&
              field.role === "description") ? null : inlineAbility ? (
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
export function recordClassification(presentation: RecordPresentationView) {
  if (
    presentation.body.kind === "hazard" &&
    presentation.body.value.complex.state === "value"
  ) {
    return `${presentation.body.value.complex.value ? "Complex" : "Simple"} ${presentation.classification}`;
  }
  return presentation.classification;
}
type IdentityActions = ReactNode | ((secondaryActions: ReactNode) => ReactNode);
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
  referenceTitle,
  onReference,
  stacked = true,
  headingRef,
  headingId,
  actions,
  showOwnerReturn = true,
  badgeExplanations,
}: {
  identity?: RecordSummaryView;
  stacked?: boolean;
  headingRef?: Ref<HTMLHeadingElement>;
  headingId?: string;
  actions?: IdentityActions;
  showOwnerReturn?: boolean;
  badgeExplanations?: ReactNode;
  classification: string;
  rankControl?: ReactNode;
  actionCost?: FactView<ActionCostView>;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  referenceRecord?: RecordNavigationView;
  referenceTitle?: string;
  onReference: RecordReferenceHandler;
  visibleOwnerRecordKey?: string;
  onOwnerReturn?: (
    parent: string,
    landing: RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
}) {
  const traits = identity?.traits.filter((trait) => trait.kind !== "rarity") ?? [];
  const hasRarity = Boolean(identity?.rarity?.trim());
  const secondaryActions = referenceRecord ? (
    <RecordProvenanceAction
      navigation={referenceRecord}
      title={referenceTitle}
      onReference={onReference}
    />
  ) : null;
  return (
    <DocumentIdentityHeader
      title={
        <>
          {identity?.title ?? "Reference"}{" "}
          {actionCost?.state === "value" && actionCost.value?.glyph && (
            <ActionCost fact={actionCost} />
          )}
        </>
      }
      classification={classification}
      classificationAction={rankControl}
      stacked={stacked}
      headingRef={headingRef}
      headingId={headingId}
      actions={
        typeof actions === "function" ? (
          actions(secondaryActions)
        ) : secondaryActions && actions ? (
          <Space wrap align="start">
            {secondaryActions}
            {actions}
          </Space>
        ) : (
          (actions ?? secondaryActions)
        )
      }
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
            href={ownerReturnHref(root.record_key, activityAddress(selection))}
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              if (onOwnerReturn)
                onOwnerReturn(
                  root.record_key,
                  activityAddress(selection),
                  event.currentTarget,
                );
              else
                navigatePath(
                  ownerReturnHref(root.record_key, activityAddress(selection)),
                );
            }}
          >
            {root.title}
          </Button>
        ) : null
      }
    >
      {(hasRarity || traits.length > 0 || badgeExplanations) && (
        <div className="badge-row">
          <RecordRarity rarity={identity?.rarity ?? null} />
          {traits.map((trait) => (
            <TraitBadge key={`${trait.kind}-${trait.value}`} trait={trait} />
          ))}
          {badgeExplanations}
        </div>
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
  actions?: IdentityActions;
}) {
  const body = presentation?.body;
  const identityChanges =
    body?.kind === "spell"
      ? body.value.rank_changes?.filter((change) =>
          ["traits", "rarity"].includes(change.fact.kind),
        )
      : undefined;
  const documentIdentity = stacked;
  const provenance = selection?.embedded_path.length
    ? relationships.find(
        (relationship) =>
          relationship.origin === "structured" &&
          relationship.kind === "CompendiumProvenance" &&
          relationship.status === "resolved_record",
      )
    : undefined;
  return (
    <RecordIdentity
      stacked={documentIdentity}
      headingRef={headingRef}
      headingId={headingId}
      actions={actions}
      showOwnerReturn={showOwnerReturn}
      identity={presentation?.identity}
      badgeExplanations={
        identityChanges?.length
          ? identityChanges.map((change) => (
              <SpellRankValue
                key={change.fact.kind}
                change={change}
                label={change.fact.kind === "traits" ? "Traits" : "Rarity"}
              >
                {change.fact.kind === "traits" ? "Trait changes" : "Rarity change"}
              </SpellRankValue>
            ))
          : undefined
      }
      classification={presentation ? recordClassification(presentation) : ""}
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
      referenceTitle={provenance?.target_title}
      referenceRecord={provenance?.target ?? undefined}
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
          introduction={
            body.kind === "creature" &&
            !encounter &&
            flavorFields[0].role === "notes" &&
            ["html", "plain"].includes(flavorFields[0].body.kind)
              ? flavorFields[0]
              : undefined
          }
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
        body.kind !== "creature" &&
        body.kind !== "hazard" && (
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
        <HazardReference
          hazard={body.value}
          encounter={encounter}
          encounterContent={actorEncounter}
          reading={reading}
          onReference={onReference}
          activityRolls={activityRolls}
          castingAnnotations={castingAnnotations}
          content={{
            description: flavor,
            detection: presentation.content
              .filter((field) => field.role === "stealth" && contentFieldVisible(field))
              .map((field) => (
                <PreparedContent
                  key={JSON.stringify(field.locator)}
                  content={field}
                  relationships={relationships}
                  onReference={onReference}
                />
              )),
            disable: (
              <ContentFields
                fields={presentation.content.filter(
                  (field) => field.role === "disable",
                )}
                relationships={relationships}
                onReference={onReference}
              />
            ),
            routine: (
              <ContentFields
                fields={presentation.content.filter(
                  (field) => field.role === "routine",
                )}
                relationships={relationships}
                onReference={onReference}
              />
            ),
            reset: (
              <ContentFields
                fields={presentation.content.filter((field) => field.role === "reset")}
                relationships={relationships}
                onReference={onReference}
              />
            ),
            other: (
              <ContentFields
                fields={presentation.content.filter(
                  (field) =>
                    !["description", "stealth", "disable", "routine", "reset"].includes(
                      field.role,
                    ),
                )}
                relationships={relationships}
                onReference={onReference}
              />
            ),
          }}
        />
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
              body.value.action_cost.state === "value" &&
              Boolean(body.value.action_cost.value?.glyph)
            }
          />
          {body.value.spell_effect && (
            <SpellEffectPreview view={body.value.spell_effect} />
          )}
          {!encounter &&
            (body.value.spell_effect && !body.value.spell_effect.unavailable ? (
              <section>
                <h3>Rules</h3>
                {flavor}
              </section>
            ) : (
              flavor
            ))}
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
                  href={destinationHref(
                    recordDestination({
                      recordKey: link.navigation.record_key,
                      selection: link.navigation,
                    }),
                  )}
                  onClick={(event) => {
                    if (!isPrimaryActivation(event)) return;
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
