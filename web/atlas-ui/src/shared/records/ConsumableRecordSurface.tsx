import { Button, Collapse, Descriptions, Empty, Space, Tag } from "antd";
import type {
  ConsumableFactView,
  ConsumableOccurrenceView,
  ConsumableSourceStateView,
  ConsumableSurfaceView,
  RecordSurfaceMetadataView,
  RecordSurfaceView,
} from "../../generated/atlas";
import { RecordHeader } from "./CreatureRecordSurface";
import { RichContent, type ReferenceHandler } from "./RecordRichContent";
import {
  RecordSurfaceIssues,
  RecordSurfaceReferences,
} from "./RecordSurfaceSupplement";
import { formatSlug } from "./recordFormatting";

type DetailProps = {
  body: ConsumableSurfaceView;
  issues: RecordSurfaceView["issues"];
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  onReferencesOpen?: () => void;
  onReferenceLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  references: RecordSurfaceView["references"];
  referencesLoading?: boolean;
  showTitle: boolean;
};

export function ConsumableDetailSurface({
  body,
  issues,
  metadata,
  onReference,
  onReferencesOpen,
  onReferenceLimit,
  references,
  referencesLoading,
  showTitle,
}: DetailProps) {
  return (
    <article className="record-surface record-surface--record-detail consumable-sheet">
      <RecordHeader
        metadata={metadata}
        onReference={onReference}
        showTitle={showTitle}
      />
      <ConsumableDescription body={body} onReference={onReference} />
      <ConsumableFacts body={body} onReference={onReference} />
      <RecordSurfaceIssues issues={issues} />
      <ConsumableProvenance metadata={metadata} />
      <RecordSurfaceReferences
        loading={referencesLoading}
        onDisclosureOpen={onReferencesOpen}
        onReference={onReference}
        onRequestLimit={onReferenceLimit}
        recordKey={metadata.record_key}
        references={references}
      />
    </article>
  );
}

function ConsumableProvenance({ metadata }: { metadata: RecordSurfaceMetadataView }) {
  const items = [
    item("Record ID", metadata.record_key),
    item("Publication", metadata.source?.publication_title),
    item("Source pack", metadata.source?.pack_label),
    item("Source path", metadata.source?.source_path),
  ].filter((entry): entry is { key: string; label: string; children: string } =>
    Boolean(entry),
  );
  return (
    <Collapse
      className="record-surface__secondary"
      ghost
      items={[
        {
          key: "provenance",
          label: "Source & provenance",
          children: <Descriptions column={1} items={items} size="small" />,
        },
      ]}
      size="small"
    />
  );
}

export function ConsumableSearchCompactSurface({
  body,
  metadata,
}: {
  body: ConsumableSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  return (
    <article className="record-surface record-surface--search-compact consumable-sheet">
      <RecordHeader metadata={metadata} />
      <Space size="small" wrap>
        <KnownTag fact={body.category} />
        <KnownTag fact={body.usage} />
        <KnownTag fact={body.bulk} prefix="Bulk " />
      </Space>
    </article>
  );
}

export function ConsumableOccurrences({
  occurrences,
  onReference,
}: {
  occurrences: ConsumableOccurrenceView[] | undefined;
  onReference: ReferenceHandler;
}) {
  if (!occurrences?.length) return null;
  return (
    <section aria-label="Consumables" className="record-surface__secondary">
      <h3>Consumables</h3>
      <Collapse
        ghost
        items={occurrences.map((occurrence) => ({
          key: occurrence.occurrence_id,
          label: (
            <Space size="small" wrap>
              <span>{occurrence.name}</span>
              <StateSummary state={occurrence.source_state} />
            </Space>
          ),
          children: (
            <OccurrenceBody occurrence={occurrence} onReference={onReference} />
          ),
        }))}
        size="small"
      />
    </section>
  );
}

function OccurrenceBody({
  occurrence,
  onReference,
}: {
  occurrence: ConsumableOccurrenceView;
  onReference: ReferenceHandler;
}) {
  return (
    <Space direction="vertical" size="small">
      {occurrence.target.state === "resolved" ? (
        <Button
          size="small"
          type="link"
          onClick={() => {
            if (occurrence.target.state === "resolved")
              onReference(occurrence.target.record_key);
          }}
        >
          Open canonical consumable
        </Button>
      ) : null}
      {occurrence.target.state === "resolved" &&
      occurrence.target.mismatch_fields.length ? (
        <Tag color="warning">Local source differences retained</Tag>
      ) : null}
      {occurrence.target.state === "parent_owned" ? (
        <Tag>{parentOwnedReason(occurrence.target.reason)}</Tag>
      ) : null}
      <SourceState state={occurrence.source_state} />
      {occurrence.definition ? (
        <ConsumableFacts
          body={occurrence.definition}
          onReference={onReference}
          showSourceState={false}
        />
      ) : null}
      {!occurrence.definition && occurrence.spell_child ? (
        <SpellChildLink child={occurrence.spell_child} onReference={onReference} />
      ) : null}
      {(occurrence.content ?? []).map((content) => (
        <RichContent
          content={content}
          key={content.content_key}
          onReference={onReference}
        />
      ))}
    </Space>
  );
}

function ConsumableDescription({
  body,
  onReference,
}: {
  body: ConsumableSurfaceView;
  onReference: ReferenceHandler;
}) {
  if (!body.content?.length) return null;
  return (
    <section aria-label="Description">
      <h3>Description</h3>
      {(body.content ?? []).map((content) => (
        <RichContent
          content={content}
          key={content.content_key}
          onReference={onReference}
        />
      ))}
    </section>
  );
}

function ConsumableFacts({
  body,
  onReference,
  showSourceState = true,
}: {
  body: ConsumableSurfaceView;
  onReference?: ReferenceHandler;
  showSourceState?: boolean;
}) {
  const items = [
    item("Category", text(body.category)),
    item("Usage", text(body.usage)),
    item("Base item", text(body.base_item)),
    item("Bulk", text(body.bulk)),
    item("Size", text(body.size)),
    item("Maximum uses", integer(body.maximum_uses)),
    item("Maximum HP", integer(body.maximum_hp)),
    item("Hardness", integer(body.hardness)),
    item("Auto-destroy", boolean(body.auto_destroy)),
    item("Price", price(body.price)),
    item("Material", material(body.material)),
    item("Publication", publication(body.publication)),
  ].filter((entry): entry is { key: string; label: string; children: string } =>
    Boolean(entry),
  );
  const damage = body.damage.state === "known" ? body.damage.value : undefined;
  const damageCategory = damage ? text(damage.category) : undefined;
  const damageLabel =
    damageCategory === "Damage" || damageCategory === "Healing"
      ? damageCategory
      : "Effect";

  return (
    <section aria-label="Consumable details">
      <h3>Details</h3>
      {items.length ? (
        <Descriptions column={{ xs: 1, sm: 2, lg: 3 }} items={items} size="small" />
      ) : (
        <Empty
          description="No typed details are available."
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      )}
      {showSourceState ? <SourceState state={body.source_state} /> : null}
      {body.spell_child ? (
        <SpellChildLink child={body.spell_child} onReference={onReference} />
      ) : null}
      {body.other_tags.state === "known" && body.other_tags.value.length ? (
        <Space aria-label="Other tags" size="small" wrap>
          {body.other_tags.value.map((tag) => (
            <Tag key={tag}>{formatSlug(tag)}</Tag>
          ))}
        </Space>
      ) : null}
      {damage ? (
        <Descriptions
          column={{ xs: 1, sm: 3 }}
          items={[
            item(damageLabel, text(damage.formula)),
            item("Type", text(damage.damage_type)),
            damageLabel === "Effect" ? item("Kind", damageCategory) : undefined,
          ].filter((entry): entry is { key: string; label: string; children: string } =>
            Boolean(entry),
          )}
          size="small"
        />
      ) : null}
    </section>
  );
}

function SourceState({ state }: { state: ConsumableSourceStateView }) {
  const equipped = state.equipped.state === "known" ? state.equipped.value : undefined;
  const items = [
    item("Quantity", integer(state.quantity)),
    item("Uses remaining", integer(state.current_uses)),
    item("Current HP", integer(state.current_hp)),
    item("Container", text(state.container_id)),
    item("Carry type", equipped ? text(equipped.carry_type) : undefined),
    item("Hands held", equipped ? integer(equipped.hands_held) : undefined),
    item("In slot", equipped ? boolean(equipped.in_slot) : undefined),
  ].filter((entry): entry is { key: string; label: string; children: string } =>
    Boolean(entry),
  );
  return items.length ? <Descriptions items={items} size="small" /> : null;
}

function SpellChildLink({
  child,
  onReference,
}: {
  child: NonNullable<ConsumableSurfaceView["spell_child"]>;
  onReference?: ReferenceHandler;
}) {
  if (!child.target_record_key || !onReference) {
    return <Tag>Embedded spell target unavailable</Tag>;
  }
  return (
    <Button
      size="small"
      type="link"
      onClick={() => onReference(child.target_record_key!)}
    >
      Open embedded spell
    </Button>
  );
}

function StateSummary({ state }: { state: ConsumableSourceStateView }) {
  const quantity = integer(state.quantity);
  const uses = integer(state.current_uses);
  return (
    <>
      {quantity ? <Tag>Quantity {quantity}</Tag> : null}
      {uses ? <Tag>{uses} uses remaining</Tag> : null}
    </>
  );
}

function KnownTag({
  fact,
  prefix = "",
}: {
  fact: ConsumableFactView<string>;
  prefix?: string;
}) {
  return fact.state === "known" ? (
    <Tag>
      {prefix}
      {formatSlug(fact.value)}
    </Tag>
  ) : null;
}

function item(label: string, children: string | undefined) {
  return children ? { key: label, label, children } : undefined;
}

function text(fact: ConsumableFactView<string>) {
  return fact.state === "known" && fact.value ? formatSlug(fact.value) : undefined;
}

function integer(fact: ConsumableFactView<number>) {
  return fact.state === "known" ? String(fact.value) : undefined;
}

function boolean(fact: ConsumableFactView<boolean>) {
  return fact.state === "known" ? (fact.value ? "Yes" : "No") : undefined;
}

function price(fact: ConsumableSurfaceView["price"]): string | undefined {
  if (fact.state !== "known" || fact.value.denominations.state !== "known")
    return undefined;
  const values = fact.value.denominations.value.map(
    (entry) => `${entry.amount} ${entry.denomination}`,
  );
  if (fact.value.per.state === "known") values.push(`per ${fact.value.per.value}`);
  return values.join(" · ") || undefined;
}

function material(fact: ConsumableSurfaceView["material"]): string | undefined {
  if (fact.state !== "known") return undefined;
  return (
    [
      text(fact.value.grade),
      text(fact.value.material_type),
      fact.value.effects.state === "known"
        ? fact.value.effects.value.map(formatSlug).join(", ")
        : undefined,
    ]
      .filter(Boolean)
      .join(" · ") || undefined
  );
}

function publication(fact: ConsumableSurfaceView["publication"]): string | undefined {
  if (fact.state !== "known") return undefined;
  return (
    [
      text(fact.value.title),
      text(fact.value.license),
      fact.value.remaster.state === "known" && fact.value.remaster.value
        ? "Remaster"
        : undefined,
    ]
      .filter(Boolean)
      .join(" · ") || undefined
  );
}

function parentOwnedReason(
  reason: Extract<
    ConsumableOccurrenceView["target"],
    { state: "parent_owned" }
  >["reason"],
) {
  switch (reason) {
    case "no_locator":
      return "Embedded definition";
    case "malformed_or_duplicate_locator":
      return "Embedded definition · source link unavailable";
    case "target_missing":
      return "Embedded definition · canonical target missing";
    case "wrong_document_or_family":
      return "Embedded definition · canonical target has another type";
  }
}
