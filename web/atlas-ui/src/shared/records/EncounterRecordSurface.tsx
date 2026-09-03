import { Alert, Button, Collapse, Empty, Popover, Space, Tag } from "antd";
import { Info } from "lucide-react";
import type React from "react";
import type {
  CreatureSurfaceView,
  CreatureSurfaceIwrView,
  CreatureSurfaceSenseView,
  EncounterRuntimeAutomationLimitationView,
  EncounterRuntimeActionCostKindView,
  EncounterRuntimeActivityView,
  EncounterRuntimeSpellView,
  EncounterRuntimeSpellcastingView,
  EncounterRuntimeView,
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
  ReferenceAndSourceContent,
  SurfaceSection,
  TraitRow,
} from "./CreatureRecordSurface";
import { formatRank, formatSigned, formatSlug } from "./recordFormatting";
import {
  narrativeContent,
  RecordReference,
  RichContent,
  type ReferenceHandler,
} from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";

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
  runtime,
  slots,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
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
        <SurfaceSection className="record-surface-card--conditions" title="Conditions">
          {slots.conditions ?? <RuntimeConditions runtime={runtime} />}
        </SurfaceSection>
      </div>
      <div className="record-surface-structured__grid">
        <div className="record-surface-structured__column">
          <RuntimeCoreFacts body={body} runtime={runtime} />
          <RuntimeActionBudget runtime={runtime} />
          <RuntimeMovement runtime={runtime} />
          <RuntimeSkills runtime={runtime} />
          {slots.notes && (
            <SurfaceSection title="Participant Note">{slots.notes}</SurfaceSection>
          )}
        </div>
        <div className="record-surface-structured__column">
          <RuntimeActivities onReference={onReference} runtime={runtime} />
          <RuntimeSpellcasting onReference={onReference} runtime={runtime} />
          <RuntimeResources runtime={runtime} />
          <RuntimeAutomationLimitations runtime={runtime} />
        </div>
      </div>
      <Collapse
        className="record-surface__secondary"
        ghost
        items={[
          ...(narrative.length
            ? [
                {
                  key: "description",
                  label: "Description & Lore",
                  children: narrative.map((document) => (
                    <RichContent
                      content={document}
                      key={document.content_key}
                      onReference={onReference}
                    />
                  )),
                },
              ]
            : []),
          {
            key: "source",
            label: "References & Source",
            children: (
              <ReferenceAndSourceContent
                body={body}
                metadata={metadata}
                onReference={onReference}
              />
            ),
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
  return (
    <SurfaceSection title="Turn Economy">
      <dl className="creature-sheet__stat-list">
        <RuntimeCountFact value={budget.actions} />
        <RuntimeCountFact value={budget.reactions} />
      </dl>
      <div className="encounter-action-capabilities">
        <Tag>{budget.can_act.available ? "Can act" : "Cannot act"}</Tag>
        <Tag>{budget.can_react.available ? "Can react" : "Cannot react"}</Tag>
      </div>
      {(budget.can_act.reason || budget.can_react.reason) && (
        <ul className="encounter-runtime-notes">
          {budget.can_act.reason && <li>{budget.can_act.reason}</li>}
          {budget.can_react.reason && <li>{budget.can_react.reason}</li>}
        </ul>
      )}
      <RuntimeNotes notes={budget.notes} />
    </SurfaceSection>
  );
}

function RuntimeVitals({ runtime }: { runtime: EncounterRuntimeView | undefined }) {
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

function RuntimeConditions({ runtime }: { runtime: EncounterRuntimeView | undefined }) {
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
  return (
    <SurfaceSection title="Movement">
      <div className="creature-sheet__movement-list">
        {speeds.map((speed) => (
          <div className="creature-sheet__movement" key={speed.movement_type}>
            <span>{speed.label}</span>
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
          </div>
        ))}
      </div>
    </SurfaceSection>
  );
}

function RuntimeSkills({ runtime }: { runtime: EncounterRuntimeView | undefined }) {
  if (!runtime?.skills?.length) return null;
  return (
    <SurfaceSection title="Skills">
      <div className="creature-sheet__chip-list">
        {runtime.skills.map((skill) => (
          <span className="creature-sheet__skill" key={skill.skill_id}>
            <span>{skill.label}</span>
            <RuntimeAdjustedValue
              adjustments={skill.modifier.modifiers}
              adjusted={skill.modifier.adjusted_value}
              base={skill.modifier.base_value}
              label={skill.label}
              provenance={skill.modifier.provenance}
              signed
              suppressedAdjustments={skill.modifier.suppressed_modifiers}
            />
          </span>
        ))}
      </div>
    </SurfaceSection>
  );
}

function RuntimeActivities({
  onReference,
  runtime,
}: {
  onReference: ReferenceHandler;
  runtime: EncounterRuntimeView | undefined;
}) {
  if (!runtime?.activities?.length) return null;
  return (
    <SurfaceSection title="Actions & Abilities">
      <div className="creature-sheet__activity-list">
        {runtime.activities.map((activity) => (
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
        {activity.action_cost && (
          <span className="creature-sheet__action-cost">
            {formatRuntimeActionCost(activity.action_cost.value)}
          </span>
        )}
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
  runtime,
}: {
  onReference: ReferenceHandler;
  runtime: EncounterRuntimeView | undefined;
}) {
  const entries = runtime?.spellcasting ?? [];
  const standalone = runtime?.standalone_spells ?? [];
  if (!entries.length && !standalone.length) return null;
  return (
    <SurfaceSection title="Spellcasting">
      <Collapse
        className="record-surface__inline-disclosure"
        ghost
        items={[
          ...entries.map((entry) => ({
            key: entry.entry_id,
            label: <RuntimeSpellcastingHeading entry={entry} />,
            children: <RuntimeSpellRoster entry={entry} onReference={onReference} />,
          })),
          ...(standalone.length
            ? [
                {
                  key: "standalone",
                  label: <strong>Standalone Spells</strong>,
                  children: standalone.map((spell) => (
                    <RuntimeSpell
                      key={spell.occurrence_id}
                      onReference={onReference}
                      spell={spell}
                    />
                  )),
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
    entry.dc ? `DC ${entry.dc.adjusted_value}` : undefined,
    entry.attack ? `attack ${formatSigned(entry.attack.adjusted_value)}` : undefined,
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
}: {
  entry: EncounterRuntimeSpellcastingView;
  onReference: ReferenceHandler;
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
            <div>
              {slot && (
                <span className="creature-sheet__spell-slots">
                  <RuntimeCountValue value={slot} /> slot
                  {slot.adjusted_value === 1 ? "" : "s"}
                </span>
              )}
              {spells.map((spell, index) => (
                <span key={spell.occurrence_id}>
                  {index > 0 && ", "}
                  <RecordReference
                    label={spell.label}
                    onReference={onReference}
                    recordKey={spell.target_record_key}
                  />
                </span>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function RuntimeSpell({
  onReference,
  spell,
}: {
  onReference: ReferenceHandler;
  spell: EncounterRuntimeSpellView;
}) {
  return (
    <div className="creature-sheet__standalone-spell">
      <RecordReference
        label={spell.label}
        onReference={onReference}
        recordKey={spell.target_record_key}
      />
      {spell.content?.map((document) => (
        <RichContent
          content={document}
          key={document.content_key}
          onReference={onReference}
        />
      ))}
    </div>
  );
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
    contextItem("languages", "Languages", awareness?.languages?.join(", ")),
    contextItem("language-notes", "Language notes", awareness?.language_details),
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

function RuntimeCountFact({ value }: { value: RuntimeCountView }) {
  return (
    <div>
      <dt>{value.label}</dt>
      <dd>
        <RuntimeCountValue value={value} />
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

function RuntimeAutomationLimitations({
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

function formatRuntimeActionCost(cost: EncounterRuntimeActionCostKindView) {
  if (cost.kind === "passive") return "Passive";
  if (cost.kind === "reaction") return "Reaction";
  if (cost.kind === "free_action") return "Free action";
  if (cost.kind === "actions") {
    return `${cost.count} action${cost.count === 1 ? "" : "s"}`;
  }
  return cost.value;
}
