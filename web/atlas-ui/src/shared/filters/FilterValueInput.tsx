import { Input, Select } from "antd";
import { useState } from "react";
import { AdvancedVocabularyPicker } from "./FilterChoicePicker";
import { actorMetric } from "./actorMetricFields";
import { ProficiencyRankInput } from "./ProficiencyRankInput";
import type {
  FilterEditorFieldView,
  FilterVocabularyScope,
  FilterDiscoveryContext,
} from "../../generated/atlas";
import { useFilterOptionCounts } from "./useFilterDiscovery";

export function FilterValueInput({
  field,
  operator,
  excluded = false,
  value,
  onChange,
  scope,
  label = field.label,
  countContext,
  countsEnabled = true,
  countQueryKey,
  clauseId = null,
  countLimit = 0,
}: {
  field: FilterEditorFieldView;
  operator: string;
  excluded?: boolean;
  value: unknown;
  onChange: (value: unknown) => void;
  scope: FilterVocabularyScope;
  label?: string;
  countContext?: FilterDiscoveryContext;
  countsEnabled?: boolean;
  countQueryKey?: readonly unknown[];
  clauseId?: string | null;
  countLimit?: number;
}) {
  const definition = field.definition;
  const [open, setOpen] = useState(false);
  const [choiceSearch, setChoiceSearch] = useState("");
  const selectedChoices = (
    Array.isArray(value) ? value : value == null || value === "" ? [] : [value]
  ).map(String);
  const visibleChoices = field.choices.filter(
    (choice) =>
      selectedChoices.includes(String(choice.value)) ||
      choice.label.toLowerCase().includes(choiceSearch.trim().toLowerCase()),
  );
  const counts = useFilterOptionCounts(
    countContext,
    definition.id,
    clauseId,
    visibleChoices.map((choice) => choice.value),
    open && countsEnabled,
    countLimit,
    countQueryKey,
  );
  const operand = field.operators.find((entry) => entry.operator === operator)?.operand;
  if (operand === "state")
    return (
      <Select
        className="filter-value-control"
        aria-label={label + " availability"}
        value={value || undefined}
        options={[
          { value: "value", label: "Has a known value" },
          { value: "missing", label: "Missing from source" },
          { value: "null", label: "Explicitly empty" },
          { value: "invalid", label: "Could not be parsed" },
          { value: "not_applicable", label: "Does not apply" },
        ]}
        onChange={onChange}
      />
    );
  if (actorMetric(definition.id)?.measure === "rank")
    return (
      <ProficiencyRankInput
        label={label}
        value={value}
        range={operand === "range"}
        onChange={onChange}
      />
    );
  if (operand === "range") {
    const range = Array.isArray(value) ? value : ["", ""];
    return (
      <div className="filter-range">
        <Input
          aria-label={label + " minimum"}
          name={definition.id + "-minimum"}
          inputMode="decimal"
          value={range[0] ?? ""}
          placeholder="Minimum…"
          onChange={(event) => onChange([event.target.value, range[1]])}
        />
        <span>to</span>
        <Input
          aria-label={label + " maximum"}
          name={definition.id + "-maximum"}
          inputMode="decimal"
          value={range[1] ?? ""}
          placeholder="Maximum…"
          onChange={(event) => onChange([range[0], event.target.value])}
        />
      </div>
    );
  }
  const multiple = operand === "list";
  if (field.short_choices)
    return (
      <Select
        className="filter-value-control"
        aria-label={label}
        placeholder={"Select " + label.toLowerCase() + "…"}
        mode={multiple ? "multiple" : undefined}
        allowClear
        showSearch={{ optionFilterProp: "label", onSearch: setChoiceSearch }}
        value={
          value === "" || value == null
            ? undefined
            : definition.field_type === "boolean"
              ? String(value)
              : value
        }
        options={field.choices.map((choice) => ({
          value: String(choice.value),
          label: choice.label,
          disabled:
            !excluded &&
            counts.get(String(choice.value)) === 0n &&
            !selectedChoices.includes(String(choice.value)) &&
            !["neq", "excludes_any"].includes(operator),
        }))}
        optionRender={(option) => {
          return (
            <span>
              {option.label}
              {counts.has(String(option.value))
                ? ` (${new Intl.NumberFormat().format(counts.get(String(option.value))!)})`
                : ""}
            </span>
          );
        }}
        onOpenChange={setOpen}
        onChange={(next) => onChange(next ?? (multiple ? [] : ""))}
      />
    );
  if (definition.field_type === "string" || definition.field_type === "set")
    return (
      <AdvancedVocabularyPicker
        field={field}
        shape={multiple ? "list" : "scalar"}
        value={value}
        onChange={onChange}
        scope={scope}
        label={label}
        countContext={countContext}
        countsEnabled={countsEnabled}
        countQueryKey={countQueryKey}
        clauseId={clauseId}
        countLimit={countLimit}
        allowNarrowing={!excluded && !["neq", "excludes_any"].includes(operator)}
      />
    );
  return (
    <Input
      aria-label={label}
      name={definition.id}
      autoComplete="off"
      inputMode={definition.field_type === "number" ? "decimal" : undefined}
      placeholder={definition.units ? definition.units + "…" : "Value…"}
      value={String(value ?? "")}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}
