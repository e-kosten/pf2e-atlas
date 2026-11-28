import type { MouseEvent } from "react";
import { Button } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import type { RuleGroupType } from "react-querybuilder";
import type {
  FilterEditorFieldView,
  QueryFieldDefinition,
  QueryPredicate,
} from "../../generated/atlas";
import { sentenceCase } from "../ui/labels";
import { QuickSingleChoicePicker } from "./FilterChoicePicker";
import {
  choiceClauses,
  conditionValues,
  isExcluded,
  type FilterPanelState,
} from "./filterControls";
import { filterValueLabel } from "./filterValueLabel";
import { isClearedOperand } from "./queryBuilder";

export function hasDraftRefinements(
  query: RuleGroupType,
  kind: QueryFieldDefinition,
): boolean {
  return query.rules.some((rule) =>
    "rules" in rule
      ? !rule.rules.length || hasDraftRefinements(rule, kind)
      : rule.field !== kind.id || isClearedOperand(rule, kind),
  );
}

export function SearchQuickBooleanFilter({
  field,
  clause,
  applied,
  disabled,
  editsPending,
  workspace,
  selectionContainer,
  onChange,
}: {
  field: FilterEditorFieldView;
  clause: QueryPredicate | null;
  applied: boolean;
  disabled: boolean;
  editsPending: boolean;
  workspace: FilterPanelState;
  selectionContainer?: HTMLElement | null;
  onChange: (predicate: QueryPredicate | null) => void;
}) {
  const value = clause?.kind === "compare" ? String(clause.value) : null;
  return (
    <QuickSingleChoicePicker
      field={field}
      scope={workspace.vocabularyScope ?? { kind: "artifact" }}
      countContext={workspace.filterDiscoveryContext}
      countsEnabled={workspace.filterCountsEnabled !== false}
      countLimit={workspace.filterEditor?.limits.literal_list}
      countQueryKey={workspace.filterCountQueryKey}
      value={value}
      clauseId={clause?.clause_id ?? null}
      applied={applied}
      disabled={disabled}
      editsPending={editsPending}
      selectionContainer={selectionContainer}
      onChange={(next) => {
        if (next === null) {
          onChange(null);
          return;
        }
        onChange({
          kind: "compare",
          clause_id: clause?.clause_id ?? crypto.randomUUID(),
          field: field.definition.path,
          op: "eq",
          value: next === "true",
        });
      }}
    />
  );
}

/** A demoted control's committed condition remains visible beside current chips. */
export function RetainedQuickFilterChip({
  field,
  clause,
  rangeLabel,
  disabled,
  onRemove,
}: {
  field: FilterEditorFieldView;
  clause: QueryPredicate;
  rangeLabel?: string;
  disabled: boolean;
  onRemove: (event: MouseEvent<HTMLElement>) => void;
}) {
  const label =
    rangeLabel ??
    choiceClauses(clause)
      .flatMap((part) =>
        conditionValues(part).map(
          (value) =>
            (isExcluded(part) ? "− " : "") +
            sentenceCase(filterValueLabel(field, value)),
        ),
      )
      .join(", ");
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
