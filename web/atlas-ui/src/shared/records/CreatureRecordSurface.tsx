import { Collapse, Empty, Space, Tag } from "antd";
import type React from "react";
import type {
  CreatureSurfaceActionCostView,
  CreatureSurfaceActivityView,
  CreatureSurfaceContentView,
  CreatureSurfaceIwrView,
  CreatureSurfaceMovementView,
  CreatureSurfaceRelationshipView,
  CreatureSurfaceSpellView,
  CreatureSurfaceSpellcastingView,
  CreatureSurfaceView,
  RecordSurfaceMetadataView,
  RuntimeNumberView,
} from "../../generated/atlas";
import {
  contentLabel,
  narrativeContent,
  RecordReference,
  RichContent,
  type ReferenceHandler,
} from "./RecordRichContent";

export function CreatureDetailSurface({
  body,
  metadata,
  onReference,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
}) {
  const narrative = narrativeContent(body.content);
  const standalone = (body.content ?? []).filter(
    (content) => content.role === "embedded_capability",
  );
  return (
    <article className="record-surface record-surface--record-detail">
      <RecordHeader metadata={metadata} />
      <NarrativeSection content={narrative} onReference={onReference} />
      <CreatureSnapshot body={body} />
      <div className="creature-sheet__facts-grid">
        <DefensePanel body={body} />
        <SensesLanguagesPanel body={body} />
        <MovementPanel movement={body.movement} />
        <SkillsPanel body={body} />
        <AbilitiesPanel body={body} />
      </div>
      <div className="creature-sheet__mechanics-grid">
        <div className="creature-sheet__mechanics-main">
          <ActivitySection activities={body.activities} onReference={onReference} />
        </div>
        <aside className="creature-sheet__mechanics-side">
          <SpellcastingSection
            entries={body.spellcasting}
            onReference={onReference}
            standalone={standalone}
          />
          <ResourcesSection resources={body.resources} />
        </aside>
      </div>
      <ReferenceAndSourceDisclosure
        body={body}
        metadata={metadata}
        onReference={onReference}
      />
    </article>
  );
}

export function SearchCompactSurface({
  body,
  metadata,
  onReference,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
}) {
  const description = narrativeContent(body.content)[0];
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <div className="record-surface-search__heading">
          <h2>{metadata.title}</h2>
        </div>
        <IdentityMetadata compact metadata={metadata} />
        <TraitRow compact metadata={metadata} />
        {description && (
          <RichContent compact content={description} onReference={onReference} />
        )}
      </div>
      <CompactCreatureFacts body={body} />
      <div className="record-surface-search__meta">
        <strong>{sourceLabel(metadata)}</strong>
        {metadata.source?.pack_label &&
          metadata.source.pack_label !== sourceLabel(metadata) && (
            <small>{metadata.source.pack_label}</small>
          )}
      </div>
    </article>
  );
}

export function RecordHeader({ metadata }: { metadata: RecordSurfaceMetadataView }) {
  return (
    <header className="creature-sheet__header">
      <div className="creature-sheet__identity">
        <h2>{metadata.title}</h2>
        <IdentityMetadata metadata={metadata} />
        <TraitRow metadata={metadata} />
      </div>
    </header>
  );
}

export function IdentityMetadata({
  compact = false,
  metadata,
}: {
  compact?: boolean;
  metadata: RecordSurfaceMetadataView;
}) {
  return (
    <div
      className={[
        "creature-sheet__identity-meta",
        compact ? "creature-sheet__identity-meta--compact" : "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      <span className="creature-sheet__kind">
        {metadata.kind_label || formatSlug(metadata.kind)}
      </span>
      {metadata.level !== undefined && (
        <span className="creature-sheet__level">Level {metadata.level}</span>
      )}
    </div>
  );
}

export function TraitRow({
  compact = false,
  metadata,
}: {
  compact?: boolean;
  metadata: RecordSurfaceMetadataView;
}) {
  const traits = [
    ...(metadata.rarity && metadata.rarity !== "common" ? [metadata.rarity] : []),
    ...(metadata.traits ?? []),
  ];
  if (!traits.length) return null;
  return (
    <Space
      className={[
        "creature-sheet__traits",
        compact ? "creature-sheet__traits--compact" : "",
      ]
        .filter(Boolean)
        .join(" ")}
      size={[5, 5]}
      wrap
    >
      {traits.map((trait) => (
        <Tag key={trait}>{formatSlug(trait)}</Tag>
      ))}
    </Space>
  );
}

function NarrativeSection({
  content,
  onReference,
}: {
  content: CreatureSurfaceContentView[];
  onReference: ReferenceHandler;
}) {
  if (!content.length) return null;
  const [primary, ...additional] = content;
  return (
    <section className="creature-sheet__narrative" aria-labelledby="record-description">
      <div className="creature-sheet__section-heading">
        <div>
          <p className="eyebrow">About this creature</p>
          <h3 id="record-description">Description & Lore</h3>
        </div>
      </div>
      <RichContent content={primary} onReference={onReference} />
      {additional.length > 0 && (
        <Collapse
          className="record-surface__inline-disclosure"
          ghost
          items={additional.map((document) => ({
            key: document.content_key,
            label: contentLabel(document),
            children: <RichContent content={document} onReference={onReference} />,
          }))}
          size="small"
        />
      )}
    </section>
  );
}

function CreatureSnapshot({ body }: { body: CreatureSurfaceView }) {
  const facts = [
    compactFact("Perception", body.awareness?.perception, true),
    compactFact("AC", body.defenses?.armor_class),
    compactFact("HP", body.vitals?.hit_points),
    compactFact("Fortitude", body.saves?.fortitude?.modifier, true),
    compactFact("Reflex", body.saves?.reflex?.modifier, true),
    compactFact("Will", body.saves?.will?.modifier, true),
  ].filter(isCompactFact);
  if (!facts.length) return null;
  return (
    <section aria-label="Core creature statistics" className="creature-sheet__snapshot">
      {facts.map(({ label, value }) => (
        <div className="creature-sheet__snapshot-fact" key={label}>
          <span>{label}</span>
          <strong>{value}</strong>
        </div>
      ))}
    </section>
  );
}

function CompactCreatureFacts({ body }: { body: CreatureSurfaceView }) {
  const facts = [
    compactFact("Perception", body.awareness?.perception, true),
    compactFact("AC", body.defenses?.armor_class),
    compactFact("HP", body.vitals?.hit_points),
    compactFact("Fort", body.saves?.fortitude?.modifier, true),
    compactFact("Ref", body.saves?.reflex?.modifier, true),
    compactFact("Will", body.saves?.will?.modifier, true),
  ].filter(isCompactFact);
  return (
    <dl className="record-surface-search__facts">
      {facts.map(({ label, value }) => (
        <Fact key={label} label={label} value={value} />
      ))}
    </dl>
  );
}

function DefensePanel({ body }: { body: CreatureSurfaceView }) {
  const { defenses, saves, vitals } = body;
  const hasAdditionalDefenseContext = Boolean(
    defenses?.armor_class_details ||
    vitals?.details ||
    saves?.all_saves_note ||
    defenses?.immunities?.length ||
    defenses?.resistances?.length ||
    defenses?.weaknesses?.length,
  );
  if (!hasAdditionalDefenseContext) return null;
  return (
    <SurfaceSection className="creature-sheet__panel--defenses" title="Defenses">
      {defenses?.armor_class_details && (
        <p className="creature-sheet__detail-note">{defenses.armor_class_details}</p>
      )}
      {vitals?.details && (
        <p className="creature-sheet__detail-note">{vitals.details}</p>
      )}
      {saves?.all_saves_note && (
        <p className="creature-sheet__detail-note">{saves.all_saves_note}</p>
      )}
      <IwrGroup label="Immunities" values={defenses?.immunities} />
      <IwrGroup label="Resistances" values={defenses?.resistances} />
      <IwrGroup label="Weaknesses" values={defenses?.weaknesses} />
    </SurfaceSection>
  );
}

function SensesLanguagesPanel({ body }: { body: CreatureSurfaceView }) {
  const awareness = body.awareness;
  if (
    !awareness ||
    (!awareness.senses?.length &&
      !awareness.languages?.length &&
      !awareness.details &&
      !awareness.language_details)
  )
    return null;
  return (
    <SurfaceSection
      className="creature-sheet__panel--senses"
      title="Senses & Languages"
    >
      {awareness.senses?.length ? (
        <FactLine
          label="Senses"
          value={awareness.senses
            .map((sense) =>
              [
                formatSlug(sense.kind),
                sense.acuity ? `(${formatSlug(sense.acuity)})` : "",
                sense.range_feet === undefined ? "" : `${sense.range_feet} ft`,
              ]
                .filter(Boolean)
                .join(" "),
            )
            .join(", ")}
        />
      ) : null}
      {awareness.languages?.length ? (
        <FactLine label="Languages" value={awareness.languages.join(", ")} />
      ) : null}
      {awareness.details && (
        <p className="creature-sheet__detail-note">{awareness.details}</p>
      )}
      {awareness.language_details && (
        <p className="creature-sheet__detail-note">{awareness.language_details}</p>
      )}
    </SurfaceSection>
  );
}

export function MovementPanel({
  movement,
}: {
  movement: CreatureSurfaceMovementView[] | undefined;
}) {
  if (!movement?.length) return null;
  return (
    <SurfaceSection className="creature-sheet__panel--movement" title="Movement">
      <div className="creature-sheet__movement-list">
        {movement.map((entry) => (
          <div className="creature-sheet__movement" key={entry.component_id}>
            <span>{formatSlug(entry.label ?? entry.mode)}</span>
            <strong>
              {entry.speed_feet === undefined ? "—" : `${entry.speed_feet} ft`}
            </strong>
            {entry.details && <small>{entry.details}</small>}
          </div>
        ))}
      </div>
    </SurfaceSection>
  );
}

function SkillsPanel({ body }: { body: CreatureSurfaceView }) {
  if (!body.skills?.length) return null;
  return (
    <SurfaceSection className="creature-sheet__panel--skills" title="Skills">
      <div className="creature-sheet__chip-list">
        {body.skills.map((skill) => (
          <span className="creature-sheet__skill" key={skill.component_id}>
            <span>{skill.label}</span>
            {skill.modifier !== undefined && (
              <strong>{formatSigned(skill.modifier)}</strong>
            )}
            {skill.note && <small>{skill.note}</small>}
          </span>
        ))}
      </div>
    </SurfaceSection>
  );
}

function AbilitiesPanel({ body }: { body: CreatureSurfaceView }) {
  const abilities = body.abilities;
  if (!abilities) return null;
  const values = [
    ["Str", abilities.strength],
    ["Dex", abilities.dexterity],
    ["Con", abilities.constitution],
    ["Int", abilities.intelligence],
    ["Wis", abilities.wisdom],
    ["Cha", abilities.charisma],
  ].filter((entry): entry is [string, number] => entry[1] !== undefined);
  if (!values.length) return null;
  return (
    <SurfaceSection
      className="creature-sheet__panel--abilities"
      title="Ability Modifiers"
    >
      <dl className="creature-sheet__ability-grid">
        {values.map(([label, value]) => (
          <Fact key={label} label={label} signed value={value} />
        ))}
      </dl>
    </SurfaceSection>
  );
}

function ActivitySection({
  activities,
  onReference,
}: {
  activities: CreatureSurfaceActivityView[] | undefined;
  onReference: ReferenceHandler;
}) {
  if (!activities?.length) return null;
  return (
    <SurfaceSection className="creature-sheet__activities" title="Actions & Abilities">
      <div className="creature-sheet__activity-list">
        {[...activities]
          .sort((left, right) => left.authored_order - right.authored_order)
          .map((activity) => (
            <StaticActivity
              activity={activity}
              key={activity.occurrence_id}
              onReference={onReference}
            />
          ))}
      </div>
    </SurfaceSection>
  );
}

function StaticActivity({
  activity,
  onReference,
}: {
  activity: CreatureSurfaceActivityView;
  onReference: ReferenceHandler;
}) {
  const content = activity.content ?? [];
  const summary = <ActivitySummary activity={activity} />;
  return content.length ? (
    <Collapse
      className="creature-sheet__activity creature-sheet__activity--expandable"
      ghost
      items={[
        {
          key: activity.occurrence_id,
          label: summary,
          children: content.map((document) => (
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

function ActivitySummary({ activity }: { activity: CreatureSurfaceActivityView }) {
  return (
    <div className="creature-sheet__activity-summary">
      <div className="creature-sheet__activity-heading">
        <strong>{activity.label}</strong>
        <ActionCost cost={activity.action_cost} />
      </div>
      {activity.traits?.length ? (
        <Space className="creature-sheet__activity-traits" size={[4, 4]} wrap>
          {activity.traits.map((trait) => (
            <Tag key={trait}>{formatSlug(trait)}</Tag>
          ))}
        </Space>
      ) : null}
      {(activity.rolls?.length || activity.damage?.length) && (
        <div className="creature-sheet__activity-mechanics">
          {activity.rolls?.map((roll) => (
            <span key={roll.roll_id}>
              {roll.label}
              {roll.modifier === undefined ? "" : ` ${formatSigned(roll.modifier)}`}
            </span>
          ))}
          {activity.damage?.map((damage) => (
            <span key={damage.damage_id}>
              {[damage.formula, damage.damage_type, damage.category]
                .filter(Boolean)
                .join(" ")}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

function ActionCost({ cost }: { cost: CreatureSurfaceActionCostView | undefined }) {
  if (!cost || cost.cost_type === "passive") return null;
  const label =
    cost.cost_type === "actions"
      ? `${cost.count} action${cost.count === 1 ? "" : "s"}`
      : cost.cost_type === "free_action"
        ? "Free action"
        : cost.cost_type === "reaction"
          ? "Reaction"
          : cost.value;
  return <span className="creature-sheet__action-cost">{label}</span>;
}

function SpellcastingSection({
  entries,
  onReference,
  standalone,
}: {
  entries: CreatureSurfaceSpellcastingView[] | undefined;
  onReference: ReferenceHandler;
  standalone: CreatureSurfaceContentView[];
}) {
  if (!entries?.length && !standalone.length) return null;
  const items: NonNullable<React.ComponentProps<typeof Collapse>["items"]> = [
    ...(entries ?? [])
      .slice()
      .sort((left, right) => left.authored_order - right.authored_order)
      .map((entry) => ({
        key: entry.occurrence_id,
        label: <SpellcastingHeading entry={entry} />,
        children: <SpellRoster onReference={onReference} spells={entry.spells} />,
      })),
    ...(standalone.length
      ? [
          {
            key: "standalone",
            label: <strong>Standalone Spells & Rituals</strong>,
            children: (
              <div className="creature-sheet__standalone-content">
                {standalone.map((document) => (
                  <section key={document.content_key}>
                    <h4>{contentLabel(document)}</h4>
                    <RichContent content={document} onReference={onReference} />
                  </section>
                ))}
              </div>
            ),
          },
        ]
      : []),
  ];
  return (
    <SurfaceSection className="creature-sheet__spellcasting" title="Spellcasting">
      <Collapse
        className="record-surface__inline-disclosure"
        defaultActiveKey={items.length === 1 ? [String(items[0]?.key ?? "")] : []}
        ghost
        items={items}
        size="small"
      />
    </SurfaceSection>
  );
}

function SpellcastingHeading({ entry }: { entry: CreatureSurfaceSpellcastingView }) {
  const meta = [
    entry.tradition ? formatSlug(entry.tradition) : undefined,
    entry.preparation ? formatSlug(entry.preparation) : undefined,
    entry.difficulty_class === undefined ? undefined : `DC ${entry.difficulty_class}`,
    entry.attack_modifier === undefined
      ? undefined
      : `attack ${formatSigned(entry.attack_modifier)}`,
  ].filter(Boolean);
  return (
    <span className="creature-sheet__spell-heading">
      <strong>{entry.label}</strong>
      {meta.length > 0 && <small>{meta.join(" · ")}</small>}
    </span>
  );
}

function SpellRoster({
  onReference,
  spells,
}: {
  onReference: ReferenceHandler;
  spells: CreatureSurfaceSpellView[] | undefined;
}) {
  if (!spells?.length) {
    return (
      <Empty description="No spells listed" image={Empty.PRESENTED_IMAGE_SIMPLE} />
    );
  }
  return (
    <div className="creature-sheet__spell-roster">
      {groupSpells(spells).map(([rank, ranked]) => (
        <div className="creature-sheet__spell-rank" key={rank}>
          <strong>{rank}</strong>
          <div>
            {ranked.map((spell, index) => (
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
      ))}
    </div>
  );
}

function ResourcesSection({
  resources,
}: {
  resources: CreatureSurfaceView["resources"];
}) {
  if (!resources?.length) return null;
  return (
    <SurfaceSection title="Resources">
      <div className="creature-sheet__resource-list">
        {resources.map((resource) => (
          <div key={resource.component_id}>
            <span>{resource.label}</span>
            {resource.maximum !== undefined && <strong>{resource.maximum}</strong>}
          </div>
        ))}
      </div>
    </SurfaceSection>
  );
}

export function ReferenceAndSourceDisclosure({
  body,
  metadata,
  onReference,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
}) {
  return (
    <Collapse
      className="record-surface__secondary"
      ghost
      items={[
        {
          key: "references-source",
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
  );
}

export function ReferenceAndSourceContent({
  body,
  metadata,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
}) {
  return (
    <div className="creature-sheet__reference-source">
      {body.relationships?.length ? (
        <section>
          <h4>References</h4>
          <ul className="creature-sheet__reference-list">
            {body.relationships.map((relationship) => (
              <RelationshipRow
                key={`${relationship.source_occurrence_id}:${relationship.kind}`}
                relationship={relationship}
              />
            ))}
          </ul>
        </section>
      ) : null}
      <SourceDetails metadata={metadata} />
    </div>
  );
}

function RelationshipRow({
  relationship,
}: {
  relationship: CreatureSurfaceRelationshipView;
}) {
  const target = relationship.target;
  const label =
    relationship.contextual_label ??
    (target.target_type === "occurrence" ? target.occurrence_id : target.source_id);
  return (
    <li>
      <span>{formatSlug(relationship.kind)}</span>
      <code>{label}</code>
    </li>
  );
}

function SourceDetails({ metadata }: { metadata: RecordSurfaceMetadataView }) {
  const source = metadata.source;
  return (
    <section>
      <h4>Provenance</h4>
      <dl className="creature-sheet__provenance-list">
        {metadata.record_key && (
          <Fact
            className="creature-sheet__provenance-row"
            label="Record ID"
            value={metadata.record_key}
          />
        )}
        {source?.publication_title && (
          <Fact
            className="creature-sheet__provenance-row"
            label="Publication"
            value={source.publication_title}
          />
        )}
        {source?.pack_label && (
          <Fact
            className="creature-sheet__provenance-row"
            label="Source pack"
            value={source.pack_label}
          />
        )}
        {source?.source_path && (
          <Fact
            className="creature-sheet__provenance-row"
            label="Source path"
            value={source.source_path}
          />
        )}
      </dl>
    </section>
  );
}

export function SurfaceSection({
  children,
  className,
  title,
}: {
  children: React.ReactNode;
  className?: string;
  title: string;
}) {
  return (
    <section className={["creature-sheet__panel", className].filter(Boolean).join(" ")}>
      <h3>{title}</h3>
      {children}
    </section>
  );
}

export function Fact({
  className,
  label,
  signed = false,
  value,
}: {
  className?: string;
  label: string;
  signed?: boolean;
  value: number | string;
}) {
  return (
    <div className={className}>
      <dt>{label}</dt>
      <dd>{typeof value === "number" && signed ? formatSigned(value) : value}</dd>
    </div>
  );
}

export function RuntimeFact({
  label,
  signed = false,
  value,
}: {
  label: string;
  signed?: boolean;
  value: RuntimeNumberView;
}) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>
        <RuntimeAdjustedValue
          adjusted={value.adjusted_value}
          base={value.base_value}
          signed={signed}
        />
      </dd>
    </div>
  );
}

export function RuntimeAdjustedValue({
  adjusted,
  base,
  signed = false,
  suffix = "",
}: {
  adjusted: number;
  base: number;
  signed?: boolean;
  suffix?: string;
}) {
  const format = (value: number) => `${signed ? formatSigned(value) : value}${suffix}`;
  return (
    <span className={adjusted === base ? "" : "creature-sheet__adjusted"}>
      {format(adjusted)}
      {adjusted !== base && <small> base {format(base)}</small>}
    </span>
  );
}

function FactLine({ label, value }: { label: string; value: string }) {
  return (
    <p className="creature-sheet__fact-line">
      <strong>{label}</strong>
      <span>{value}</span>
    </p>
  );
}

function IwrGroup({
  label,
  values,
}: {
  label: string;
  values: CreatureSurfaceIwrView[] | undefined;
}) {
  if (!values?.length) return null;
  return <FactLine label={label} value={values.map(formatIwr).join(", ")} />;
}

function formatIwr(value: CreatureSurfaceIwrView) {
  const exceptions = value.exceptions?.length
    ? ` (except ${value.exceptions.join(", ")})`
    : "";
  const doubled = value.double_vs?.length
    ? `; double vs. ${value.double_vs.join(", ")}`
    : "";
  return `${formatSlug(value.kind)}${value.amount === undefined ? "" : ` ${value.amount}`}${exceptions}${doubled}`;
}

export function formatSigned(value: number) {
  return value >= 0 ? `+${value}` : value.toString();
}

export function formatSlug(value: string) {
  return value
    .replace(/[_-]+/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function sourceLabel(metadata: RecordSurfaceMetadataView) {
  return metadata.source?.publication_title ?? metadata.source?.pack_label ?? "";
}

type CompactFact = { label: string; value: string };
function compactFact(
  label: string,
  value: number | undefined,
  signed = false,
): CompactFact | null {
  return value === undefined
    ? null
    : { label, value: signed ? formatSigned(value) : value.toString() };
}
function isCompactFact(value: CompactFact | null): value is CompactFact {
  return value !== null;
}

function groupSpells(
  spells: CreatureSurfaceSpellView[],
): Array<[string, CreatureSurfaceSpellView[]]> {
  const groups = new Map<string, CreatureSurfaceSpellView[]>();
  for (const spell of [...spells].sort(
    (left, right) => left.authored_order - right.authored_order,
  )) {
    const rank = formatRank(spell.rank);
    groups.set(rank, [...(groups.get(rank) ?? []), spell]);
  }
  return [...groups.entries()].sort(
    ([left], [right]) => rankSortValue(right) - rankSortValue(left),
  );
}

export function formatRank(rank: number | undefined) {
  if (rank === undefined) return "Unranked";
  if (rank === 0) return "Cantrips";
  const suffix =
    rank % 10 === 1 && rank % 100 !== 11
      ? "st"
      : rank % 10 === 2 && rank % 100 !== 12
        ? "nd"
        : rank % 10 === 3 && rank % 100 !== 13
          ? "rd"
          : "th";
  return `${rank}${suffix}`;
}

function rankSortValue(rank: string) {
  if (rank === "Cantrips") return 0;
  if (rank === "Unranked") return -1;
  return Number.parseInt(rank, 10);
}
