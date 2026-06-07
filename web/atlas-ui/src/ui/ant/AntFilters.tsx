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
} from "antd";
import { Search } from "lucide-react";
import type { FilterEditorFieldView } from "../../generated/atlas";
import { SORT_OPTIONS } from "../../state/searchState";
import {
  addVisibleFilter,
  additionalFilterGroups,
  additionalVisibleFilterIds,
  booleanForField,
  controlKindForField,
  discoveredOptions,
  editorFieldForId,
  labelForField,
  rangeForField,
  removeVisibleFilter,
  setBooleanForField,
  setRangeForField,
  setValuesForField,
  valuesForField,
  visibleEditorFilterFields,
} from "../filterControls";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";

export function AntFilters({ workspace }: { workspace: AtlasWorkspaceState }) {
  const { search, setSearch } = workspace;
  const standardFilterFields = visibleEditorFilterFields(workspace);
  const optionalFilterIds = additionalVisibleFilterIds(workspace);
  const addFilterGroups = additionalFilterGroups(workspace);

  return (
    <aside className="filter-panel">
      {workspace.errorMessage ? (
        <Alert showIcon type="error" message={workspace.errorMessage} />
      ) : null}
      <Form className="ant-filter-form" layout="vertical" size="middle">
        <Form.Item label="Search">
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
                  <Form.Item label="Sort">
                    <Select
                      options={SORT_OPTIONS}
                      value={search.sort}
                      onChange={(sort) => setSearch({ ...search, sort })}
                    />
                  </Form.Item>
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

  if (controlKind === "range") {
    const range = rangeForField(search, field.id);
    const minLabel = field.control.kind === "range" ? field.control.min_label : "Min";
    const maxLabel = field.control.kind === "range" ? field.control.max_label : "Max";
    return (
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
    );
  }

  if (controlKind === "boolean") {
    return (
      <Form.Item label={field.label}>
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

  return (
    <>
      <Form.Item label={field.label}>
        <Select
          mode="multiple"
          loading={workspace.filterDiscoveryLoading}
          options={discoveredOptions(workspace, field.id)}
          value={valuesForField(search, field.id)}
          onChange={(values) => setSearch(setValuesForField(search, field.id, values))}
        />
      </Form.Item>
      {field.id === "traits" ? (
        <>
          <Checkbox
            checked={search.traitOperator === "include_any"}
            onChange={(event) =>
              setSearch({
                ...search,
                traitOperator: event.target.checked ? "include_any" : "include_all",
              })
            }
          >
            Match any selected trait
          </Checkbox>
          <Form.Item label={`Excluded ${field.label.toLowerCase()}`}>
            <Select
              mode="multiple"
              loading={workspace.filterDiscoveryLoading}
              options={discoveredOptions(workspace, field.id)}
              value={search.excludedTraits}
              onChange={(excludedTraits) => setSearch({ ...search, excludedTraits })}
            />
          </Form.Item>
        </>
      ) : null}
    </>
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

  return (
    <Select
      mode="multiple"
      loading={workspace.filterDiscoveryLoading}
      options={discoveredOptions(workspace, fieldId)}
      value={valuesForField(search, fieldId)}
      onChange={(values) => setSearch(setValuesForField(search, fieldId, values))}
    />
  );
}
