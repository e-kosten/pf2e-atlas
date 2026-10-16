import { Alert, AutoComplete, Button, Input, Select, Space, Typography } from "antd";
import { useEffect, useState } from "react";
import type {
  FilterEditorFieldView,
  FilterVocabularyScope,
  FilterDiscoveryContext,
} from "../../generated/atlas";
import {
  useFilterOptionCounts,
  useFilterVocabulary,
  VOCABULARY_PAGE_SIZE,
} from "./useFilterDiscovery";

export function FilterValueInput({
  field,
  operator,
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
  const counts = useFilterOptionCounts(
    countContext,
    definition.id,
    clauseId,
    field.choices.map((choice) => choice.value),
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
  if (field.choices.length || definition.field_type === "boolean")
    return (
      <Select
        className="filter-value-control"
        aria-label={label}
        mode={multiple ? "multiple" : undefined}
        allowClear
        showSearch={{ optionFilterProp: "label" }}
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
        }))}
        onOpenChange={setOpen}
        optionRender={(option) => (
          <span>
            {option.label}
            {counts.has(String(option.value))
              ? ` (${new Intl.NumberFormat().format(counts.get(String(option.value))!)})`
              : ""}
          </span>
        )}
        onChange={(next) => onChange(next ?? (multiple ? [] : ""))}
      />
    );
  if (definition.field_type === "string" || definition.field_type === "set")
    return (
      <OpenVocabularyInput
        field={field}
        multiple={multiple}
        value={value}
        onChange={onChange}
        scope={scope}
        label={label}
        countContext={countContext}
        countsEnabled={countsEnabled}
        countQueryKey={countQueryKey}
        clauseId={clauseId}
        countLimit={countLimit}
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

function OpenVocabularyInput({
  field,
  multiple,
  value,
  onChange,
  scope,
  label,
  countContext,
  countsEnabled,
  countQueryKey,
  clauseId,
  countLimit,
}: {
  field: FilterEditorFieldView;
  multiple: boolean;
  value: unknown;
  onChange: (value: unknown) => void;
  scope: FilterVocabularyScope;
  label: string;
  countContext?: FilterDiscoveryContext;
  countsEnabled: boolean;
  countQueryKey?: readonly unknown[];
  clauseId: string | null;
  countLimit: number;
}) {
  const selected = Array.isArray(value)
    ? value.map(String)
    : value
      ? [String(value)]
      : [];
  const [text, setText] = useState("");
  const [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [open, setOpen] = useState(false);
  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setSearch(text);
    }, 250);
    return () => window.clearTimeout(timeout);
  }, [text]);
  function searchValues(next: string) {
    setText(next);
    setOffset(0);
  }
  const query = useFilterVocabulary(
    scope,
    field.definition.id,
    search,
    offset,
    selected,
  );
  const values = query.data?.values;
  const options = [
    ...new Set([
      ...selected,
      ...(values?.selected_values.map(String) ?? []),
      ...(values?.options.map((option) => String(option.value)) ?? []),
    ]),
  ].map((option) => ({ value: option, label: option }));
  const counts = useFilterOptionCounts(
    countContext,
    field.definition.id,
    clauseId,
    options.map((option) => option.value),
    open && countsEnabled && !!values && !query.isPlaceholderData,
    countLimit,
    countQueryKey,
  );
  const total = values?.total_values ?? 0n;
  const paging = (
    <div className="filter-vocabulary-footer">
      <Typography.Text type="secondary">
        {scope.kind === "saved_list"
          ? "Available values in this list"
          : "Available values in this artifact"}
      </Typography.Text>
      <Typography.Text type="secondary">Includes legacy entries.</Typography.Text>
      <Space size="small">
        <Button
          size="small"
          disabled={offset === 0 || query.isFetching}
          onClick={() => setOffset(Math.max(0, offset - VOCABULARY_PAGE_SIZE))}
        >
          Previous values
        </Button>
        <Button
          size="small"
          disabled={BigInt(offset + VOCABULARY_PAGE_SIZE) >= total || query.isFetching}
          onClick={() => setOffset(offset + VOCABULARY_PAGE_SIZE)}
        >
          More values
        </Button>
      </Space>
    </div>
  );
  return (
    <div className="filter-value-input">
      {multiple ? (
        <Select
          aria-label={label}
          mode="tags"
          allowClear
          value={selected}
          options={options}
          onOpenChange={setOpen}
          optionRender={(option) => (
            <span>
              {option.label}
              {counts.has(String(option.value))
                ? ` (${new Intl.NumberFormat().format(counts.get(String(option.value))!)})`
                : ""}
            </span>
          )}
          placeholder={"Search " + field.label.toLowerCase() + "…"}
          loading={query.isFetching}
          showSearch={{
            filterOption: false,
            searchValue: text,
            onSearch: searchValues,
          }}
          onSelect={() => searchValues("")}
          onClear={() => searchValues("")}
          onChange={onChange}
          popupRender={(menu) => (
            <>
              {menu}
              {paging}
            </>
          )}
        />
      ) : (
        <AutoComplete
          aria-label={label}
          value={String(value ?? "")}
          options={options.map((option) => ({
            ...option,
            label: (
              <span>
                {option.label}
                {counts.has(option.value)
                  ? ` (${new Intl.NumberFormat().format(counts.get(option.value)!)})`
                  : ""}
              </span>
            ),
          }))}
          onOpenChange={setOpen}
          placeholder="Search available values…"
          showSearch={{ filterOption: false, onSearch: searchValues }}
          onChange={onChange}
          popupRender={(menu) => (
            <>
              {menu}
              {paging}
            </>
          )}
        />
      )}
      {query.error && (
        <Alert
          type="error"
          title="Values could not be loaded"
          description={query.error.message}
          action={
            <Button size="small" onClick={() => void query.refetch()}>
              Retry values
            </Button>
          }
        />
      )}
    </div>
  );
}
