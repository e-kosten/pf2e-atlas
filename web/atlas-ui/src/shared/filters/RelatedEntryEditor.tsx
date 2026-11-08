import { Button, Checkbox, Select, Typography, type SelectProps } from "antd";
import { useId, useRef, useState, type ReactNode } from "react";
import type { RuleGroupType } from "react-querybuilder";
import type {
  FilterEditorFieldView,
  QueryRelationshipDirection,
} from "../../generated/atlas";
import { AtlasModal } from "../ui/overlays/AtlasModal";
import { isBuilderGroup } from "./builderEdits";
import { filterValueLabel } from "./filterValueLabel";
import {
  BOOLEAN_CONDITION,
  RELATED_CONDITION,
  builderPredicate,
  emptyGroup,
  isClearedOperand,
  type PredicateRule,
} from "./queryBuilder";

export function isEmbeddedEntryField(
  field: FilterEditorFieldView | undefined,
): boolean {
  return (
    field?.definition.field_type === "collection" &&
    field.definition.scope === null &&
    field.definition.path.startsWith("embedded.")
  );
}

function presenceOnly(group: RuleGroupType): boolean {
  const [rule] = group.rules;
  return (
    !rule ||
    (group.rules.length === 1 &&
      !group.not &&
      !("rules" in rule) &&
      rule.field === BOOLEAN_CONDITION &&
      !(rule as PredicateRule).meta?.atlasExcluded &&
      rule.value === true)
  );
}

function linkVerb(direction: QueryRelationshipDirection, excluded = false) {
  if (direction === "outgoing") return excluded ? "Not linked from" : "Linked from";
  return excluded ? "Does not link to" : "Links to";
}

function simpleSummary(
  query: RuleGroupType,
  fields: FilterEditorFieldView[],
  scope: string | null,
) {
  if (
    query.not ||
    query.combinator !== "and" ||
    !query.rules.length ||
    query.rules.length > 2
  )
    return "Matching filters";
  const labels = [];
  for (const rule of query.rules) {
    if ("rules" in rule || (rule as PredicateRule).meta?.atlasExcluded)
      return "Matching filters";
    const field = fields.find(
      (field) => field.definition.id === rule.field && field.definition.scope === scope,
    );
    const operator = field?.operators.find(
      (option) => option.operator === rule.operator,
    );
    const value = rule.value;
    if (
      !field ||
      !operator ||
      isClearedOperand(rule, field.definition) ||
      (typeof value !== "string" && typeof value !== "number") ||
      operator.operand !== "scalar"
    )
      return "Matching filters";
    const label = filterValueLabel(field, String(value));
    if (label.length > 40) return "Matching filters";
    labels.push([field.label, operator.label, label].join(" "));
  }
  return labels.join(" · ");
}

export function RelatedEntrySummary({
  rule,
  fields,
}: {
  rule: PredicateRule;
  fields: FilterEditorFieldView[];
}) {
  const collection = fields.find(
    (field) => field.definition.id === rule.field && isEmbeddedEntryField(field),
  );
  const query = collection ? rule.value : rule.value?.query;
  if (!isBuilderGroup(query)) return null;
  const excluded = rule.meta?.atlasExcluded ?? false;
  return (
    <Typography.Text type="secondary" className="filter-link-summary">
      {collection
        ? (excluded ? "Does not contain" : "Contains") + " · " + collection.label
        : linkVerb(rule.operator === "outgoing" ? "outgoing" : "incoming", excluded)}
      {" · "}
      {collection && presenceOnly(query)
        ? "Any entry"
        : simpleSummary(query, fields, collection?.definition.path ?? null)}
      {!collection && rule.value?.includeContents === false && " · Main entry only"}
    </Typography.Text>
  );
}

type Draft = {
  relation: "contains" | QueryRelationshipDirection;
  collection: string | undefined;
  query: RuleGroupType;
  includeContents: boolean;
  excluded: boolean;
};

function RelatedEntrySelect<Value extends string>({
  label,
  ...props
}: SelectProps<Value> & { label: string }) {
  const id = useId();
  return (
    <div className="related-entry-editor__field">
      <label htmlFor={id}>{label}</label>
      <Select
        {...props}
        id={id}
        styles={{
          root: { width: "100%" },
          popup: { listItem: { whiteSpace: "normal", overflowWrap: "anywhere" } },
        }}
      />
    </div>
  );
}

function filterDescription(draft: Draft, scope: string | null) {
  if (draft.relation === "incoming")
    return "Find entries that link to an entry matching this filter group.";
  if (draft.relation === "outgoing")
    return "Find entries linked from an entry matching this filter group.";
  if (!draft.query.rules.length)
    return "Matches any entry of the selected type. Add conditions to narrow the match.";
  return scope === "embedded.spells"
    ? "Find entries with a spell that matches this filter group."
    : "Find entries with a contained entry that matches this filter group.";
}

export function RelatedEntryEditor({
  rule,
  fields,
  disabled,
  linkLimitReached,
  onApply,
  renderConditions,
}: {
  rule: PredicateRule;
  fields: FilterEditorFieldView[];
  disabled?: boolean;
  linkLimitReached: boolean;
  onApply: (rule: PredicateRule) => void;
  renderConditions: (
    query: RuleGroupType,
    change: (query: RuleGroupType) => void,
    scope: string | null,
  ) => ReactNode;
}) {
  const collections = fields.filter(isEmbeddedEntryField);
  const applied = collections.find((field) => field.definition.id === rule.field);
  const appliedQuery = applied ? rule.value : rule.value?.query;
  const appliedGroup = isBuilderGroup(appliedQuery) ? appliedQuery : emptyGroup();
  const appliedLink = !applied && isBuilderGroup(appliedQuery);
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<Draft>();
  const opener = useRef<HTMLButtonElement>(null);
  const exclusions = rule.meta?.atlasExcluded ?? false;
  const configured = !!applied || appliedLink;
  function begin() {
    setDraft({
      relation:
        applied || !appliedLink
          ? "contains"
          : rule.operator === "outgoing"
            ? "outgoing"
            : "incoming",
      collection: applied?.definition.id,
      query: applied && presenceOnly(appliedGroup) ? emptyGroup() : appliedGroup,
      includeContents: rule.value?.includeContents ?? true,
      excluded: exclusions,
    });
    setOpen(true);
  }
  const collection = collections.find(
    (field) => field.definition.id === draft?.collection,
  );
  const scope = draft?.relation === "contains" ? collection?.definition.path : null;
  function complete(group: RuleGroupType, memberScope: string | null): boolean {
    return (
      group.rules.length > 0 &&
      group.rules.every((rule) => {
        if ("rules" in rule) return complete(rule, memberScope);
        const field = fields.find(
          (field) =>
            field.definition.id === rule.field &&
            field.definition.scope === memberScope,
        );
        if (rule.operator === "exists" && field?.definition.path === "senses")
          return isBuilderGroup(rule.value) && complete(rule.value, "senses");
        return field
          ? !isClearedOperand(rule, field.definition)
          : rule.field === BOOLEAN_CONDITION;
      })
    );
  }
  const exceedsLinkLimit = draft && draft.relation !== "contains" && linkLimitReached;
  let valid = false;
  if (draft && scope !== undefined && !exceedsLinkLimit) {
    if (draft.relation === "contains" && !draft.query.rules.length) valid = true;
    else if (complete(draft.query, scope)) {
      try {
        valid = !!builderPredicate(
          draft.query,
          fields.map((field) => field.definition),
          scope,
        );
      } catch {
        // The shared condition controls present the field-specific validation.
      }
    }
  }
  function apply() {
    if (!draft || !valid) return;
    const base = { ...rule, meta: { ...rule.meta, atlasExcluded: draft.excluded } };
    if (draft.relation === "contains" && collection) {
      onApply({
        ...base,
        field: collection.definition.id,
        operator: "exists",
        value: draft.query,
      });
    } else if (draft.relation !== "contains") {
      onApply({
        ...base,
        field: RELATED_CONDITION,
        operator: draft.relation,
        value: { query: draft.query, includeContents: draft.includeContents },
      });
    }
    setOpen(false);
  }

  return (
    <div className="filter-rule-value">
      <Button
        className="filter-related-action"
        ref={opener}
        disabled={disabled}
        onClick={begin}
      >
        {configured ? "Edit related entries" : "Configure related entries"}
      </Button>
      <AtlasModal
        title="Related entries"
        width={640}
        open={open}
        onCancel={() => setOpen(false)}
        onOk={apply}
        okText="Apply"
        okButtonProps={{ disabled: !valid }}
        afterClose={() => opener.current?.focus({ preventScroll: true })}
        destroyOnHidden
      >
        {draft && (
          <div className="related-entry-editor">
            <div className="related-entry-editor__choices">
              <RelatedEntrySelect
                label="Relationship"
                value={draft.relation}
                options={[
                  { value: "contains", label: "Contains" },
                  { value: "incoming", label: "Links to" },
                  { value: "outgoing", label: "Linked from" },
                ]}
                onChange={(relation: Draft["relation"]) => {
                  const changedScope =
                    (draft.relation === "contains") !== (relation === "contains");
                  setDraft({
                    ...draft,
                    relation,
                    ...(changedScope ? { query: emptyGroup() } : {}),
                  });
                }}
              />
              {draft.relation === "contains" && (
                <RelatedEntrySelect
                  label="Entry type"
                  placeholder="Choose a type…"
                  value={draft.collection}
                  showSearch={{ optionFilterProp: "label" }}
                  options={collections.map((field) => ({
                    value: field.definition.id,
                    label: field.label,
                  }))}
                  onChange={(collection) =>
                    setDraft({ ...draft, collection, query: emptyGroup() })
                  }
                />
              )}
              {draft.relation !== "contains" && (
                <RelatedEntrySelect
                  label="Link scope"
                  value={draft.includeContents ? "whole" : "main"}
                  options={[
                    { value: "whole", label: "Whole entry" },
                    { value: "main", label: "Main entry only" },
                  ]}
                  onChange={(value) =>
                    setDraft({ ...draft, includeContents: value === "whole" })
                  }
                />
              )}
            </div>
            {draft.relation !== "contains" && (
              <Typography.Text type="secondary">
                {draft.relation === "incoming"
                  ? "Check links in each search result"
                  : "Check links in the related entry"}
                {draft.includeContents ? " and its contents." : " itself."}
              </Typography.Text>
            )}
            {scope !== undefined && (
              <>
                <div>
                  <Typography.Text strong>Related entry filters</Typography.Text>
                  <br />
                  <Typography.Text type="secondary">
                    {filterDescription(draft, scope)}
                  </Typography.Text>
                </div>
                {renderConditions(
                  draft.query,
                  (query) => setDraft({ ...draft, query }),
                  scope,
                )}
              </>
            )}
            {exceedsLinkLimit && (
              <Typography.Text role="status">
                Remove a link condition before adding another.
              </Typography.Text>
            )}
            <Checkbox
              checked={draft.excluded}
              onChange={(event) =>
                setDraft({ ...draft, excluded: event.target.checked })
              }
            >
              Exclude matching records
            </Checkbox>
          </div>
        )}
      </AtlasModal>
    </div>
  );
}
