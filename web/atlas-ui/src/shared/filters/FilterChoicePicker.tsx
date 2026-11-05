import { ErrorNotice } from "../feedback/ErrorNotice";
import { Button, Checkbox, Space, Typography } from "antd";
import { useEffect, useState } from "react";
import type {
  FilterEditorFieldView,
  FilterVocabularyScope,
  FilterDiscoveryContext,
  QueryPredicate,
} from "../../generated/atlas";
import { sentenceCase } from "../ui/labels";
import { ChoicePicker, type ChoiceState } from "../ui/pickers/ChoicePicker";
import {
  changeChoiceValue,
  choiceClauses,
  choiceValueState,
  conditionValues,
} from "./filterControls";
import { filterValueLabel } from "./filterValueLabel";
import {
  findClauseIds,
  useFilterOptionCounts,
  useFilterVocabulary,
  VOCABULARY_PAGE_SIZE,
} from "./useFilterDiscovery";

type DiscoveryProps = {
  field: FilterEditorFieldView;
  scope: FilterVocabularyScope;
  countContext?: FilterDiscoveryContext;
  countsEnabled?: boolean;
  countQueryKey?: readonly unknown[];
  countLimit?: number;
};

type ChoiceEdit =
  | {
      kind: "condition";
      clause: QueryPredicate | null;
      onChange: (next: QueryPredicate | null) => void;
    }
  | {
      kind: "operand";
      shape: "list" | "scalar";
      value: unknown;
      clauseId: string | null;
      allowNarrowing: boolean;
      onChange: (next: unknown) => void;
    };

export function CommonChoicePicker({
  clause,
  onChange,
  ...discovery
}: DiscoveryProps & Omit<Extract<ChoiceEdit, { kind: "condition" }>, "kind">) {
  return (
    <FilterChoicePicker {...discovery} edit={{ kind: "condition", clause, onChange }} />
  );
}

export function AdvancedVocabularyPicker({
  shape,
  value,
  clauseId,
  allowNarrowing,
  onChange,
  label,
  ...discovery
}: DiscoveryProps &
  Omit<Extract<ChoiceEdit, { kind: "operand" }>, "kind"> & { label: string }) {
  return (
    <FilterChoicePicker
      {...discovery}
      label={label}
      edit={{ kind: "operand", shape, value, clauseId, allowNarrowing, onChange }}
    />
  );
}

function FilterChoicePicker({
  field,
  label = field.label,
  scope,
  edit,
  countContext,
  countsEnabled = true,
  countQueryKey,
  countLimit = 0,
}: DiscoveryProps & { label?: string; edit: ChoiceEdit }) {
  const selected =
    edit.kind === "condition"
      ? choiceClauses(edit.clause).flatMap(conditionValues)
      : (Array.isArray(edit.value) ? edit.value : edit.value ? [edit.value] : []).map(
          String,
        );
  const [open, setOpen] = useState(false);
  const [text, setText] = useState("");
  const [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [showUnavailable, setShowUnavailable] = useState(false);
  const closedChoices = field.short_choices;
  const allowNarrowing = edit.kind === "condition" || edit.allowNarrowing;
  const clauseId =
    edit.kind === "operand"
      ? edit.clauseId
      : (findClauseIds(
          countContext?.filter ?? null,
          field.definition.path,
          field.definition.scope,
        )[0] ?? null);
  const query = useFilterVocabulary(
    scope,
    field.definition.id,
    search,
    offset,
    selected,
    open && !closedChoices,
    countContext && countsEnabled && allowNarrowing && !showUnavailable
      ? { context: countContext, clauseId }
      : undefined,
  );
  useEffect(() => {
    const timeout = window.setTimeout(() => setSearch(text), 250);
    return () => window.clearTimeout(timeout);
  }, [text]);

  function labelForValue(value: string) {
    return sentenceCase(filterValueLabel(field, value));
  }
  const choices = (
    closedChoices ? field.choices : (query.data?.values.options ?? [])
  ).map((option) => ({
    value: String(option.value),
    label: labelForValue(String(option.value)),
  }));
  const contextualCounts =
    countsEnabled && !query.error ? query.data?.counts : undefined;
  const extraCounts = useFilterOptionCounts(
    countContext,
    field.definition.id,
    clauseId,
    [
      ...new Set([
        ...selected,
        ...choices
          .filter(
            (choice) =>
              !closedChoices ||
              choice.label.toLowerCase().includes(text.trim().toLowerCase()),
          )
          .map((choice) => choice.value),
      ]),
    ].filter((value) => !contextualCounts?.has(value)),
    open && countsEnabled && (closedChoices || !!query.data),
    countLimit,
    countQueryKey,
  );
  const counts = contextualCounts
    ? new Map([...contextualCounts, ...extraCounts])
    : extraCounts;
  const total = query.data?.values.total_values ?? 0n;
  const pageSize = BigInt(VOCABULARY_PAGE_SIZE);
  const pages = (total + pageSize - 1n) / pageSize;
  const literal = text.trim();
  const matchingValue =
    literal &&
    choices.find(
      (choice) =>
        choice.value === literal ||
        choice.label.toLowerCase() === literal.toLowerCase(),
    );
  const exact =
    edit.kind === "operand" &&
    !field.definition.choices.length &&
    literal &&
    !matchingValue
      ? literal
      : null;

  const settled = text === search && !query.isFetching;
  const enterValue = matchingValue
    ? matchingValue.value
    : field.definition.path !== "physical.base_type" || settled
      ? exact
      : null;

  function searchValues(next: string) {
    setText(next);
    setOffset(0);
  }
  function activate(value: string, next: ChoiceState): void | "close" {
    if (edit.kind === "condition") {
      edit.onChange(changeChoiceValue(field.definition, edit.clause, value, next));
    } else if (edit.shape === "scalar") {
      edit.onChange(next === "neutral" ? "" : value);
      if (next !== "neutral") return "close";
    } else {
      edit.onChange(
        next === "neutral"
          ? selected.filter((entry) => entry !== value)
          : [...new Set([...selected, value])],
      );
      if (next !== "neutral") searchValues("");
    }
  }

  return (
    <ChoicePicker
      labelForValue={labelForValue}
      id={field.definition.id}
      label={label}
      choices={choices}
      counts={counts}
      selected={selected}
      stateOf={(value) =>
        edit.kind === "condition"
          ? choiceValueState(edit.clause, value)
          : selected.includes(value)
            ? "include"
            : "neutral"
      }
      onChange={activate}
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) {
          setText("");
          setSearch("");
          setOffset(0);
        }
      }}
      text={text}
      onTextChange={searchValues}
      emptyMessage={query.error && !closedChoices ? null : "No matching values"}
      filterChoices={closedChoices}
      loading={!closedChoices && query.isFetching}
      pageKey={query.data?.pageKey}
      valueEditing={
        edit.kind === "operand"
          ? {
              triggerText: selected.length ? "Edit values" : "Choose a value…",
              enterValue: enterValue ?? undefined,
              manualValue: settled ? (exact ?? undefined) : undefined,
            }
          : undefined
      }
    >
      {!closedChoices && (
        <div className="common-choice-footer">
          {allowNarrowing && (
            <Checkbox
              checked={showUnavailable}
              onChange={(event) => {
                setShowUnavailable(event.target.checked);
                setOffset(0);
              }}
            >
              Show unavailable
            </Checkbox>
          )}
          {query.data && (
            <Typography.Text type="secondary" role="status">
              {query.isFetching ? (
                "Loading values…"
              ) : (
                <>
                  {new Intl.NumberFormat().format(total)} values
                  {scope.kind === "saved_list" ? " in this list" : ""}
                  {pages > 1n
                    ? " · Page " + (offset / VOCABULARY_PAGE_SIZE + 1) + " of " + pages
                    : ""}
                </>
              )}
            </Typography.Text>
          )}
          <Space size="small">
            <Button
              size="small"
              aria-label="Previous values"
              disabled={offset === 0 || query.isFetching}
              onClick={() => setOffset(Math.max(0, offset - VOCABULARY_PAGE_SIZE))}
            >
              Previous
            </Button>
            <Button
              size="small"
              aria-label="More values"
              disabled={BigInt(offset) + pageSize >= total || query.isFetching}
              onClick={() => setOffset(offset + VOCABULARY_PAGE_SIZE)}
            >
              Next
            </Button>
          </Space>
        </div>
      )}
      {query.error && !closedChoices && (
        <ErrorNotice
          error={query.error}
          compact
          title="Values could not be loaded"
          action={
            <Button size="small" onClick={() => void query.refetch()}>
              Retry values
            </Button>
          }
        />
      )}
    </ChoicePicker>
  );
}
