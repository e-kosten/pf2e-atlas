import { ErrorNotice } from "../feedback/ErrorNotice";
import { Button, Checkbox, Space, Spin, Typography } from "antd";
import { useEffect, useState } from "react";
import type {
  FilterEditorFieldView,
  FilterVocabularyScope,
  FilterDiscoveryContext,
  QueryPredicate,
} from "../../generated/atlas";
import { sentenceCase } from "../ui/labels";
import { ChoicePicker } from "../ui/pickers/ChoicePicker";
import {
  changeChoiceValue,
  choiceClauses,
  choiceValueState,
  conditionValues,
} from "./filterControls";
import {
  findClauseIds,
  useFilterOptionCounts,
  useFilterVocabulary,
  VOCABULARY_PAGE_SIZE,
} from "./useFilterDiscovery";

/** Backend-owned categorical fields adapt vocabulary and clauses to the shared picker. */
export function CommonChoicePicker({
  field,
  clause,
  scope,
  onChange,
  countContext,
  countsEnabled = true,
  countQueryKey,
  countLimit = 0,
}: {
  field: FilterEditorFieldView;
  clause: QueryPredicate | null;
  scope: FilterVocabularyScope;
  onChange: (next: QueryPredicate | null) => void;
  countContext?: FilterDiscoveryContext;
  countsEnabled?: boolean;
  countQueryKey?: readonly unknown[];
  countLimit?: number;
}) {
  const selected = choiceClauses(clause).flatMap(conditionValues);
  const [open, setOpen] = useState(false);
  const [text, setText] = useState("");
  const [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [showUnavailable, setShowUnavailable] = useState(false);
  const closedChoices = field.short_choices;
  const ids = findClauseIds(
    countContext?.filter ?? null,
    field.definition.path,
    field.definition.scope,
  );
  const query = useFilterVocabulary(
    scope,
    field.definition.id,
    search,
    offset,
    selected,
    open && !closedChoices,
    countContext && countsEnabled && !showUnavailable
      ? { context: countContext, clauseId: ids[0] ?? null }
      : undefined,
  );
  useEffect(() => {
    const timeout = window.setTimeout(() => setSearch(text), 250);
    return () => window.clearTimeout(timeout);
  }, [text]);
  const choices = closedChoices
    ? field.choices.map((choice) => ({
        value: String(choice.value),
        label: sentenceCase(choice.label),
      }))
    : (query.data?.values.options ?? []).map((option) => ({
        value: String(option.value),
        label: sentenceCase(String(option.value)),
      }));
  const counts = useFilterOptionCounts(
    countContext,
    field.definition.id,
    ids[0] ?? null,
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
    ],
    open && countsEnabled && (closedChoices || !!query.data),
    countLimit,
    countQueryKey,
  );
  return (
    <ChoicePicker
      labelForValue={sentenceCase}
      id={field.definition.id}
      label={field.label}
      choices={choices}
      counts={counts}
      selected={selected}
      stateOf={(value) => choiceValueState(clause, value)}
      onChange={(value, state) =>
        onChange(changeChoiceValue(field.definition, clause, value, state))
      }
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
      onTextChange={(next) => {
        setText(next);
        setOffset(0);
      }}
      emptyMessage={query.error && !closedChoices ? null : "No matching values"}
      filterChoices={closedChoices}
      loading={!closedChoices && query.isLoading}
    >
      {!closedChoices && (
        <div className="common-choice-footer">
          <Checkbox
            checked={showUnavailable}
            onChange={(event) => {
              setShowUnavailable(event.target.checked);
              setOffset(0);
            }}
          >
            Show unavailable
          </Checkbox>
          {query.isLoading && <Spin size="small" />}
          {query.data && (
            <Typography.Text type="secondary" role="status">
              {new Intl.NumberFormat().format(query.data.values.total_values)} values
              {scope.kind === "saved_list" ? " in this list" : ""}
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
              disabled={
                BigInt(offset + VOCABULARY_PAGE_SIZE) >=
                  (query.data?.values.total_values ?? 0n) || query.isFetching
              }
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
