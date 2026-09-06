import { Alert, Button, Collapse, InputNumber, Select, Space, Tag } from "antd";
import { useState } from "react";
import type React from "react";
import type {
  RecordSurfaceMetadataView,
  SpellAreaView,
  SpellCastingView,
  SpellClassificationView,
  SpellDamagePatchSetView,
  SpellDamagePatchView,
  SpellDamageView,
  SpellDefenseView,
  SpellDurationView,
  SpellFactView,
  SpellFormResultView,
  SpellFormView,
  SpellHeighteningPatchView,
  SpellHeighteningView,
  SpellPatchView,
  SpellResolvedDefinitionView,
  SpellResolvedFieldView,
  SpellRitualView,
  SpellRuleDetailView,
  SpellRulePredicateView,
  SpellRuleView,
  SpellSourceValueView,
  SpellSurfaceView,
  SpellTargetingView,
  SpellTextPatchSetView,
} from "../../generated/atlas";
import {
  NarrativeSection,
  RecordHeader,
  SurfaceSection,
} from "./CreatureRecordSurface";
import { narrativeContent, type ReferenceHandler } from "./RecordRichContent";
import { RecordKeyValueList, type RecordKeyValueItem } from "./RecordKeyValueList";
import { formatRank, formatSlug } from "./recordFormatting";

export type SpellFormSelection = {
  castRank: number;
  formId: string;
};

type SpellDetailSurfaceProps = {
  body: SpellSurfaceView;
  catalog: SpellSurfaceView;
  metadata: RecordSurfaceMetadataView;
  onReference: ReferenceHandler;
  onSelectionChange?: (selection: SpellFormSelection) => void;
  selection?: SpellFormSelection;
  selectionLoading?: boolean;
  showTitle: boolean;
};

export function SpellDetailSurface({
  body,
  catalog,
  metadata,
  onReference,
  onSelectionChange,
  selection,
  selectionLoading = false,
  showTitle,
}: SpellDetailSurfaceProps) {
  const content = narrativeContent(body.content);
  return (
    <article className="record-surface record-surface--record-detail spell-sheet">
      <RecordHeader
        metadata={metadata}
        onReference={onReference}
        showTitle={showTitle}
      />
      <NarrativeSection
        content={content}
        headingId="spell-overview"
        onReference={onReference}
        title="Overview"
      />
      <div className="spell-sheet__summary-grid">
        <ClassificationSection value={body.definition.classification} />
        <CastingSection value={body.definition.casting} />
        <TargetingSection value={body.definition.targeting} />
        <DefenseSection value={body.definition.defense} />
        <DamageSection value={body.definition.damage} />
        <DurationSection value={body.definition.duration} />
      </div>
      <HeighteningSection value={body.definition.heightening} />
      <FormsSection
        body={body}
        catalog={catalog}
        loading={selectionLoading}
        onSelectionChange={onSelectionChange}
        selection={selection}
      />
      <RitualSection value={body.definition.ritual} />
      <RulesSection value={body.definition.rules} />
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
  const classification = knownValue(body.definition.classification);
  const rank = classification && knownValue(classification.rank);
  const traditions = classification && knownValue(classification.traditions);
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <div className="record-surface-search__heading">
          <h2>{metadata.title}</h2>
        </div>
        <div className="creature-sheet__identity-meta creature-sheet__identity-meta--compact">
          <span>{metadata.kind_label || formatSlug(metadata.kind)}</span>
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

function ClassificationSection({
  value,
}: {
  value: SpellFactView<SpellClassificationView>;
}) {
  return (
    <SurfaceSection title="Classification">
      <FactGroup
        ariaLabel="Spell classification"
        fact={value}
        render={(classification) => [
          item("rank", "Rank", factNode(classification.rank, formatRank)),
          item("traits", "Traits", factNode(classification.traits, formatList)),
          item(
            "traditions",
            "Traditions",
            factNode(classification.traditions, formatList),
          ),
        ]}
      />
    </SurfaceSection>
  );
}

function CastingSection({ value }: { value: SpellFactView<SpellCastingView> }) {
  return (
    <SurfaceSection title="Casting">
      <FactGroup
        ariaLabel="Spell casting"
        fact={value}
        render={(casting) => [
          item("time", "Cast", factNode(casting.time)),
          item("cost", "Cost", factNode(casting.cost)),
          item("requirements", "Requirements", factNode(casting.requirements)),
          item(
            "counteraction",
            "Counteraction",
            factNode(casting.counteraction, formatBoolean),
          ),
        ]}
      />
    </SurfaceSection>
  );
}

function TargetingSection({ value }: { value: SpellFactView<SpellTargetingView> }) {
  return (
    <SurfaceSection title="Targeting">
      <FactGroup
        ariaLabel="Spell targeting"
        fact={value}
        render={(targeting) => [
          item("target", "Targets", factNode(targeting.target)),
          item(
            "range",
            "Range",
            factNode(targeting.range, (range) => range.authored_text),
          ),
          item("area", "Area", factNode(targeting.area, renderArea)),
        ]}
      />
    </SurfaceSection>
  );
}

function DefenseSection({ value }: { value: SpellFactView<SpellDefenseView> }) {
  return (
    <SurfaceSection title="Defense">
      <FactGroup
        ariaLabel="Spell defense"
        fact={value}
        render={(defense) => [
          item("passive", "Passive", factNode(defense.passive)),
          item(
            "save-statistic",
            "Save",
            factNode(defense.save, (save) => factNode(save.statistic)),
          ),
          item(
            "save-basic",
            "Basic save",
            factNode(defense.save, (save) => factNode(save.basic, formatBoolean)),
          ),
        ]}
      />
    </SurfaceSection>
  );
}

function DamageSection({ value }: { value: SpellFactView<SpellDamageView[]> }) {
  return (
    <SurfaceSection className="spell-sheet__span" title="Damage">
      <FactCollection
        fact={value}
        emptyLabel="No authored damage"
        render={(damage) => <DamageList damage={damage} />}
      />
    </SurfaceSection>
  );
}

function DamageList({ damage }: { damage: SpellDamageView[] }) {
  return (
    <ol aria-label="Spell damage" className="spell-sheet__member-list">
      {damage.map((member) => (
        <li key={`${member.key}:${member.order}`}>
          <div className="spell-sheet__member-heading">
            <strong>Damage {member.order + 1}</strong>
            <code>{member.key}</code>
          </div>
          <RecordKeyValueList
            items={damageItems(member)}
            ariaLabel={`Damage ${member.order + 1}`}
          />
        </li>
      ))}
    </ol>
  );
}

function damageItems(damage: SpellDamageView): RecordKeyValueItem[] {
  return [
    item("formula", "Formula", factNode(damage.formula)),
    item("type", "Type", factNode(damage.damage_type, formatSlug)),
    item("category", "Category", factNode(damage.category, formatSlug)),
    item("kinds", "Kinds", factNode(damage.kinds, formatList)),
    item("materials", "Materials", factNode(damage.materials, formatList)),
    item(
      "apply-modifier",
      "Apply modifier",
      factNode(damage.apply_modifier, formatBoolean),
    ),
  ];
}

function DurationSection({ value }: { value: SpellFactView<SpellDurationView> }) {
  return (
    <SurfaceSection title="Duration">
      <FactGroup
        ariaLabel="Spell duration"
        fact={value}
        render={(duration) => [
          item("value", "Duration", factNode(duration.value)),
          item("sustained", "Sustained", factNode(duration.sustained, formatBoolean)),
        ]}
      />
    </SurfaceSection>
  );
}

function HeighteningSection({ value }: { value: SpellFactView<SpellHeighteningView> }) {
  return (
    <SurfaceSection title="Heightening">
      <FactCollection
        fact={value}
        render={(heightening) => <Heightening value={heightening} />}
      />
    </SurfaceSection>
  );
}

function Heightening({ value }: { value: SpellHeighteningView }) {
  if (value.kind === "interval") {
    return (
      <RecordKeyValueList
        ariaLabel="Interval heightening"
        items={[
          item("interval", "Interval", factNode(value.interval)),
          item("area", "Area increase", factNode(value.area)),
          item(
            "damage",
            "Damage increases",
            factNode(value.damage, (members) => <KeyedValues values={members} />),
          ),
        ]}
      />
    );
  }
  return (
    <Collapse
      ghost
      items={value.layers.map((layer) => ({
        key: `${layer.order}:${sourceValueText(layer.rank)}`,
        label: `Rank ${sourceValueText(layer.rank)}`,
        children: <SpellPatch patch={layer.patch} />,
      }))}
      size="small"
    />
  );
}

function FormsSection({
  body,
  catalog,
  loading,
  onSelectionChange,
  selection,
}: {
  body: SpellSurfaceView;
  catalog: SpellSurfaceView;
  loading: boolean;
  onSelectionChange?: (selection: SpellFormSelection) => void;
  selection?: SpellFormSelection;
}) {
  const [draftFormId, setDraftFormId] = useState<string>();
  const [draftRank, setDraftRank] = useState<number | null>(null);

  const selected = selectedFormFor(body, selection);
  return (
    <SurfaceSection title="Spell forms">
      {catalog.form_catalog_unavailable ? (
        <Alert
          message={`Form catalog unavailable: ${formatSlug(catalog.form_catalog_unavailable)}`}
          showIcon
          type="warning"
        />
      ) : null}
      {onSelectionChange && catalog.forms.length ? (
        <div
          aria-busy={loading}
          aria-label="Resolve spell form"
          className="spell-sheet__form-controls"
        >
          <Select
            aria-label="Spell form"
            onChange={(formId) => {
              setDraftFormId(formId);
              const form = catalog.forms.find((candidate) => candidate.id === formId);
              if (form) setDraftRank(form.cast_rank);
            }}
            options={catalog.forms.map((form) => ({
              label: form.label,
              value: form.id,
            }))}
            placeholder="Choose a form"
            value={draftFormId}
          />
          <InputNumber
            aria-label="Cast rank"
            onChange={(rank) => setDraftRank(rank)}
            placeholder="Rank"
            precision={0}
            value={draftRank}
          />
          <Button
            disabled={draftFormId === undefined || draftRank === null}
            onClick={() => {
              if (draftFormId !== undefined && draftRank !== null) {
                onSelectionChange({ formId: draftFormId, castRank: draftRank });
              }
            }}
            type="primary"
          >
            Resolve form
          </Button>
          {loading ? <span role="status">Resolving selected form…</span> : null}
        </div>
      ) : null}
      {selection && !loading && !selected ? (
        <Alert
          message="The selected form response did not match the current form and rank."
          showIcon
          type="warning"
        />
      ) : null}
      {selected ? (
        <SelectedForm result={selected.result} selection={selection!} />
      ) : null}
      <Collapse
        className="record-surface__inline-disclosure"
        ghost
        items={catalog.forms.map((form) => ({
          key: form.id,
          label: form.label,
          children: <CatalogForm form={form} />,
        }))}
        size="small"
      />
    </SurfaceSection>
  );
}

function CatalogForm({ form }: { form: SpellFormView }) {
  return (
    <div className="spell-sheet__catalog-form">
      <RecordKeyValueList
        ariaLabel={`${form.label} identity`}
        items={[
          item("kind", "Kind", formatSlug(form.kind)),
          item("rank", "Catalog rank", formatRank(form.cast_rank)),
          item("order", "Order", form.order),
        ]}
      />
      {form.authored_patch ? (
        <div>
          <h4>Authored patch</h4>
          <SpellPatch patch={form.authored_patch} />
        </div>
      ) : null}
      <div>
        <h4>Catalog result</h4>
        <FormResult result={form.result} />
      </div>
    </div>
  );
}

function SelectedForm({
  result,
  selection,
}: {
  result: SpellFormResultView;
  selection: SpellFormSelection;
}) {
  return (
    <div aria-live="polite" className="spell-sheet__selected-form">
      <h4>Selected form at {formatRank(selection.castRank)} rank</h4>
      <FormResult result={result} />
    </div>
  );
}

function FormResult({ result }: { result: SpellFormResultView }) {
  if (result.state === "unavailable") {
    const detail =
      result.reason.reason === "cast_rank_below_base"
        ? `Base rank ${result.reason.base_rank}; selected rank ${result.reason.cast_rank}.`
        : undefined;
    return (
      <Alert
        description={detail}
        message={`Form unavailable: ${formatSlug(result.reason.reason)}`}
        showIcon
        type="warning"
      />
    );
  }
  return <ResolvedDefinition definition={result.definition} />;
}

function ResolvedDefinition({
  definition,
}: {
  definition: SpellResolvedDefinitionView;
}) {
  return (
    <div className="spell-sheet__resolved-definition">
      <RecordKeyValueList
        ariaLabel="Resolved heightening"
        items={[
          item(
            "applied-ranks",
            "Applied fixed ranks",
            definition.applied_fixed_ranks.length
              ? definition.applied_fixed_ranks.map(formatRank).join(", ")
              : "None",
          ),
        ]}
      />
      <ResolvedField
        field="Classification"
        value={definition.classification}
        render={(fact) => <ClassificationSection value={fact} />}
      />
      <ResolvedField
        field="Casting"
        value={definition.casting}
        render={(fact) => <CastingSection value={fact} />}
      />
      <ResolvedField
        field="Targeting"
        value={definition.targeting}
        render={(fact) => <TargetingSection value={fact} />}
      />
      <ResolvedField
        field="Defense"
        value={definition.defense}
        render={(fact) => <DefenseSection value={fact} />}
      />
      <ResolvedField
        field="Damage"
        value={definition.damage}
        render={(fact) => <DamageSection value={fact} />}
      />
      <ResolvedField
        field="Duration"
        value={definition.duration}
        render={(fact) => <DurationSection value={fact} />}
      />
      <ResolvedField
        field="Heightening"
        value={definition.heightening}
        render={(fact) => <HeighteningSection value={fact} />}
      />
      <ResolvedField
        field="Rules"
        value={definition.rules}
        render={(fact) => <RulesSection value={fact} />}
      />
    </div>
  );
}

function ResolvedField<T>({
  field,
  render,
  value,
}: {
  field: string;
  render: (fact: SpellFactView<T>) => React.ReactNode;
  value: SpellResolvedFieldView<T>;
}) {
  if (value.state === "available") return render(value.value);
  return (
    <Alert
      description={`${formatSlug(value.unavailable.source)} patch`}
      message={`${field} unavailable: ${formatSlug(value.unavailable.reason)}`}
      showIcon
      type="warning"
    />
  );
}

function SpellPatch({ patch }: { patch: SpellPatchView }) {
  return (
    <div className="spell-sheet__patch">
      <PatchFact label="Classification" value={patch.classification}>
        {(value) => <ClassificationSection value={{ state: "known", value }} />}
      </PatchFact>
      <PatchFact label="Casting" value={patch.casting}>
        {(value) => <CastingSection value={{ state: "known", value }} />}
      </PatchFact>
      <PatchFact label="Targeting" value={patch.targeting}>
        {(value) => <TargetingSection value={{ state: "known", value }} />}
      </PatchFact>
      <PatchFact label="Defense" value={patch.defense}>
        {(value) => <DefenseSection value={{ state: "known", value }} />}
      </PatchFact>
      <PatchFact label="Damage" value={patch.damage}>
        {(value) => <DamagePatchSet value={value} />}
      </PatchFact>
      <PatchFact label="Duration" value={patch.duration}>
        {(value) => <DurationSection value={{ state: "known", value }} />}
      </PatchFact>
      <PatchFact label="Heightening" value={patch.heightening}>
        {(value) => <HeighteningPatch value={value} />}
      </PatchFact>
      <PatchFact label="Rules" value={patch.rules}>
        {(value) => <RuleList rules={value} />}
      </PatchFact>
      {patch.unsupported_fields?.length ? (
        <Alert
          message={`Unavailable patch fields: ${patch.unsupported_fields.map(formatSlug).join(", ")}`}
          showIcon
          type="warning"
        />
      ) : null}
    </div>
  );
}

function PatchFact<T>({
  children,
  label,
  value,
}: {
  children: (value: T) => React.ReactNode;
  label: string;
  value: SpellFactView<T>;
}) {
  if (value.state === "missing") return null;
  if (value.state !== "known") {
    return (
      <Alert
        message={`${label} patch: ${factStateLabel(value)}`}
        showIcon
        type="warning"
      />
    );
  }
  return <div className="spell-sheet__patch-group">{children(value.value)}</div>;
}

function DamagePatchSet({ value }: { value: SpellDamagePatchSetView }) {
  return (
    <ol aria-label="Damage patch members" className="spell-sheet__member-list">
      {value.members.map((member) => (
        <li key={`${member.key}:${member.order}`}>
          <div className="spell-sheet__member-heading">
            <strong>{formatSlug(member.operation.operation)}</strong>
            <code>{member.key}</code>
          </div>
          {member.operation.operation === "merge" ? (
            <DamagePatch value={member.operation.value} />
          ) : null}
        </li>
      ))}
    </ol>
  );
}

function DamagePatch({ value }: { value: SpellDamagePatchView }) {
  return (
    <RecordKeyValueList
      ariaLabel="Damage patch"
      items={[
        item("formula", "Formula", factNode(value.formula)),
        item("type", "Type", factNode(value.damage_type, formatSlug)),
        item("category", "Category", factNode(value.category, formatSlug)),
        item("kinds", "Kinds", factNode(value.kinds, formatList)),
        item("materials", "Materials", factNode(value.materials, formatList)),
        item(
          "apply-modifier",
          "Apply modifier",
          factNode(value.apply_modifier, formatBoolean),
        ),
      ]}
    />
  );
}

function HeighteningPatch({ value }: { value: SpellHeighteningPatchView }) {
  return (
    <RecordKeyValueList
      ariaLabel="Heightening patch"
      items={[
        item("kind", "Kind", factNode(value.kind, formatSlug)),
        item("interval", "Interval", factNode(value.interval)),
        item("area", "Area increase", factNode(value.area)),
        item("damage", "Damage", factNode(value.damage, renderTextPatchSet)),
      ]}
    />
  );
}

function RitualSection({ value }: { value: SpellFactView<SpellRitualView> }) {
  if (value.state === "missing") return null;
  return (
    <SurfaceSection title="Ritual">
      <FactGroup
        ariaLabel="Ritual checks"
        fact={value}
        render={(ritual) => [
          item("primary", "Primary check", factNode(ritual.primary_check)),
          item(
            "secondary-casters",
            "Secondary casters",
            factNode(ritual.secondary_casters),
          ),
          item(
            "secondary-checks",
            "Secondary checks",
            factNode(ritual.secondary_checks),
          ),
        ]}
      />
    </SurfaceSection>
  );
}

function RulesSection({ value }: { value: SpellFactView<SpellRuleView[]> }) {
  if (value.state === "missing") return null;
  return (
    <SurfaceSection title="Rules">
      <FactCollection
        fact={value}
        emptyLabel="No authored rules"
        render={(rules) => <RuleList rules={rules} />}
      />
    </SurfaceSection>
  );
}

function RuleList({ rules }: { rules: SpellRuleView[] }) {
  return (
    <ol aria-label="Spell rules" className="spell-sheet__member-list">
      {rules.map((rule) => (
        <li key={`${rule.order}:${rule.rule.kind}`}>
          <div className="spell-sheet__member-heading">
            <strong>{formatSlug(rule.rule.kind)}</strong>
            <span>Rule {rule.order + 1}</span>
          </div>
          <RuleDetails rule={rule.rule} />
        </li>
      ))}
    </ol>
  );
}

function RuleDetails({ rule }: { rule: SpellRuleDetailView }) {
  if (rule.kind === "unsupported") {
    return <Alert message="Rule details unavailable" showIcon type="warning" />;
  }
  switch (rule.kind) {
    case "damage_dice":
      return (
        <RuleFacts
          items={[
            item("selector", "Selector", factNode(rule.value.selector)),
            item(
              "predicate",
              "Predicate",
              factNode(rule.value.predicate, predicatesNode),
            ),
            item("dice", "Dice", factNode(rule.value.dice_number)),
            item("die-size", "Die size", factNode(rule.value.die_size)),
            item(
              "damage-type",
              "Damage type",
              factNode(rule.value.damage_type, formatSlug),
            ),
            item(
              "hide-disabled",
              "Hide if disabled",
              factNode(rule.value.hide_if_disabled, formatBoolean),
            ),
          ]}
        />
      );
    case "ephemeral_effect":
      return (
        <RuleFacts
          items={[
            item(
              "predicate",
              "Predicate",
              factNode(rule.value.predicate, predicatesNode),
            ),
            item("selectors", "Selectors", factNode(rule.value.selectors, formatList)),
            item("uuid", "Effect", factNode(rule.value.uuid)),
          ]}
        />
      );
    case "damage_alteration":
      return (
        <RuleFacts
          items={[
            item("mode", "Mode", factNode(rule.value.mode, formatSlug)),
            item(
              "predicate",
              "Predicate",
              factNode(rule.value.predicate, predicatesNode),
            ),
            item("property", "Property", factNode(rule.value.property)),
            item("selectors", "Selectors", factNode(rule.value.selectors, formatList)),
            item("slug", "Slug", factNode(rule.value.slug)),
            item("value", "Value", factNode(rule.value.value)),
          ]}
        />
      );
    case "roll_option":
      return (
        <RuleFacts
          items={[
            item("domain", "Domain", factNode(rule.value.domain)),
            item("label", "Label", factNode(rule.value.label)),
            item("option", "Option", factNode(rule.value.option)),
            item("placement", "Placement", factNode(rule.value.placement)),
            item(
              "predicate",
              "Predicate",
              factNode(rule.value.predicate, predicatesNode),
            ),
            item(
              "suboptions",
              "Suboptions",
              factNode(rule.value.suboptions, (suboptions) => (
                <ul className="spell-sheet__inline-list">
                  {suboptions.map((suboption, index) => (
                    <li key={index}>
                      {factNode(suboption.label)}: {factNode(suboption.value)}
                    </li>
                  ))}
                </ul>
              )),
            ),
            item(
              "toggleable",
              "Toggleable",
              factNode(rule.value.toggleable, formatBoolean),
            ),
          ]}
        />
      );
    case "item_alteration":
      return (
        <RuleFacts
          items={[
            item("item", "Item", factNode(rule.value.item_id)),
            item("mode", "Mode", factNode(rule.value.mode, formatSlug)),
            item(
              "predicate",
              "Predicate",
              factNode(rule.value.predicate, predicatesNode),
            ),
            item("property", "Property", factNode(rule.value.property)),
            item("value", "Value", factNode(rule.value.value)),
          ]}
        />
      );
  }
}

function RuleFacts({ items }: { items: RecordKeyValueItem[] }) {
  return <RecordKeyValueList ariaLabel="Rule details" items={items} />;
}

function SpellSourceDisclosure({ metadata }: { metadata: RecordSurfaceMetadataView }) {
  const source = metadata.source;
  const facts = [
    item("record", "Record ID", metadata.record_key ?? "Unavailable"),
    item("publication", "Publication", source?.publication_title ?? "Unavailable"),
    item("pack", "Source pack", source?.pack_label ?? "Unavailable"),
    item("path", "Source path", source?.source_path ?? "Unavailable"),
  ];
  return (
    <Collapse
      className="record-surface__secondary"
      ghost
      items={[
        {
          key: "references-source",
          label: "References & Source",
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

function FactGroup<T>({
  ariaLabel,
  fact,
  render,
}: {
  ariaLabel: string;
  fact: SpellFactView<T>;
  render: (value: T) => RecordKeyValueItem[];
}) {
  if (fact.state !== "known") return <Availability fact={fact} />;
  return <RecordKeyValueList ariaLabel={ariaLabel} items={render(fact.value)} />;
}

function FactCollection<T>({
  emptyLabel,
  fact,
  render,
}: {
  emptyLabel?: string;
  fact: SpellFactView<T>;
  render: (value: T) => React.ReactNode;
}) {
  if (fact.state !== "known") return <Availability fact={fact} />;
  if (Array.isArray(fact.value) && fact.value.length === 0) {
    return <span className="spell-sheet__availability">{emptyLabel ?? "None"}</span>;
  }
  return render(fact.value);
}

function Availability<T>({ fact }: { fact: SpellFactView<T> }) {
  return <span className="spell-sheet__availability">{factStateLabel(fact)}</span>;
}

function factNode<T>(
  fact: SpellFactView<T>,
  render: (value: T) => React.ReactNode = String,
) {
  return fact.state === "known" ? render(fact.value) : factStateNode(fact);
}

function factStateNode<T>(fact: SpellFactView<T>) {
  return <span className="spell-sheet__availability">{factStateLabel(fact)}</span>;
}

function factStateLabel<T>(fact: SpellFactView<T>) {
  switch (fact.state) {
    case "missing":
      return "Not authored";
    case "null":
      return "Authored null";
    case "unsupported":
      return "Unavailable";
    case "known":
      return "Available";
  }
}

function knownValue<T>(fact: SpellFactView<T>): T | undefined {
  return fact.state === "known" ? fact.value : undefined;
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

function renderArea(area: SpellAreaView) {
  return (
    <RecordKeyValueList
      ariaLabel="Spell area"
      items={[
        item("value", "Size", factNode(area.value)),
        item("type", "Type", factNode(area.area_type, formatSlug)),
        item("legacy-type", "Legacy type", factNode(area.legacy_area_type, formatSlug)),
        item("details", "Details", factNode(area.details)),
      ]}
    />
  );
}

function sourceValueText(value: SpellSourceValueView<number>) {
  return value.state === "known" ? value.value.toString() : "Unavailable";
}

function KeyedValues({
  values,
}: {
  values: Array<{ key: string; order: number; value: string }>;
}) {
  return (
    <ul className="spell-sheet__inline-list">
      {values.map((value) => (
        <li key={`${value.key}:${value.order}`}>
          <code>{value.key}</code>: {value.value}
        </li>
      ))}
    </ul>
  );
}

function renderTextPatchSet(set: SpellTextPatchSetView) {
  return (
    <ul className="spell-sheet__inline-list">
      {set.members.map((member) => (
        <li key={`${member.key}:${member.order}`}>
          <code>{member.key}</code>: {formatSlug(member.operation.operation)}
          {member.operation.operation === "merge"
            ? ` (${factStateLabel(member.operation.value)}${member.operation.value.state === "known" ? `: ${member.operation.value.value}` : ""})`
            : ""}
        </li>
      ))}
    </ul>
  );
}

function predicatesNode(predicates: SpellRulePredicateView[]) {
  return predicates.length
    ? predicates
        .map((predicate) => {
          if (predicate.kind === "term") return predicate.value;
          if (predicate.kind === "or") return `any of ${predicate.value.join(", ")}`;
          return "Unavailable predicate";
        })
        .join("; ")
    : "None";
}

function selectedFormFor(
  body: SpellSurfaceView,
  selection: SpellFormSelection | undefined,
) {
  if (!selection || !body.selected_form) return undefined;
  return body.selected_form.id === selection.formId &&
    body.selected_form.cast_rank === selection.castRank
    ? body.selected_form
    : undefined;
}
