import { Alert, Button, Collapse, InputNumber, Select, Space, Tag } from "antd";
import { useState } from "react";
import type React from "react";
import type {
  RecordSurfaceMetadataView,
  RecordSurfaceView,
  SpellAreaView,
  SpellCastingView,
  SpellDamageView,
  SpellDefenseView,
  SpellDurationView,
  SpellFactView,
  SpellFixedHeighteningChangeView,
  SpellFormResultView,
  SpellHeighteningView,
  SpellResolvedDefinitionView,
  SpellResolvedFieldView,
  SpellRitualView,
  SpellRuleDetailView,
  SpellRulePredicateView,
  SpellRuleView,
  SpellSourceValueView,
  SpellSurfaceView,
  SpellTargetingView,
} from "../../generated/atlas";
import {
  NarrativeSection,
  RecordHeader,
  SurfaceSection,
} from "./CreatureRecordSurface";
import { narrativeContent, type ReferenceHandler } from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import {
  RecordSurfaceIssues,
  RecordSurfaceReferences,
} from "./RecordSurfaceSupplement";
import { ActionGlyph, actionCostLabel } from "./ActionGlyph";
import { formatRank, formatSlug } from "./recordFormatting";

export type SpellFormSelection = {
  castRank: number;
  formId: string;
};

type SpellDetailSurfaceProps = {
  body: SpellSurfaceView;
  catalog: SpellSurfaceView;
  issues: NonNullable<RecordSurfaceView["issues"]> | undefined;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  onReferencesOpen?: () => void;
  onReferenceLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  onSelectionChange?: (selection: SpellFormSelection) => void;
  references: NonNullable<RecordSurfaceView["references"]> | undefined;
  referencesLoading?: boolean;
  selection?: SpellFormSelection;
  selectionError?: string;
  selectionLoading?: boolean;
  selectionUnavailable?: boolean;
  showTitle: boolean;
};

export function SpellDetailSurface({
  body,
  catalog,
  issues,
  metadata,
  onReference,
  onReferencesOpen,
  onReferenceLimit,
  onSelectionChange,
  references,
  referencesLoading,
  selection,
  selectionError,
  selectionLoading = false,
  selectionUnavailable = false,
  showTitle,
}: SpellDetailSurfaceProps) {
  const content = narrativeContent(body.content);
  const effective = body.effective_form;
  const definition =
    effective?.result.state === "available" ? effective.result.definition : undefined;
  const classification =
    definition?.classification.state === "available"
      ? knownValue(definition.classification.value)
      : undefined;
  const rank = classification && knownValue(classification.rank);
  const traits = classification && knownValue(classification.traits);
  const headerMetadata = {
    ...metadata,
    kind_label: body.family === "ritual" ? "Ritual" : "Spell",
    level: rank,
    traits: traits ?? [],
  };
  return (
    <article className="record-surface record-surface--record-detail spell-sheet">
      <RecordHeader
        levelLabel="Rank"
        metadata={headerMetadata}
        onReference={onReference}
        showTitle={showTitle}
      />
      <div className="spell-sheet__workspace">
        <aside className="spell-sheet__summary" aria-label="Spell quick facts">
          {definition ? <SpellQuickFacts definition={definition} /> : null}
          <FormsSection
            key={`${metadata.record_key}:${body.effective_form.id}:${body.effective_form.cast_rank}`}
            body={body}
            catalog={catalog}
            loading={selectionLoading}
            onSelectionChange={onSelectionChange}
            selection={selection}
            selectionError={selectionError}
            selectionUnavailable={selectionUnavailable}
          />
        </aside>
        <div className="spell-sheet__main">
          <FormResult result={effective.result} />
          <NarrativeSection
            content={content}
            headingId="spell-overview"
            onReference={onReference}
            title="Authored overview"
          />
        </div>
      </div>
      <RecordSurfaceIssues issues={issues} />
      <RecordSurfaceReferences
        loading={referencesLoading}
        onDisclosureOpen={onReferencesOpen}
        onRequestLimit={onReferenceLimit}
        onReference={onReference}
        references={references}
      />
      <SpellSourceDisclosure metadata={metadata} />
    </article>
  );
}

export function SpellSearchCompactSurface({
  body,
  metadata,
}: {
  body: SpellSurfaceView;
  metadata: RecordSurfaceMetadataView;
}) {
  const definition =
    body.effective_form.result.state === "available"
      ? body.effective_form.result.definition
      : undefined;
  const classification =
    definition?.classification.state === "available"
      ? knownValue(definition.classification.value)
      : undefined;
  const rank = classification && knownValue(classification.rank);
  const traditions = classification && knownValue(classification.traditions);
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <div className="record-surface-search__heading">
          <h2>{metadata.title}</h2>
        </div>
        <div className="creature-sheet__identity-meta creature-sheet__identity-meta--compact">
          <span>{body.family === "ritual" ? "Ritual" : "Spell"}</span>
          {rank !== undefined ? <span>{formatRank(rank)} rank</span> : null}
        </div>
        {traditions?.length ? (
          <Space size={[5, 5]} wrap>
            {traditions.map((tradition) => (
              <Tag key={tradition}>{formatSlug(tradition)}</Tag>
            ))}
          </Space>
        ) : null}
      </div>
      <div className="record-surface-search__meta">
        <strong>
          {metadata.source?.publication_title ?? metadata.source?.pack_label}
        </strong>
        {metadata.source?.pack_label &&
        metadata.source.pack_label !== metadata.source.publication_title ? (
          <small>{metadata.source.pack_label}</small>
        ) : null}
      </div>
    </article>
  );
}

function SpellQuickFacts({ definition }: { definition: SpellResolvedDefinitionView }) {
  const casting = meaningfulKnown(availableFact(definition.casting));
  const targeting = meaningfulKnown(availableFact(definition.targeting));
  const defense = meaningfulKnown(availableFact(definition.defense));
  const damage = meaningfulKnown(availableFact(definition.damage));
  const save = defense && meaningfulKnown(defense.save);
  const statistic = save && meaningfulKnown(save.statistic);
  const range = targeting && meaningfulKnown(targeting.range);
  const area = targeting && meaningfulKnown(targeting.area);
  const ritual = ritualFactItems(definition.ritual);
  return (
    <>
      {damage?.map((member, index) => (
        <p className="spell-sheet__quick-effect" key={index}>
          <strong>
            <span>{damageSummary(member)}</span> {member.label.toLowerCase()}
          </strong>
        </p>
      ))}
      <p className="spell-sheet__quick-casting">
        {casting?.action_cost ? <ActionGlyph cost={casting.action_cost} /> : null}
        {[
          casting?.action_cost
            ? actionCostLabel(casting.action_cost)
            : casting && meaningfulKnown(casting.time),
          range?.authored_text.trim() && `Range ${range.authored_text.trim()}`,
          areaText(area),
          statistic &&
            `${save && knownValue(save.basic) === true ? "Basic " : ""}${formatSlug(statistic)}`,
        ]
          .filter(Boolean)
          .join(" · ")}
      </p>
      {ritual.length ? (
        <div aria-label="Ritual casting requirements">
          <strong>Ritual requirements</strong>
          {ritual.map((entry) => (
            <p key={entry.key}>
              {entry.label}: {entry.value}
            </p>
          ))}
        </div>
      ) : null}
    </>
  );
}

function CastingSection({
  traditions,
  value,
}: {
  traditions?: SpellFactView<string[]>;
  value: SpellFactView<SpellCastingView>;
}) {
  const casting = meaningfulKnown(value);
  if (!casting) return null;
  const items = [
    casting.action_cost
      ? item("time", "Cast", <ActionGlyph cost={casting.action_cost} />)
      : factItem("time", "Cast", casting.time),
    factItem("cost", "Cost", casting.cost),
    factItem("requirements", "Requirements", casting.requirements),
    factItem("counteraction", "Counteraction", casting.counteraction, formatBoolean),
    traditions ? factItem("traditions", "Traditions", traditions, formatList) : null,
  ].filter((entry): entry is RecordKeyValueItem => entry !== null);
  if (!items.length) return null;
  return (
    <SurfaceSection title="Casting">
      <RecordKeyValueList ariaLabel="Spell casting" items={items} />
    </SurfaceSection>
  );
}

function RangeAndTargetsSection({
  defense,
  duration,
  targeting,
}: {
  defense: SpellFactView<SpellDefenseView>;
  duration: SpellFactView<SpellDurationView>;
  targeting: SpellFactView<SpellTargetingView>;
}) {
  const targetingValue = meaningfulKnown(targeting);
  const defenseValue = meaningfulKnown(defense);
  const durationValue = meaningfulKnown(duration);
  const area = targetingValue && meaningfulKnown(targetingValue.area);
  const range = targetingValue && meaningfulKnown(targetingValue.range);
  const save = defenseValue && meaningfulKnown(defenseValue.save);
  const saveStatistic = save && meaningfulKnown(save.statistic);
  const basicSave = save && knownValue(save.basic);
  const durationText = durationValue && meaningfulKnown(durationValue.value);
  const sustained = durationValue && knownValue(durationValue.sustained);
  const items = [
    targetingValue ? factItem("target", "Targets", targetingValue.target) : null,
    range?.authored_text.trim()
      ? item("range", "Range", range.authored_text.trim())
      : null,
    areaText(area) ? item("area", "Area", areaText(area)) : null,
    defenseValue ? factItem("passive", "Defense", defenseValue.passive) : null,
    saveStatistic
      ? item(
          "save",
          "Save",
          `${basicSave === true ? "Basic " : ""}${formatSlug(saveStatistic)}`,
        )
      : null,
    durationText ? item("duration", "Duration", durationText) : null,
    durationText || sustained === true
      ? item("sustained", "Sustained", formatBoolean(Boolean(sustained)))
      : null,
  ].filter((entry): entry is RecordKeyValueItem => entry !== null);
  if (!items.length) return null;
  return (
    <SurfaceSection title="Range & targets">
      <RecordKeyValueList ariaLabel="Spell range and targets" items={items} />
    </SurfaceSection>
  );
}

function EffectSection({ value }: { value: SpellFactView<SpellDamageView[]> }) {
  const damage = meaningfulKnown(value);
  if (!damage) return null;
  return (
    <SurfaceSection title="Effect">
      <DamageList damage={damage} />
    </SurfaceSection>
  );
}

function DamageList({ damage }: { damage: SpellDamageView[] }) {
  return (
    <ol aria-label="Spell damage" className="spell-sheet__member-list">
      {damage.map((member, index) => (
        <li key={`${member.label}:${index}`}>
          <div className="spell-sheet__member-heading">
            <strong>
              {damageSummary(member)} {member.label.toLowerCase()}
            </strong>
          </div>
          {damageQualifiers(member).length ? (
            <Collapse
              ghost
              size="small"
              items={[
                {
                  key: "qualifiers",
                  label: "Effect details",
                  children: (
                    <RecordKeyValueList
                      ariaLabel={`${member.label} qualifiers`}
                      items={damageQualifiers(member)}
                    />
                  ),
                },
              ]}
            />
          ) : null}
        </li>
      ))}
    </ol>
  );
}

function damageSummary(damage: SpellDamageView) {
  return [
    meaningfulKnown(damage.formula),
    meaningfulKnown(damage.damage_type) &&
      formatSlug(meaningfulKnown(damage.damage_type)!),
  ]
    .filter((value): value is string => Boolean(value))
    .join(" ");
}

function damageQualifiers(damage: SpellDamageView): RecordKeyValueItem[] {
  return [
    factItem("category", "Category", damage.category, formatSlug),
    factItem("kinds", "Kinds", damage.kinds, formatList),
    factItem("materials", "Materials", damage.materials, formatList),
    factItem("apply-modifier", "Apply modifier", damage.apply_modifier, formatBoolean),
  ].filter((entry): entry is RecordKeyValueItem => entry !== null);
}

function HeighteningSection({
  appliedFixedRanks,
  value,
}: {
  appliedFixedRanks: number[];
  value: SpellFactView<SpellHeighteningView>;
}) {
  const heightening = meaningfulKnown(value);
  if (!heightening || !heighteningHasContent(heightening)) return null;
  return (
    <SurfaceSection title="Heightening">
      {appliedFixedRanks.length ? (
        <p className="spell-sheet__applied-ranks">
          Applied ranks {appliedFixedRanks.map(formatRank).join(", ")}
        </p>
      ) : null}
      <Heightening value={heightening} />
    </SurfaceSection>
  );
}

function Heightening({ value }: { value: SpellHeighteningView }) {
  if (value.kind === "interval") {
    const interval = meaningfulKnown(value.interval);
    const area = meaningfulKnown(value.area);
    const damage = meaningfulKnown(value.damage);
    return (
      <div className="spell-sheet__heightening-summary">
        {interval !== undefined ? <strong>Heightened (+{interval})</strong> : null}
        <ul className="spell-sheet__inline-list">
          {area !== undefined ? <li>Area increases by {area} feet</li> : null}
          {damage?.map((member, index) => (
            <li key={`${member.label}:${index}`}>
              {member.label} increases by {member.value}
            </li>
          ))}
        </ul>
      </div>
    );
  }
  return (
    <ul className="spell-sheet__member-list">
      {value.layers.flatMap((layer, index) => {
        const rank = sourceKnown(layer.rank);
        if (rank === undefined || !layer.changes.length) return [];
        return [
          <li key={`${rank}:${index}`}>
            <div className="spell-sheet__member-heading">
              <strong>At rank {rank}</strong>
            </div>
            <ul className="spell-sheet__inline-list">
              {layer.changes.map((change, changeIndex) => (
                <li key={changeIndex}>{fixedChangeSummary(change)}</li>
              ))}
            </ul>
          </li>,
        ];
      })}
    </ul>
  );
}

function heighteningHasContent(value: SpellHeighteningView) {
  if (value.kind === "fixed") {
    return value.layers.some(
      (layer) => sourceKnown(layer.rank) !== undefined && layer.changes.length > 0,
    );
  }
  return (
    meaningfulKnown(value.interval) !== undefined ||
    meaningfulKnown(value.area) !== undefined ||
    meaningfulKnown(value.damage) !== undefined
  );
}

function fixedChangeSummary(change: SpellFixedHeighteningChangeView) {
  if (change.field !== "effect") return `${formatSlug(change.field)} changes`;
  if (change.operation === "delete") return `${change.label} is removed`;
  if (!change.value) return `${change.label} changes`;
  const details = [
    meaningfulKnown(change.value.formula),
    meaningfulKnown(change.value.damage_type) &&
      formatSlug(meaningfulKnown(change.value.damage_type)!),
    meaningfulKnown(change.value.category) &&
      formatSlug(meaningfulKnown(change.value.category)!),
    meaningfulKnown(change.value.kinds)?.map(formatSlug).join(", "),
    meaningfulKnown(change.value.materials)?.map(formatSlug).join(", "),
    knownValue(change.value.apply_modifier) === undefined
      ? undefined
      : `apply modifier ${formatBoolean(knownValue(change.value.apply_modifier)!)}`,
  ].filter((detail): detail is string => Boolean(detail));
  return details.length
    ? `${change.label}: ${details.join(" · ")}`
    : `${change.label} changes`;
}

function FormsSection({
  body,
  catalog,
  loading,
  onSelectionChange,
  selection,
  selectionError,
  selectionUnavailable,
}: {
  body: SpellSurfaceView;
  catalog: SpellSurfaceView;
  loading: boolean;
  onSelectionChange?: (selection: SpellFormSelection) => void;
  selection?: SpellFormSelection;
  selectionError?: string;
  selectionUnavailable: boolean;
}) {
  const initialForm = catalog.forms.find((form) => form.id === body.effective_form.id);
  const [draftFormId, setDraftFormId] = useState<string | undefined>(initialForm?.id);
  const [draftRank, setDraftRank] = useState<number | null>(
    body.effective_form.cast_rank,
  );
  if (!onSelectionChange || !catalog.forms.length) return null;
  const draftForm = catalog.forms.find((form) => form.id === draftFormId);
  const displayed = selectionLabel(catalog, body.effective_form);
  const requested = selection && selectionLabel(catalog, selection);
  const matchesRequest =
    !selection ||
    (body.effective_form.id === selection.formId &&
      body.effective_form.cast_rank === selection.castRank);
  return (
    <SurfaceSection className="spell-sheet__form-section" title="Form & rank">
      <div
        aria-busy={loading}
        aria-label="Resolve spell form"
        className="spell-sheet__form-controls"
      >
        <div className="spell-sheet__form-control">
          <span>Form</span>
          {catalog.forms.length === 1 ? (
            <strong>{catalog.forms[0].label}</strong>
          ) : (
            <Select
              aria-label="Spell form"
              onChange={(formId) => {
                setDraftFormId(formId);
                const form = catalog.forms.find((candidate) => candidate.id === formId);
                if (form) setDraftRank(form.minimum_cast_rank);
              }}
              options={catalog.forms.map((form) => ({
                label: form.label,
                value: form.id,
              }))}
              placeholder="Choose a form"
              value={draftFormId}
              virtual={false}
            />
          )}
        </div>
        <div className="spell-sheet__form-control">
          <span>Cast rank</span>
          <InputNumber
            aria-label="Cast rank"
            max={255}
            min={draftForm?.minimum_cast_rank ?? 0}
            onChange={(rank) => setDraftRank(rank)}
            placeholder="Rank"
            precision={0}
            value={draftRank}
          />
        </div>
        <Button
          disabled={draftFormId === undefined || draftRank === null}
          onClick={() => {
            if (draftFormId !== undefined && draftRank !== null) {
              onSelectionChange({ formId: draftFormId, castRank: draftRank });
            }
          }}
          type="primary"
        >
          Apply
        </Button>
      </div>
      <p aria-live="polite" className="spell-sheet__selection-summary">
        Showing {displayed}.
      </p>
      {loading && selection ? (
        <Alert
          description={`Showing ${displayed} until the requested result is available.`}
          message={`Resolving ${requested ?? "selected form"}.`}
          showIcon
          type="info"
        />
      ) : null}
      {!loading && selectionError ? (
        <Alert
          description={`${selectionError} Showing ${displayed}.`}
          message="Unable to resolve the selected form"
          showIcon
          type="error"
        />
      ) : null}
      {!loading && !selectionError && selectionUnavailable ? (
        <Alert
          description={`Showing ${displayed}.`}
          message={`${requested ?? "The selected form"} is unavailable.`}
          showIcon
          type="warning"
        />
      ) : null}
      {!loading && !selectionError && !selectionUnavailable && !matchesRequest ? (
        <Alert
          description={`Showing ${displayed}.`}
          message="The returned result did not match the requested form and rank."
          showIcon
          type="warning"
        />
      ) : null}
    </SurfaceSection>
  );
}

function FormResult({ result }: { result: SpellFormResultView }) {
  if (result.state === "unavailable") return null;
  return <ResolvedDefinition definition={result.definition} />;
}

function ResolvedDefinition({
  definition,
}: {
  definition: SpellResolvedDefinitionView;
}) {
  const traditions =
    definition.classification.state === "available"
      ? meaningfulKnown(definition.classification.value)?.traditions
      : undefined;
  const casting = availableFact(definition.casting);
  const targeting = availableFact(definition.targeting);
  const defense = availableFact(definition.defense);
  const damage = availableFact(definition.damage);
  const duration = availableFact(definition.duration);
  const heightening = availableFact(definition.heightening);
  const rules = availableFact(definition.rules);
  return (
    <div className="spell-sheet__resolved-definition">
      <CastingSection value={casting} traditions={traditions} />
      <RangeAndTargetsSection
        defense={defense}
        duration={duration}
        targeting={targeting}
      />
      <EffectSection value={damage} />
      <HeighteningSection
        appliedFixedRanks={definition.applied_fixed_ranks}
        value={heightening}
      />
      <SecondaryMechanics ritual={definition.ritual} rules={rules} />
    </div>
  );
}

function availableFact<T>(value: SpellResolvedFieldView<T>): SpellFactView<T> {
  return value.state === "available" ? value.value : { state: "missing" };
}

function SecondaryMechanics({
  ritual,
  rules,
}: {
  ritual: SpellFactView<SpellRitualView>;
  rules: SpellFactView<SpellRuleView[]>;
}) {
  const ritualItems = ritualFactItems(ritual);
  const ruleValues = meaningfulKnown(rules);
  const items = [
    ritualItems.length
      ? {
          key: "ritual",
          label: "Ritual requirements",
          children: (
            <RecordKeyValueList ariaLabel="Ritual checks" items={ritualItems} />
          ),
        }
      : null,
    ruleValues
      ? {
          key: "rules",
          label: "Rule effects",
          children: <RuleList rules={ruleValues} />,
        }
      : null,
  ].filter((entry): entry is NonNullable<typeof entry> => entry !== null);
  if (!items.length) return null;
  return (
    <SurfaceSection title="Additional mechanics">
      <Collapse
        className="record-surface__inline-disclosure"
        ghost
        items={items}
        size="small"
      />
    </SurfaceSection>
  );
}

function ritualFactItems(value: SpellFactView<SpellRitualView>) {
  const ritual = meaningfulKnown(value);
  if (!ritual) return [];
  return [
    factItem("primary", "Primary check", ritual.primary_check),
    factItem("secondary-casters", "Secondary casters", ritual.secondary_casters),
    factItem("secondary-checks", "Secondary checks", ritual.secondary_checks),
  ].filter((entry): entry is RecordKeyValueItem => entry !== null);
}

function RuleList({ rules }: { rules: SpellRuleView[] }) {
  return (
    <ol aria-label="Spell rules" className="spell-sheet__member-list">
      {rules.map((rule) => (
        <li key={`${rule.order}:${rule.rule.kind}`}>
          <div className="spell-sheet__member-heading">
            <strong>{formatSlug(rule.rule.kind)}</strong>
          </div>
          <RuleDetails rule={rule.rule} />
        </li>
      ))}
    </ol>
  );
}

function RuleDetails({ rule }: { rule: SpellRuleDetailView }) {
  if (rule.kind === "unsupported") return null;
  switch (rule.kind) {
    case "damage_dice":
      return (
        <RuleFacts
          items={[
            factPresenceItem(
              "selector",
              "Scope",
              rule.value.selector,
              "Selected damage scope",
            ),
            factItem(
              "predicate",
              "Conditions",
              rule.value.predicate,
              predicatesSummary,
            ),
            factItem("dice", "Dice", rule.value.dice_number),
            factItem("die-size", "Die size", rule.value.die_size),
            factItem("damage-type", "Damage type", rule.value.damage_type, formatSlug),
            factItem(
              "hide-disabled",
              "Hide if disabled",
              rule.value.hide_if_disabled,
              formatBoolean,
            ),
          ]}
        />
      );
    case "ephemeral_effect":
      return (
        <RuleFacts
          items={[
            factItem(
              "predicate",
              "Conditions",
              rule.value.predicate,
              predicatesSummary,
            ),
            factItem("selectors", "Scope", rule.value.selectors, scopeCountSummary),
            factPresenceItem(
              "uuid",
              "Effect",
              rule.value.uuid,
              "Typed effect reference retained",
            ),
          ]}
        />
      );
    case "damage_alteration":
      return (
        <RuleFacts
          items={[
            factItem("mode", "Mode", rule.value.mode, formatSlug),
            factItem(
              "predicate",
              "Conditions",
              rule.value.predicate,
              predicatesSummary,
            ),
            factItem("property", "Property", rule.value.property, formatSlug),
            factItem("selectors", "Scope", rule.value.selectors, scopeCountSummary),
            factPresenceItem(
              "slug",
              "Effect",
              rule.value.slug,
              "Typed effect identifier retained",
            ),
            factItem("value", "Adjustment", rule.value.value),
          ]}
        />
      );
    case "roll_option":
      return (
        <RuleFacts
          items={[
            factItem("domain", "Domain", rule.value.domain, formatSlug),
            factItem("label", "Label", rule.value.label),
            factItem("option", "Option", rule.value.option, formatSlug),
            factItem("placement", "Placement", rule.value.placement, formatSlug),
            factItem(
              "predicate",
              "Conditions",
              rule.value.predicate,
              predicatesSummary,
            ),
            factItem(
              "suboptions",
              "Suboptions",
              rule.value.suboptions,
              (suboptions) => (
                <ul className="spell-sheet__inline-list">
                  {suboptions.flatMap((suboption, index) => {
                    const label = meaningfulKnown(suboption.label);
                    return label ? [<li key={index}>{label}</li>] : [];
                  })}
                </ul>
              ),
            ),
            factItem("toggleable", "Toggleable", rule.value.toggleable, formatBoolean),
          ]}
        />
      );
    case "item_alteration":
      return (
        <RuleFacts
          items={[
            factPresenceItem(
              "item",
              "Item",
              rule.value.item_id,
              "Typed item reference retained",
            ),
            factItem("mode", "Mode", rule.value.mode, formatSlug),
            factItem(
              "predicate",
              "Conditions",
              rule.value.predicate,
              predicatesSummary,
            ),
            factItem("property", "Property", rule.value.property, formatSlug),
            factItem("value", "Adjustment", rule.value.value),
          ]}
        />
      );
  }
}

function RuleFacts({ items }: { items: Array<RecordKeyValueItem | null> }) {
  return (
    <RecordKeyValueList
      ariaLabel="Rule details"
      items={items.filter((entry): entry is RecordKeyValueItem => entry !== null)}
    />
  );
}

function SpellSourceDisclosure({ metadata }: { metadata: RecordSurfaceMetadataView }) {
  const source = metadata.source;
  const facts = [
    source?.publication_title
      ? item("publication", "Publication", source.publication_title)
      : null,
    source?.pack_label ? item("pack", "Source pack", source.pack_label) : null,
  ].filter((entry): entry is RecordKeyValueItem => entry !== null);
  if (!facts.length) return null;
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
              ariaLabel="Spell source"
              items={facts}
              labelWidth="provenance"
            />
          ),
        },
      ]}
      size="small"
    />
  );
}

function knownValue<T>(fact: SpellFactView<T>): T | undefined {
  return fact.state === "known" ? fact.value : undefined;
}

function meaningfulKnown<T>(fact: SpellFactView<T>): T | undefined {
  const value = knownValue(fact);
  if (typeof value === "string" && !value.trim()) return undefined;
  if (Array.isArray(value) && !value.length) return undefined;
  return value;
}

function sourceKnown<T>(value: SpellSourceValueView<T>): T | undefined {
  return value.state === "known" ? value.value : undefined;
}

function factItem<T>(
  key: React.Key,
  label: React.ReactNode,
  fact: SpellFactView<T>,
  render: (value: T) => React.ReactNode = String,
) {
  const value = meaningfulKnown(fact);
  return value === undefined ? null : item(key, label, render(value));
}

function factPresenceItem<T>(
  key: React.Key,
  label: React.ReactNode,
  fact: SpellFactView<T>,
  text: string,
) {
  return meaningfulKnown(fact) === undefined ? null : item(key, label, text);
}

function item(key: React.Key, label: React.ReactNode, value: React.ReactNode) {
  return { key, label, value };
}

function formatList(values: string[]) {
  return values.length ? values.map(formatSlug).join(", ") : "None";
}

function formatBoolean(value: boolean) {
  return value ? "Yes" : "No";
}

function areaText(area: SpellAreaView | false | undefined) {
  if (!area) return undefined;
  const size = meaningfulKnown(area.value);
  const type = meaningfulKnown(area.area_type);
  const details = meaningfulKnown(area.details);
  const parts = [
    size === undefined ? undefined : `${size}-foot`,
    type ? formatSlug(type).toLowerCase() : undefined,
  ].filter((part): part is string => Boolean(part));
  const formattedArea = parts.length ? parts.join(" ") : undefined;
  return formattedArea && details
    ? `${formattedArea} — ${details}`
    : (formattedArea ?? details);
}

function scopeCountSummary(scopes: string[]) {
  return `${scopes.length} selected ${scopes.length === 1 ? "scope" : "scopes"}`;
}

function predicatesSummary(predicates: SpellRulePredicateView[]) {
  const alternatives = predicates.reduce(
    (count, predicate) =>
      predicate.kind === "term"
        ? count + 1
        : predicate.kind === "or"
          ? count + predicate.value.length
          : count,
    0,
  );
  return `${alternatives} typed ${alternatives === 1 ? "condition" : "conditions"}`;
}

function selectionLabel(
  catalog: SpellSurfaceView,
  selection: SpellFormSelection | { id: string; cast_rank: number },
) {
  const id = "formId" in selection ? selection.formId : selection.id;
  const rank = "castRank" in selection ? selection.castRank : selection.cast_rank;
  const form = catalog.forms.find((candidate) => candidate.id === id);
  return `${form?.label ?? "Selected form"} at rank ${rank}`;
}
