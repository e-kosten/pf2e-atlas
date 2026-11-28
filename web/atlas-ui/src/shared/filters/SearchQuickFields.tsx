import type { MouseEvent, ReactNode } from "react";
import { Button } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import type { RuleGroupType } from "react-querybuilder";
import type { FilterEditorFieldView, QueryPredicate } from "../../generated/atlas";
import { sentenceCase } from "../ui/labels";
import { QuickSingleChoicePicker } from "./FilterChoicePicker";
import {
  choiceClauses,
  collectionCategory,
  conditionValues,
  isExcluded,
  type FilterPanelState,
} from "./filterControls";
import { filterValueLabel } from "./filterValueLabel";

export function hasDraftRefinements(query: RuleGroupType): boolean {
  return query.rules.some((rule) =>
    "rules" in rule ? hasDraftRefinements(rule) : rule.field !== "kind",
  );
}

/** Boolean and collection operands keep their original typed predicates. */
export function SearchQuickSingleFilter({
  field,
  member,
  clause,
  applied,
  disabled,
  workspace,
  selectionContainer,
  onChange,
}: {
  field: FilterEditorFieldView;
  member?: FilterEditorFieldView;
  clause: QueryPredicate | null;
  applied: boolean;
  disabled: boolean;
  workspace: FilterPanelState;
  selectionContainer?: HTMLElement | null;
  onChange: (predicate: QueryPredicate | null) => void;
}) {
  const current = member ? collectionCategory(clause, member.definition.path) : clause;
  const value = current?.kind === "compare" ? String(current.value) : null;
  const pickerField = member ? { ...member, label: field.label } : field;
  return (
    <QuickSingleChoicePicker
      field={pickerField}
      scope={workspace.vocabularyScope ?? { kind: "artifact" }}
      countContext={workspace.filterDiscoveryContext}
      countsEnabled={workspace.filterCountsEnabled !== false}
      countLimit={workspace.filterEditor?.limits.literal_list}
      countQueryKey={workspace.filterCountQueryKey}
      value={value}
      clauseId={current?.clause_id ?? null}
      applied={applied}
      disabled={disabled}
      selectionContainer={selectionContainer}
      onChange={(next) => {
        if (next === null) {
          onChange(null);
          return;
        }
        const comparison: QueryPredicate = {
          kind: "compare",
          clause_id: current?.clause_id ?? crypto.randomUUID(),
          field: pickerField.definition.path,
          op: "eq",
          value: member ? next : next === "true",
        };
        onChange(
          member
            ? {
                kind: "exists",
                clause_id: clause?.clause_id ?? crypto.randomUUID(),
                collection: field.definition.path,
                scope_id:
                  clause?.kind === "exists" ? clause.scope_id : crypto.randomUUID(),
                predicate: comparison,
              }
            : comparison,
        );
      }}
    />
  );
}

/** A demoted control's committed condition remains visible beside current chips. */
export function RetainedQuickFilterChip({
  field,
  member,
  clause,
  rangeLabel,
  disabled,
  onRemove,
}: {
  field: FilterEditorFieldView;
  member?: FilterEditorFieldView;
  clause: QueryPredicate;
  rangeLabel?: string;
  disabled: boolean;
  onRemove: (event: MouseEvent<HTMLElement>) => void;
}) {
  let label: ReactNode = rangeLabel;
  if (member) {
    const category = collectionCategory(clause, member.definition.path);
    if (!category) return null;
    label = sentenceCase(filterValueLabel(member, String(category.value)));
  } else if (!rangeLabel) {
    label = choiceClauses(clause)
      .flatMap((part) =>
        conditionValues(part).map(
          (value) =>
            (isExcluded(part) ? "− " : "") +
            sentenceCase(filterValueLabel(field, value)),
        ),
      )
      .join(", ");
  }
  return (
    <Button
      className="applied-filter-chip"
      size="small"
      color="default"
      variant="filled"
      aria-label={`Remove ${field.label} filter`}
      disabled={disabled}
      onClick={onRemove}
    >
      {field.label}: {label}
      <CloseOutlined aria-hidden="true" />
    </Button>
  );
}
