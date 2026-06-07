import { Button, Checkbox, Input, InputNumber, Select, Space, Table, Tag } from "antd";
import type { ColumnsType } from "antd/es/table";
import { ChevronLeft, ChevronRight, Search } from "lucide-react";
import type { ResultWindowRow } from "../generated/atlas";
import {
  KIND_OPTIONS,
  RARITY_OPTIONS,
  SORT_OPTIONS,
  TRAIT_OPTIONS,
} from "../state/searchState";
import { RecordPresentation } from "./recordPresentation";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

type AntPrototypeProps = {
  workspace: AtlasWorkspaceState;
};

export function AntPrototype({ workspace }: AntPrototypeProps) {
  const { search, setSearch, resultPage } = workspace;
  const columns: ColumnsType<ResultWindowRow> = [
    {
      title: "Record",
      dataIndex: ["record", "title"],
      render: (_, row) => (
        <button
          className="row-link"
          onClick={() => workspace.selectRecord(row.record.record_key)}
          type="button"
        >
          <span>{row.record.title}</span>
          <small>{row.record.record_key}</small>
        </button>
      ),
    },
    {
      title: "Kind",
      dataIndex: ["record", "kind_label"],
      width: 130,
    },
    {
      title: "Level",
      dataIndex: ["record", "level_label"],
      width: 92,
      render: (value) => value ?? "",
    },
    {
      title: "Traits",
      dataIndex: ["record", "traits"],
      render: (_, row) => (
        <Space size={[4, 4]} wrap>
          {(row.record.traits ?? []).slice(0, 4).map((trait) => (
            <Tag key={trait.value}>{trait.label}</Tag>
          ))}
        </Space>
      ),
    },
  ];

  return (
    <main className="workspace-grid workspace-grid--ant">
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
      <section className="results-panel">
        <div className="panel-heading">
          <div>
            <h2>Results</h2>
            <p>
              {resultPage
                ? `${resultPage.page.total.toLocaleString()} records`
                : "No result window yet"}
            </p>
          </div>
          <PaginationControls workspace={workspace} />
        </div>
        <Table
          columns={columns}
          dataSource={resultPage?.rows ?? []}
          loading={workspace.resultsLoading}
          pagination={false}
          rowKey={(row) => row.record.record_key}
          size="middle"
        />
      </section>
      <section className="detail-panel">
        <RecordPresentation
          detail={workspace.recordDetail}
          loading={workspace.detailLoading}
          onReference={workspace.selectRecord}
        />
      </section>
    </main>
  );
}

function PaginationControls({ workspace }: { workspace: AtlasWorkspaceState }) {
  const page = workspace.resultPage?.page;
  return (
    <div className="pager">
      <Button
        disabled={workspace.pageNumber <= 1}
        icon={<ChevronLeft size={16} />}
        onClick={() => workspace.setPageNumber(workspace.pageNumber - 1)}
      />
      <span>{page ? `${page.number}` : workspace.pageNumber}</span>
      <Button
        disabled={!page?.has_more}
        icon={<ChevronRight size={16} />}
        onClick={() => workspace.setPageNumber(page?.next_page ?? workspace.pageNumber + 1)}
      />
    </div>
  );
}
