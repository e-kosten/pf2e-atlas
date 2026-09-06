import { Alert, Collapse, Space, Tag } from "antd";
import type React from "react";
import type {
  EncounterRuntimeView,
  HazardSurfaceActivityView,
  HazardSurfaceDefensesView,
  HazardSurfaceIwrView,
  HazardSurfaceLifecycleView,
  HazardSurfaceRuleView,
  HazardSurfaceView,
  RecordSurfaceMetadataView,
} from "../../generated/atlas";
import { ActionGlyph, actionCostLabel } from "./ActionGlyph";
import { RecordHeader, SurfaceSection, TraitRow } from "./CreatureRecordSurface";
import {
  RuntimeActivities,
  RuntimeAutomationLimitations,
  RuntimeConditions,
  RuntimeVitals,
  type EncounterRecordSurfaceSlots,
} from "./EncounterRecordSurface";
import { RichBlocks, RichContent, type ReferenceHandler } from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import { formatSigned, formatSlug } from "./recordFormatting";

export function HazardDetailSurface({
  body,
  metadata,
  onReference,
  showTitle,
}: {
  body: HazardSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  showTitle: boolean;
}) {
  return (
    <article className="record-surface record-surface--record-detail">
      <RecordHeader
        metadata={metadata}
        onReference={onReference}
        showTitle={showTitle}
      />
      <LifecycleBlock
        blocks={body.lifecycle?.description}
        keyPrefix="hazard-description"
        onReference={onReference}
      />
      <div className="creature-sheet__facts-grid">
        <div className="creature-sheet__facts-column creature-sheet__facts-column--primary">
          <HazardSummary body={body} onReference={onReference} />
          <HazardDefenses defenses={body.defenses} onReference={onReference} />
        </div>
        <div className="creature-sheet__facts-column creature-sheet__facts-column--secondary">
          <HazardLifecycle lifecycle={body.lifecycle} onReference={onReference} />
        </div>
      </div>
      <HazardActivities activities={body.activities} onReference={onReference} />
      <HazardGeneralContent body={body} onReference={onReference} />
      <HazardRelationships body={body} />
      <HazardAvailability body={body} />
      <HazardSourceDisclosure body={body} metadata={metadata} />
    </article>
  );
}

export function HazardSearchCompactSurface({
  body,
  metadata,
}: {
  body: HazardSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <div className="record-surface-search__heading">
          <h2>{metadata.title}</h2>
        </div>
        <div className="creature-sheet__identity-meta creature-sheet__identity-meta--compact">
          <span className="creature-sheet__kind">{metadata.kind_label}</span>
          {metadata.level !== undefined ? (
            <span className="creature-sheet__level">Level {metadata.level}</span>
          ) : null}
          {body.complexity ? <span>{formatSlug(body.complexity)}</span> : null}
        </div>
        <TraitRow compact metadata={metadata} />
        {body.teaser ? (
          <p className="record-surface-search__teaser">{body.teaser}</p>
        ) : null}
      </div>
      <dl className="record-surface-search__facts">
        {body.detection?.difficulty_class !== undefined ? (
          <div>
            <dt>Detection DC</dt>
            <dd>{body.detection.difficulty_class}</dd>
          </div>
        ) : null}
        {body.defenses?.armor_class !== undefined ? (
          <div>
            <dt>AC</dt>
            <dd>{body.defenses.armor_class}</dd>
          </div>
        ) : null}
      </dl>
      <div className="record-surface-search__meta">
        <strong>
          {body.complexity ? `${formatSlug(body.complexity)} hazard` : "Hazard"}
        </strong>
        {metadata.source?.pack_label ? (
          <small>{metadata.source.pack_label}</small>
        ) : null}
      </div>
    </article>
  );
}

export function HazardEncounterSurface({
  body,
  metadata,
  onReference,
  runtime,
  slots,
}: {
  body: HazardSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  runtime: EncounterRuntimeView | undefined;
  slots: EncounterRecordSurfaceSlots;
}) {
  const hazard = runtime?.hazard;
  return (
    <article className="record-surface record-surface--encounter-participant">
      <header className="record-surface-structured__header">
        <div>
          <div className="record-surface-structured__title-row">
            <h2>{metadata.title}</h2>
            <Space size="small" wrap>
              <Tag>{hazard?.state === "disabled" ? "Disabled" : "Active"}</Tag>
              {body.complexity ? <Tag>{formatSlug(body.complexity)}</Tag> : null}
            </Space>
          </div>
          <TraitRow compact metadata={metadata} />
        </div>
        <div className="record-surface-structured__header-actions">
          {slots.header_actions}
        </div>
      </header>
      {slots.header ? (
        <div className="record-surface-structured__participant">{slots.header}</div>
      ) : null}
      <div className="record-surface-structured__runtime">
        <SurfaceSection className="record-surface-card--vitals" title="Vitals">
          {slots.vitals ?? <RuntimeVitals runtime={runtime} />}
        </SurfaceSection>
        {slots.conditions ?? (
          <SurfaceSection
            className="record-surface-card--conditions"
            title="Conditions"
          >
            <RuntimeConditions runtime={runtime} />
          </SurfaceSection>
        )}
      </div>
      {!runtime ? (
        <Alert message="Runtime facts are unavailable." showIcon type="info" />
      ) : null}
      <div className="record-surface-structured__grid">
        <div className="record-surface-structured__column">
          <HazardRuntimeFacts body={body} runtime={runtime} />
          <HazardLifecycle lifecycle={body.lifecycle} onReference={onReference} />
          {slots.notes ? (
            <SurfaceSection title="Participant Note">{slots.notes}</SurfaceSection>
          ) : null}
        </div>
        <div className="record-surface-structured__column">
          <RuntimeActivities
            activityType="active"
            onReference={onReference}
            runtime={runtime}
            title="Actions"
          />
          <RuntimeActivities
            activityType="passive"
            onReference={onReference}
            runtime={runtime}
            title="Passives"
          />
          <RuntimeAutomationLimitations runtime={runtime} />
        </div>
      </div>
      <LifecycleBlock
        blocks={body.lifecycle?.description}
        keyPrefix="encounter-hazard-description"
        onReference={onReference}
        title="Description"
      />
      <HazardGeneralContent body={body} onReference={onReference} />
      <HazardAvailability body={body} />
      <HazardSourceDisclosure body={body} metadata={metadata} />
    </article>
  );
}

function HazardSummary({
  body,
  onReference,
}: {
  body: HazardSurfaceView;
  onReference: ReferenceHandler;
}) {
  const facts: RecordKeyValueItem[] = [
    textItem(
      "complexity",
      "Complexity",
      body.complexity && formatSlug(body.complexity),
    ),
    textItem("size", "Size", body.size && formatSlug(body.size)),
    numberItem("stealth", "Stealth", body.detection?.stealth_modifier, true),
    numberItem("detection", "Detection DC", body.detection?.difficulty_class),
    textItem("sound", "Emits sound", formatEmitsSound(body)),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  if (!facts.length && !body.detection?.details?.length) return null;
  return (
    <SurfaceSection title="Hazard">
      {facts.length ? (
        <RecordKeyValueList ariaLabel="Hazard facts" items={facts} />
      ) : null}
      <LifecycleBlock
        blocks={body.detection?.details}
        keyPrefix="hazard-detection"
        onReference={onReference}
      />
    </SurfaceSection>
  );
}

function HazardDefenses({
  defenses,
  onReference,
}: {
  defenses: HazardSurfaceDefensesView | undefined;
  onReference: ReferenceHandler;
}) {
  if (!defenses) return null;
  const hp = defenses.hit_points;
  const saves = defenses.saves;
  const facts: RecordKeyValueItem[] = [
    numberItem("ac", "Armor Class", defenses.armor_class),
    numberItem("hardness", "Hardness", defenses.hardness),
    numberItem("hp", "Hit Points", hp?.current),
    numberItem("maximum-hp", "Maximum HP", hp?.maximum),
    numberItem("temporary-hp", "Temporary HP", hp?.temporary),
    numberItem("broken-threshold", "Broken Threshold", hp?.broken_threshold),
    numberItem("fortitude", "Fortitude", saves?.fortitude, true),
    numberItem("reflex", "Reflex", saves?.reflex, true),
    numberItem("will", "Will", saves?.will, true),
    iwrItem("immunities", "Immunities", defenses.immunities),
    iwrItem("weaknesses", "Weaknesses", defenses.weaknesses),
    iwrItem("resistances", "Resistances", defenses.resistances),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  if (!facts.length && !hp?.details?.length) return null;
  return (
    <SurfaceSection title="Defenses">
      {facts.length ? (
        <RecordKeyValueList ariaLabel="Hazard defenses" items={facts} />
      ) : null}
      <LifecycleBlock
        blocks={hp?.details}
        keyPrefix="hazard-hit-points"
        onReference={onReference}
      />
    </SurfaceSection>
  );
}

function HazardLifecycle({
  lifecycle,
  onReference,
}: {
  lifecycle: HazardSurfaceLifecycleView | undefined;
  onReference: ReferenceHandler;
}) {
  if (!lifecycle) return null;
  const items = [
    ["disable", "Disable", lifecycle.disable],
    ["routine", "Routine", lifecycle.routine],
    ["reset", "Reset", lifecycle.reset],
  ] as const;
  const present = items.filter(([, , blocks]) => blocks?.length);
  if (!present.length) return null;
  return (
    <SurfaceSection title="Lifecycle">
      <Collapse
        ghost
        items={present.map(([key, label, blocks]) => ({
          key,
          label,
          children: (
            <RichBlocks
              blocks={blocks ?? []}
              keyPrefix={`hazard-${key}`}
              onReference={onReference}
            />
          ),
        }))}
        size="small"
      />
    </SurfaceSection>
  );
}

function HazardActivities({
  activities,
  onReference,
}: {
  activities: HazardSurfaceActivityView[] | undefined;
  onReference: ReferenceHandler;
}) {
  if (!activities?.length) return null;
  return (
    <SurfaceSection title="Activities">
      <Collapse
        defaultActiveKey={activities.map((activity) => activity.occurrence_id)}
        ghost
        items={activities.map((activity) => ({
          key: activity.occurrence_id,
          label: <HazardActivityHeading activity={activity} />,
          children: (
            <HazardActivityDetails activity={activity} onReference={onReference} />
          ),
        }))}
        size="small"
      />
    </SurfaceSection>
  );
}

function HazardActivityHeading({ activity }: { activity: HazardSurfaceActivityView }) {
  return (
    <Space size="small" wrap>
      <strong>{activity.label}</strong>
      {activity.action_cost ? <ActionGlyph cost={activity.action_cost} /> : null}
      <Tag>{formatSlug(activity.activity_type)}</Tag>
      {activity.attack_bonus !== undefined ? (
        <span>{formatSigned(activity.attack_bonus)}</span>
      ) : null}
    </Space>
  );
}

function HazardActivityDetails({
  activity,
  onReference,
}: {
  activity: HazardSurfaceActivityView;
  onReference: ReferenceHandler;
}) {
  return (
    <div className="encounter-runtime-activity-details">
      {activity.traits?.length ? (
        <Space size={[4, 4]} wrap>
          {activity.traits.map((trait) => (
            <Tag key={trait}>{formatSlug(trait)}</Tag>
          ))}
        </Space>
      ) : null}
      {activity.action_cost ? <p>{actionCostLabel(activity.action_cost)}</p> : null}
      {activity.child_type ? <p>Source type: {activity.child_type}</p> : null}
      {activity.category ? <p>Category: {formatSlug(activity.category)}</p> : null}
      {activity.death_note !== undefined ? (
        <p>Death note: {activity.death_note ? "Yes" : "No"}</p>
      ) : null}
      {activity.self_effect ? (
        <p>
          Self effect: {activity.self_effect.label ?? "Unlabeled"}
          {activity.self_effect.target_uuid
            ? ` (${activity.self_effect.target_uuid})`
            : ""}
        </p>
      ) : null}
      {activity.frequency ? <p>{formatFrequency(activity.frequency)}</p> : null}
      {activity.attack_effects?.length ? (
        <p>Effects: {activity.attack_effects.join(", ")}</p>
      ) : null}
      {activity.damage?.map((damage) => (
        <p key={damage.damage_id}>
          {[damage.formula, damage.damage_type, damage.category]
            .filter(Boolean)
            .join(" ")}
        </p>
      ))}
      {activity.rules?.map((rule) => (
        <HazardRule
          key={`${rule.authored_order}:${rule.rule_type}`}
          onReference={onReference}
          rule={rule}
        />
      ))}
      {activity.content?.map((content) => (
        <RichContent
          content={content}
          key={content.content_key}
          onReference={onReference}
        />
      ))}
      <small>Occurrence {activity.occurrence_id}</small>
    </div>
  );
}

function HazardRule({
  onReference,
  rule,
}: {
  onReference: ReferenceHandler;
  rule: HazardSurfaceRuleView;
}) {
  switch (rule.rule_type) {
    case "immunity":
      return (
        <p>
          Immunity{rule.mode ? ` (${rule.mode})` : ""}:{" "}
          {rule.immunity_types?.join(", ") || "—"}
        </p>
      );
    case "active_effect_like":
      return (
        <p>
          Active effect{rule.mode ? ` (${rule.mode})` : ""}: {rule.path ?? "—"}
          {rule.value !== undefined ? ` = ${rule.value ? "true" : "false"}` : ""}
        </p>
      );
    case "aura":
      return (
        <p>
          Aura:{" "}
          {rule.radius !== undefined ? `${rule.radius} feet` : "radius unavailable"}
          {rule.slug ? ` (${rule.slug})` : ""}
          {rule.traits?.length ? `; ${rule.traits.join(", ")}` : ""}
        </p>
      );
    case "damage_dice":
      return (
        <p>
          Damage dice:{" "}
          {[rule.dice_number, rule.die_size, rule.damage_type, rule.selector]
            .filter((value) => value !== undefined)
            .join(" ") || "—"}
          {rule.critical !== undefined
            ? `; critical ${rule.critical ? "yes" : "no"}`
            : ""}
        </p>
      );
    case "flat_modifier":
      return (
        <p>
          Flat modifier: {rule.value === undefined ? "—" : formatSigned(rule.value)}
          {rule.damage_type ? ` ${rule.damage_type}` : ""}
          {rule.selector ? ` (${rule.selector})` : ""}
          {rule.critical !== undefined
            ? `; critical ${rule.critical ? "yes" : "no"}`
            : ""}
        </p>
      );
    case "note":
      return (
        <div>
          <p>
            Note{rule.title ? `: ${rule.title}` : ""}
            {rule.outcomes?.length ? ` (${rule.outcomes.join(", ")})` : ""}
          </p>
          <RichBlocks
            blocks={rule.text ?? []}
            keyPrefix={`hazard-rule-note-${rule.authored_order}`}
            onReference={onReference}
          />
        </div>
      );
    case "unsupported":
      return <p>Unsupported authored rule</p>;
  }
}

function HazardRuntimeFacts({
  body,
  runtime,
}: {
  body: HazardSurfaceView;
  runtime: EncounterRuntimeView | undefined;
}) {
  const hazard = runtime?.hazard;
  if (!runtime || !hazard) return null;
  const facts: RecordKeyValueItem[] = [
    numberItem("ac", "Armor Class", runtime.defenses?.armor_class.adjusted_value),
    numberItem(
      "fortitude",
      "Fortitude",
      runtime.saves?.fortitude?.adjusted_value,
      true,
    ),
    numberItem("reflex", "Reflex", runtime.saves?.reflex?.adjusted_value, true),
    numberItem("will", "Will", runtime.saves?.will?.adjusted_value, true),
    numberItem("hardness", "Hardness", body.defenses?.hardness),
    numberItem("detection", "Detection DC", hazard.detection_dc?.adjusted_value),
    numberItem(
      "broken-threshold",
      "Broken Threshold",
      hazard.broken_threshold?.adjusted_value,
    ),
    hazard.initiative_suggestion
      ? textItem(
          "initiative-suggestion",
          "Initiative suggestion",
          `${formatSlug(hazard.initiative_suggestion.statistic)} ${formatSigned(hazard.initiative_suggestion.modifier.adjusted_value)}`,
        )
      : null,
    iwrItem("immunities", "Immunities", body.defenses?.immunities),
    iwrItem("weaknesses", "Weaknesses", body.defenses?.weaknesses),
    iwrItem("resistances", "Resistances", body.defenses?.resistances),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  return facts.length ? (
    <SurfaceSection title="Hazard runtime">
      <RecordKeyValueList ariaLabel="Hazard runtime facts" items={facts} />
    </SurfaceSection>
  ) : null;
}

function HazardGeneralContent({
  body,
  onReference,
}: {
  body: HazardSurfaceView;
  onReference: ReferenceHandler;
}) {
  if (!body.content?.length) return null;
  return (
    <SurfaceSection title="Additional content">
      {body.content.map((content) => (
        <RichContent
          content={content}
          key={content.content_key}
          onReference={onReference}
        />
      ))}
    </SurfaceSection>
  );
}

function HazardRelationships({ body }: { body: HazardSurfaceView }) {
  if (!body.relationships?.length) return null;
  return (
    <SurfaceSection title="Relationships">
      <ul>
        {body.relationships.map((relationship) => {
          const target =
            relationship.target.target_type === "entity"
              ? `Entity ${relationship.target.entity_id}`
              : `Occurrence ${relationship.target.occurrence_id}`;
          return (
            <li key={relationship.relationship_id}>
              {target}
              {relationship.source_occurrence_id
                ? ` from ${relationship.source_occurrence_id}`
                : ""}
            </li>
          );
        })}
      </ul>
    </SurfaceSection>
  );
}

function HazardAvailability({ body }: { body: HazardSurfaceView }) {
  if (!body.unavailable_fields?.length) return null;
  return (
    <Collapse
      className="record-surface__secondary"
      ghost
      items={[
        {
          key: "availability",
          label: "Data availability",
          children: (
            <ul className="encounter-runtime-limitations">
              {body.unavailable_fields.map((unavailable, index) => (
                <li
                  key={`${unavailable.field}:${unavailable.component_id ?? "record"}:${index}`}
                >
                  <Tag>{formatSlug(unavailable.state)}</Tag>
                  <span>{unavailable.message}</span>
                  <small>{unavailable.field}</small>
                </li>
              ))}
            </ul>
          ),
        },
      ]}
      size="small"
    />
  );
}

function HazardSourceDisclosure({
  body,
  metadata,
}: {
  body: HazardSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  const license = body.provenance.publication_license;
  return (
    <Collapse
      className="record-surface__secondary"
      ghost
      items={[
        {
          key: "source",
          label: "References & Source",
          children: (
            <RecordKeyValueList
              ariaLabel="Hazard source"
              items={[
                {
                  key: "pack",
                  label: "Pack",
                  value: metadata.source?.pack_label ?? "—",
                },
                {
                  key: "path",
                  label: "Source path",
                  value: body.provenance.source_path,
                },
                {
                  key: "license",
                  label: "License",
                  value:
                    license.state === "value"
                      ? license.value
                      : formatSlug(license.state),
                },
              ]}
            />
          ),
        },
      ]}
      size="small"
    />
  );
}

function LifecycleBlock({
  blocks,
  keyPrefix,
  onReference,
  title,
}: {
  blocks: HazardSurfaceLifecycleView["description"];
  keyPrefix: string;
  onReference: ReferenceHandler;
  title?: string;
}) {
  if (!blocks?.length) return null;
  const content = (
    <RichBlocks blocks={blocks} keyPrefix={keyPrefix} onReference={onReference} />
  );
  return title ? <SurfaceSection title={title}>{content}</SurfaceSection> : content;
}

function numberItem(
  key: string,
  label: string,
  value: number | undefined,
  signed = false,
): RecordKeyValueItem | null {
  return value === undefined
    ? null
    : { key, label, value: signed ? formatSigned(value) : value.toString() };
}

function textItem(
  key: string,
  label: string,
  value: string | undefined,
): RecordKeyValueItem | null {
  return value === undefined ? null : { key, label, value };
}

function iwrItem(
  key: string,
  label: string,
  values: HazardSurfaceIwrView[] | undefined,
): RecordKeyValueItem | null {
  if (!values?.length) return null;
  return {
    key,
    label,
    value: values.map(formatIwr).join(", "),
  };
}

function formatIwr(value: HazardSurfaceIwrView) {
  return [
    formatSlug(value.kind),
    value.amount,
    value.exceptions?.length ? `except ${value.exceptions.join(", ")}` : undefined,
    value.double_vs?.length ? `double vs ${value.double_vs.join(", ")}` : undefined,
  ]
    .filter((part) => part !== undefined)
    .join(" ");
}

function formatFrequency(
  frequency: NonNullable<HazardSurfaceActivityView["frequency"]>,
) {
  const count = frequency.maximum ?? frequency.value;
  return [
    count === undefined ? undefined : `${count} use${count === 1 ? "" : "s"}`,
    frequency.period,
  ]
    .filter(Boolean)
    .join(" per ");
}

function formatEmitsSound(body: HazardSurfaceView): string | undefined {
  if (!body.emits_sound) return undefined;
  return body.emits_sound.sound_type === "boolean"
    ? body.emits_sound.value
      ? "Yes"
      : "No"
    : body.emits_sound.value;
}
