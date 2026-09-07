import { Alert, Button, Collapse, Empty, Popover, Space, Tag, Tooltip } from "antd";
import { Info } from "lucide-react";
import { useId } from "react";
import type React from "react";
import type {
  CreatureSurfaceView,
  CreatureSurfaceIwrView,
  CreatureSurfaceSenseView,
  EncounterRuntimeAutomationLimitationView,
  EncounterRuntimeActivityView,
  EncounterRuntimeSpellView,
  EncounterRuntimeSpellcastingView,
  EncounterRuntimeView,
  EncounterSpellCastBlockedReasonView,
  EncounterSpellCastRequest,
  EncounterSpellCastUnavailableReasonView,
  RecordSurfaceMetadataView,
  RuntimeAdjustmentView,
  RuntimeCountSegmentView,
  RuntimeCountView,
  RuntimeEffectNoteView,
  RuntimeFactProvenanceView,
  RuntimeFormulaView,
  RuntimeNumberView,
  RuntimeRollView,
} from "../../generated/atlas";
import {
  CreatureSourceContent,
  NarrativeSection,
  SurfaceSection,
  TraitRow,
} from "./CreatureRecordSurface";
import { ActionGlyph } from "./ActionGlyph";
import { formatRank, formatSigned, formatSlug } from "./recordFormatting";
import {
  narrativeContent,
  RichContent,
  type ReferenceHandler,
} from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import { SpellPreviewPopover } from "./SpellOccurrencePreviewPopover";

export type EncounterRecordSurfaceSlots = {
  conditions?: React.ReactNode;
  header?: React.ReactNode;
  header_actions?: React.ReactNode;
  notes?: React.ReactNode;
  vitals?: React.ReactNode;
};

export function EncounterParticipantSurface({
  body,
  metadata,
  onReference,
  onSpellCast,
  runtime,
  slots,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  onSpellCast?: (request: EncounterSpellCastRequest) => void;
  runtime: EncounterRuntimeView | undefined;
  slots: EncounterRecordSurfaceSlots;
}) {
  const narrative = narrativeContent(body.content);
  return (
    <article className="record-surface record-surface--encounter-participant">
      <header className="record-surface-structured__header">
        <div>
          <div className="record-surface-structured__title-row">
            <h2>{metadata.title}</h2>
            <RuntimeIdentity metadata={metadata} runtime={runtime} />
          </div>
          <TraitRow compact metadata={metadata} />
        </div>
        <div className="record-surface-structured__header-actions">
          {slots.header_actions}
        </div>
      </header>
      {slots.header && (
        <div className="record-surface-structured__participant">{slots.header}</div>
      )}
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
      <div className="record-surface-structured__grid">
        <div className="record-surface-structured__column">
          <RuntimeCoreFacts body={body} runtime={runtime} />
          <RuntimeActionBudget runtime={runtime} />
          <RuntimeMovement runtime={runtime} />
          <RuntimeSkills runtime={runtime} />
          <RuntimeCommunication body={body} />
          {slots.notes && (
            <SurfaceSection title="Participant Note">{slots.notes}</SurfaceSection>
          )}
        </div>
        <div className="record-surface-structured__column">
          <RuntimeActivities
            activityType="active"
            onReference={onReference}
            runtime={runtime}
            title="Actions"
          />
          <RuntimeSpellcasting
            onReference={onReference}
            onSpellCast={onSpellCast}
            runtime={runtime}
          />
          <RuntimeActivities
            activityType="passive"
            onReference={onReference}
            runtime={runtime}
            title="Passives"
          />
          <RuntimeResources runtime={runtime} />
          <RuntimeAutomationLimitations runtime={runtime} />
        </div>
      </div>
      <NarrativeSection
        content={narrative}
        headingId="encounter-description"
        onReference={onReference}
        title="Description & Lore"
      />
      <Collapse
        className="record-surface__secondary"
        ghost
        items={[
          {
            key: "source",
            label: "Source & provenance",
            children: <CreatureSourceContent body={body} metadata={metadata} />,
          },
        ]}
        size="small"
      />
    </article>
  );
}

function RuntimeIdentity({
  metadata,
  runtime,
}: {
  metadata: RecordSurfaceMetadataView;
  runtime: EncounterRuntimeView | undefined;
}) {
  return (
    <div className="creature-sheet__identity-meta creature-sheet__identity-meta--compact">
      <span className="creature-sheet__kind">
        {metadata.kind_label || formatSlug(metadata.kind)}
      </span>
      {(runtime?.level || metadata.level !== undefined) && (
        <span className="creature-sheet__level">
          Level{" "}
          {runtime?.level ? (
            <RuntimeNumberValue value={runtime.level} />
          ) : (
            metadata.level
          )}
        </span>
      )}
    </div>
  );
}

function RuntimeCoreFacts({
  body,
  runtime,
}: {
  body: CreatureSurfaceView;
  runtime: EncounterRuntimeView | undefined;
}) {
  if (!runtime) {
    return <Alert message="Runtime facts are unavailable." showIcon type="info" />;
  }
  const facts = [
    ["AC", runtime.defenses?.armor_class],
    ["Fortitude", runtime.saves?.fortitude],
    ["Reflex", runtime.saves?.reflex],
    ["Will", runtime.saves?.will],
    ["Perception", runtime.awareness?.perception],
  ].filter((entry): entry is [string, RuntimeNumberView] => entry[1] !== undefined);
  const abilities = runtime.abilities
    ? [
        ["Str", runtime.abilities.strength],
        ["Dex", runtime.abilities.dexterity],
        ["Con", runtime.abilities.constitution],
        ["Int", runtime.abilities.intelligence],
        ["Wis", runtime.abilities.wisdom],
        ["Cha", runtime.abilities.charisma],
      ].filter((entry): entry is [string, RuntimeNumberView] => entry[1] !== undefined)
    : [];
  if (!facts.length && !abilities.length) return null;
  return (
    <SurfaceSection title="Combat Snapshot">
      <dl className="creature-sheet__stat-list">
        {facts.map(([label, value]) => (
          <RuntimeNumberFact key={label} signed={label !== "AC"} value={value} />
        ))}
      </dl>
      {abilities.length > 0 && (
        <dl className="creature-sheet__ability-grid">
          {abilities.map(([label, value]) => (
            <RuntimeNumberFact key={label} label={label} signed value={value} />
          ))}
        </dl>
      )}
      <RuntimeCanonicalContext body={body} />
    </SurfaceSection>
  );
}

function RuntimeActionBudget({
  runtime,
}: {
  runtime: EncounterRuntimeView | undefined;
}) {
  const budget = runtime?.action_budget;
  if (!budget) return null;
  const fullyAvailable = budget.can_act.available && budget.can_react.available;
  return (
    <SurfaceSection title="Turn Economy">
      {fullyAvailable ? (
        <div
          aria-label={`${budget.actions.adjusted_value} ${budget.actions.label}, ${budget.reactions.adjusted_value} ${budget.reactions.label}; can act and react`}
          className="encounter-action-summary"
        >
          <span>
            <strong>{budget.actions.adjusted_value}</strong> {budget.actions.label}
          </span>
          <span>
            <strong>{budget.reactions.adjusted_value}</strong> {budget.reactions.label}
          </span>
        </div>
      ) : (
        <>
          <dl className="creature-sheet__stat-list">
            <div>
              <dt>{budget.actions.label}</dt>
              <dd>{budget.actions.adjusted_value}</dd>
            </div>
            <div>
              <dt>{budget.reactions.label}</dt>
              <dd>{budget.reactions.adjusted_value}</dd>
            </div>
          </dl>
          <div className="encounter-action-capabilities">
            <div className="encounter-action-capability">
              <Tag>{budget.can_act.available ? "Can act" : "Cannot act"}</Tag>
              {budget.can_act.reason ? <small>{budget.can_act.reason}</small> : null}
            </div>
            <div className="encounter-action-capability">
              <Tag>{budget.can_react.available ? "Can react" : "Cannot react"}</Tag>
              {budget.can_react.reason ? (
                <small>{budget.can_react.reason}</small>
              ) : null}
            </div>
          </div>
        </>
      )}
      <RuntimeNotes notes={budget.notes} />
    </SurfaceSection>
  );
}

export function RuntimeVitals({
  runtime,
}: {
  runtime: EncounterRuntimeView | undefined;
}) {
  const vitals = runtime?.vitals;
  if (!vitals) return <span className="record-surface__muted">No runtime vitals</span>;
  return (
    <p className="record-surface__runtime-vitals">
      <strong>
        {vitals.current_hp ?? "—"} / {vitals.maximum_hp?.adjusted_value ?? "—"} HP
      </strong>
      {vitals.temporary_hp > 0 && <span>+{vitals.temporary_hp} temporary</span>}
    </p>
  );
}

export function RuntimeConditions({
  runtime,
}: {
  runtime: EncounterRuntimeView | undefined;
}) {
  if (!runtime?.conditions?.length) {
    return <span className="record-surface__muted">No active conditions</span>;
  }
  return (
    <Space size={[4, 4]} wrap>
      {runtime.conditions.map((condition) => (
        <Tag key={condition.condition_id}>
          {condition.name}
          {condition.value === undefined ? "" : ` ${condition.value}`}
        </Tag>
      ))}
    </Space>
  );
}

function RuntimeMovement({ runtime }: { runtime: EncounterRuntimeView | undefined }) {
  const speeds = runtime?.movement?.speeds;
  if (!speeds?.length) return null;
  const items: RecordKeyValueItem[] = speeds.map((speed) => ({
    key: speed.movement_type,
    label: speed.label,
    value: (
      <RuntimeAdjustedValue
        adjustments={speed.adjustments}
        adjusted={speed.adjusted_value_feet}
        base={speed.base_value_feet}
        label={speed.label}
        notes={speed.notes}
        provenance={speed.provenance}
        suppressedAdjustments={speed.suppressed_adjustments}
        suffix=" ft"
      />
    ),
  }));
  return (
    <SurfaceSection title="Movement">
      <RecordKeyValueList ariaLabel="Movement" items={items} />
    </SurfaceSection>
  );
}

function RuntimeSkills({ runtime }: { runtime: EncounterRuntimeView | undefined }) {
  if (!runtime?.skills?.length) return null;
  return (
    <SurfaceSection className="creature-sheet__panel--skills" title="Skills">
      <ul aria-label="Skills" className="creature-sheet__skill-grid">
        {runtime.skills.map((skill) => (
          <li className="creature-sheet__skill-cell" key={skill.skill_id}>
            <div className="creature-sheet__skill-heading">
              <span>{skill.label}</span>
              <strong>
                <RuntimeAdjustedValue
                  adjustments={skill.modifier.modifiers}
                  adjusted={skill.modifier.adjusted_value}
                  base={skill.modifier.base_value}
                  label={skill.label}
                  provenance={skill.modifier.provenance}
                  signed
                  suppressedAdjustments={skill.modifier.suppressed_modifiers}
                />
              </strong>
            </div>
          </li>
        ))}
      </ul>
    </SurfaceSection>
  );
}

function RuntimeCommunication({ body }: { body: CreatureSurfaceView }) {
  const awareness = body.awareness;
  const languages = awareness?.languages ?? [];
  const items: RecordKeyValueItem[] = [];
  if (languages.length) {
    items.push({
      key: "languages",
      label: "Languages",
      value: (
        <ul aria-label="Languages" className="creature-sheet__compact-multi-value-list">
          {languages.map((language, index) => (
            <li
              className="creature-sheet__compact-multi-value-item"
              key={`${language}:${index}`}
            >
              <span>{formatSlug(language)}</span>
            </li>
          ))}
        </ul>
      ),
    });
  }
  if (awareness?.language_details) {
    items.push({
      key: "language-details",
      label: "Details",
      rowClassName: "creature-sheet__fact-group-note",
      value: awareness.language_details,
    });
  }
  if (!items.length) return null;
  return (
    <SurfaceSection title="Languages & communication">
      <RecordKeyValueList
        ariaLabel="Languages and communication"
        className="creature-sheet__fact-group-rows"
        items={items}
      />
    </SurfaceSection>
  );
}

export function RuntimeActivities({
  activityType,
  onReference,
  runtime,
  title,
}: {
  activityType: "active" | "passive";
  onReference: ReferenceHandler;
  runtime: EncounterRuntimeView | undefined;
  title: string;
}) {
  const activities = (runtime?.activities ?? []).filter((activity) =>
    activityType === "passive"
      ? activity.action_cost?.value.kind === "passive"
      : activity.action_cost?.value.kind !== "passive",
  );
  if (!activities.length) return null;
  return (
    <SurfaceSection title={title}>
      <div className="creature-sheet__activity-list">
        {activities.map((activity) => (
          <RuntimeActivity
            activity={activity}
            key={activity.activity_id}
            onReference={onReference}
          />
        ))}
      </div>
    </SurfaceSection>
  );
}

function RuntimeActivity({
  activity,
  onReference,
}: {
  activity: EncounterRuntimeActivityView;
  onReference: ReferenceHandler;
}) {
  const summary = (
    <div className="creature-sheet__activity-summary">
      <div className="creature-sheet__activity-heading">
        <strong>{activity.label}</strong>
        {activity.action_cost && <ActionGlyph cost={activity.action_cost.value} />}
      </div>
      {activity.traits?.length ? (
        <Space size={[4, 4]} wrap>
          {activity.traits.map((trait) => (
            <Tag key={trait}>{formatSlug(trait)}</Tag>
          ))}
        </Space>
      ) : null}
      <div className="creature-sheet__activity-mechanics">
        {activity.rolls?.map((roll) => (
          <span key={roll.roll_id}>
            {roll.label} {formatSigned(roll.adjusted_value)}
          </span>
        ))}
        {activity.damage?.map((damage) => (
          <span key={damage.damage_id}>
            {[damage.adjusted_formula ?? damage.formula, damage.damage_type]
              .filter(Boolean)
              .join(" ")}
          </span>
        ))}
      </div>
    </div>
  );
  const details = <RuntimeActivityDetails activity={activity} />;
  return activity.content?.length || hasRuntimeActivityDetails(activity) ? (
    <Collapse
      className="creature-sheet__activity creature-sheet__activity--expandable"
      defaultActiveKey={[activity.activity_id]}
      ghost
      items={[
        {
          key: activity.activity_id,
          label: summary,
          children: (
            <div className="encounter-runtime-activity-details">
              {details}
              {activity.content?.map((document) => (
                <RichContent
                  content={document}
                  key={document.content_key}
                  onReference={onReference}
                />
              ))}
            </div>
          ),
        },
      ]}
      size="small"
    />
  ) : (
    <article className="creature-sheet__activity">{summary}</article>
  );
}

function RuntimeSpellcasting({
  onReference,
  onSpellCast,
  runtime,
}: {
  onReference: ReferenceHandler;
  onSpellCast?: (request: EncounterSpellCastRequest) => void;
  runtime: EncounterRuntimeView | undefined;
}) {
  const entries = runtime?.spellcasting ?? [];
  const standalone = runtime?.standalone_spells ?? [];
  if (!entries.length && !standalone.length) return null;
  return (
    <SurfaceSection title="Spellcasting">
      <Collapse
        className="record-surface__inline-disclosure"
        defaultActiveKey={[
          ...entries.map((entry) => entry.entry_id),
          ...(standalone.length ? ["standalone"] : []),
        ]}
        destroyOnHidden
        ghost
        items={[
          ...entries.map((entry) => ({
            key: entry.entry_id,
            label: <RuntimeSpellcastingHeading entry={entry} />,
            children: (
              <RuntimeSpellRoster
                entry={entry}
                onReference={onReference}
                onSpellCast={onSpellCast}
              />
            ),
          })),
          ...(standalone.length
            ? [
                {
                  key: "standalone",
                  label: <strong>Standalone Spells</strong>,
                  children: (
                    <RuntimeSpellGroup
                      onReference={onReference}
                      onSpellCast={onSpellCast}
                      spells={standalone}
                    />
                  ),
                },
              ]
            : []),
        ]}
        size="small"
      />
    </SurfaceSection>
  );
}

function RuntimeSpellcastingHeading({
  entry,
}: {
  entry: EncounterRuntimeSpellcastingView;
}) {
  const meta = [
    entry.tradition ? formatSlug(entry.tradition) : undefined,
    entry.preparation ? formatSlug(entry.preparation) : undefined,
  ].filter(Boolean);
  return (
    <span className="creature-sheet__spell-heading">
      <strong>{entry.label}</strong>
      {meta.length > 0 && <small>{meta.join(" · ")}</small>}
    </span>
  );
}

function RuntimeSpellRoster({
  entry,
  onReference,
  onSpellCast,
}: {
  entry: EncounterRuntimeSpellcastingView;
  onReference: ReferenceHandler;
  onSpellCast?: (request: EncounterSpellCastRequest) => void;
}) {
  const groups = groupRuntimeSpells(entry.spells ?? []);
  const slots = new Map((entry.slots ?? []).map((slot) => [slot.rank, slot.maximum]));
  if (!groups.length && !slots.size) {
    return (
      <Empty description="No spells listed" image={Empty.PRESENTED_IMAGE_SIMPLE} />
    );
  }
  return (
    <div className="creature-sheet__spell-roster">
      <RuntimeSpellcastingMechanics entry={entry} />
      {groups.map(([rank, spells]) => {
        const slot = typeof rank === "number" ? slots.get(rank) : undefined;
        return (
          <div className="creature-sheet__spell-rank" key={rank}>
            <strong>{rank === "Unranked" ? rank : formatRank(rank)}</strong>
            <div className="encounter-runtime-spell-rank-content">
              {slot && (
                <span className="creature-sheet__spell-slots">
                  <RuntimeCountValue value={slot} /> slot
                  {slot.adjusted_value === 1 ? "" : "s"}
                </span>
              )}
              <RuntimeSpellGroup
                onReference={onReference}
                onSpellCast={onSpellCast}
                spells={spells}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}

function RuntimeSpellGroup({
  onReference,
  onSpellCast,
  spells,
}: {
  onReference: ReferenceHandler;
  onSpellCast?: (request: EncounterSpellCastRequest) => void;
  spells: EncounterRuntimeSpellView[];
}) {
  return (
    <div className="creature-sheet__spell-links">
      {spells.map((spell, index) => (
        <span
          className="creature-sheet__spell-link encounter-runtime-spell"
          key={spell.occurrence_id}
        >
          <RuntimeSpell
            onReference={onReference}
            onSpellCast={onSpellCast}
            spell={spell}
          />
          {index < spells.length - 1 ? (
            <span aria-hidden="true" className="creature-sheet__spell-separator">
              ,
            </span>
          ) : null}
        </span>
      ))}
    </div>
  );
}

function RuntimeSpell({
  onReference,
  onSpellCast,
  spell,
}: {
  onReference: ReferenceHandler;
  onSpellCast?: (request: EncounterSpellCastRequest) => void;
  spell: EncounterRuntimeSpellView;
}) {
  const metadata = [
    spell.rank === undefined ? undefined : formatRank(spell.rank),
    ...(spell.traits ?? []).map(formatSlug),
  ].filter((value): value is string => Boolean(value));
  return (
    <>
      <SpellPreviewPopover
        actionCost={spell.activity?.action_cost?.value}
        actions={<RuntimeSpellControls onSpellCast={onSpellCast} spell={spell} />}
        label={spell.label}
        metadata={metadata.join(" · ")}
        onOpenSpellRecord={onReference}
        previewContent={
          <div className="encounter-runtime-spell-details">
            <p className="encounter-runtime-spell-availability">
              <strong>{spellCastStateLabel(spell.cast)}</strong>
            </p>
            {spell.activity && <RuntimeActivityDetails activity={spell.activity} />}
            {spell.content?.map((document) => (
              <RichContent
                content={document}
                key={document.content_key}
                onReference={onReference}
              />
            ))}
          </div>
        }
        targetRecordKey={spell.target_record_key}
      />
      {spell.activity?.action_cost ? (
        <span className="encounter-runtime-spell-cost">
          <ActionGlyph cost={spell.activity.action_cost.value} />
        </span>
      ) : null}
      <small className="encounter-runtime-spell-state">
        {spellCastStateLabel(spell.cast)}
      </small>
    </>
  );
}

function RuntimeSpellControls({
  onSpellCast,
  spell,
}: {
  onSpellCast?: (request: EncounterSpellCastRequest) => void;
  spell: EncounterRuntimeSpellView;
}) {
  const controlId = useId();
  const target = spell.cast.spend_target;
  const blockedReason = spell.cast.blocked_reason
    ? spellCastBlockedReasonLabel(spell.cast.blocked_reason)
    : undefined;
  const castDisabledReason = !onSpellCast
    ? "Casting controls are unavailable"
    : !target
      ? "Casting target unavailable"
      : (blockedReason ??
        (!spell.cast.available ? "Casting is unavailable" : undefined));
  const tracked =
    spell.cast.state.state_type === "tracked" ? spell.cast.state : undefined;
  const restoreDisabledReason = !onSpellCast
    ? "Restore controls are unavailable"
    : !target
      ? "Casting target unavailable"
      : !tracked
        ? spell.cast.state.state_type === "at_will"
          ? "At-will spells do not consume uses"
          : "Tracked uses are unavailable"
        : tracked.remaining >= tracked.initial_remaining
          ? "Already at the creation baseline"
          : undefined;
  const mutate = (operation: EncounterSpellCastRequest["operation"]) => {
    if (!onSpellCast || !target) return;
    onSpellCast({
      spell_occurrence_id: spell.occurrence_id,
      spend_target: target,
      operation,
    });
  };
  return (
    <div className="encounter-runtime-spell-controls">
      <Space size="small">
        <Tooltip title={castDisabledReason}>
          <span>
            <Button
              aria-describedby={
                castDisabledReason ? `${controlId}-cast-reason` : undefined
              }
              aria-label={`Cast ${spell.label}`}
              disabled={Boolean(castDisabledReason)}
              onClick={(event) => {
                event.stopPropagation();
                mutate("cast_one");
              }}
              size="small"
              type="primary"
            >
              Cast
            </Button>
          </span>
        </Tooltip>
        <Tooltip title={restoreDisabledReason}>
          <span>
            <Button
              aria-describedby={
                restoreDisabledReason ? `${controlId}-restore-reason` : undefined
              }
              aria-label={`Restore one use of ${spell.label}`}
              disabled={Boolean(restoreDisabledReason)}
              onClick={(event) => {
                event.stopPropagation();
                mutate("restore_one");
              }}
              size="small"
            >
              Restore
            </Button>
          </span>
        </Tooltip>
      </Space>
      {castDisabledReason ? (
        <span className="sr-only" id={`${controlId}-cast-reason`}>
          {castDisabledReason}
        </span>
      ) : null}
      {restoreDisabledReason ? (
        <span className="sr-only" id={`${controlId}-restore-reason`}>
          {restoreDisabledReason}
        </span>
      ) : null}
    </div>
  );
}

function spellCastStateLabel(cast: EncounterRuntimeSpellView["cast"]): string {
  if (cast.state.state_type === "at_will") return "At will";
  if (cast.state.state_type === "tracked") {
    return `${cast.state.remaining} of ${cast.state.maximum} remaining`;
  }
  return `Unavailable: ${spellCastUnavailableReasonLabel(cast.state.reason)}`;
}

function spellCastBlockedReasonLabel(
  reason: EncounterSpellCastBlockedReasonView,
): string {
  const labels: Record<EncounterSpellCastBlockedReasonView, string> = {
    exhausted: "Cast unavailable at 0 remaining uses",
    participant_defeated: "Defeated participants cannot cast",
    state_unavailable: "Casting state unavailable",
  };
  return labels[reason];
}

function spellCastUnavailableReasonLabel(
  reason: EncounterSpellCastUnavailableReasonView,
): string {
  const labels: Record<EncounterSpellCastUnavailableReasonView, string> = {
    missing_current: "current uses were not provided by the source",
    missing_maximum: "maximum uses were not provided by the source",
    unsafe_integer: "the source value is outside the supported range",
    missing_identity: "the source does not provide a stable casting identity",
    ambiguous_ownership: "the source casting ownership is ambiguous",
    unsupported_preparation: "this preparation type is not supported",
    unresolved_participant: "the participant record could not be resolved",
    state_unavailable: "tracked casting state is unavailable",
  };
  return labels[reason];
}

function RuntimeResources({ runtime }: { runtime: EncounterRuntimeView | undefined }) {
  if (!runtime?.resources?.length) return null;
  const facts: RecordKeyValueItem[] = runtime.resources.map((resource) => ({
    key: resource.resource_id,
    label: resource.label,
    value: (
      <span
        aria-label={`${resource.current?.adjusted_value ?? "Unavailable"} of ${resource.maximum.adjusted_value} ${resource.label}`}
        className="encounter-runtime-resource-value"
      >
        {resource.current ? (
          <RuntimeNumberValue
            label={`${resource.label} current`}
            value={resource.current}
          />
        ) : (
          "—"
        )}
        <span aria-hidden="true">/</span>
        <RuntimeNumberValue
          label={`${resource.label} maximum`}
          value={resource.maximum}
        />
      </span>
    ),
  }));
  return (
    <SurfaceSection title="Resources">
      <RecordKeyValueList ariaLabel="Resources" items={facts} />
    </SurfaceSection>
  );
}

function RuntimeCanonicalContext({ body }: { body: CreatureSurfaceView }) {
  const defense = body.defenses;
  const awareness = body.awareness;
  const saves = body.saves;
  const shield = defense?.shield;
  const items = [
    contextItem("ac-context", "AC context", defense?.armor_class_details),
    contextItem("hardness", "Hardness", defense?.hardness),
    contextItem(
      "shield",
      "Shield",
      shield
        ? [
            shield.armor_class_bonus === undefined
              ? undefined
              : `+${shield.armor_class_bonus} AC`,
            shield.hardness === undefined ? undefined : `Hardness ${shield.hardness}`,
            shield.maximum_hit_points === undefined
              ? undefined
              : `${shield.maximum_hit_points} HP`,
            shield.broken_threshold === undefined
              ? undefined
              : `BT ${shield.broken_threshold}`,
          ]
            .filter(Boolean)
            .join(" · ")
        : undefined,
    ),
    contextItem("immunities", "Immunities", formatIwrList(defense?.immunities)),
    contextItem("weaknesses", "Weaknesses", formatIwrList(defense?.weaknesses)),
    contextItem("resistances", "Resistances", formatIwrList(defense?.resistances)),
    contextItem("all-saves", "Save notes", saves?.all_saves_note),
    contextItem("fortitude-note", "Fortitude", saves?.fortitude?.details),
    contextItem("reflex-note", "Reflex", saves?.reflex?.details),
    contextItem("will-note", "Will", saves?.will?.details),
    contextItem("perception-note", "Perception", awareness?.details),
    contextItem("senses", "Senses", awareness?.senses?.map(formatSense).join(", ")),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  if (!items.length) return null;
  return (
    <RecordKeyValueList
      ariaLabel="Runtime canonical context"
      items={items}
      labelWidth="provenance"
    />
  );
}

function RuntimeNumberFact({
  label,
  signed = false,
  value,
}: {
  label?: string;
  signed?: boolean;
  value: RuntimeNumberView;
}) {
  return (
    <div>
      <dt>{label ?? value.label}</dt>
      <dd>
        <RuntimeNumberValue label={label} signed={signed} value={value} />
      </dd>
    </div>
  );
}

function RuntimeNumberValue({
  label,
  signed = false,
  suffix = "",
  value,
}: {
  label?: string;
  signed?: boolean;
  suffix?: string;
  value: RuntimeNumberView;
}) {
  return (
    <RuntimeAdjustedValue
      adjustments={value.modifiers}
      adjusted={value.adjusted_value}
      base={value.base_value}
      label={label ?? value.label}
      provenance={value.provenance}
      signed={signed}
      suffix={suffix}
      suppressedAdjustments={value.suppressed_modifiers}
    />
  );
}

function RuntimeCountValue({ value }: { value: RuntimeCountView }) {
  return (
    <RuntimeAdjustedValue
      adjustments={value.adjustments}
      adjusted={value.adjusted_value}
      base={value.base_value}
      label={value.label}
      provenance={value.provenance}
      segments={value.segments}
      suppressedAdjustments={value.suppressed_adjustments}
    />
  );
}

type RuntimeDelta = Pick<RuntimeAdjustmentView, "label" | "provenance" | "value"> & {
  reason?: string;
};

function RuntimeAdjustedValue({
  adjustments,
  adjusted,
  base,
  label,
  notes,
  provenance,
  segments,
  signed = false,
  suffix = "",
  suppressedAdjustments,
}: {
  adjustments?: RuntimeDelta[];
  adjusted: number;
  base: number;
  label: string;
  notes?: RuntimeEffectNoteView[];
  provenance: RuntimeFactProvenanceView;
  segments?: RuntimeCountSegmentView[];
  signed?: boolean;
  suffix?: string;
  suppressedAdjustments?: RuntimeDelta[];
}) {
  const format = (value: number) => `${signed ? formatSigned(value) : value}${suffix}`;
  const hasDetails = Boolean(
    adjusted !== base ||
    adjustments?.length ||
    suppressedAdjustments?.length ||
    segments?.length ||
    notes?.length,
  );
  return (
    <span
      className={[
        "encounter-runtime-value",
        adjusted === base ? "" : "creature-sheet__adjusted",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      <span>{format(adjusted)}</span>
      {adjusted !== base && <small>base {format(base)}</small>}
      {hasDetails && (
        <RuntimeDetailsButton
          adjustments={adjustments}
          adjusted={format(adjusted)}
          base={format(base)}
          label={label}
          notes={notes}
          provenance={provenance}
          segments={segments}
          suppressedAdjustments={suppressedAdjustments}
        />
      )}
    </span>
  );
}

function RuntimeDetailsButton({
  adjustments,
  adjusted,
  base,
  label,
  notes,
  provenance,
  segments,
  suppressedAdjustments,
}: {
  adjustments?: RuntimeDelta[];
  adjusted: React.ReactNode;
  base: React.ReactNode;
  label: string;
  notes?: RuntimeEffectNoteView[];
  provenance: RuntimeFactProvenanceView;
  segments?: RuntimeCountSegmentView[];
  suppressedAdjustments?: RuntimeDelta[];
}) {
  return (
    <Popover
      content={
        <div className="encounter-runtime-explanation">
          <dl>
            <div>
              <dt>Current</dt>
              <dd>{adjusted}</dd>
            </div>
            <div>
              <dt>Base</dt>
              <dd>{base}</dd>
            </div>
            <div>
              <dt>Source</dt>
              <dd>{runtimeSourceLabel(provenance)}</dd>
            </div>
          </dl>
          <RuntimeDeltaList label="Applied" values={adjustments} />
          <RuntimeDeltaList label="Suppressed" values={suppressedAdjustments} />
          {segments?.length ? (
            <section>
              <strong>Budget</strong>
              <ul>
                {segments.map((segment, index) => (
                  <li key={`${segment.label}:${index}`}>
                    <span>
                      {segment.label}: {segment.value}
                    </span>
                    {segment.restricted && (
                      <small>{segment.reason ?? "Restricted"}</small>
                    )}
                  </li>
                ))}
              </ul>
            </section>
          ) : null}
          <RuntimeNotes notes={notes} />
        </div>
      }
      placement="bottom"
      title={`${label} details`}
      trigger="click"
    >
      <Button
        aria-label={`${label} adjustment details`}
        className="encounter-runtime-value__details"
        icon={<Info size={12} />}
        size="small"
        type="text"
      />
    </Popover>
  );
}

function RuntimeDeltaList({
  label,
  values,
}: {
  label: string;
  values?: RuntimeDelta[];
}) {
  if (!values?.length) return null;
  return (
    <section>
      <strong>{label}</strong>
      <ul>
        {values.map((value, index) => (
          <li key={`${value.label}:${index}`}>
            <span>
              {value.label} {formatSigned(value.value)}
            </span>
            <small>
              {[value.reason, runtimeSourceLabel(value.provenance)]
                .filter(Boolean)
                .join(" · ")}
            </small>
          </li>
        ))}
      </ul>
    </section>
  );
}

function RuntimeNotes({ notes }: { notes: RuntimeEffectNoteView[] | undefined }) {
  if (!notes?.length) return null;
  return (
    <ul className="encounter-runtime-notes">
      {notes.map((note, index) => (
        <li key={`${note.label}:${index}`}>
          <strong>{note.label}</strong> {note.reason}
        </li>
      ))}
    </ul>
  );
}

function RuntimeActivityDetails({
  activity,
}: {
  activity: EncounterRuntimeActivityView;
}) {
  const facts = [
    contextItem("frequency", "Frequency", formatRuntimeFrequency(activity.frequency)),
    contextItem("uses", "Uses", formatRuntimeUses(activity.uses)),
    contextItem(
      "usage",
      "Usage",
      activity.usage === "ambiguous" ? "Requires adjudication" : undefined,
    ),
  ].filter((item): item is RecordKeyValueItem => item !== null);
  const adjustedRolls = (activity.rolls ?? []).filter(hasRuntimeRollDetails);
  const adjustedDamage = (activity.damage ?? []).filter(hasRuntimeFormulaDetails);
  return (
    <>
      {facts.length ? (
        <RecordKeyValueList
          ariaLabel={`${activity.label} runtime details`}
          items={facts}
        />
      ) : null}
      {(adjustedRolls.length > 0 || adjustedDamage.length > 0) && (
        <div className="encounter-runtime-adjustment-links">
          {adjustedRolls.map((roll) => (
            <RuntimeRollDetails key={roll.roll_id} roll={roll} />
          ))}
          {adjustedDamage.map((damage) => (
            <RuntimeFormulaDetails damage={damage} key={damage.damage_id} />
          ))}
        </div>
      )}
      {activity.modes?.length ? (
        <ul className="encounter-runtime-modes">
          {activity.modes.map((mode) => (
            <li key={mode.mode_id}>
              <strong>{mode.label}</strong>
              <span>
                {[mode.target, mode.range, mode.time].filter(Boolean).join(" · ")}
              </span>
            </li>
          ))}
        </ul>
      ) : null}
    </>
  );
}

function RuntimeSpellcastingMechanics({
  entry,
}: {
  entry: EncounterRuntimeSpellcastingView;
}) {
  if (!entry.attack && !entry.dc) return null;
  return (
    <dl className="encounter-runtime-spellcasting-mechanics">
      {entry.dc && <RuntimeRollFact label="DC" roll={entry.dc} />}
      {entry.attack && (
        <RuntimeRollFact label="Spell attack" roll={entry.attack} signed />
      )}
    </dl>
  );
}

function RuntimeRollFact({
  label,
  roll,
  signed = false,
}: {
  label: string;
  roll: RuntimeRollView;
  signed?: boolean;
}) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>
        <RuntimeAdjustedValue
          adjustments={roll.modifiers}
          adjusted={roll.adjusted_value}
          base={roll.base_value}
          label={roll.label}
          provenance={roll.provenance}
          signed={signed}
          suppressedAdjustments={roll.suppressed_modifiers}
        />
      </dd>
    </div>
  );
}

function RuntimeRollDetails({ roll }: { roll: RuntimeRollView }) {
  const format = (value: number) =>
    roll.surface === "attack_roll" ? formatSigned(value) : value.toString();
  return (
    <RuntimeDetailsButton
      adjustments={roll.modifiers}
      adjusted={format(roll.adjusted_value)}
      base={format(roll.base_value)}
      label={roll.label}
      provenance={roll.provenance}
      suppressedAdjustments={roll.suppressed_modifiers}
    />
  );
}

function RuntimeFormulaDetails({ damage }: { damage: RuntimeFormulaView }) {
  const label =
    damage.label ??
    (damage.damage_type ? `${formatSlug(damage.damage_type)} damage` : "Damage");
  return (
    <RuntimeDetailsButton
      adjustments={damage.modifiers}
      adjusted={damage.adjusted_formula ?? damage.formula}
      base={damage.formula}
      label={label}
      provenance={damage.provenance}
    />
  );
}

function hasRuntimeActivityDetails(activity: EncounterRuntimeActivityView) {
  return Boolean(
    activity.frequency ||
    activity.uses ||
    activity.usage === "ambiguous" ||
    activity.modes?.length ||
    activity.rolls?.some(hasRuntimeRollDetails) ||
    activity.damage?.some(hasRuntimeFormulaDetails),
  );
}

function hasRuntimeRollDetails(roll: RuntimeRollView) {
  return Boolean(
    roll.base_value !== roll.adjusted_value ||
    roll.modifiers?.length ||
    roll.suppressed_modifiers?.length,
  );
}

function hasRuntimeFormulaDetails(damage: RuntimeFormulaView) {
  return Boolean(damage.adjusted_formula || damage.modifiers?.length);
}

function formatRuntimeFrequency(frequency: EncounterRuntimeActivityView["frequency"]) {
  if (!frequency) return undefined;
  if (frequency.maximum !== undefined && frequency.period) {
    return `${frequency.maximum} per ${formatSlug(frequency.period)}`;
  }
  if (frequency.maximum !== undefined) return `${frequency.maximum} maximum`;
  return frequency.period ? formatSlug(frequency.period) : undefined;
}

function formatRuntimeUses(uses: EncounterRuntimeActivityView["uses"]) {
  return uses?.maximum === undefined ? undefined : `${uses.maximum} maximum`;
}

function runtimeSourceLabel(provenance: RuntimeFactProvenanceView) {
  const source = provenance.source;
  if (source.source_type === "canonical_record") return "Source record";
  if (source.source_type === "participant_state") return "Participant state";
  if (source.source_type === "participant_variant") {
    return `${formatSlug(source.variant)} variant`;
  }
  if (source.source_type === "condition") return source.label;
  return formatSlug(source.rule);
}

function limitationTargetLabel(limitation: EncounterRuntimeAutomationLimitationView) {
  const target = limitation.target;
  if (target.target_type === "participant") return "Participant";
  if (target.target_type === "condition") return "Condition";
  if (target.target_type === "activity") return "Action or ability";
  if (target.target_type === "spell") return "Spell";
  return "Spellcasting";
}

function contextItem(
  key: React.Key,
  label: React.ReactNode,
  value: React.ReactNode | undefined | null,
): RecordKeyValueItem | null {
  return value === undefined || value === null || value === ""
    ? null
    : { key, label, value };
}

function formatSense(sense: CreatureSurfaceSenseView) {
  return [
    formatSlug(sense.kind),
    sense.acuity ? formatSlug(sense.acuity) : undefined,
    sense.range_feet === undefined ? undefined : `${sense.range_feet} ft`,
  ]
    .filter(Boolean)
    .join(" ");
}

function formatIwrList(values: CreatureSurfaceIwrView[] | undefined) {
  if (!values?.length) return undefined;
  return values
    .map((value) => {
      const exceptions = value.exceptions?.length
        ? ` (except ${value.exceptions.join(", ")})`
        : "";
      const doubled = value.double_vs?.length
        ? `; double vs. ${value.double_vs.join(", ")}`
        : "";
      return `${formatSlug(value.kind)}${
        value.amount === undefined ? "" : ` ${value.amount}`
      }${exceptions}${doubled}`;
    })
    .join(", ");
}

export function RuntimeAutomationLimitations({
  runtime,
}: {
  runtime: EncounterRuntimeView | undefined;
}) {
  if (!runtime?.automation_limitations?.length) return null;
  return (
    <SurfaceSection title="Automation Notes">
      <ul className="encounter-runtime-limitations">
        {runtime.automation_limitations.map((limitation, index) => (
          <li key={`${limitation.code}:${index}`}>
            <Tag>{limitationTargetLabel(limitation)}</Tag>
            <span>{limitation.message}</span>
          </li>
        ))}
      </ul>
    </SurfaceSection>
  );
}

function groupRuntimeSpells(
  spells: EncounterRuntimeSpellView[],
): Array<[number | "Unranked", EncounterRuntimeSpellView[]]> {
  const groups = new Map<number | "Unranked", EncounterRuntimeSpellView[]>();
  for (const spell of [...spells].sort(
    (left, right) => left.authored_order - right.authored_order,
  )) {
    const rank = spell.rank ?? "Unranked";
    groups.set(rank, [...(groups.get(rank) ?? []), spell]);
  }
  return [...groups.entries()].sort(([left], [right]) => {
    const leftValue = typeof left === "number" ? left : -1;
    const rightValue = typeof right === "number" ? right : -1;
    return rightValue - leftValue;
  });
}
