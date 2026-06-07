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
import {
  KIND_OPTIONS,
  RARITY_OPTIONS,
  SORT_OPTIONS,
  TRAIT_OPTIONS,
} from "../../state/searchState";
import {
  addVisibleFilter,
  additionalFilterGroups,
  booleanForField,
  controlKindForField,
  discoveredOptions,
  labelForField,
  rangeForField,
  removeVisibleFilter,
  setBooleanForField,
  setRangeForField,
  setValuesForField,
  valuesForField,
} from "../filterControls";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";

export function AntFilters({ workspace }: { workspace: AtlasWorkspaceState }) {
  const { search, setSearch } = workspace;
  const optionalFilterIds = search.visibleFilterIds.filter(
    (fieldId) => !["kind", "rarity", "traits", "level"].includes(fieldId),
  );
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
          defaultActiveKey={["standard", "more", "options"]}
          ghost
          items={[
            {
              key: "standard",
              label: "Standard filters",
              children: (
                <div className="filter-section">
                  <Form.Item label="Kinds">
                    <Select
                      mode="multiple"
                      options={discoveredOptions(workspace, "kind", KIND_OPTIONS)}
                      value={search.kinds}
                      onChange={(kinds) => setSearch({ ...search, kinds })}
                    />
                  </Form.Item>
                  <Form.Item label="Rarity">
                    <Select
                      mode="multiple"
                      options={discoveredOptions(workspace, "rarity", RARITY_OPTIONS)}
                      value={search.rarity}
                      onChange={(rarity) => setSearch({ ...search, rarity })}
                    />
                  </Form.Item>
                  <Form.Item label="Traits">
                    <Select
                      mode="multiple"
                      options={discoveredOptions(workspace, "traits", TRAIT_OPTIONS)}
                      value={search.traits}
                      onChange={(traits) => setSearch({ ...search, traits })}
                    />
                  </Form.Item>
                  <Checkbox
                    checked={search.traitOperator === "include_any"}
                    onChange={(event) =>
                      setSearch({
                        ...search,
                        traitOperator: event.target.checked
                          ? "include_any"
                          : "include_all",
                      })
                    }
                  >
                    Match any selected trait
                  </Checkbox>
                  <Form.Item label="Excluded traits">
                    <Select
                      mode="multiple"
                      options={discoveredOptions(workspace, "traits", TRAIT_OPTIONS)}
                      value={search.excludedTraits}
                      onChange={(excludedTraits) =>
                        setSearch({ ...search, excludedTraits })
                      }
                    />
                  </Form.Item>
                  <div className="control-row">
                    <Form.Item label="Min level">
                      <InputNumber
                        min={0}
                        max={30}
                        value={search.levelMin}
                        onChange={(levelMin) =>
                          setSearch({ ...search, levelMin: levelMin ?? null })
                        }
                      />
                    </Form.Item>
                    <Form.Item label="Max level">
                      <InputNumber
                        min={0}
                        max={30}
                        value={search.levelMax}
                        onChange={(levelMax) =>
                          setSearch({ ...search, levelMax: levelMax ?? null })
                        }
                      />
                    </Form.Item>
                  </div>
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
                    <Empty
                      description="No additional filters"
                      image={Empty.PRESENTED_IMAGE_SIMPLE}
                    />
                  ) : (
                    optionalFilterIds.map((fieldId) => (
                      <Form.Item
                        key={fieldId}
                        label={
                          <div className="control-heading">
                            <span>
                              {labelForField(workspace.filterFields?.fields, fieldId)}
                            </span>
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

function OptionalFilterControl({
  workspace,
  fieldId,
}: {
  workspace: AtlasWorkspaceState;
  fieldId: string;
}) {
  const { search, setSearch } = workspace;
  const controlKind = controlKindForField(workspace.filterFields?.fields, fieldId);

  if (controlKind === "range") {
    const range = rangeForField(search, fieldId);
    return (
      <div className="control-row">
        <InputNumber
          placeholder="Min"
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
        options={discoveredOptions(workspace, fieldId, [
          { value: "true", label: "Yes" },
          { value: "false", label: "No" },
        ])}
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
      options={discoveredOptions(workspace, fieldId, [])}
      value={valuesForField(search, fieldId)}
      onChange={(values) => setSearch(setValuesForField(search, fieldId, values))}
    />
  );
}
