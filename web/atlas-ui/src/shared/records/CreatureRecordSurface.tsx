import { RightOutlined } from "@ant-design/icons";
import { Alert, Button, Collapse, Empty, Space, Tag } from "antd";
import type React from "react";
import type {
  CreatureSurfaceActivityView,
  CreatureSurfaceContentView,
  CreatureSurfaceShieldView,
  CreatureSurfaceSkillPredicateView,
  CreatureSurfaceSpellView,
  CreatureSurfaceSpellcastingView,
  CreatureSurfaceView,
  RecordSurfaceEditionCounterpartView,
  RecordSurfaceMetadataView,
  RecordSurfaceView,
  RuntimeNumberView,
} from "../../generated/atlas";
import { ActionGlyph, actionCostLabel } from "./ActionGlyph";
import { DefenseIwrList, DefenseNote, DefenseStats } from "./RecordDefensePrimitives";
import { contentLabel, RichContent, type ReferenceHandler } from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import {
  RecordSurfaceIssues,
  RecordSurfaceReferences,
} from "./RecordSurfaceSupplement";
import { formatRank, formatSigned, formatSlug } from "./recordFormatting";
import { SpellOccurrencePreviewPopover } from "./SpellOccurrencePreviewPopover";

export function CreatureDetailSurface({
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
  body: CreatureSurfaceView;
  issues: NonNullable<RecordSurfaceView["issues"]> | undefined;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  onReferencesOpen?: () => void;
  onReferenceLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  references: NonNullable<RecordSurfaceView["references"]> | undefined;
  referencesLoading?: boolean;
  showTitle: boolean;
}) {
  const overview = overviewContent(body.content);
  const standalone = body.standalone_spells ?? [];
  return (
    <article className="record-surface record-surface--record-detail">
      <RecordHeader
        metadata={metadata}
        onReference={onReference}
        showTitle={showTitle}
      />
      <EditionNotice metadata={metadata} onReference={onReference} />
      <OverviewSection content={overview} onReference={onReference} />
      <div className="creature-sheet__facts-grid">
        <div className="creature-sheet__facts-column creature-sheet__facts-column--primary">
          <DefensePanel body={body} />
          <AbilitiesPanel body={body} />
        </div>
        <div className="creature-sheet__facts-column creature-sheet__facts-column--secondary">
          <ProfileAwarenessPanel body={body} />
          <SkillsPanel body={body} />
        </div>
      </div>
      <div className="creature-sheet__mechanics-grid">
        <div className="creature-sheet__mechanics-main">
          <ActivitySections activities={body.activities} onReference={onReference} />
        </div>
        <aside className="creature-sheet__mechanics-side">
          <SpellcastingSection
            entries={body.spellcasting}
            onReference={onReference}
            standalone={standalone}
          />
          <RitualsSection rituals={body.rituals} />
          <EquipmentSection equipment={body.equipment} />
          <LoreSection lore={body.lore} />
          <ResourcesSection resources={body.resources} />
        </aside>
      </div>
      <RecordSurfaceIssues issues={issues} />
      <RecordSurfaceReferences
        loading={referencesLoading}
        onDisclosureOpen={onReferencesOpen}
        onRequestLimit={onReferenceLimit}
        onReference={onReference}
        references={references}
      />
      <CreatureSourceDisclosure body={body} metadata={metadata} />
    </article>
  );
}

export function SearchCompactSurface({
  body,
  metadata,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <div className="record-surface-search__heading">
          <h2>{metadata.title}</h2>
        </div>
        <IdentityMetadata compact metadata={metadata} />
        <EditionTag metadata={metadata} />
        <TraitRow compact metadata={metadata} />
        {body.teaser ? (
          <p className="record-surface-search__teaser">{body.teaser}</p>
        ) : null}
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

function EditionTag({ metadata }: { metadata: RecordSurfaceMetadataView }) {
  if (metadata.edition?.status !== "legacy") return null;
  return (
    <div className="creature-sheet__edition-tag">
      <Tag>Legacy</Tag>
    </div>
  );
}

function EditionNotice({
  metadata,
  onReference,
}: {
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
}) {
  const edition = metadata.edition;
  if (edition?.status !== "legacy") return null;
  const navigation = edition.counterparts.length ? (
    <Space className="creature-sheet__edition-navigation" size="small" wrap>
      {edition.counterparts.map((counterpart) => (
        <Button
          key={`${counterpart.role}:${counterpart.record_key}`}
          onClick={() => onReference(counterpart.record_key)}
          size="small"
          type="link"
        >
          {counterpartActionLabel(counterpart)}
        </Button>
      ))}
    </Space>
  ) : undefined;
  return (
    <Alert
      action={navigation}
      className="creature-sheet__edition-notice"
      message="This record uses legacy rules."
      showIcon
      type="warning"
    />
  );
}

function counterpartActionLabel(counterpart: RecordSurfaceEditionCounterpartView) {
  return counterpart.role === "remastered_counterpart"
    ? `View remastered ${counterpart.title}`
    : `View legacy ${counterpart.title}`;
}

export function RecordHeader({
  levelLabel = "Level",
  metadata,
  onReference,
  showTitle = true,
}: {
  levelLabel?: "Level" | "Rank";
  metadata: RecordSurfaceMetadataView;
  onReference?: ReferenceHandler;
  showTitle?: boolean;
}) {
  return (
    <header className="creature-sheet__header">
      <div className="creature-sheet__identity">
        {showTitle ? <h2>{metadata.title}</h2> : null}
        <IdentityMetadata levelLabel={levelLabel} metadata={metadata} />
        <TraitRow metadata={metadata} />
        {onReference ? (
          <RelatedEditionMetadata metadata={metadata} onReference={onReference} />
        ) : null}
      </div>
    </header>
  );
}

function RelatedEditionMetadata({
  metadata,
  onReference,
}: {
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
}) {
  const counterparts =
    metadata.edition?.status === "remaster"
      ? metadata.edition.counterparts.filter(
          (counterpart) => counterpart.role === "legacy_counterpart",
        )
      : [];
  if (!counterparts.length) return null;
  return (
    <div aria-label="Related edition" className="creature-sheet__related-edition">
      <span>Related edition</span>
      <Space size="small" wrap>
        {counterparts.map((counterpart) => (
          <Button
            key={`${counterpart.role}:${counterpart.record_key}`}
            onClick={() => onReference(counterpart.record_key)}
            size="small"
            type="link"
          >
            {counterpartActionLabel(counterpart)}
          </Button>
        ))}
      </Space>
    </div>
  );
}

export function IdentityMetadata({
  compact = false,
  levelLabel = "Level",
  metadata,
}: {
  compact?: boolean;
  levelLabel?: "Level" | "Rank";
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
        <span className="creature-sheet__level">
          {levelLabel} {metadata.level}
        </span>
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

function OverviewSection({
  content,
  onReference,
}: {
  content: CreatureSurfaceContentView[];
  onReference: ReferenceHandler;
}) {
  return (
    <NarrativeSection
      content={content}
      headingId="record-overview"
      onReference={onReference}
      title="Overview"
    />
  );
}

export function NarrativeSection({
  content,
  headingId,
  onReference,
  title,
}: {
  content: CreatureSurfaceContentView[];
  headingId: string;
  onReference: ReferenceHandler;
  title: string;
}) {
  if (!content.length) return null;
  const [primary, ...additional] = content;
  return (
    <section className="creature-sheet__narrative" aria-labelledby={headingId}>
      <div className="creature-sheet__section-heading">
        <h3 id={headingId}>{title}</h3>
      </div>
      <RichContent content={primary} onReference={onReference} />
      {additional.length > 0 && (
        <Collapse
          className="record-surface__inline-disclosure"
          expandIcon={disclosureExpandIcon}
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
  const stats = [
    { key: "ac", label: "AC", value: defenses?.armor_class },
    { key: "hp", label: "HP", value: vitals?.hit_points },
    { key: "hardness", label: "Hardness", value: defenses?.hardness },
    {
      key: "fortitude",
      label: "Fortitude",
      signed: true,
      value: saves?.fortitude?.modifier,
    },
    {
      key: "reflex",
      label: "Reflex",
      signed: true,
      value: saves?.reflex?.modifier,
    },
    { key: "will", label: "Will", signed: true, value: saves?.will?.modifier },
  ];
  const hasDetails = Boolean(
    defenses?.armor_class_details ||
    vitals?.details ||
    saves?.all_saves_note ||
    saves?.fortitude?.details ||
    saves?.reflex?.details ||
    saves?.will?.details ||
    defenses?.shield,
  );
  const hasStats = stats.some((stat) => stat.value !== undefined);
  const hasIwr = Boolean(
    defenses?.immunities?.length ||
    defenses?.weaknesses?.length ||
    defenses?.resistances?.length,
  );
  if (!hasStats && !hasIwr && !hasDetails) return null;
  return (
    <SurfaceSection
      className="creature-sheet__panel--defenses"
      title="Defenses & Vitals"
    >
      <DefenseStats ariaLabel="Defense statistics" values={stats} />
      <DefenseNote>{defenses?.armor_class_details}</DefenseNote>
      <DefenseNote>{vitals?.details}</DefenseNote>
      <DefenseNote>{saves?.all_saves_note}</DefenseNote>
      <SaveDetails saves={saves} />
      <ShieldDetails shield={defenses?.shield} />
      <DefenseIwrList
        immunities={defenses?.immunities}
        resistances={defenses?.resistances}
        weaknesses={defenses?.weaknesses}
      />
    </SurfaceSection>
  );
}

function SaveDetails({ saves }: { saves: CreatureSurfaceView["saves"] }) {
  const items = [
    recordKeyValue("fortitude-details", "Fortitude details", saves?.fortitude?.details),
    recordKeyValue("reflex-details", "Reflex details", saves?.reflex?.details),
    recordKeyValue("will-details", "Will details", saves?.will?.details),
  ].filter(isRecordKeyValueItem);
  return items.length ? (
    <RecordKeyValueList ariaLabel="Save details" items={items} />
  ) : null;
}

function ShieldDetails({ shield }: { shield: CreatureSurfaceShieldView | undefined }) {
  if (!shield) return null;
  const items = [
    recordKeyValue(
      "shield-ac",
      "AC bonus",
      shield.armor_class_bonus === undefined
        ? undefined
        : formatSigned(shield.armor_class_bonus),
    ),
    recordKeyValue("shield-hardness", "Hardness", shield.hardness),
    recordKeyValue("shield-hp", "Maximum HP", shield.maximum_hit_points),
    recordKeyValue("shield-bt", "Broken threshold", shield.broken_threshold),
  ].filter(isRecordKeyValueItem);
  if (!items.length) return null;
  return (
    <div className="creature-sheet__shield">
      <h4>Shield</h4>
      <RecordKeyValueList ariaLabel="Shield details" items={items} />
    </div>
  );
}

function ProfileAwarenessPanel({ body }: { body: CreatureSurfaceView }) {
  const awareness = body.awareness;
  const senses = (awareness?.senses ?? [])
    .slice()
    .sort((left, right) => left.authored_order - right.authored_order)
    .map((sense, index) => ({
      key: `${sense.component_id}:${index}`,
      label: formatSlug(sense.kind),
      qualifier: [
        sense.acuity,
        sense.range_feet === undefined ? undefined : `${sense.range_feet} feet`,
      ]
        .filter((value): value is string => value !== undefined)
        .join(", "),
    }));
  const languages: CompactMultiValueItem[] = (awareness?.languages ?? []).map(
    (language, index) => ({
      key: `${language}:${index}`,
      label: formatSlug(language),
    }),
  );
  const perceptionFacts = [
    recordKeyValue(
      "perception",
      "Perception",
      awareness?.perception === undefined
        ? undefined
        : formatSigned(awareness.perception),
    ),
    recordKeyValue("senses", "Senses", compactMultiValueList("Senses", senses)),
    detailKeyValue("perception-details", awareness?.details),
  ].filter(isRecordKeyValueItem);
  const languageFacts = [
    recordKeyValue(
      "languages",
      "Languages",
      compactMultiValueList("Languages", languages),
    ),
    detailKeyValue("language-details", awareness?.language_details),
  ].filter(isRecordKeyValueItem);
  const facts = [
    recordKeyValue("size", "Size", body.size && formatSlug(body.size.value)),
    recordKeyValue(
      "adjustment",
      "Adjustment",
      body.adjustment && formatSlug(body.adjustment.value),
    ),
    recordKeyValue(
      "initiative",
      "Initiative",
      body.initiative && formatSlug(body.initiative.statistic),
    ),
    recordKeyValue("movement", "Movement", movementValue(body.movement)),
    factGroup(
      "perception-and-senses",
      "Perception & senses",
      "Perception and senses",
      perceptionFacts,
    ),
    factGroup(
      "languages-and-communication",
      "Languages & communication",
      "Languages and communication",
      languageFacts,
    ),
  ].filter(isRecordKeyValueItem);
  if (!facts.length) return null;
  return (
    <section className="creature-sheet__panel creature-sheet__panel--profile">
      <RecordKeyValueList
        ariaLabel="Creature profile facts"
        className="creature-sheet__compact-fact-grid"
        items={facts}
      />
    </section>
  );
}

function movementValue(movement: CreatureSurfaceView["movement"]) {
  if (!movement?.length) return undefined;
  return compactMultiValueList(
    "Movement",
    movement
      .slice()
      .sort((left, right) => left.authored_order - right.authored_order)
      .map((entry, index) => ({
        key: `${entry.component_id}:${index}`,
        label: formatSlug(entry.label ?? entry.mode),
        qualifier: [
          entry.speed_feet === undefined ? undefined : `${entry.speed_feet} feet`,
          entry.details,
        ]
          .filter((value): value is string => value !== undefined && value !== "")
          .join(", "),
      })),
  );
}

type CompactMultiValueItem = {
  key: React.Key;
  label: string;
  qualifier?: string;
};

function compactMultiValueList(ariaLabel: string, items: CompactMultiValueItem[]) {
  if (!items.length) return undefined;
  return (
    <ul aria-label={ariaLabel} className="creature-sheet__compact-multi-value-list">
      {items.map((item) => (
        <li className="creature-sheet__compact-multi-value-item" key={item.key}>
          <span>{item.label}</span>
          {item.qualifier ? (
            <>
              <span aria-hidden="true" className="creature-sheet__value-dash">
                —
              </span>
              <small>{item.qualifier}</small>
            </>
          ) : null}
        </li>
      ))}
    </ul>
  );
}

function detailKeyValue(key: React.Key, value: string | undefined) {
  const item = recordKeyValue(key, "Details", value);
  return item ? { ...item, rowClassName: "creature-sheet__fact-group-note" } : null;
}

function factGroup(
  key: React.Key,
  label: string,
  ariaLabel: string,
  items: RecordKeyValueItem[],
) {
  if (!items.length) return null;
  return {
    key,
    label,
    rowClassName: "creature-sheet__fact-group",
    value: (
      <RecordKeyValueList
        ariaLabel={ariaLabel}
        className="creature-sheet__fact-group-rows"
        items={items}
      />
    ),
  } satisfies RecordKeyValueItem;
}

function SkillsPanel({ body }: { body: CreatureSurfaceView }) {
  if (!body.skills?.length) return null;
  return (
    <SurfaceSection className="creature-sheet__panel--skills" title="Skills">
      <ul aria-label="Skills" className="creature-sheet__skill-grid">
        {body.skills
          .slice()
          .sort((left, right) => left.authored_order - right.authored_order)
          .map((skill) => (
            <li
              className="creature-sheet__skill-cell"
              key={`${skill.component_id}:${skill.authored_order}`}
            >
              <div className="creature-sheet__skill-heading">
                <span>{skill.label}</span>
                {skill.modifier !== undefined ? (
                  <strong>{formatSigned(skill.modifier)}</strong>
                ) : null}
              </div>
              {skill.note ? <small>{skill.note}</small> : null}
              {skill.variants?.length ? (
                <ul className="creature-sheet__skill-variants">
                  {skill.variants
                    .slice()
                    .sort((left, right) => left.authored_order - right.authored_order)
                    .map((variant) => (
                      <li key={`${variant.component_id}:${variant.authored_order}`}>
                        <span>
                          {variant.label ?? "Variant"}
                          {variant.modifier === undefined ? null : (
                            <strong>{formatSigned(variant.modifier)}</strong>
                          )}
                        </span>
                        {variant.predicates?.length ? (
                          <small>
                            {variant.predicates.map(formatSkillPredicate).join("; ")}
                          </small>
                        ) : null}
                      </li>
                    ))}
                </ul>
              ) : null}
            </li>
          ))}
      </ul>
    </SurfaceSection>
  );
}

function formatSkillPredicate(predicate: CreatureSurfaceSkillPredicateView) {
  switch (predicate.predicate_type) {
    case "term":
      return predicate.term;
    case "not":
      return `not ${predicate.term}`;
    case "any":
      return `any of ${predicate.terms.join(", ")}`;
    case "at_least":
      return `at least ${predicate.minimum} ${predicate.term}`;
  }
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

function ActivitySections({
  activities,
  onReference,
}: {
  activities: CreatureSurfaceActivityView[] | undefined;
  onReference: ReferenceHandler;
}) {
  if (!activities?.length) return null;
  const ordered = [...activities].sort(
    (left, right) => left.authored_order - right.authored_order,
  );
  const actions = ordered.filter(isActiveActivity);
  const features = ordered.filter((activity) => !isActiveActivity(activity));
  return (
    <>
      <ActivityGroup activities={actions} onReference={onReference} title="Actions" />
      <ActivityGroup activities={features} onReference={onReference} title="Passives" />
    </>
  );
}

function ActivityGroup({
  activities,
  onReference,
  title,
}: {
  activities: CreatureSurfaceActivityView[];
  onReference: ReferenceHandler;
  title: string;
}) {
  if (!activities.length) return null;
  return (
    <SurfaceSection className="creature-sheet__activities" title={title}>
      <div className="creature-sheet__activity-list">
        {activities.map((activity) => (
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

function isActiveActivity(activity: CreatureSurfaceActivityView) {
  return activity.action_cost?.cost_type !== "passive";
}

function StaticActivity({
  activity,
  onReference,
}: {
  activity: CreatureSurfaceActivityView;
  onReference: ReferenceHandler;
}) {
  const content = activity.content ?? [];
  const heading = <ActivityHeading activity={activity} />;
  return content.length ? (
    <Collapse
      className="creature-sheet__activity creature-sheet__activity--expandable"
      defaultActiveKey={[activity.occurrence_id]}
      expandIcon={disclosureExpandIcon}
      ghost
      items={[
        {
          key: activity.occurrence_id,
          label: heading,
          children: (
            <div className="creature-sheet__activity-content">
              <ActivityDetails activity={activity} />
              {content.map((document) => (
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
    <article className="creature-sheet__activity">
      <div className="creature-sheet__activity-summary">
        {heading}
        <ActivityDetails activity={activity} />
      </div>
    </article>
  );
}

function ActivityHeading({ activity }: { activity: CreatureSurfaceActivityView }) {
  return (
    <div
      aria-label={`${activity.label}, ${actionCostLabel(activity.action_cost)}`}
      className="creature-sheet__activity-heading"
    >
      <strong>{activity.label}</strong>
      <ActionGlyph cost={activity.action_cost} />
    </div>
  );
}

function ActivityDetails({ activity }: { activity: CreatureSurfaceActivityView }) {
  const details = [
    recordKeyValue("frequency", "Frequency", formatFrequency(activity.frequency)),
    recordKeyValue("requirements", "Requirements", activity.requirements),
    recordKeyValue("cost", "Cost", activity.cost),
    recordKeyValue("uses", "Uses", formatUses(activity.uses)),
    recordKeyValue(
      "self-effect",
      "Self effect",
      formatSelfEffect(activity.self_effect),
    ),
  ].filter(isRecordKeyValueItem);
  return (
    <>
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
      {details.length ? (
        <RecordKeyValueList ariaLabel={`${activity.label} details`} items={details} />
      ) : null}
    </>
  );
}

function formatFrequency(frequency: CreatureSurfaceActivityView["frequency"]) {
  return frequency?.display;
}

function formatUses(uses: CreatureSurfaceActivityView["uses"]) {
  return uses?.maximum === undefined ? undefined : `${uses.maximum} maximum`;
}

function formatSelfEffect(effect: CreatureSurfaceActivityView["self_effect"]) {
  if (!effect) return undefined;
  return [effect.label, effect.value].filter(Boolean).join(": ") || undefined;
}

function SpellcastingSection({
  entries,
  onReference,
  standalone,
}: {
  entries: CreatureSurfaceSpellcastingView[] | undefined;
  onReference: ReferenceHandler;
  standalone: CreatureSurfaceSpellView[];
}) {
  if (!entries?.length && !standalone.length) return null;
  const spellcastingItems: NonNullable<React.ComponentProps<typeof Collapse>["items"]> =
    (entries ?? [])
      .slice()
      .sort((left, right) => left.authored_order - right.authored_order)
      .map((entry) => ({
        key: entry.occurrence_id,
        label: <SpellcastingHeading entry={entry} />,
        children: (
          <SpellRoster
            onReference={onReference}
            slots={entry.slots}
            spells={entry.spells}
          />
        ),
      }));
  const orderedStandalone = standalone
    .slice()
    .sort((left, right) => left.authored_order - right.authored_order);
  const spellGroups: NonNullable<React.ComponentProps<typeof Collapse>["items"]> = [
    ...spellcastingItems,
    ...(orderedStandalone.length
      ? [
          {
            key: "standalone-spells",
            label: (
              <span className="creature-sheet__spell-heading">
                <strong>Standalone</strong>
              </span>
            ),
            children: (
              <SpellRoster onReference={onReference} spells={orderedStandalone} />
            ),
          },
        ]
      : []),
  ];
  const defaultActiveKey = spellGroups.flatMap((group) =>
    typeof group.key === "string" || typeof group.key === "number" ? [group.key] : [],
  );
  return (
    <SurfaceSection className="creature-sheet__spellcasting" title="Spells">
      {spellGroups.length ? (
        <Collapse
          className="record-surface__inline-disclosure"
          defaultActiveKey={defaultActiveKey}
          expandIcon={disclosureExpandIcon}
          ghost
          items={spellGroups}
          size="small"
        />
      ) : null}
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
  slots,
  spells,
}: {
  onReference: ReferenceHandler;
  slots?: CreatureSurfaceSpellcastingView["slots"];
  spells: CreatureSurfaceSpellView[] | undefined;
}) {
  if (!spells?.length) {
    return (
      <Empty description="No spells are listed." image={Empty.PRESENTED_IMAGE_SIMPLE} />
    );
  }
  return (
    <div className="creature-sheet__spell-roster">
      {groupSpells(spells).map(([rank, ranked]) => (
        <div className="creature-sheet__spell-rank" key={rank}>
          <div className="creature-sheet__spell-rank-heading">
            <strong>{rank}</strong>
            <SpellSlotMaximum rank={ranked[0]?.rank} slots={slots} />
          </div>
          <div className="creature-sheet__spell-links">
            {ranked.map((spell, index) => (
              <span
                className="creature-sheet__spell-link"
                key={`${spell.occurrence_id}:${spell.authored_order}`}
              >
                <SpellOccurrencePreviewPopover
                  onOpenSpellRecord={onReference}
                  onReference={onReference}
                  spell={spell}
                />
                <SpellOccurrenceContext spell={spell} />
                {index < ranked.length - 1 ? (
                  <span aria-hidden="true" className="creature-sheet__spell-separator">
                    ,
                  </span>
                ) : null}
              </span>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function SpellSlotMaximum({
  rank,
  slots,
}: {
  rank: number | undefined;
  slots: CreatureSurfaceSpellcastingView["slots"];
}) {
  if (rank === undefined) return null;
  const slot = slots?.find((candidate) => candidate.rank === rank);
  if (slot?.maximum === undefined) return null;
  return (
    <span className="creature-sheet__spell-slots">
      {slot.maximum} {slot.maximum === 1 ? "slot" : "slots"}
    </span>
  );
}

function SpellOccurrenceContext({ spell }: { spell: CreatureSurfaceSpellView }) {
  const context = spell.context;
  if (context?.uses?.maximum === undefined) return null;
  return (
    <small>
      {context.uses.maximum} {context.uses.maximum === 1 ? "use" : "uses"}
    </small>
  );
}

function RitualsSection({ rituals }: { rituals: CreatureSurfaceView["rituals"] }) {
  if (!rituals) return null;
  return (
    <SurfaceSection title="Rituals">
      <dl className="creature-sheet__stat-list">
        <Fact label="Ritual DC" value={rituals.difficulty_class} />
      </dl>
    </SurfaceSection>
  );
}

function EquipmentSection({
  equipment,
}: {
  equipment: CreatureSurfaceView["equipment"];
}) {
  if (!equipment?.length) return null;
  return (
    <SurfaceSection title="Equipment & Gear">
      <ul className="creature-sheet__equipment-list">
        {equipment
          .slice()
          .sort((left, right) => left.authored_order - right.authored_order)
          .map((item) => {
            const details = [
              item.quantity === undefined ? undefined : `Quantity ${item.quantity}`,
              item.level === undefined ? undefined : `Level ${item.level}`,
              item.usage ? `Usage ${item.usage}` : undefined,
              item.uses?.maximum === undefined
                ? undefined
                : `${item.uses.maximum} maximum uses`,
            ].filter((detail): detail is string => Boolean(detail));
            return (
              <li key={`${item.occurrence_id}:${item.authored_order}`}>
                <strong>{item.label}</strong>
                {details.length ? <small>{details.join(" · ")}</small> : null}
                {item.traits?.length ? (
                  <span className="creature-sheet__equipment-traits">
                    {item.traits.map(formatSlug).join(", ")}
                  </span>
                ) : null}
              </li>
            );
          })}
      </ul>
    </SurfaceSection>
  );
}

function LoreSection({ lore }: { lore: CreatureSurfaceView["lore"] }) {
  if (!lore?.length) return null;
  const items = lore
    .slice()
    .sort((left, right) => left.authored_order - right.authored_order)
    .map((entry) => ({
      key: `${entry.occurrence_id}:${entry.authored_order}`,
      label: entry.label,
      value: entry.modifier === undefined ? "—" : formatSigned(entry.modifier),
    }));
  return (
    <SurfaceSection title="Lore">
      <RecordKeyValueList ariaLabel="Lore skills" items={items} />
    </SurfaceSection>
  );
}

function ResourcesSection({
  resources,
}: {
  resources: CreatureSurfaceView["resources"];
}) {
  if (!resources?.length) return null;
  const facts: RecordKeyValueItem[] = resources.map((resource) => ({
    key: resource.component_id,
    label: resource.label,
    value: resource.maximum ?? "—",
  }));
  return (
    <SurfaceSection title="Resources">
      <RecordKeyValueList ariaLabel="Resources" items={facts} />
    </SurfaceSection>
  );
}

export function CreatureSourceDisclosure({
  body,
  metadata,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  return (
    <Collapse
      className="record-surface__secondary"
      expandIcon={disclosureExpandIcon}
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
  );
}

export function CreatureSourceContent({
  body,
  metadata,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  return (
    <div className="creature-sheet__reference-source">
      <SourceDetails body={body} metadata={metadata} />
    </div>
  );
}

function SourceDetails({
  body,
  metadata,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  const source = metadata.source;
  const facts = [
    recordKeyValue("record-id", "Record ID", metadata.record_key),
    recordKeyValue("publication", "Publication", source?.publication_title),
    recordKeyValue("source-pack", "Source pack", source?.pack_label),
    recordKeyValue("source-path", "Source path", source?.source_path),
  ].filter(isRecordKeyValueItem);
  const skillSources: RecordKeyValueItem[] = (body.skills ?? [])
    .slice()
    .sort((left, right) => left.authored_order - right.authored_order)
    .flatMap((skill) => {
      if (!skill.source_entries?.length) return [];
      return [
        {
          key: `${skill.component_id}:${skill.authored_order}`,
          label: skill.label,
          value: (
            <span className="creature-sheet__skill-source-values">
              {skill.source_entries
                .slice()
                .sort((left, right) => left.authored_order - right.authored_order)
                .map((entry) => (
                  <code key={`${entry.authored_order}:${entry.authored_key}`}>
                    {entry.authored_key}
                  </code>
                ))}
            </span>
          ),
        },
      ];
    });
  return (
    <section>
      <h4>Provenance</h4>
      <RecordKeyValueList
        ariaLabel="Provenance"
        items={facts}
        labelWidth="provenance"
      />
      {skillSources.length ? (
        <div className="creature-sheet__skill-source-provenance">
          <h5>Skill source keys</h5>
          <RecordKeyValueList
            ariaLabel="Skill source keys"
            items={skillSources}
            labelWidth="provenance"
          />
        </div>
      ) : null}
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
  label,
  signed = false,
  value,
}: {
  label: string;
  signed?: boolean;
  value: number | string;
}) {
  return (
    <div>
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

function recordKeyValue(
  key: React.Key,
  label: React.ReactNode,
  value: React.ReactNode | undefined | null,
): RecordKeyValueItem | null {
  return value === undefined || value === null || value === ""
    ? null
    : { key, label, value };
}

function isRecordKeyValueItem(
  value: RecordKeyValueItem | null,
): value is RecordKeyValueItem {
  return value !== null;
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

function rankSortValue(rank: string) {
  if (rank === "Cantrips") return 0;
  if (rank === "Unranked") return -1;
  return Number.parseInt(rank, 10);
}

function overviewContent(content: CreatureSurfaceContentView[] | undefined) {
  return (content ?? [])
    .filter(
      (document) =>
        document.role === "primary_description" ||
        document.role === "summary" ||
        document.role === "supplemental_rules" ||
        document.role === "embedded_capability" ||
        document.role === "generated_narrative" ||
        document.role === "journal_page",
    )
    .slice()
    .sort((left, right) => left.authored_order - right.authored_order);
}

const disclosureExpandIcon: NonNullable<
  React.ComponentProps<typeof Collapse>["expandIcon"]
> = ({ isActive }) => <RightOutlined aria-hidden="true" rotate={isActive ? 90 : 0} />;
