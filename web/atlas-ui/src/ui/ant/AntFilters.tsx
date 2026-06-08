import {
  Alert,
  Button,
  Checkbox,
  Collapse,
  Empty,
  Form,
  Input,
  InputNumber,
  Select,
  Tag,
  Tooltip,
} from "antd";
import { Minus, Plus, Search, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { FilterEditorFieldView } from "../../generated/atlas";
import type { MetricComparisonState } from "../../state/searchState";
import { SORT_OPTIONS } from "../../state/searchState";
import {
  addVisibleFilter,
  additionalFilterGroups,
  additionalVisibleFilterIds,
  booleanForField,
  clearAllFilters,
  clearFieldFilter,
  clearSelectedValueForField,
  controlKindForField,
  cycleSelectedValueForField,
  discoveredOptions,
  editorFieldForId,
  excludedValuesForField,
  hasActiveFieldFilter,
  hasActiveFilters,
  includeOperatorForField,
  labelForField,
  metricComparisonForField,
  rangeForField,
  removeVisibleFilter,
  setBooleanForField,
  setIncludeOperatorForField,
  setMetricComparisonForField,
  setRangeForField,
  valuesForField,
  valueFilterOperatorPolicy,
  visibleEditorFilterFields,
  type FilterSelectOption,
} from "../filterControls";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";

export function AntFilters({ workspace }: { workspace: AtlasWorkspaceState }) {
  const { search, setSearch } = workspace;
  const standardFilterFields = visibleEditorFilterFields(workspace);
  const optionalFilterIds = additionalVisibleFilterIds(workspace);
  const addFilterGroups = additionalFilterGroups(workspace);
  const activeFilters = hasActiveFilters(search);
  const textSearchActive =
    search.mode === "text_search" && search.query.trim().length > 0;

  return (
    <aside className="filter-panel">
      {workspace.errorMessage ? (
        <Alert showIcon type="error" message={workspace.errorMessage} />
      ) : null}
      <Form className="ant-filter-form" layout="vertical" size="middle">
        <Form.Item label="Search">
          <div className="filter-search-row">
            <Input.Search
              allowClear
              enterButton={<Search size={16} />}
              placeholder="Search records"
              value={search.query}
              onChange={(event) =>
                setSearch({
                  ...search,
                  query: event.target.value,
                  mode: event.target.value.trim() ? "text_search" : "browse",
                })
              }
              onSearch={(query) =>
                setSearch({
                  ...search,
                  query,
                  mode: query.trim() ? "text_search" : "browse",
                })
              }
            />
            <Tooltip title="Clear search and filters">
              <Button
                aria-label="Clear search and filters"
                icon={<X size={14} />}
                disabled={!activeFilters}
                onClick={() => setSearch(clearAllFilters(search))}
              />
            </Tooltip>
          </div>
        </Form.Item>
        <Collapse
          className="ant-filter-collapse"
          defaultActiveKey={["standard", "options"]}
          ghost
          items={[
            {
              key: "standard",
              label: "Standard filters",
              children: (
                <div className="filter-section">
                  {standardFilterFields.map((field) => (
                    <FilterFieldControl
                      key={field.id}
                      workspace={workspace}
                      field={field}
                    />
                  ))}
                </div>
              ),
            },
            {
              key: "more",
              label: `Added filters${
                optionalFilterIds.length > 0 ? ` (${optionalFilterIds.length})` : ""
              }`,
              children: (
                <div className="filter-section">
                  {optionalFilterIds.length === 0 ? (
                    <p className="filter-empty-note">No additional filters added.</p>
                  ) : (
                    optionalFilterIds.map((fieldId) => (
                      <Form.Item
                        key={fieldId}
                        label={
                          <div className="control-heading">
                            <span>{labelForField(workspace, fieldId)}</span>
                            <Button
                              size="small"
                              type="text"
                              onClick={() => removeVisibleFilter(workspace, fieldId)}
                            >
                              Remove
                            </Button>
                          </div>
                        }
                      >
                        <OptionalFilterControl
                          workspace={workspace}
                          fieldId={fieldId}
                        />
                      </Form.Item>
                    ))
                  )}
                  <Form.Item label="Add filter">
                    <Select
                      loading={workspace.filterDiscoveryLoading}
                      notFoundContent={
                        <Empty
                          description="No filters available"
                          image={Empty.PRESENTED_IMAGE_SIMPLE}
                        />
                      }
                      options={addFilterGroups}
                      placeholder="Choose a filter"
                      value={null}
                      onChange={(fieldId) => addVisibleFilter(workspace, fieldId)}
                    />
                  </Form.Item>
                </div>
              ),
            },
            {
              key: "options",
              label: "Result options",
              children: (
                <div className="control-row">
                  {textSearchActive ? null : (
                    <Form.Item label="Sort">
                      <Select
                        options={SORT_OPTIONS}
                        value={search.sort}
                        onChange={(sort) => setSearch({ ...search, sort })}
                      />
                    </Form.Item>
                  )}
                  <Form.Item label="Page size">
                    <InputNumber
                      min={10}
                      max={100}
                      step={5}
                      value={search.pageSize}
                      onChange={(pageSize) =>
                        setSearch({ ...search, pageSize: pageSize ?? 25 })
                      }
                    />
                  </Form.Item>
                </div>
              ),
            },
          ]}
        />
      </Form>
    </aside>
  );
}

function FilterFieldControl({
  workspace,
  field,
}: {
  workspace: AtlasWorkspaceState;
  field: FilterEditorFieldView;
}) {
  const { search, setSearch } = workspace;
  const controlKind = controlKindForField(workspace, field.id);
  const fieldLabel = (
    <FilterFieldLabel workspace={workspace} fieldId={field.id} label={field.label} />
  );

  if (controlKind === "range") {
    const range = rangeForField(search, field.id);
    const minLabel = field.control.kind === "range" ? field.control.min_label : "Min";
    const maxLabel = field.control.kind === "range" ? field.control.max_label : "Max";
    return (
      <Form.Item label={fieldLabel}>
        <div className="control-row">
          <Form.Item label={`${minLabel} ${field.label.toLowerCase()}`}>
            <InputNumber
              min={field.control.kind === "range" ? field.control.min : undefined}
              max={field.control.kind === "range" ? field.control.max : undefined}
              step={field.control.kind === "range" ? field.control.step : undefined}
              value={range.min}
              onChange={(min) =>
                setSearch(
                  setRangeForField(search, field.id, {
                    ...range,
                    min: min ?? null,
                  }),
                )
              }
            />
          </Form.Item>
          <Form.Item label={`${maxLabel} ${field.label.toLowerCase()}`}>
            <InputNumber
              min={field.control.kind === "range" ? field.control.min : undefined}
              max={field.control.kind === "range" ? field.control.max : undefined}
              step={field.control.kind === "range" ? field.control.step : undefined}
              value={range.max}
              onChange={(max) =>
                setSearch(
                  setRangeForField(search, field.id, {
                    ...range,
                    max: max ?? null,
                  }),
                )
              }
            />
          </Form.Item>
        </div>
      </Form.Item>
    );
  }

  if (controlKind === "boolean") {
    return (
      <Form.Item label={fieldLabel}>
        <Select
          allowClear
          loading={workspace.filterDiscoveryLoading}
          options={discoveredOptions(workspace, field.id)}
          value={booleanForField(search, field.id)}
          onChange={(value) =>
            setSearch(setBooleanForField(search, field.id, value ?? null))
          }
        />
      </Form.Item>
    );
  }

  if (controlKind === "metric") {
    return <MetricFilterControl workspace={workspace} field={field} />;
  }

  return (
    <>
      <Form.Item label={fieldLabel}>
        <TriStateOptionFilter workspace={workspace} fieldId={field.id} />
      </Form.Item>
      {field.id === "traits" ? (
        <Checkbox
          checked={includeOperatorForField(search, field.id) === "include_any"}
          onChange={(event) =>
            setSearch(
              setIncludeOperatorForField(
                search,
                field.id,
                event.target.checked ? "include_any" : "include_all",
              ),
            )
          }
        >
          Match any selected trait
        </Checkbox>
      ) : null}
    </>
  );
}

function FilterFieldLabel({
  workspace,
  fieldId,
  label,
}: {
  workspace: AtlasWorkspaceState;
  fieldId: string;
  label: string;
}) {
  const active = hasActiveFieldFilter(workspace.search, fieldId);
  return (
    <div className="control-heading">
      <span>{label}</span>
      {active ? (
        <Button
          aria-label={`Clear ${label} filter`}
          icon={<X size={12} />}
          size="small"
          type="text"
          onClick={() =>
            workspace.setSearch(clearFieldFilter(workspace.search, fieldId))
          }
        >
          Clear
        </Button>
      ) : null}
    </div>
  );
}

function OptionalFilterControl({
  workspace,
  fieldId,
}: {
  workspace: AtlasWorkspaceState;
  fieldId: string;
}) {
  const { search, setSearch } = workspace;
  const controlKind = controlKindForField(workspace, fieldId);

  if (controlKind === "range") {
    const range = rangeForField(search, fieldId);
    const field = editorFieldForId(workspace, fieldId);
    return (
      <div className="control-row">
        <InputNumber
          placeholder="Min"
          min={field?.control.kind === "range" ? field.control.min : undefined}
          max={field?.control.kind === "range" ? field.control.max : undefined}
          step={field?.control.kind === "range" ? field.control.step : undefined}
          value={range.min}
          onChange={(min) =>
            setSearch(
              setRangeForField(search, fieldId, {
                ...range,
                min: min ?? null,
              }),
            )
          }
        />
        <InputNumber
          placeholder="Max"
          min={field?.control.kind === "range" ? field.control.min : undefined}
          max={field?.control.kind === "range" ? field.control.max : undefined}
          step={field?.control.kind === "range" ? field.control.step : undefined}
          value={range.max}
          onChange={(max) =>
            setSearch(
              setRangeForField(search, fieldId, {
                ...range,
                max: max ?? null,
              }),
            )
          }
        />
      </div>
    );
  }

  if (controlKind === "boolean") {
    return (
      <Select
        allowClear
        loading={workspace.filterDiscoveryLoading}
        options={discoveredOptions(workspace, fieldId)}
        value={booleanForField(search, fieldId)}
        onChange={(value) =>
          setSearch(setBooleanForField(search, fieldId, value ?? null))
        }
      />
    );
  }

  if (controlKind === "metric") {
    const field = editorFieldForId(workspace, fieldId);
    return field ? <MetricFilterControl workspace={workspace} field={field} /> : null;
  }

  return <TriStateOptionFilter workspace={workspace} fieldId={fieldId} />;
}

function TriStateOptionFilter({
  workspace,
  fieldId,
}: {
  workspace: AtlasWorkspaceState;
  fieldId: string;
}) {
  const { search, setSearch } = workspace;
  const rootRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const field = editorFieldForId(workspace, fieldId);
  const operatorPolicy = valueFilterOperatorPolicy(field);
  const options = discoveredOptions(workspace, fieldId);
  const [openedOptions, setOpenedOptions] = useState(options);
  const includedValues = valuesForField(search, fieldId);
  const excludedValues = excludedValuesForField(search, fieldId);
  const includedValueSet = new Set(includedValues);
  const excludedValueSet = new Set(excludedValues);
  const selectedValues = [...includedValues, ...excludedValues];
  const displayedOptions = open
    ? refreshedOptionSnapshot(openedOptions, options)
    : options;
  const filteredOptions = displayedOptions.filter((option) =>
    option.label.toLowerCase().includes(query.trim().toLowerCase()),
  );

  useEffect(() => {
    if (!open) {
      return;
    }
    function handleClick(event: MouseEvent) {
      if (event.target instanceof Node && !rootRef.current?.contains(event.target)) {
        setOpen(false);
        setQuery("");
      }
    }

    document.addEventListener("click", handleClick);
    return () => {
      document.removeEventListener("click", handleClick);
    };
  }, [open]);

  function setMenuOpen(nextOpen: boolean) {
    if (nextOpen) {
      setOpenedOptions(options);
    }
    setOpen(nextOpen);
    if (!nextOpen) {
      setQuery("");
    }
  }

  function stateIcon(state: "included" | "excluded" | "neutral") {
    if (state === "included") {
      return <Plus aria-hidden="true" size={12} strokeWidth={2.5} />;
    }
    if (state === "excluded") {
      return <Minus aria-hidden="true" size={12} strokeWidth={2.5} />;
    }
    return null;
  }

  function optionState(value: string): "included" | "excluded" | "neutral" {
    if (includedValueSet.has(value)) {
      return "included";
    }
    if (excludedValueSet.has(value)) {
      return "excluded";
    }
    return "neutral";
  }

  function optionLabel(value: string): string {
    return (
      displayedOptions.find((option) => option.value === value)?.label ??
      options.find((option) => option.value === value)?.label ??
      value
    );
  }

  const content = (
    <div className="tri-state-filter-menu">
      <Input
        allowClear
        placeholder="Search options"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
      />
      <div className="tri-state-filter-options">
        {filteredOptions.length > 0 ? (
          filteredOptions.map((option) => {
            const state = optionState(option.value);
            return (
              <button
                key={option.value}
                type="button"
                aria-label={option.label}
                className={`filter-option-row is-${state}`}
                disabled={option.disabled}
                onClick={(event) => {
                  event.stopPropagation();
                  setSearch(
                    cycleSelectedValueForField(
                      search,
                      fieldId,
                      option.value,
                      operatorPolicy,
                    ),
                  );
                }}
              >
                <span className="filter-option-state-marker">{stateIcon(state)}</span>
                <span>{option.label}</span>
              </button>
            );
          })
        ) : (
          <Empty
            description="No options"
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            className="tri-state-filter-empty"
          />
        )}
      </div>
    </div>
  );

  return (
    <div className="tri-state-filter-root" ref={rootRef}>
      <div
        className={`tri-state-filter-trigger ${open ? "is-open" : ""}`}
        role="button"
        tabIndex={0}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={`Edit ${labelForField(workspace, fieldId)} filter`}
        onClick={() => setMenuOpen(true)}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            setMenuOpen(true);
          }
          if (event.key === "Escape") {
            setMenuOpen(false);
          }
        }}
      >
        {selectedValues.length > 0 ? (
          selectedValues.map((value) => {
            const excluded = excludedValueSet.has(value);
            return (
              <Tag
                key={value}
                className={`filter-value-tag ${
                  excluded ? "is-excluded" : "is-included"
                }`}
              >
                <span className="filter-value-tag-marker">
                  {stateIcon(excluded ? "excluded" : "included")}
                </span>
                <span className="filter-value-tag-label">{optionLabel(value)}</span>
                <button
                  type="button"
                  className="filter-value-tag-remove"
                  aria-label={`Remove ${optionLabel(value)} filter`}
                  onClick={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    setSearch(clearSelectedValueForField(search, fieldId, value));
                  }}
                >
                  <X aria-hidden="true" size={12} strokeWidth={2.25} />
                </button>
              </Tag>
            );
          })
        ) : (
          <span className="tri-state-filter-placeholder">
            {workspace.filterDiscoveryLoading ? "Loading options" : "Select options"}
          </span>
        )}
      </div>
      {open ? content : null}
    </div>
  );
}

function refreshedOptionSnapshot(
  openedOptions: FilterSelectOption[],
  currentOptions: FilterSelectOption[],
): FilterSelectOption[] {
  if (openedOptions.length === 0) {
    return currentOptions;
  }
  const currentByValue = new Map(
    currentOptions.map((option) => [option.value, option] as const),
  );
  return openedOptions.map((option) => currentByValue.get(option.value) ?? option);
}

function MetricFilterControl({
  workspace,
  field,
}: {
  workspace: AtlasWorkspaceState;
  field: FilterEditorFieldView;
}) {
  const { search, setSearch } = workspace;
  const current = metricComparisonForField(search, field.id);
  const [draft, setDraft] = useState<MetricComparisonState>(current);
  const keyLabel =
    field.control.kind === "metric_comparison" ? field.control.key_label : "Metric";
  const operatorLabel =
    field.control.kind === "metric_comparison"
      ? field.control.operator_label
      : "Operator";
  const valueLabel =
    field.control.kind === "metric_comparison" ? field.control.value_label : "Value";

  function update(next: MetricComparisonState) {
    setDraft(next);
    setSearch(setMetricComparisonForField(search, field.id, next));
  }

  return (
    <div className="control-row metric-filter-control">
      <Form.Item label={keyLabel}>
        <Select
          showSearch
          allowClear
          loading={workspace.filterDiscoveryLoading}
          optionFilterProp="label"
          options={discoveredOptions(workspace, field.id)}
          value={draft.key}
          onChange={(key) => update({ ...draft, key: key ?? null })}
        />
      </Form.Item>
      <Form.Item label={operatorLabel}>
        <Select
          options={[
            { value: "gte", label: ">=" },
            { value: "lte", label: "<=" },
            { value: "gt", label: ">" },
            { value: "lt", label: "<" },
            { value: "eq", label: "=" },
          ]}
          value={draft.op}
          onChange={(op) => update({ ...draft, op })}
        />
      </Form.Item>
      <Form.Item label={valueLabel}>
        <InputNumber
          value={draft.value}
          onChange={(value) => update({ ...draft, value: value ?? null })}
        />
      </Form.Item>
    </div>
  );
}
