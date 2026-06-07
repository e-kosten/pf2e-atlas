import { Button, Space, Table, Tag } from "antd";
import type { ColumnsType } from "antd/es/table";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { ResultWindowRow } from "../../generated/atlas";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";

export function AntResults({ workspace }: { workspace: AtlasWorkspaceState }) {
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
    <section className="results-panel">
      <div className="panel-heading">
        <div>
          <h2>Results</h2>
          <p>
            {workspace.resultPage
              ? `${workspace.resultPage.page.total.toLocaleString()} records`
              : "No result window yet"}
          </p>
        </div>
        <PaginationControls workspace={workspace} />
      </div>
      <div className="results-scroll">
        <Table
          columns={columns}
          dataSource={workspace.resultPage?.rows ?? []}
          loading={workspace.resultsLoading}
          pagination={false}
          rowKey={(row) => row.record.record_key}
          size="middle"
        />
      </div>
    </section>
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
        onClick={() =>
          workspace.setPageNumber(page?.next_page ?? workspace.pageNumber + 1)
        }
      />
    </div>
  );
}
