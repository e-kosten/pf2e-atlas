import { Button, Checkbox, Modal, Select, Typography, type SelectProps } from "antd";
import { useId, useRef, useState, type ReactNode } from "react";
import type { RuleGroupType } from "react-querybuilder";
import type {
  FilterEditorFieldView,
  QueryRelationshipDirection,
} from "../../generated/atlas";
import { isBuilderGroup } from "./builderEdits";
import { filterValueLabel } from "./filterValueLabel";
import {
  BOOLEAN_CONDITION,
  LINK_CONDITION,
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
  mode: "any" | "filters" | "exact";
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

export function RelatedEntryEditor({
  rule,
  fields,
  disabled,
  linkLimitReached,
  onApply,
  onExact,
  renderConditions,
}: {
  rule: PredicateRule;
  fields: FilterEditorFieldView[];
  disabled?: boolean;
  linkLimitReached: boolean;
  onApply: (rule: PredicateRule) => void;
  onExact: (rule: PredicateRule, opener?: HTMLElement) => void;
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
  const exactLink = rule.field === LINK_CONDITION;
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState<Draft>();
  const opener = useRef<HTMLButtonElement>(null);
  const handoff = useRef<PredicateRule | undefined>(undefined);
  const exclusions = rule.meta?.atlasExcluded ?? false;
  const configured = !!applied || appliedLink || !!rule.value?.selection;
  function begin() {
    setDraft({
      relation:
        applied || (!appliedLink && !exactLink)
          ? "contains"
          : rule.operator === "outgoing"
            ? "outgoing"
            : "incoming",
      collection: applied?.definition.id,
      mode: exactLink
        ? "exact"
        : applied && presenceOnly(appliedGroup)
          ? "any"
          : "filters",
      query: appliedGroup,
      includeContents:
        (exactLink ? rule.value?.include_contents : rule.value?.includeContents) ??
        true,
      excluded: exclusions,
    });
    setOpen(true);
  }
  const collection = collections.find(
    (field) => field.definition.id === draft?.collection,
  );
  const scope = draft?.relation === "contains" ? collection?.definition.path : null;
  function complete(group: RuleGroupType): boolean {
    return (
      group.rules.length > 0 &&
      group.rules.every((rule) => {
        if ("rules" in rule) return complete(rule);
        const field = fields.find(
          (field) =>
            field.definition.id === rule.field && field.definition.scope === scope,
        );
        return field
          ? !isClearedOperand(rule, field.definition)
          : rule.field === BOOLEAN_CONDITION;
      })
    );
  }
  const exceedsLinkLimit = draft && draft.relation !== "contains" && linkLimitReached;
  let valid = false;
  if (draft && scope !== undefined && !exceedsLinkLimit) {
    if (draft.mode !== "filters") valid = true;
    else if (complete(draft.query)) {
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
        value: draft.mode === "any" ? emptyGroup() : draft.query,
      });
    } else if (draft.relation !== "contains") {
      if (draft.mode === "exact") {
        handoff.current = {
          ...base,
          field: LINK_CONDITION,
          operator: draft.relation,
          value: {
            include_contents: draft.includeContents,
            selection:
              exactLink &&
              rule.operator === draft.relation &&
              rule.value?.include_contents === draft.includeContents
                ? rule.value?.selection
                : undefined,
          },
        };
      } else {
        onApply({
          ...base,
          field: RELATED_CONDITION,
          operator: draft.relation,
          value: { query: draft.query, includeContents: draft.includeContents },
        });
      }
    }
    setOpen(false);
  }

  return (
    <div className="filter-rule-value">
      <Button ref={opener} disabled={disabled} onClick={begin}>
        {configured ? "Edit related entries" : "Configure related entries"}
      </Button>
      <Modal
        title="Related entries"
        width={640}
        open={open}
        onCancel={() => setOpen(false)}
        onOk={apply}
        okText={draft?.mode === "exact" ? "Choose record" : "Apply"}
        okButtonProps={{ disabled: !valid }}
        afterClose={() => {
          const pending = handoff.current;
          handoff.current = undefined;
          if (pending) onExact(pending, opener.current ?? undefined);
          else opener.current?.focus({ preventScroll: true });
        }}
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
                    ...(changedScope ? { query: emptyGroup(), mode: "filters" } : {}),
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
              {(collection || draft.relation !== "contains") && (
                <RelatedEntrySelect
                  label="Which related entries?"
                  value={draft.mode}
                  options={[
                    { value: "filters", label: "Entries matching filters" },
                    draft.relation === "contains"
                      ? { value: "any", label: "Any entry of this type" }
                      : { value: "exact", label: "A specific record" },
                  ]}
                  onChange={(mode: Draft["mode"]) => setDraft({ ...draft, mode })}
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
            {draft.mode === "filters" && scope !== undefined && (
              <>
                <div>
                  <Typography.Text strong>Related entry filters</Typography.Text>
                  <br />
                  <Typography.Text type="secondary">
                    Atlas checks the conditions against one related entry at a time.
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
      </Modal>
    </div>
  );
}
