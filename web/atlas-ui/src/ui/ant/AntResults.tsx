import { Space, Table, Tag } from "antd";
import type { ColumnsType } from "antd/es/table";
import type { ResultWindowRow } from "../../generated/atlas";
import { handleResultKeyboard, useActiveResultScroll } from "../resultKeyboard";
import type { AtlasWorkspaceState } from "../useAtlasWorkspace";

export function AntResults({ workspace }: { workspace: AtlasWorkspaceState }) {
  const scrollRef = useActiveResultScroll<HTMLDivElement>(
    workspace.activeResultKey,
  );
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
      <div
        aria-label="Results"
        className="results-scroll results-scroll--focusable"
        onKeyDown={(event) => handleResultKeyboard(event, workspace)}
        ref={scrollRef}
        tabIndex={0}
      >
        <Table
          columns={columns}
          dataSource={workspace.resultPage?.rows ?? []}
          loading={workspace.resultsLoading}
          pagination={false}
          rowClassName={(row) =>
            row.record.record_key === workspace.activeResultKey
              ? "result-row result-row--active"
              : "result-row"
          }
          onRow={(row) => ({
            "data-active-result":
              row.record.record_key === workspace.activeResultKey
                ? "true"
                : undefined,
            onMouseEnter: () => workspace.focusResult(row.record.record_key),
          })}
          rowKey={(row) => row.record.record_key}
          size="middle"
        />
      </div>
    </section>
  );
}
