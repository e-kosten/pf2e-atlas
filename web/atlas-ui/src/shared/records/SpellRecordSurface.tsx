import { Button, Collapse, InputNumber, Select, Space, Tag, Typography } from "antd";
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
import { ActionGlyph } from "./ActionGlyph";
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
      <div className="spell-sheet__heading">
        <RecordHeader
          levelLabel="Rank"
          metadata={headerMetadata}
          onReference={onReference}
          showTitle={showTitle}
        />
        <FormsSection
          key={metadata.record_key}
          body={body}
          catalog={catalog}
          loading={selectionLoading}
          onSelectionChange={onSelectionChange}
          selection={selection}
          selectionError={selectionError}
          selectionUnavailable={selectionUnavailable}
        />
      </div>
      <NarrativeSection
        content={content}
        headingId="spell-description"
        onReference={onReference}
        title="Description"
      />
      <FormResult result={effective.result} />
      <RecordSurfaceIssues issues={issues} />
      <RecordSurfaceReferences
        recordKey={metadata.record_key}
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

function CastingSection({
  traditions,
  ritual,
  value,
}: {
  traditions?: SpellFactView<string[]>;
  ritual: SpellFactView<SpellRitualView>;
  value: SpellFactView<SpellCastingView>;
}) {
  const casting = meaningfulKnown(value);
  const items = [
    casting?.action_cost
      ? item("time", "Cast", <ActionGlyph cost={casting.action_cost} />)
      : casting
        ? factItem("time", "Cast", casting.time)
        : null,
    casting ? factItem("cost", "Cost", casting.cost) : null,
    casting ? factItem("requirements", "Requirements", casting.requirements) : null,
    casting
      ? factItem("counteraction", "Counteraction", casting.counteraction, formatBoolean)
      : null,
    traditions ? factItem("traditions", "Traditions", traditions, formatList) : null,
    ...ritualFactItems(ritual),
  ].filter((entry): entry is RecordKeyValueItem => entry !== null);
  if (!items.length) return null;
  return (
    <div className="spell-sheet__mechanics-group">
      <RecordKeyValueList ariaLabel="Spell casting" items={items} />
    </div>
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
      ? {
          ...item("range", "Range", range.authored_text.trim()),
          rowClassName: "spell-sheet__primary-fact",
        }
      : null,
    areaText(area)
      ? {
          ...item("area", "Area", areaText(area)),
          rowClassName: "spell-sheet__primary-fact",
        }
      : null,
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
    <div className="spell-sheet__mechanics-group">
      <RecordKeyValueList ariaLabel="Spell range and targets" items={items} />
    </div>
  );
}

function EffectSection({ value }: { value: SpellFactView<SpellDamageView[]> }) {
  const damage = meaningfulKnown(value);
  if (!damage) return null;
  return (
    <div className="spell-sheet__mechanics-group">
      <DamageList damage={damage} />
    </div>
  );
}

function DamageList({ damage }: { damage: SpellDamageView[] }) {
  return (
    <ol aria-label="Spell damage" className="spell-sheet__member-list">
      {damage.map((member, index) => (
        <li key={`${member.label}:${index}`}>
          <div className="spell-sheet__member-heading">
            <strong>
              <span>{damageSummary(member)}</span> {member.label.toLowerCase()}
            </strong>
          </div>
          {damageQualifiers(member).length ? (
            <RecordKeyValueList
              ariaLabel={`${member.label} qualifiers`}
              items={damageQualifiers(member)}
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
    factItem("category", "Damage category", damage.category, formatSlug),
    factItem("kinds", "Effect type", damage.kinds, formatList),
    factItem("materials", "Damage materials", damage.materials, formatList),
    factItem(
      "apply-modifier",
      "Spellcasting ability modifier",
      damage.apply_modifier,
      formatBoolean,
    ),
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
  const [rankNotice, setRankNotice] = useState("");
  const [draftRank, setDraftRank] = useState<number | null>(
    body.effective_form.cast_rank,
  );
  const [previousEffective, setPreviousEffective] = useState({
    formId: body.effective_form.id,
    castRank: body.effective_form.cast_rank,
  });
  if (
    previousEffective.formId !== body.effective_form.id ||
    previousEffective.castRank !== body.effective_form.cast_rank
  ) {
    setPreviousEffective({
      formId: body.effective_form.id,
      castRank: body.effective_form.cast_rank,
    });
    // Follow a refreshed base until the user has requested a selection. Selected
    // responses must not remount controls or overwrite a newer local draft.
    if (!selection) {
      setDraftFormId(initialForm?.id);
      setDraftRank(body.effective_form.cast_rank);
    }
  }
  if (!onSelectionChange || !catalog.forms.length) return null;
  const draftForm = catalog.forms.find((form) => form.id === draftFormId);
  const defaultForm = catalog.forms.find((form) => form.kind === "base");
  const appliedModified =
    defaultForm !== undefined &&
    (body.effective_form.id !== defaultForm.id ||
      body.effective_form.cast_rank !== defaultForm.minimum_cast_rank);
  const canReset =
    defaultForm !== undefined &&
    (appliedModified ||
      draftFormId !== defaultForm.id ||
      draftRank !== defaultForm.minimum_cast_rank ||
      (selection !== undefined &&
        (selection.formId !== defaultForm.id ||
          selection.castRank !== defaultForm.minimum_cast_rank)));
  const requested = selection && selectionLabel(catalog, selection);
  const matchesRequest =
    !selection ||
    (body.effective_form.id === selection.formId &&
      body.effective_form.cast_rank === selection.castRank);
  return (
    <section
      className={`spell-sheet__form-section${catalog.forms.length === 1 ? " spell-sheet__form-section--static" : ""}`}
      aria-label="Form & rank"
    >
      <div
        aria-busy={loading}
        aria-label="Resolve spell form"
        className="spell-sheet__form-controls"
      >
        <div className="spell-sheet__form-control spell-sheet__form-choice">
          <span>Form</span>
          {catalog.forms.length === 1 ? (
            <strong>{catalog.forms[0].label}</strong>
          ) : (
            <Select
              aria-label="Spell form"
              className="spell-sheet__form-select"
              optionRender={(option) => (
                <span className="spell-sheet__form-option">{option.label}</span>
              )}
              labelRender={(option) => (
                <span className="spell-sheet__form-option">{option.label}</span>
              )}
              onChange={(formId) => {
                setDraftFormId(formId);
                const form = catalog.forms.find((candidate) => candidate.id === formId);
                if (
                  form &&
                  (draftRank === null ||
                    draftRank < form.minimum_cast_rank ||
                    draftRank > 255)
                ) {
                  setDraftRank(form.minimum_cast_rank);
                  setRankNotice(
                    `Cast rank reset to ${form.minimum_cast_rank}, the minimum for this form.`,
                  );
                } else setRankNotice("");
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
        <div className="spell-sheet__rank-actions">
          <div className="spell-sheet__form-control">
            <span>Cast rank</span>
            <InputNumber
              aria-label="Cast rank"
              max={255}
              min={draftForm?.minimum_cast_rank ?? 0}
              onChange={(rank) => {
                setDraftRank(rank);
                setRankNotice("");
              }}
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
          <Button
            style={{ visibility: canReset ? "visible" : "hidden" }}
            disabled={!canReset}
            onClick={() => {
              if (defaultForm) {
                setRankNotice("");
                setDraftFormId(defaultForm.id);
                setDraftRank(defaultForm.minimum_cast_rank);
                onSelectionChange({
                  formId: defaultForm.id,
                  castRank: defaultForm.minimum_cast_rank,
                });
              }
            }}
          >
            Reset
          </Button>
        </div>
      </div>
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className="spell-sheet__selection-summary"
      >
        <div className="spell-sheet__applied-selection">
          <span>Applied rank {body.effective_form.cast_rank}</span>
          <Tag style={{ visibility: appliedModified ? "visible" : "hidden" }}>
            Modified from default
          </Tag>
        </div>
        <Typography.Text
          type={!loading && selectionError ? "danger" : "secondary"}
          className="spell-sheet__selection-status"
        >
          {loading && selection
            ? `Resolving ${requested ?? "selected form"}.`
            : selectionError
              ? `Unable to resolve the selected form. ${selectionError}`
              : selectionUnavailable
                ? `${requested ?? "The selected form"} is unavailable.`
                : !matchesRequest
                  ? "The returned result did not match the requested form and rank."
                  : rankNotice || null}
        </Typography.Text>
      </div>
    </section>
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
    <section aria-label="Spell mechanics" className="spell-sheet__resolved-definition">
      <CastingSection
        value={casting}
        traditions={traditions}
        ritual={definition.ritual}
      />
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
      <SecondaryMechanics rules={rules} />
    </section>
  );
}

function availableFact<T>(value: SpellResolvedFieldView<T>): SpellFactView<T> {
  return value.state === "available" ? value.value : { state: "missing" };
}

function SecondaryMechanics({ rules }: { rules: SpellFactView<SpellRuleView[]> }) {
  const ruleValues = meaningfulKnown(rules);
  const items = [
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
