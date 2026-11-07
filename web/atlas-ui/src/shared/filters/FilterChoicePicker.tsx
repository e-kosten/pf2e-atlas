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
import { ChoicePicker, type ChoiceState } from "../ui/pickers/ChoicePicker";
import { ReferenceHelpLink, ReferenceHelpProvider } from "../records/ReferenceHelp";
import { useRecordPreview } from "../records/useRecordPreview";
import { PickerReference } from "../records/PickerReference";
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
  selectionContainer?: HTMLElement | null;
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
  selectionContainer,
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
  const [fullCatalogKey, setFullCatalogKey] = useState<string | null>(null);
  const closedChoices = field.short_choices;
  const isActions = field.definition.id === "actions";
  const labelScope = JSON.stringify([scope, field.definition.id]);
  const [retained, setRetained] = useState<{
    scope: string;
    labels: ReadonlyMap<string, string>;
  }>();
  const retainedLabels = retained?.scope === labelScope ? retained.labels : undefined;
  const unresolvedActions = isActions
    ? selected.filter((value) => !retainedLabels?.has(value))
    : [];
  // Contextual suggestions may omit a saved action. Resolve its label separately.
  const actionLabels = useFilterVocabulary(
    scope,
    field.definition.id,
    "",
    0,
    unresolvedActions,
    unresolvedActions.length > 0,
  );
  const loadSelectedLabels =
    !isActions && field.allow_custom_values === false && selected.length > 0;
  const allowNarrowing = edit.kind === "condition" || edit.allowNarrowing;
  const narrowingContext = countsEnabled && allowNarrowing ? countContext : undefined;
  const clauseId =
    edit.kind === "operand"
      ? edit.clauseId
      : (findClauseIds(
          countContext?.filter ?? null,
          field.definition.path,
          field.definition.scope,
        )[0] ?? null);
  const narrowingKey = narrowingContext
    ? JSON.stringify(
        [scope, field.definition.id, narrowingContext, clauseId],
        (_, value) => (typeof value === "bigint" ? value.toString() : value),
      )
    : null;
  const showAllValues = narrowingKey !== null && fullCatalogKey === narrowingKey;
  const query = useFilterVocabulary(
    scope,
    field.definition.id,
    search,
    offset,
    selected,
    (open || loadSelectedLabels) && !closedChoices,
    open && narrowingContext && !showAllValues
      ? { context: narrowingContext, clauseId }
      : undefined,
  );
  useEffect(() => {
    const timeout = window.setTimeout(() => setSearch(text), 250);
    return () => window.clearTimeout(timeout);
  }, [text]);

  const titleCaseDefense =
    field.definition.path === "type" &&
    ["defenses.immunities", "defenses.weaknesses", "defenses.resistances"].includes(
      field.definition.scope ?? "",
    );
  const displayLabel = (label: string) =>
    titleCaseDefense
      ? label.replace(
          /(^|[\s-])(\p{Ll})/gu,
          (_, prefix: string, letter: string) =>
            prefix + letter.toLocaleUpperCase("en"),
        )
      : label;
  const choices = (closedChoices ? field.choices : (query.data?.values.options ?? []))
    // An empty string clears an Advanced operand, so it cannot be a choice.
    .filter(
      (option) =>
        edit.kind !== "operand" ||
        typeof option.value !== "string" ||
        option.value.trim() !== "",
    )
    // A stale action ID can be returned as a fallback label without a resolved record.
    .filter((option) => !isActions || option.help?.destination.kind === "record")
    .map((option) => ({
      value: String(option.value),
      label: displayLabel(option.label),
      help: option.help,
    }));
  // A vocabulary page can disappear when the picker changes search or count scope.
  // Keep the resolved labels of saved selections independently of that page.
  if (unresolvedActions.length) {
    const learned = [
      ...(actionLabels.data?.values.options ?? []),
      ...(query.isPlaceholderData ? [] : choices),
    ].filter(
      ({ value, help }) =>
        help?.destination.kind === "record" &&
        typeof value === "string" &&
        unresolvedActions.includes(value),
    );
    if (learned.length) {
      setRetained({
        scope: labelScope,
        labels: new Map([
          ...(retainedLabels ?? []),
          ...learned.map(({ value, label }) => [String(value), label] as const),
        ]),
      });
    }
  }
  function labelForValue(value: string) {
    return (
      choices.find((option) => option.value === value)?.label ??
      retainedLabels?.get(value) ??
      (isActions
        ? actionLabels.isPending || actionLabels.isFetching
          ? "Loading action…"
          : "Unavailable action"
        : displayLabel(sentenceCase(filterValueLabel(field, value))))
    );
  }
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
  // Contextual pages also carry all selected options, including those outside
  // the text search. The Advanced operand already displays these in its field.
  const searchTerm = search.trim().toLowerCase();
  const offSearch =
    edit.kind === "operand" && query.data?.contextual && searchTerm
      ? choices.filter(
          ({ value, label }) =>
            selected.includes(value) &&
            !label.toLowerCase().includes(searchTerm) &&
            !value.toLowerCase().includes(searchTerm),
        )
      : [];
  const displayedChoices = offSearch.length
    ? choices.filter((choice) => !offSearch.includes(choice))
    : choices;
  const total = query.data?.values.total_values ?? 0n;
  const displayedTotal = total - BigInt(offSearch.length);
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
    field.allow_custom_values !== false &&
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
    const option =
      next !== "neutral" && (isActions || edit.kind === "operand")
        ? choices.find((choice) => choice.value === value)
        : undefined;
    if (option && retainedLabels?.get(value) !== option.label) {
      setRetained({
        scope: labelScope,
        labels: new Map(retainedLabels).set(value, option.label),
      });
    }
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

  const preview = useRecordPreview();
  return (
    <ReferenceHelpProvider onHelp={preview.openHelp} onReference={preview.open}>
      <ChoicePicker
        reading={
          preview.destination
            ? {
                origin: preview.anchor,
                onDismiss: preview.closeForNavigation,
                onBack: preview.readingHistory.canGoBack
                  ? preview.readingHistory.onBack
                  : preview.close,
                content: <PickerReference preview={preview} label={label} />,
              }
            : undefined
        }
        optionAction={
          !closedChoices || choices.some((choice) => choice.help?.destination)
            ? (option) => {
                const help = choices.find(
                  (choice) => choice.value === option.value,
                )?.help;
                return help?.destination ? (
                  <ReferenceHelpLink
                    help={help}
                    generation={countContext?.expected_artifact_generation ?? undefined}
                  >
                    Read<span className="sr-only"> {option.label}</span>
                  </ReferenceHelpLink>
                ) : null;
              }
            : undefined
        }
        selectionContainer={selectionContainer}
        labelForValue={labelForValue}
        id={field.definition.id}
        label={label}
        choices={displayedChoices}
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
                triggerText:
                  edit.shape === "scalar" && selected.length
                    ? labelForValue(selected[0])
                    : selected.length
                      ? "Add value"
                      : field.definition.path === "consumable.damage.type"
                        ? "Choose type…"
                        : "Choose " + label.toLowerCase() + "…",
                scalar: edit.shape === "scalar",
                enterValue: enterValue ?? undefined,
                manualValue: settled ? (exact ?? undefined) : undefined,
                onClear: () => edit.onChange(edit.shape === "scalar" ? "" : []),
              }
            : undefined
        }
      >
        {!closedChoices && (
          <div className="common-choice-footer">
            {(showAllValues || (query.data?.contextual && !query.error)) && (
              <Checkbox
                checked={showAllValues}
                onChange={(event) => {
                  setFullCatalogKey(event.target.checked ? narrowingKey : null);
                  setOffset(0);
                }}
              >
                Show all values
              </Checkbox>
            )}
            <div className="common-choice-progress">
              {query.data && (
                <Typography.Text type="secondary" role="status">
                  {new Intl.NumberFormat().format(displayedTotal)}{" "}
                  {displayedTotal === 1n ? "value" : "values"}
                  {scope.kind === "saved_list" ? " in this list" : ""}
                  {pages > 1n
                    ? " · Page " +
                      (query.data.offset / VOCABULARY_PAGE_SIZE + 1) +
                      " of " +
                      pages
                    : ""}
                </Typography.Text>
              )}
              <Spin
                size="small"
                spinning={query.isFetching}
                delay={500}
                aria-label="Loading values"
              />
            </div>
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
    </ReferenceHelpProvider>
  );
}
