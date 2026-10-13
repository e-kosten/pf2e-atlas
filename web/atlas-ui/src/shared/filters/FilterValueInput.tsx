import { Alert, AutoComplete, Button, Input, Select, Space, Typography } from "antd";
import { useEffect, useState } from "react";
import type {
  FilterEditorFieldView,
  FilterVocabularyScope,
} from "../../generated/atlas";
import { useFilterVocabulary, VOCABULARY_PAGE_SIZE } from "./useFilterDiscovery";

export function FilterValueInput({
  field,
  operator,
  value,
  onChange,
  scope,
  label = field.label,
}: {
  field: FilterEditorFieldView;
  operator: string;
  value: unknown;
  onChange: (value: unknown) => void;
  scope: FilterVocabularyScope;
  label?: string;
}) {
  const definition = field.definition;
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
}: {
  field: FilterEditorFieldView;
  multiple: boolean;
  value: unknown;
  onChange: (value: unknown) => void;
  scope: FilterVocabularyScope;
  label: string;
}) {
  const selected = Array.isArray(value)
    ? value.map(String)
    : value
      ? [String(value)]
      : [];
  const [text, setText] = useState("");
  const [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0);
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
  const total = values?.total_values ?? 0n;
  const paging = (
    <div className="filter-vocabulary-footer">
      <Typography.Text type="secondary">
        {scope.kind === "saved_list"
          ? "Available values in this list"
          : "Available values in this artifact"}
      </Typography.Text>
      <Typography.Text type="secondary">
        Includes legacy entries; values do not have match counts.
      </Typography.Text>
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
          options={options}
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
