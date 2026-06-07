import { Checkbox, Input, InputNumber, Select } from "antd";
import { Search } from "lucide-react";
import {
  KIND_OPTIONS,
  RARITY_OPTIONS,
  SORT_OPTIONS,
  TRAIT_OPTIONS,
} from "../../state/searchState";
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
          options={KIND_OPTIONS}
          value={search.kinds}
          onChange={(kinds) => setSearch({ ...search, kinds })}
        />
      </div>
      <div className="control-group">
        <label>Rarity</label>
        <Select
          mode="multiple"
          options={RARITY_OPTIONS}
          value={search.rarity}
          onChange={(rarity) => setSearch({ ...search, rarity })}
        />
      </div>
      <div className="control-group">
        <label>Traits</label>
        <Select
          mode="multiple"
          options={TRAIT_OPTIONS}
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
          options={TRAIT_OPTIONS}
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
    </aside>
  );
}
