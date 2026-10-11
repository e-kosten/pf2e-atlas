import { Button, Collapse, Tag } from "antd";
import type {
  PreparedContentFieldView,
  RecordNavigationView,
  RecordPresentationView,
  RecordRelationshipView,
  RecordSummaryView,
} from "../../generated/atlas";
import { ActorActivities, ActorActivity, ActorFacts } from "./ActorReference";
import { FactLine, FactRow, NumberFact, factText } from "./presentationFacts";
import { PreparedContent, type RecordReferenceHandler } from "./PreparedContent";
import { SpellReference } from "./SpellReference";

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
  return (
    <>
      {fields.map((field) => (
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

export function RecordIdentity({
  identity,
  root,
  selection,
  onReference,
}: {
  identity: RecordSummaryView;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  onReference: RecordReferenceHandler;
}) {
  return (
    <header className="record-reference__header">
      <div className="record-reference__title">
        <h2>{identity.title}</h2>
        <span>
          {identity.kind_label}
          {identity.level_label ? ` ${identity.level_label}` : ""}
        </span>
      </div>
      <div className="badge-row">
        {identity.rarity && <Tag>{identity.rarity}</Tag>}
        {identity.traits.map((trait) => (
          <Tag key={`${trait.kind}-${trait.value}`}>{trait.label}</Tag>
        ))}
      </div>
      {identity.publication && <p className="muted">{identity.publication}</p>}
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
      <p className="record-key">{identity.record_key}</p>
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
}: {
  presentation: RecordPresentationView;
  root?: RecordSummaryView;
  selection?: RecordNavigationView;
  relationships?: RecordRelationshipView[];
  onReference: RecordReferenceHandler;
  compact?: boolean;
  encounter?: boolean;
  hideIdentity?: boolean;
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
      className={`record-reference${compact ? " record-reference--compact" : ""}`}
    >
      {!hideIdentity && (
        <RecordIdentity
          identity={presentation.identity}
          root={root}
          selection={selection}
          onReference={onReference}
        />
      )}
      {body.kind === "creature" && (
        <ActorFacts actor={body.value} compact={compact} hideHitPoints={encounter} />
      )}
      {body.kind === "hazard" && (
        <>
          {!compact && !encounter && (
            <ContentFields
              fields={presentation.content.filter(
                (field) => field.role === "description",
              )}
              relationships={relationships}
              onReference={onReference}
            />
          )}
          <section className="hazard-reference__detection">
            <FactRow label="Hazard">
              {factText(body.value.complex, (complex) =>
                complex ? "Complex" : "Simple",
              )}
            </FactRow>
            <FactRow label="Stealth">
              <NumberFact fact={body.value.stealth} signed />
            </FactRow>
          </section>
          {!encounter && (
            <ContentFields
              fields={presentation.content.filter((field) => field.role === "stealth")}
              relationships={relationships}
              onReference={onReference}
            />
          )}
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
            hazard
          />
          <FactRow label="Hardness">
            <NumberFact fact={body.value.hardness} />
          </FactRow>
        </>
      )}
      {body.kind === "spell" && <SpellReference spell={body.value} compact={compact} />}
      {body.kind === "activity" && (
        <ActorActivity activity={body.value} onReference={onReference} selected />
      )}
      {body.kind === "physical_reference" && (
        <div className="record-fact-list">
          <FactLine label="Usage" fact={body.value.usage} />
          <FactLine label="Price" fact={body.value.price} />
          <FactLine label="Price per" fact={body.value.price_per} />
          <FactLine label="Bulk" fact={body.value.bulk} />
        </div>
      )}
      {body.kind === "roll_table" && (
        <FactLine label="Formula" fact={body.value.formula} />
      )}
      {body.kind === "table_result" && (
        <div className="record-fact-list">
          <FactLine label="Range" fact={body.value.range} />
          <FactLine label="Weight" fact={body.value.weight} />
          <FactLine label="Result type" fact={body.value.result_type} />
        </div>
      )}
      {actor && !compact && <ActorActivities actor={actor} onReference={onReference} />}
      {!compact &&
        (encounter ? null : body.kind === "hazard" ? (
          <ContentFields
            fields={presentation.content.filter(
              (field) => !["disable", "stealth", "description"].includes(field.role),
            )}
            relationships={relationships}
            onReference={onReference}
          />
        ) : (
          content
        ))}
      {compact && presentation.content.length > 0 && (
        <Collapse
          size="small"
          defaultActiveKey={["content"]}
          items={[{ key: "content", label: "Description", children: content }]}
        />
      )}
      {!compact && owned.length > 0 && (
        <section className="record-reference__owned">
          <h3>
            {presentation.identity.kind === "journal" ? "Pages" : "Owned records"}
          </h3>
          {owned.map((link, index) => (
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
              {link.usage && <span className="muted"> {link.usage}</span>}
            </div>
          ))}
        </section>
      )}
    </article>
  );
}
