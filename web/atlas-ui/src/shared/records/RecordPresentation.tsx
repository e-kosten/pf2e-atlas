import { ReferenceProfiles, ReferenceFact } from "./ReferenceProfiles";
import { JournalContents } from "./JournalContents";
import { DocumentIdentityHeader } from "./DocumentIdentityHeader";
import { TraitBadge } from "./TraitBadge";
import { RollTableReading } from "./RollTableReading";
import { RecordRarity } from "./RecordRarity";
import { SpellRankControl } from "./SpellRankControl";
import { recordReadingIdentity } from "./useRetainedRecord";
import type { ReactNode, Ref } from "react";
import { Button, Collapse } from "antd";
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
  ActorFacts,
  ActorSecondaryFacts,
  Iwr,
} from "./ActorReference";
import { FactLine, factText, isEmptyFactValue } from "./presentationFacts";
import { PreparedContent, type RecordReferenceHandler } from "./PreparedContent";
import { SpellReference } from "./SpellReference";
import { PhysicalReference } from "./PhysicalReference";
import { ActionCost } from "./ActionCost";
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
  originalRecord,
  onReference,
  stacked = false,
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
  originalRecord?: RecordNavigationView;
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
      identityActions={
        originalRecord && (
          <Button
            className="record-reference__original"
            type="link"
            size="small"
            href={atlasRoutePath({
              kind: "record",
              recordKey: originalRecord.record_key,
              selection: originalRecord,
            })}
            onClick={(event) => {
              if (!shouldHandleAtlasRouteClick(event)) return;
              event.preventDefault();
              onReference(
                originalRecord.record_key,
                event.currentTarget,
                originalRecord,
              );
            }}
          >
            View original
          </Button>
        )
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
  stacked = false,
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
  const documentIdentity = stacked || body?.kind !== "creature";
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
      originalRecord={
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
}: {
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
        body.kind !== "activity" && (
          <ReferenceProfiles body={body} onReference={onReference}>
            {!encounter && flavor}
          </ReferenceProfiles>
        )
      )}
      {body.kind === "creature" && encounter && (
        <ActorSecondaryFacts actor={body.value} />
      )}
      {body.kind === "creature" && (
        <ActorFacts
          actor={body.value}
          hideHitPoints={encounter}
          encounter={encounter}
        />
      )}
      {body.kind === "hazard" && (
        <>
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
              .filter((field) => field.role === "stealth" && contentFieldVisible(field))
              .map((field) => (
                <PreparedContent
                  key={JSON.stringify(field.locator)}
                  content={field}
                  relationships={relationships}
                  onReference={onReference}
                />
              ))}
          </section>
          {!encounter && (
            <ContentFields
              fields={presentation.content.filter((field) => field.role === "disable")}
              relationships={relationships}
              onReference={onReference}
            />
          )}
          {body.value.defense_block.value !== false && (
            <section className="reference-profile__section">
              <h3>Defenses</h3>
              <div className="reference-profile__groups">
                <div className="reference-profile__facts">
                  <ReferenceFact label="AC" fact={body.value.actor.armor_class} />
                  <ReferenceFact label="HP" fact={body.value.actor.maximum_hp} />
                  <ReferenceFact label="Hardness" fact={body.value.hardness} />
                </div>
                {body.value.show_saves && (
                  <div className="reference-profile__facts">
                    {(["fortitude", "reflex", "will"] as const).map((save) => (
                      <ReferenceFact
                        key={save}
                        label={save[0].toUpperCase() + save.slice(1)}
                        fact={body.value.actor.saves[save]}
                        format={(value) => `${Number(value) >= 0 ? "+" : ""}${value}`}
                      />
                    ))}
                  </div>
                )}
                <Iwr label="Immunities" fact={body.value.actor.immunities} />
                <Iwr label="Weaknesses" fact={body.value.actor.weaknesses} />
                <Iwr label="Resistances" fact={body.value.actor.resistances} />
                <ReferenceFact
                  label="AC details"
                  fact={body.value.actor.defense_notes.armor_class}
                  optional
                />
                <ReferenceFact
                  label="HP details"
                  fact={body.value.actor.defense_notes.hit_points}
                  optional
                />
                <ReferenceFact
                  label="Save details"
                  fact={body.value.actor.defense_notes.all_saves}
                  optional
                />
                {(["fortitude", "reflex", "will"] as const).map((save) => (
                  <ReferenceFact
                    key={save}
                    label={`${save} details`}
                    fact={body.value.actor.defense_notes[save]}
                    optional
                  />
                ))}
              </div>
            </section>
          )}
        </>
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
      {body.kind === "hazard" && encounter && (
        <ActorSecondaryFacts actor={body.value.actor} hazard />
      )}
      {actor && (
        <ActorActivities
          castingAnnotations={castingAnnotations}
          essential={body.kind === "hazard"}
          actor={actor}
          activityRolls={activityRolls}
          onReference={onReference}
          encounter={encounter}
          reading={reading}
        />
      )}
      {!encounter && body.kind === "hazard" && (
        <ContentFields
          fields={presentation.content.filter(
            (field) => !["disable", "stealth", "description"].includes(field.role),
          )}
          relationships={relationships}
          onReference={onReference}
        />
      )}
      {encounter && presentation.content.some(contentFieldVisible) && (
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
        <JournalContents pages={body.value.pages} onReference={onReference} />
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
