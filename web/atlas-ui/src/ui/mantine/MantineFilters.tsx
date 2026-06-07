import {
  Button,
  Checkbox,
  Group,
  MultiSelect,
  NumberInput,
  Select,
  TextInput,
} from "@mantine/core";
import { Search } from "lucide-react";
import {
  KIND_OPTIONS,
  RARITY_OPTIONS,
  SORT_OPTIONS,
  TRAIT_OPTIONS,
  type SortKey,
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

export function MantineFilters({
  workspace,
}: {
  workspace: AtlasWorkspaceState;
}) {
  const { search, setSearch } = workspace;

  return (
    <aside className="filter-panel">
      <TextInput
        leftSection={<Search size={16} />}
        placeholder="Search records"
        value={search.query}
        onChange={(event) =>
          setSearch({
            ...search,
            query: event.currentTarget.value,
            mode: event.currentTarget.value.trim() ? "text_search" : "browse",
          })
        }
      />
      <MultiSelect
        data={discoveredOptions(workspace, "kind", KIND_OPTIONS)}
        label="Kinds"
        searchable
        value={search.kinds}
        onChange={(kinds) => setSearch({ ...search, kinds })}
      />
      <MultiSelect
        data={discoveredOptions(workspace, "rarity", RARITY_OPTIONS)}
        label="Rarity"
        value={search.rarity}
        onChange={(rarity) => setSearch({ ...search, rarity })}
      />
      <MultiSelect
        data={discoveredOptions(workspace, "traits", TRAIT_OPTIONS)}
        label="Traits"
        searchable
        value={search.traits}
        onChange={(traits) => setSearch({ ...search, traits })}
      />
      <Checkbox
        checked={search.traitOperator === "include_any"}
        label="Match any selected trait"
        onChange={(event) =>
          setSearch({
            ...search,
            traitOperator: event.currentTarget.checked
              ? "include_any"
              : "include_all",
          })
        }
      />
      <MultiSelect
        data={discoveredOptions(workspace, "traits", TRAIT_OPTIONS)}
        label="Excluded traits"
        searchable
        value={search.excludedTraits}
        onChange={(excludedTraits) => setSearch({ ...search, excludedTraits })}
      />
      <Group grow align="end">
        <NumberInput
          label="Min level"
          min={0}
          max={30}
          value={search.levelMin ?? ""}
          onChange={(levelMin) =>
            setSearch({
              ...search,
              levelMin: typeof levelMin === "number" ? levelMin : null,
            })
          }
        />
        <NumberInput
          label="Max level"
          min={0}
          max={30}
          value={search.levelMax ?? ""}
          onChange={(levelMax) =>
            setSearch({
              ...search,
              levelMax: typeof levelMax === "number" ? levelMax : null,
            })
          }
        />
      </Group>
      <Group grow align="end">
        <Select
          data={SORT_OPTIONS}
          label="Sort"
          value={search.sort}
          onChange={(sort) =>
            setSearch({ ...search, sort: (sort ?? "record_key") as SortKey })
          }
        />
        <NumberInput
          label="Page size"
          min={10}
          max={100}
          step={5}
          value={search.pageSize}
          onChange={(pageSize) =>
            setSearch({
              ...search,
              pageSize: typeof pageSize === "number" ? pageSize : 25,
            })
          }
        />
      </Group>
      {search.visibleFilterIds
        .filter((fieldId) => !["kind", "rarity", "traits", "level"].includes(fieldId))
        .map((fieldId) => (
          <div className="control-group" key={fieldId}>
            <div className="control-heading">
              <label>{labelForField(workspace.filterFields?.fields, fieldId)}</label>
              <Button
                size="xs"
                variant="subtle"
                onClick={() => removeVisibleFilter(workspace, fieldId)}
              >
                Remove
              </Button>
            </div>
            <OptionalFilterControl workspace={workspace} fieldId={fieldId} />
          </div>
        ))}
      <Select
        data={additionalFilterGroups(workspace).map((group) => ({
          group: group.label,
          items: group.options,
        }))}
        label="Add filter"
        placeholder="Choose a filter"
        value={null}
        onChange={(fieldId) => addVisibleFilter(workspace, fieldId)}
      />
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
      <Group grow align="end">
        <NumberInput
          placeholder="Min"
          value={range.min ?? ""}
          onChange={(min) =>
            setSearch(
              setRangeForField(search, fieldId, {
                ...range,
                min: typeof min === "number" ? min : null,
              }),
            )
          }
        />
        <NumberInput
          placeholder="Max"
          value={range.max ?? ""}
          onChange={(max) =>
            setSearch(
              setRangeForField(search, fieldId, {
                ...range,
                max: typeof max === "number" ? max : null,
              }),
            )
          }
        />
      </Group>
    );
  }

  if (controlKind === "boolean") {
    return (
      <Select
        clearable
        data={discoveredOptions(workspace, fieldId, [
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
    <MultiSelect
      data={discoveredOptions(workspace, fieldId, [])}
      searchable
      value={valuesForField(search, fieldId)}
      onChange={(values) =>
        setSearch(setValuesForField(search, fieldId, values))
      }
    />
  );
}
