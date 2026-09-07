import { Alert, Collapse, Space, Tag, Typography } from "antd";
import type React from "react";
import type {
  EncounterRuntimeView,
  HazardSurfaceActivityView,
  HazardSurfaceDefensesView,
  HazardSurfaceLifecycleView,
  HazardSurfaceRuleView,
  HazardSurfaceSourceMetadataFactView,
  HazardSurfaceView,
  RecordSurfaceMetadataView,
  RecordSurfaceView,
} from "../../generated/atlas";
import { ActionGlyph } from "./ActionGlyph";
import { RecordHeader, SurfaceSection, TraitRow } from "./CreatureRecordSurface";
import {
  RuntimeActivities,
  RuntimeAutomationLimitations,
  RuntimeConditions,
  RuntimeVitals,
  type EncounterRecordSurfaceSlots,
} from "./EncounterRecordSurface";
import { RichBlocks, RichContent, type ReferenceHandler } from "./RecordRichContent";
import { DefenseIwrList, DefenseStats } from "./RecordDefensePrimitives";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import {
  RecordSurfaceIssues,
  RecordSurfaceReferences,
} from "./RecordSurfaceSupplement";
import { formatSigned, formatSlug } from "./recordFormatting";

type RecordSurfaceIssue = NonNullable<RecordSurfaceView["issues"]>[number];
type RecordSurfaceReferences = NonNullable<RecordSurfaceView["references"]>;

export function HazardDetailSurface({
  body,
  issues,
  metadata,
  onReference,
  onReferencesOpen,
  onReferenceLimit,
  references,
  referencesLoading,
  showTitle,
}: {
  body: HazardSurfaceView;
  issues: RecordSurfaceIssue[] | undefined;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  onReferencesOpen?: () => void;
  onReferenceLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  references: RecordSurfaceReferences | undefined;
  referencesLoading?: boolean;
  showTitle: boolean;
}) {
  return (
    <article className="record-surface record-surface--record-detail">
      <RecordHeader
        metadata={metadata}
        onReference={onReference}
        showTitle={showTitle}
      />
      <HazardOverview body={body} onReference={onReference} />
      <HazardDetectionAndDisable body={body} onReference={onReference} />
      <HazardDefensesPanel defenses={body.defenses} onReference={onReference} />
      {body.lifecycle?.routine?.length ? (
        <Typography.Link href="#hazard-operation">Jump to routine</Typography.Link>
      ) : null}
      <HazardActivities activities={body.activities} onReference={onReference} />
      <HazardOperation lifecycle={body.lifecycle} onReference={onReference} />
      <HazardGeneralContent body={body} onReference={onReference} />
      <RecordSurfaceIssues issues={issues} />
      <RecordSurfaceReferences
        loading={referencesLoading}
        onDisclosureOpen={onReferencesOpen}
        onRequestLimit={onReferenceLimit}
        onReference={onReference}
        references={references}
      />
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
  issues,
  metadata,
  onReference,
  runtime,
  slots,
}: {
  body: HazardSurfaceView;
  issues: RecordSurfaceIssue[] | undefined;
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
      <RecordSurfaceIssues issues={issues} />
      <HazardSourceDisclosure body={body} metadata={metadata} />
    </article>
  );
}

function HazardOverview({
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
    textItem("sound", "Detectable by hearing", formatEmitsSound(body)),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  const description = body.lifecycle?.description;
  if (!facts.length && !description?.length) return null;
  return (
    <SurfaceSection title="Overview">
      <LifecycleBlock
        blocks={description}
        keyPrefix="hazard-description"
        onReference={onReference}
      />
      {facts.length ? (
        <RecordKeyValueList ariaLabel="Hazard overview facts" items={facts} />
      ) : null}
    </SurfaceSection>
  );
}

function HazardDetectionAndDisable({
  body,
  onReference,
}: {
  body: HazardSurfaceView;
  onReference: ReferenceHandler;
}) {
  const facts = [
    numberItem("stealth", "Stealth", body.detection?.stealth_modifier, true),
    numberItem("detection", "Detection DC", body.detection?.difficulty_class),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  const detectionDetails = body.detection?.details;
  const disable = body.lifecycle?.disable;
  if (!facts.length && !detectionDetails?.length && !disable?.length) return null;
  return (
    <SurfaceSection title="Detection & disable">
      {facts.length ? (
        <RecordKeyValueList ariaLabel="Hazard detection" items={facts} />
      ) : null}
      <LifecycleBlock
        blocks={detectionDetails}
        keyPrefix="hazard-detection"
        onReference={onReference}
      />
      {disable?.length ? (
        <div className="hazard-sheet__lifecycle-entry">
          <h4>Disable</h4>
          <RichBlocks
            blocks={disable}
            keyPrefix="hazard-disable"
            onReference={onReference}
          />
        </div>
      ) : null}
    </SurfaceSection>
  );
}

function HazardDefensesPanel({
  defenses,
  onReference,
}: {
  defenses: HazardSurfaceDefensesView | undefined;
  onReference: ReferenceHandler;
}) {
  if (!defenses) return null;
  const hp = defenses.hit_points;
  const stats = [
    { key: "ac", label: "AC", value: defenses.armor_class },
    {
      key: "hp",
      label: "HP",
      qualifier:
        hp?.temporary !== undefined && hp.temporary !== 0
          ? `${formatSigned(hp.temporary)} temporary`
          : undefined,
      value: hazardHitPoints(hp),
    },
    { key: "hardness", label: "Hardness", value: defenses.hardness },
    { key: "broken-threshold", label: "BT", value: hp?.broken_threshold },
    {
      key: "fortitude",
      label: "Fort",
      signed: true,
      value: defenses.saves?.fortitude,
    },
    {
      key: "reflex",
      label: "Ref",
      signed: true,
      value: defenses.saves?.reflex,
    },
    { key: "will", label: "Will", signed: true, value: defenses.saves?.will },
  ];
  const hasStats = stats.some((stat) => stat.value !== undefined);
  const hasIwr = Boolean(
    defenses.immunities?.length ||
    defenses.weaknesses?.length ||
    defenses.resistances?.length,
  );
  if (!hasStats && !hasIwr && !hp?.details?.length) return null;
  return (
    <SurfaceSection
      className="creature-sheet__panel--defenses hazard-sheet__defenses"
      title="Defenses & Structure"
    >
      <DefenseStats ariaLabel="Hazard defense statistics" values={stats} />
      <LifecycleBlock
        blocks={hp?.details}
        keyPrefix="hazard-hit-points"
        onReference={onReference}
      />
      <DefenseIwrList
        immunities={defenses.immunities}
        resistances={defenses.resistances}
        weaknesses={defenses.weaknesses}
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

function HazardOperation({
  lifecycle,
  onReference,
}: {
  lifecycle: HazardSurfaceLifecycleView | undefined;
  onReference: ReferenceHandler;
}) {
  if (!lifecycle) return null;
  const items = [
    ["routine", "Routine", lifecycle.routine],
    ["reset", "Reset", lifecycle.reset],
  ] as const;
  const present = items.filter(([, , blocks]) => blocks?.length);
  if (!present.length) return null;
  return (
    <SurfaceSection title="Operation">
      <span id="hazard-operation" />
      {present.map(([key, label, blocks]) => (
        <div className="hazard-sheet__lifecycle-entry" key={key}>
          <h4>{label}</h4>
          {blocks?.length ? (
            <RichBlocks
              blocks={blocks}
              keyPrefix={`hazard-${key}`}
              onReference={onReference}
            />
          ) : null}
        </div>
      ))}
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
            <div id={`hazard-activity-${activity.occurrence_id}`}>
              <HazardActivityDetails activity={activity} onReference={onReference} />
            </div>
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
      <Tag>
        {activity.attack_mode
          ? `${formatSlug(activity.attack_mode)} Strike`
          : activity.activity_type === "unsupported_child"
            ? "Content only"
            : activity.action_cost?.cost_type === "reaction"
              ? "Reaction"
              : activity.action_cost?.cost_type === "passive"
                ? "Passive"
                : formatSlug(activity.activity_type)}
      </Tag>
      {activity.attack_bonus !== undefined ? (
        <span>Attack {formatSigned(activity.attack_bonus)}</span>
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
      {activity.category ? <p>Category: {formatSlug(activity.category)}</p> : null}
      {activity.death_note !== undefined ? (
        <p>Death note: {activity.death_note ? "Yes" : "No"}</p>
      ) : null}
      {activity.self_effect ? (
        <p>Self effect: {activity.self_effect.label ?? "Linked effect"}</p>
      ) : null}
      {activity.frequency ? <p>{formatFrequency(activity.frequency)}</p> : null}
      {activity.attack_effects?.length ? (
        <p>
          Effects: {activity.attack_effects.map(hazardAttackEffectLabel).join(", ")}
        </p>
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
          Active effect{rule.mode ? ` (${formatSlug(rule.mode)})` : ""}
          {rule.value !== undefined ? `: ${rule.value ? "enabled" : "disabled"}` : ""}
        </p>
      );
    case "aura":
      return (
        <p>
          Aura:{" "}
          {rule.radius !== undefined ? `${rule.radius} feet` : "radius unavailable"}
          {rule.traits?.length ? `; ${rule.traits.join(", ")}` : ""}
        </p>
      );
    case "damage_dice":
      return (
        <p>
          Damage dice:{" "}
          {[rule.dice_number, rule.die_size, rule.damage_type]
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
      return null;
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
  const stats = [
    {
      key: "ac",
      label: "AC",
      value: runtime.defenses?.armor_class.adjusted_value,
    },
    { key: "hardness", label: "Hardness", value: body.defenses?.hardness },
    {
      key: "broken-threshold",
      label: "BT",
      value: hazard.broken_threshold?.adjusted_value,
    },
    {
      key: "fortitude",
      label: "Fort",
      signed: true,
      value: runtime.saves?.fortitude?.adjusted_value,
    },
    {
      key: "reflex",
      label: "Ref",
      signed: true,
      value: runtime.saves?.reflex?.adjusted_value,
    },
    {
      key: "will",
      label: "Will",
      signed: true,
      value: runtime.saves?.will?.adjusted_value,
    },
    {
      key: "detection",
      label: "Detection DC",
      value: hazard.detection_dc?.adjusted_value,
    },
  ];
  const hasStats = stats.some((stat) => stat.value !== undefined);
  const hasIwr = Boolean(
    body.defenses?.immunities?.length ||
    body.defenses?.weaknesses?.length ||
    body.defenses?.resistances?.length,
  );
  return hasStats || hasIwr || hazard.initiative_suggestion ? (
    <SurfaceSection title="Hazard runtime">
      <DefenseStats ariaLabel="Hazard runtime statistics" values={stats} />
      {hazard.initiative_suggestion ? (
        <p className="creature-sheet__detail-note">
          Initiative suggestion: {formatSlug(hazard.initiative_suggestion.statistic)}{" "}
          {formatSigned(hazard.initiative_suggestion.modifier.adjusted_value)}
        </p>
      ) : null}
      <DefenseIwrList
        immunities={body.defenses?.immunities}
        resistances={body.defenses?.resistances}
        weaknesses={body.defenses?.weaknesses}
      />
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

function HazardSourceDisclosure({
  body,
  metadata,
}: {
  body: HazardSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  const license = body.provenance.publication_license;
  const metadataFacts = body.provenance.source_metadata
    .map(sourceMetadataItem)
    .filter((item): item is RecordKeyValueItem => item !== null);
  const sourceFacts = [
    textItem("publication", "Publication", metadata.source?.publication_title),
    textItem("pack", "Source pack", metadata.source?.pack_label),
    license.state === "value" ? textItem("license", "License", license.value) : null,
    ...metadataFacts,
  ].filter((item): item is RecordKeyValueItem => item !== null);
  if (!sourceFacts.length) return null;
  return (
    <Collapse
      className="record-surface__secondary"
      ghost
      items={[
        {
          key: "source",
          label: "Source & provenance",
          children: (
            <RecordKeyValueList
              ariaLabel="Hazard provenance"
              items={sourceFacts}
              labelWidth="provenance"
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

function hazardHitPoints(
  hitPoints: HazardSurfaceDefensesView["hit_points"],
): string | undefined {
  if (!hitPoints) return undefined;
  const { current, maximum } = hitPoints;
  if (maximum !== undefined && (current === undefined || current === maximum)) {
    return maximum.toString();
  }
  if (current !== undefined && maximum !== undefined) return `${current}/${maximum}`;
  return current?.toString();
}

function sourceMetadataItem(
  fact: HazardSurfaceSourceMetadataFactView,
  index: number,
): RecordKeyValueItem | null {
  if (fact.value.state !== "typed") return null;
  const key = `${fact.field}:${index}`;
  const component =
    "component_label" in fact && fact.component_label
      ? `${fact.component_label} · `
      : "";
  switch (fact.field) {
    case "token_name":
      return textItem(key, "Token name", fact.value.value.trim() || undefined);
    case "has_health":
      return textItem(
        key,
        "Health compatibility",
        fact.value.value ? "Health present" : "No health",
      );
    case "temporary_maximum":
      return textItem(key, "Temporary maximum", fact.value.value.toString());
    case "save_detail":
      return textItem(
        key,
        `${formatSlug(fact.save)} source note`,
        fact.value.value.trim() || undefined,
      );
    case "item_rarity":
      return textItem(
        key,
        `${component}Component rarity`,
        fact.value.value.trim() ? formatSlug(fact.value.value) : undefined,
      );
    case "item_lineage":
      return fact.value.value.compendium_source.state === "typed"
        ? textItem(key, `${component}Component lineage`, "Compendium source recorded")
        : null;
    case "strike_attack":
      return textItem(
        key,
        `${component}Strike source attack`,
        formatSigned(fact.value.value),
      );
    case "strike_weapon_type":
      return textItem(
        key,
        `${component}Strike source mode`,
        formatSlug(fact.value.value),
      );
    case "strike_attack_effects_custom":
      return textItem(
        key,
        `${component}Custom attack effect`,
        fact.value.value.trim() || undefined,
      );
  }
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

function hazardAttackEffectLabel(
  effect: NonNullable<HazardSurfaceActivityView["attack_effects"]>[number],
) {
  switch (effect) {
    case "no_multiple_attack_penalty":
      return "No multiple attack penalty";
    case "independent_limbs":
      return "Independent limbs";
  }
}

function formatEmitsSound(body: HazardSurfaceView): string | undefined {
  if (!body.emits_sound) return undefined;
  return body.emits_sound.sound_type === "boolean"
    ? body.emits_sound.value
      ? "Always — eligible for hearing detection; other detection conditions still apply."
      : "Never — not eligible for hearing detection."
    : "During encounters — eligible for hearing detection while participating in a started encounter. Other detection conditions still apply; this does not describe an audio clip or guarantee discovery.";
}
