import { Button, Checkbox, Input, InputNumber, Select } from "antd";
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

  return (
    <aside className="filter-panel">
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
      <div className="control-group">
        <label>Kinds</label>
        <Select
          mode="multiple"
          options={discoveredOptions(workspace, "kind", KIND_OPTIONS)}
          value={search.kinds}
          onChange={(kinds) => setSearch({ ...search, kinds })}
        />
      </div>
      <div className="control-group">
        <label>Rarity</label>
        <Select
          mode="multiple"
          options={discoveredOptions(workspace, "rarity", RARITY_OPTIONS)}
          value={search.rarity}
          onChange={(rarity) => setSearch({ ...search, rarity })}
        />
      </div>
      <div className="control-group">
        <label>Traits</label>
        <Select
          mode="multiple"
          options={discoveredOptions(workspace, "traits", TRAIT_OPTIONS)}
          value={search.traits}
          onChange={(traits) => setSearch({ ...search, traits })}
        />
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
      </div>
      <div className="control-group">
        <label>Excluded traits</label>
        <Select
          mode="multiple"
          options={discoveredOptions(workspace, "traits", TRAIT_OPTIONS)}
          value={search.excludedTraits}
          onChange={(excludedTraits) =>
            setSearch({ ...search, excludedTraits })
          }
        />
      </div>
      <div className="control-row">
        <div className="control-group">
          <label>Min level</label>
          <InputNumber
            min={0}
            max={30}
            value={search.levelMin}
            onChange={(levelMin) =>
              setSearch({ ...search, levelMin: levelMin ?? null })
            }
          />
        </div>
        <div className="control-group">
          <label>Max level</label>
          <InputNumber
            min={0}
            max={30}
            value={search.levelMax}
            onChange={(levelMax) =>
              setSearch({ ...search, levelMax: levelMax ?? null })
            }
          />
        </div>
      </div>
      <div className="control-row">
        <div className="control-group">
          <label>Sort</label>
          <Select
            options={SORT_OPTIONS}
            value={search.sort}
            onChange={(sort) => setSearch({ ...search, sort })}
          />
        </div>
        <div className="control-group">
          <label>Page size</label>
          <InputNumber
            min={10}
            max={100}
            step={5}
            value={search.pageSize}
            onChange={(pageSize) =>
              setSearch({ ...search, pageSize: pageSize ?? 25 })
            }
          />
        </div>
      </div>
      {search.visibleFilterIds
        .filter((fieldId) => !["kind", "rarity", "traits", "level"].includes(fieldId))
        .map((fieldId) => (
          <div className="control-group" key={fieldId}>
            <div className="control-heading">
              <label>{labelForField(workspace.filterFields?.fields, fieldId)}</label>
              <Button
                size="small"
                type="text"
                onClick={() => removeVisibleFilter(workspace, fieldId)}
              >
                Remove
              </Button>
            </div>
            <OptionalFilterControl workspace={workspace} fieldId={fieldId} />
          </div>
        ))}
      <div className="control-group">
        <label>Add filter</label>
        <Select
          placeholder="Choose a filter"
          options={additionalFilterGroups(workspace)}
          value={null}
          onChange={(fieldId) => addVisibleFilter(workspace, fieldId)}
        />
      </div>
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
      onChange={(values) =>
        setSearch(setValuesForField(search, fieldId, values))
      }
    />
  );
}
