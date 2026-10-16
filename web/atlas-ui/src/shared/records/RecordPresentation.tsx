import { sentenceCase } from "../ui/labels";
import { Button, Collapse, Tag } from "antd";
import type {
  PreparedContentFieldView,
  RecordNavigationView,
  RecordPresentationView,
  RecordRelationshipView,
  RecordSummaryView,
  ActionCostView,
  FactView,
  PhysicalPriceView,
} from "../../generated/atlas";
import {
  ActorActivities,
  ActorActivity,
  ActorFacts,
  ActorSecondaryFacts,
} from "./ActorReference";
import {
  FactLine,
  FactRow,
  NumberFact,
  factText,
  hasReadingFact,
  isEmptyFactValue,
} from "./presentationFacts";
import { PreparedContent, type RecordReferenceHandler } from "./PreparedContent";
import { SpellReference } from "./SpellReference";
import { ActionCost } from "./ActionCost";
import type { OwnedReadingContext } from "./OwnedAbilityReading";
import { activityAddress } from "./ownedNavigation";
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
}: {
  fields: PreparedContentFieldView[];
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
  inlineAbility?: boolean;
}) {
  const visibleFields = fields.filter(contentFieldVisible);
  return (
    <>
      {visibleFields.map((field) => (
        <section
          className="record-reference__content"
          key={JSON.stringify(field.locator)}
        >
          {inlineAbility ? (
            field.role !== "description" && (
              <h5>{contentRoleLabels[field.role] ?? field.role}</h5>
            )
          ) : (
            <h3>{contentRoleLabels[field.role] ?? field.role}</h3>
          )}
          <PreparedContent
            content={field}
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
  actionCost,
  root,
  selection,
  onOwnerReturn,
  originalRecord,
  onReference,
}: {
  identity: RecordSummaryView;
  classification: string;
  actionCost?: FactView<ActionCostView>;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  originalRecord?: RecordNavigationView;
  onReference?: RecordReferenceHandler;
  onOwnerReturn?: (
    parent: string,
    landing: RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
}) {
  return (
    <header className="record-reference__header">
      {root && selection?.owners.length ? (
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
      ) : null}
      {originalRecord && (
        <Button
          type="link"
          size="small"
          href={atlasRoutePath({
            kind: "record",
            recordKey: originalRecord.record_key,
          })}
          onClick={(event) => {
            if (!onReference || !shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            onReference(originalRecord.record_key, event.currentTarget, originalRecord);
          }}
        >
          Original record
        </Button>
      )}
      <div className="record-reference__title">
        <h2>
          {identity.title} {actionCost && <ActionCost fact={actionCost} />}
        </h2>
        <p className="record-reference__kind">{classification}</p>
      </div>
      <div className="badge-row">
        {identity.rarity?.trim() && identity.rarity !== "common" && (
          <span className="record-reference__rarity">
            {sentenceCase(identity.rarity)}
          </span>
        )}
        {identity.traits.map((trait) => (
          <Tag key={`${trait.kind}-${trait.value}`}>{sentenceCase(trait.label)}</Tag>
        ))}
      </div>
      {identity.publication?.trim() && <p className="muted">{identity.publication}</p>}
    </header>
  );
}

export function RecordPresentation({
  presentation,
  root,
  selection,
  relationships = [],
  onReference,
  compact = false,
  encounter = false,
  hideIdentity = false,
  reading,
  onOwnerReturn,
}: {
  presentation: RecordPresentationView;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
  compact?: boolean;
  encounter?: boolean;
  hideIdentity?: boolean;
  reading?: OwnedReadingContext;
  onOwnerReturn?: (
    parent: string,
    landing: RecordNavigationView,
    anchor: HTMLElement,
  ) => void;
}) {
  const body = presentation.body;
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
  const owned = presentation.owned.filter(
    (link) => !usedOwned.includes(JSON.stringify(link.navigation)),
  );
  return (
    <article
      className={`record-reference${compact ? " record-reference--compact" : ""}${encounter ? " record-reference--encounter" : ""}`}
    >
      {!hideIdentity && (
        <RecordIdentity
          identity={presentation.identity}
          classification={presentation.classification}
          actionCost={body.kind === "activity" ? body.value.action_cost : undefined}
          root={root}
          selection={selection}
          onOwnerReturn={onOwnerReturn}
          originalRecord={
            selection?.owners.length
              ? (relationships.find(
                  (relationship) =>
                    relationship.origin === "structured" &&
                    relationship.kind === "CompendiumProvenance" &&
                    relationship.status === "resolved_record",
                )?.target ?? undefined)
              : undefined
          }
          onReference={onReference}
        />
      )}
      {!encounter && (
        <div className="record-reference__flavor">
          <ContentFields
            fields={
              body.kind === "hazard"
                ? presentation.content.filter((field) => field.role === "description")
                : presentation.content
            }
            relationships={relationships}
            onReference={onReference}
          />
        </div>
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
            <FactRow label="Hazard">
              {factText(body.value.complex, (complex) =>
                complex ? "Complex" : "Simple",
              )}
            </FactRow>
            <FactRow label="Stealth">
              <NumberFact fact={body.value.stealth} signed />
            </FactRow>
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
          <ActorFacts
            actor={body.value.actor}
            hideHitPoints={encounter}
            encounter={encounter}
            hazard
            hideSaves={!body.value.show_saves}
            hidePrimaryDefenses={
              body.value.defense_block.state === "value" &&
              body.value.defense_block.value === false
            }
          />
          {!(
            body.value.defense_block.state === "value" &&
            body.value.defense_block.value === false
          ) && (
            <FactRow label="Hardness">
              <NumberFact fact={body.value.hardness} />
            </FactRow>
          )}
          {body.value.defense_block.state !== "value" && (
            <FactRow label="Defense block">
              {factText(body.value.defense_block, (value) =>
                value ? "Present" : "Absent",
              )}
            </FactRow>
          )}
        </>
      )}
      {body.kind === "spell" && <SpellReference spell={body.value} />}
      {body.kind === "activity" && (
        <ActorActivity
          activity={body.value}
          onReference={onReference}
          selected
          hideCost={!hideIdentity}
        />
      )}
      {body.kind === "physical_reference" && (
        <div className="record-fact-list">
          <FactLine label="Usage" fact={body.value.usage} />
          <PhysicalPrice fact={body.value.price} />
          {body.value.price_per.state !== "missing" && (
            <FactLine label="Price per" fact={body.value.price_per} />
          )}
          <FactLine label="Bulk" fact={body.value.bulk} />
        </div>
      )}
      {body.kind === "roll_table" && (
        <FactLine label="Formula" fact={body.value.formula} required />
      )}
      {body.kind === "table_result" && (
        <div className="record-fact-list">
          <FactLine label="Range" fact={body.value.range} required />
          <FactLine label="Weight" fact={body.value.weight} />
          <FactLine label="Result type" fact={body.value.result_type} required />
        </div>
      )}
      {actor && (
        <ActorActivities
          actor={actor}
          onReference={onReference}
          encounter={encounter}
          reading={reading}
        />
      )}
      {actor && encounter && (
        <ActorSecondaryFacts actor={actor} hazard={body.kind === "hazard"} />
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
      {(body.kind === "roll_table" ? (body.value.results.value ?? []) : owned).length >
        0 && (
        <section className="record-reference__owned">
          <h3>
            {body.kind === "roll_table"
              ? "Results"
              : presentation.identity.kind === "journal"
                ? "Pages"
                : "Owned records"}
          </h3>
          {(body.kind === "roll_table" ? (body.value.results.value ?? []) : owned).map(
            (link, index) => (
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
            ),
          )}
        </section>
      )}
    </article>
  );
}
function PhysicalPrice({ fact }: { fact: FactView<PhysicalPriceView> }) {
  if (!hasReadingFact(fact, { required: true })) return null;
  if (fact.state !== "value" || !fact.value)
    return <FactLine label="Price" fact={fact} required />;
  const coins = Object.entries(fact.value).filter(
    ([, value]) => !["missing", "not_applicable"].includes(value.state),
  );
  return (
    <FactRow label="Price">
      {coins.length
        ? coins.map(([unit, value], index) => (
            <span key={unit}>
              {index ? ", " : ""}
              <NumberFact fact={value} /> {unit}
            </span>
          ))
        : "Empty"}
    </FactRow>
  );
}
