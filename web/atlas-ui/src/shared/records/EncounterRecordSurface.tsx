import { Alert, Collapse, Empty, Space, Tag } from "antd";
import type React from "react";
import type {
  CreatureSurfaceView,
  EncounterRuntimeActionCostKindView,
  EncounterRuntimeActivityView,
  EncounterRuntimeSpellView,
  EncounterRuntimeSpellcastingView,
  EncounterRuntimeView,
  RecordSurfaceMetadataView,
  RuntimeNumberView,
} from "../../generated/atlas";
import {
  formatRank,
  formatSigned,
  formatSlug,
  IdentityMetadata,
  ReferenceAndSourceContent,
  RuntimeAdjustedValue,
  RuntimeFact,
  SurfaceSection,
  TraitRow,
} from "./CreatureRecordSurface";
import {
  narrativeContent,
  RecordReference,
  RichContent,
  type ReferenceHandler,
} from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";

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
  slots: Record<string, React.ReactNode>;
}) {
  const narrative = narrativeContent(body.content);
  return (
    <article className="record-surface record-surface--encounter-participant">
      <header className="record-surface-structured__header">
        <div>
          <div className="record-surface-structured__title-row">
            <h2>{metadata.title}</h2>
            <IdentityMetadata compact metadata={metadata} />
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
          <RuntimeCoreFacts runtime={runtime} />
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

function RuntimeCoreFacts({ runtime }: { runtime: EncounterRuntimeView | undefined }) {
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
          <RuntimeFact
            key={label}
            label={label}
            signed={label !== "AC"}
            value={value}
          />
        ))}
      </dl>
      {abilities.length > 0 && (
        <dl className="creature-sheet__ability-grid">
          {abilities.map(([label, value]) => (
            <RuntimeFact key={label} label={label} signed value={value} />
          ))}
        </dl>
      )}
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
        <RuntimeFact label="Actions" value={budget.actions} />
        <RuntimeFact label="Reactions" value={budget.reactions} />
      </dl>
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
              adjusted={speed.adjusted_value_feet}
              base={speed.base_value_feet}
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
              adjusted={skill.modifier.adjusted_value}
              base={skill.modifier.base_value}
              signed
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
  return activity.content?.length ? (
    <Collapse
      className="creature-sheet__activity creature-sheet__activity--expandable"
      ghost
      items={[
        {
          key: activity.activity_id,
          label: summary,
          children: activity.content.map((document) => (
            <RichContent
              content={document}
              key={document.content_key}
              onReference={onReference}
            />
          )),
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
      {groups.map(([rank, spells]) => {
        const slot = typeof rank === "number" ? slots.get(rank) : undefined;
        return (
          <div className="creature-sheet__spell-rank" key={rank}>
            <strong>{rank === "Unranked" ? rank : formatRank(rank)}</strong>
            <div>
              {slot && (
                <span className="creature-sheet__spell-slots">
                  {slot.adjusted_value} slot{slot.adjusted_value === 1 ? "" : "s"}
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
    value: `${resource.current?.adjusted_value ?? "—"} / ${resource.maximum.adjusted_value}`,
  }));
  return (
    <SurfaceSection title="Resources">
      <RecordKeyValueList ariaLabel="Resources" items={facts} />
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
