import { Button, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Pencil, Plus } from "lucide-react";
import { useState } from "react";
import type { SavedListSummaryView } from "../../generated/atlas";
import {
  listEditPath,
  listPath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "../../app/routes";
import { CreateListModal } from "./CreateListModal";
import { useSavedLists } from "./savedListQueries";
import { formatDate, listCountLabel } from "./listUtils";

type ListIndexViewProps = {
  route: Extract<AtlasRoute, { kind: "lists" }>;
};

export function ListIndexView(_props: ListIndexViewProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const lists = useSavedLists();

  return (
    <main className="list-index-view">
      <section className="list-index-view__toolbar">
        <div>
          <h2>Saved Lists</h2>
          <p>{listCountLabel(lists.data?.lists.length ?? 0)}</p>
        </div>
        <Button
          icon={<Plus size={16} />}
          onClick={() => setCreateOpen(true)}
          type="primary"
        >
          New List
        </Button>
      </section>
      <section className="list-index-view__table">
        <Table
          columns={listIndexColumns()}
          dataSource={lists.data?.lists ?? []}
          loading={lists.isLoading || lists.isFetching}
          locale={{ emptyText: "No saved lists" }}
          pagination={false}
          rowKey={(list) => list.list_key}
          size="middle"
        />
      </section>
      <CreateListModal
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onCreated={(slug) => {
          setCreateOpen(false);
          navigateToAtlasRoute({ kind: "list", slug, selectedRecordKey: null });
        }}
      />
    </main>
  );
}

function listIndexColumns(): ColumnsType<SavedListSummaryView> {
  return [
    {
      title: "List",
      render: (_, list) => (
        <a
          className="row-link row-link--anchor"
          href={listPath(list.slug)}
          onClick={(event) => {
            if (!shouldHandleAtlasRouteClick(event)) {
              return;
            }
            event.preventDefault();
            navigateToAtlasRoute({
              kind: "list",
              slug: list.slug,
              selectedRecordKey: null,
            });
          }}
        >
          <span>{list.name}</span>
          <small>{list.slug}</small>
        </a>
      ),
    },
    {
      title: "Description",
      dataIndex: "description",
      render: (value) => value ?? "",
    },
    {
      title: "Updated",
      dataIndex: "updated_at",
      width: 180,
      render: (value) => formatDate(value),
    },
    {
      title: "",
      width: 72,
      render: (_, list) => (
        <a
          aria-label={`Edit ${list.name}`}
          className="pane-toggle"
          href={listEditPath(list.slug)}
          onClick={(event) => {
            if (!shouldHandleAtlasRouteClick(event)) {
              return;
            }
            event.preventDefault();
            navigateToAtlasRoute({ kind: "listEdit", slug: list.slug });
          }}
          title="Edit"
        >
          <Pencil size={15} />
        </a>
      ),
    },
  ];
}
