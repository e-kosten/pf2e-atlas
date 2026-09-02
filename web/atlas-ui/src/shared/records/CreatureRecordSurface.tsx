import { RightOutlined } from "@ant-design/icons";
import { Alert, Button, Collapse, Empty, Space, Tag } from "antd";
import type React from "react";
import type {
  CreatureSurfaceActivityView,
  CreatureSurfaceContentView,
  CreatureSurfaceIwrView,
  CreatureSurfaceMovementView,
  CreatureSurfaceRelationshipView,
  CreatureSurfaceShieldView,
  CreatureSurfaceSkillPredicateView,
  CreatureSurfaceSpellView,
  CreatureSurfaceSpellcastingView,
  CreatureSurfaceView,
  RecordSurfaceEditionCounterpartView,
  RecordSurfaceMetadataView,
  RuntimeNumberView,
} from "../../generated/atlas";
import { ActionGlyph } from "./ActionGlyph";
import { DataAvailabilityDisclosure } from "./CreatureDataAvailability";
import { contentLabel, RichContent, type ReferenceHandler } from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import { formatRank, formatSigned, formatSlug } from "./recordFormatting";
import { SpellOccurrencePreviewPopover } from "./SpellOccurrencePreviewPopover";

export function CreatureDetailSurface({
  body,
  metadata,
  onReference,
  showTitle,
}: {
  body: CreatureSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  showTitle: boolean;
}) {
  const overview = overviewContent(body.content);
  const standalone = body.standalone_spells ?? [];
  return (
    <article className="record-surface record-surface--record-detail">
      <RecordHeader metadata={metadata} showTitle={showTitle} />
      <EditionNotice metadata={metadata} onReference={onReference} />
      <CreatureProfileFacts body={body} />
      <OverviewSection content={overview} onReference={onReference} />
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
          <RitualsSection rituals={body.rituals} />
          <EquipmentSection equipment={body.equipment} />
          <LoreSection lore={body.lore} />
          <ResourcesSection resources={body.resources} />
        </aside>
      </div>
      <DataAvailabilityDisclosure unavailable={body.unavailable_domains} />
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

function CreatureProfileFacts({ body }: { body: CreatureSurfaceView }) {
  const facts = [
    body.size ? { label: "Size", value: formatSlug(body.size.value) } : null,
    body.adjustment
      ? { label: "Adjustment", value: formatSlug(body.adjustment.value) }
      : null,
    body.initiative
      ? { label: "Initiative", value: formatSlug(body.initiative.statistic) }
      : null,
  ].filter((fact): fact is { label: string; value: string } => fact !== null);
  if (!facts.length) return null;
  return (
    <dl aria-label="Creature profile" className="creature-sheet__profile-facts">
      {facts.map((fact) => (
        <Fact key={fact.label} label={fact.label} value={fact.value} />
      ))}
    </dl>
  );
}

function EditionTag({ metadata }: { metadata: RecordSurfaceMetadataView }) {
  if (!metadata.edition) return null;
  return (
    <div className="creature-sheet__edition-tag">
      <Tag>{metadata.edition.status === "legacy" ? "Legacy" : "Remastered"}</Tag>
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
  if (!edition) return null;
  return (
    <Alert
      action={
        edition.counterparts.length ? (
          <Space size="small" wrap>
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
        ) : undefined
      }
      className="creature-sheet__edition-notice"
      message={
        edition.status === "legacy"
          ? "This record uses legacy rules."
          : "This record uses remastered rules."
      }
      showIcon
      type={edition.status === "legacy" ? "warning" : "info"}
    />
  );
}

function counterpartActionLabel(counterpart: RecordSurfaceEditionCounterpartView) {
  return counterpart.role === "remastered_counterpart"
    ? `View remastered ${counterpart.title}`
    : `View legacy ${counterpart.title}`;
}

export function RecordHeader({
  metadata,
  showTitle = true,
}: {
  metadata: RecordSurfaceMetadataView;
  showTitle?: boolean;
}) {
  return (
    <header className="creature-sheet__header">
      <div className="creature-sheet__identity">
        {showTitle ? <h2>{metadata.title}</h2> : null}
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

function OverviewSection({
  content,
  onReference,
}: {
  content: CreatureSurfaceContentView[];
  onReference: ReferenceHandler;
}) {
  if (!content.length) return null;
  const [primary, ...additional] = content;
  return (
    <section className="creature-sheet__narrative" aria-labelledby="record-overview">
      <div className="creature-sheet__section-heading">
        <h3 id="record-overview">Overview</h3>
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
    compactFact("AC", defenses?.armor_class),
    compactFact("HP", vitals?.hit_points),
    compactFact("Hardness", defenses?.hardness),
    compactFact("Fortitude", saves?.fortitude?.modifier, true),
    compactFact("Reflex", saves?.reflex?.modifier, true),
    compactFact("Will", saves?.will?.modifier, true),
  ].filter(isCompactFact);
  const iwr = [
    iwrKeyValue("immunities", "Immunities", defenses?.immunities),
    iwrKeyValue("weaknesses", "Weaknesses", defenses?.weaknesses),
    iwrKeyValue("resistances", "Resistances", defenses?.resistances),
  ].filter(isRecordKeyValueItem);
  const hasDetails = Boolean(
    defenses?.armor_class_details ||
    vitals?.details ||
    saves?.all_saves_note ||
    saves?.fortitude?.details ||
    saves?.reflex?.details ||
    saves?.will?.details ||
    defenses?.shield,
  );
  if (!stats.length && !iwr.length && !hasDetails) return null;
  return (
    <SurfaceSection
      className="creature-sheet__panel--defenses"
      title="Defenses & Vitals"
    >
      {stats.length ? (
        <dl aria-label="Defense statistics" className="creature-sheet__defense-stats">
          {stats.map(({ label, value }) => (
            <Fact key={label} label={label} value={value} />
          ))}
        </dl>
      ) : null}
      {defenses?.armor_class_details && (
        <p className="creature-sheet__detail-note">{defenses.armor_class_details}</p>
      )}
      {vitals?.details && (
        <p className="creature-sheet__detail-note">{vitals.details}</p>
      )}
      {saves?.all_saves_note && (
        <p className="creature-sheet__detail-note">{saves.all_saves_note}</p>
      )}
      <SaveDetails saves={saves} />
      <ShieldDetails shield={defenses?.shield} />
      <RecordKeyValueList
        ariaLabel="Immunities, weaknesses, and resistances"
        items={iwr}
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

function SensesLanguagesPanel({ body }: { body: CreatureSurfaceView }) {
  const awareness = body.awareness;
  if (!awareness) return null;
  const facts = [
    recordKeyValue(
      "senses",
      "Senses",
      awareness.senses
        ?.map((sense) =>
          [
            formatSlug(sense.kind),
            sense.acuity ? `(${formatSlug(sense.acuity)})` : "",
            sense.range_feet === undefined ? "" : `${sense.range_feet} ft`,
          ]
            .filter(Boolean)
            .join(" "),
        )
        .join(", "),
    ),
    recordKeyValue("languages", "Languages", awareness.languages?.join(", ")),
  ].filter(isRecordKeyValueItem);
  if (!facts.length && !awareness.details && !awareness.language_details) return null;
  return (
    <SurfaceSection
      className="creature-sheet__panel--senses"
      title="Senses & Languages"
    >
      {awareness.perception !== undefined ? (
        <dl aria-label="Perception" className="creature-sheet__perception-stat">
          <Fact label="Perception" signed value={awareness.perception} />
        </dl>
      ) : null}
      <RecordKeyValueList ariaLabel="Senses and languages" items={facts} />
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
  const facts: RecordKeyValueItem[] = movement
    .slice()
    .sort((left, right) => left.authored_order - right.authored_order)
    .map((entry, index) => ({
      key: `${entry.component_id}:${index}`,
      label: formatSlug(entry.label ?? entry.mode),
      value: (
        <span className="creature-sheet__movement-value">
          <strong>
            {entry.speed_feet === undefined ? "—" : `${entry.speed_feet} ft`}
          </strong>
          {entry.details && <small>{entry.details}</small>}
        </span>
      ),
    }));
  return (
    <SurfaceSection className="creature-sheet__panel--movement" title="Movement">
      <RecordKeyValueList ariaLabel="Movement speeds" items={facts} />
    </SurfaceSection>
  );
}

function SkillsPanel({ body }: { body: CreatureSurfaceView }) {
  if (!body.skills?.length) return null;
  return (
    <SurfaceSection className="creature-sheet__panel--skills" title="Skills">
      <div className="creature-sheet__skill-list">
        {body.skills
          .slice()
          .sort((left, right) => left.authored_order - right.authored_order)
          .map((skill) => (
            <article
              className="creature-sheet__skill"
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
                        <span>{variant.label ?? "Variant"}</span>
                        {variant.modifier === undefined ? null : (
                          <strong>{formatSigned(variant.modifier)}</strong>
                        )}
                        {variant.predicates?.length ? (
                          <small>
                            {variant.predicates.map(formatSkillPredicate).join("; ")}
                          </small>
                        ) : null}
                      </li>
                    ))}
                </ul>
              ) : null}
              {skill.source_entries?.length ? (
                <div className="creature-sheet__skill-source">
                  <span>Source key</span>
                  {skill.source_entries
                    .slice()
                    .sort((left, right) => left.authored_order - right.authored_order)
                    .map((entry) => (
                      <code key={`${entry.authored_order}:${entry.authored_key}`}>
                        {entry.authored_key}
                      </code>
                    ))}
                </div>
              ) : null}
            </article>
          ))}
      </div>
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
      expandIcon={disclosureExpandIcon}
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
  const details = [
    recordKeyValue("category", "Category", activity.category),
    recordKeyValue("frequency", "Frequency", formatFrequency(activity.frequency)),
    recordKeyValue("requirements", "Requirements", activity.requirements),
    recordKeyValue("cost", "Cost", activity.cost),
    recordKeyValue("uses", "Uses", formatUses(activity.uses)),
    recordKeyValue(
      "self-effect",
      "Self effect",
      formatSelfEffect(activity.self_effect),
    ),
    recordKeyValue(
      "attack-effects",
      "Attack effects",
      activity.attack_effects?.join(", "),
    ),
  ].filter(isRecordKeyValueItem);
  return (
    <div className="creature-sheet__activity-summary">
      <div className="creature-sheet__activity-heading">
        <strong>{activity.label}</strong>
        <ActionGlyph cost={activity.action_cost} />
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
      {details.length ? (
        <RecordKeyValueList ariaLabel={`${activity.label} details`} items={details} />
      ) : null}
    </div>
  );
}

function formatFrequency(frequency: CreatureSurfaceActivityView["frequency"]) {
  if (!frequency) return undefined;
  if (frequency.maximum !== undefined && frequency.period) {
    return `${frequency.maximum} per ${frequency.period}`;
  }
  return frequency.maximum ?? frequency.period;
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
            {ranked.map((spell) => (
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
  if (!context) return null;
  const details = [
    context.contextual_label,
    context.group ? `Group ${context.group}` : undefined,
    context.location ? `Location ${context.location}` : undefined,
    context.slot ? `Slot ${context.slot}` : undefined,
    context.uses?.maximum === undefined
      ? undefined
      : `${context.uses.maximum} ${context.uses.maximum === 1 ? "use" : "uses"}`,
  ].filter((detail): detail is string => Boolean(detail));
  return details.length ? <small>{details.join(" · ")}</small> : null;
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
      expandIcon={disclosureExpandIcon}
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
  const facts = [
    recordKeyValue("record-id", "Record ID", metadata.record_key),
    recordKeyValue("publication", "Publication", source?.publication_title),
    recordKeyValue("source-pack", "Source pack", source?.pack_label),
    recordKeyValue("source-path", "Source path", source?.source_path),
  ].filter(isRecordKeyValueItem);
  return (
    <section>
      <h4>Provenance</h4>
      <RecordKeyValueList
        ariaLabel="Provenance"
        items={facts}
        labelWidth="provenance"
      />
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

function iwrKeyValue(
  key: React.Key,
  label: string,
  values: CreatureSurfaceIwrView[] | undefined,
): RecordKeyValueItem | null {
  return recordKeyValue(key, label, values?.map(formatIwr).join(", "));
}

function isRecordKeyValueItem(
  value: RecordKeyValueItem | null,
): value is RecordKeyValueItem {
  return value !== null;
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
