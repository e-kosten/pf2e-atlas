import { Table, Tag, Typography } from "antd";
import type { ColumnsType } from "antd/es/table";
import type { ResultWindowRow } from "../../generated/atlas";
import { handleResultKeyboard, useActiveResultScroll } from "./resultKeyboard";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

const RESULT_COLUMNS: ColumnsType<ResultWindowRow> = [
  {
    title: "Name",
    dataIndex: ["record", "title"],
    key: "title",
    render: (_value, row) => (
      <span className="result-title">
        <Typography.Text strong>{row.record.title}</Typography.Text>
        <Typography.Text type="secondary">{row.record.record_key}</Typography.Text>
      </span>
    ),
  },
  {
    title: "Kind",
    dataIndex: ["record", "kind_label"],
    key: "kind",
    render: (_value, row) => row.record.kind_label || row.record.kind,
    width: 110,
  },
  {
    title: "Level",
    dataIndex: ["record", "level_label"],
    key: "level",
    render: (_value, row) => row.record.level_label ?? "",
    width: 76,
  },
  {
    title: "Traits",
    dataIndex: ["record", "traits"],
    key: "traits",
    render: (_value, row) => <TraitTags row={row} />,
  },
  {
    title: "Source",
    key: "source",
    render: (_value, row) => row.record.publication ?? row.record.pack ?? "",
    width: 140,
  },
  {
    title: "Match",
    key: "match",
    render: (_value, row) => row.match_summary?.label ?? row.record.preview ?? "",
  },
];

export function ResultTable({ workspace }: { workspace: SearchWorkspaceState }) {
  const scrollRef = useActiveResultScroll<HTMLDivElement>(workspace.activeResultKey);
  const rows = workspace.resultPage?.rows ?? [];
  return (
    <section className="results-panel">
      <div
        aria-label="Results"
        aria-busy={workspace.resultsLoading || workspace.resultsRefreshing}
        className="results-scroll"
        ref={scrollRef}
      >
        <Table<ResultWindowRow>
          aria-label="Results"
          className="result-table"
          columns={RESULT_COLUMNS}
          dataSource={rows}
          loading={workspace.resultsLoading}
          locale={{ emptyText: "No results" }}
          onRow={(row) => ({
            "data-active-result":
              row.record.record_key === workspace.activeResultKey ? "true" : undefined,
            onClick: () => workspace.selectRecord(row.record.record_key),
            onFocus: () => workspace.focusResult(row.record.record_key),
            onKeyDown: (event) => {
              if (event.key === " ") {
                event.preventDefault();
                workspace.selectRecord(row.record.record_key);
                return;
              }
              handleResultKeyboard(event, workspace);
            },
            onMouseEnter: () => workspace.focusResult(row.record.record_key),
            role: "button",
            tabIndex: 0,
          })}
          pagination={false}
          rowClassName={(row) =>
            row.record.record_key === workspace.activeResultKey
              ? "result-row ant-table-row-selected"
              : "result-row"
          }
          rowKey={(row) => row.record.record_key}
          size="middle"
        />
      </div>
    </section>
  );
}

function TraitTags({ row }: { row: ResultWindowRow }) {
  const traits = row.record.traits ?? [];
  if (traits.length === 0) {
    return "";
  }
  const visibleTraits = traits.slice(0, 4);
  const hiddenCount = traits.length - visibleTraits.length;
  return (
    <span className="result-traits">
      {visibleTraits.map((trait) => (
        <Tag key={`${trait.kind}-${trait.value}`}>{trait.label}</Tag>
      ))}
      {hiddenCount > 0 && (
        <Typography.Text type="secondary">+{hiddenCount}</Typography.Text>
      )}
    </span>
  );
}
