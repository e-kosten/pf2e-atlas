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
}: {
  fields: PreparedContentFieldView[];
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
}) {
  const visibleFields = fields.filter(contentFieldVisible);
  return (
    <>
      {visibleFields.map((field) => (
        <section
          className="record-reference__content"
          key={JSON.stringify(field.locator)}
        >
          <h3>{contentRoleLabels[field.role] ?? field.role}</h3>
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
  onReference,
}: {
  identity: RecordSummaryView;
  classification: string;
  actionCost?: FactView<ActionCostView>;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  onReference: RecordReferenceHandler;
}) {
  return (
    <header className="record-reference__header">
      <div className="record-reference__title">
        <h2>
          {identity.title} {actionCost && <ActionCost fact={actionCost} />}
        </h2>
        <p className="record-reference__kind">{classification}</p>
      </div>
      <div className="badge-row">
        {identity.rarity?.trim() && <Tag>{identity.rarity}</Tag>}
        {identity.traits.map((trait) => (
          <Tag key={`${trait.kind}-${trait.value}`}>{trait.label}</Tag>
        ))}
      </div>
      {identity.publication?.trim() && <p className="muted">{identity.publication}</p>}
      {root && selection?.owners.length ? (
        <Button
          type="link"
          onClick={(event) =>
            onReference(root.record_key, event.currentTarget, {
              record_key: root.record_key,
              owners: [],
              field: null,
              passage: null,
              source_fingerprint: selection.source_fingerprint,
            })
          }
        >
          From {root.title}
        </Button>
      ) : null}
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
  hideActivityCost = false,
}: {
  presentation: RecordPresentationView;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
  compact?: boolean;
  encounter?: boolean;
  hideIdentity?: boolean;
  hideActivityCost?: boolean;
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
          compact={compact}
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
          {!compact && !encounter && (
            <ContentFields
              fields={presentation.content.filter((field) => field.role === "disable")}
              relationships={relationships}
              onReference={onReference}
            />
          )}
          <ActorFacts
            actor={body.value.actor}
            compact={compact}
            hideHitPoints={encounter}
            encounter={encounter}
            hazard
            hidePrimaryDefenses={
              body.value.defense_block.state === "value" &&
              body.value.defense_block.value === false
            }
          />
          {!(
            body.value.defense_block.state === "value" &&
            body.value.defense_block.value === false
          ) ? (
            <FactRow label="Hardness">
              <NumberFact fact={body.value.hardness} />
            </FactRow>
          ) : (
            <Collapse
              size="small"
              ghost
              items={[
                {
                  key: "authored-defenses",
                  label: "Defense details",
                  children: (
                    <div className="record-fact-list">
                      <FactRow label="AC">
                        <NumberFact fact={body.value.actor.armor_class} />
                      </FactRow>
                      <FactRow label="HP">
                        <NumberFact fact={body.value.actor.maximum_hp} />
                      </FactRow>
                      <FactRow label="Hardness">
                        <NumberFact fact={body.value.hardness} />
                      </FactRow>
                    </div>
                  ),
                },
              ]}
            />
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
      {body.kind === "spell" && <SpellReference spell={body.value} compact={compact} />}
      {body.kind === "activity" && (
        <ActorActivity
          activity={body.value}
          onReference={onReference}
          selected
          hideCost={!hideIdentity || hideActivityCost}
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
      {actor && !compact && <ActorActivities actor={actor} onReference={onReference} />}
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
      {compact && !encounter && body.kind === "hazard" && (
        <ContentFields
          fields={presentation.content.filter((field) => field.role === "disable")}
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
      {!compact &&
        (body.kind === "roll_table" ? (body.value.results.value ?? []) : owned).length >
          0 && (
          <section className="record-reference__owned">
            <h3>
              {body.kind === "roll_table"
                ? "Results"
                : presentation.identity.kind === "journal"
                  ? "Pages"
                  : "Owned records"}
            </h3>
            {(body.kind === "roll_table"
              ? (body.value.results.value ?? [])
              : owned
            ).map((link, index) => (
              <div key={index}>
                <Button
                  type="link"
                  onClick={(event) =>
                    onReference(
                      link.navigation.record_key,
                      event.currentTarget,
                      link.navigation,
                    )
                  }
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
