import { Button } from "antd";
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
import { IndexTable, stopIndexRowAction } from "../../shared/ui/tables/IndexTable";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
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
    <EntityIndexPage
      actions={
        <Button
          icon={<Plus size={16} />}
          onClick={() => setCreateOpen(true)}
          type="primary"
        >
          New List
        </Button>
      }
      className="list-index-view"
      overlays={
        <CreateListModal
          open={createOpen}
          onCancel={() => setCreateOpen(false)}
          onCreated={(slug) => {
            setCreateOpen(false);
            navigateToAtlasRoute({ kind: "list", slug, selectedRecordKey: null });
          }}
        />
      }
      summary={listCountLabel(lists.data?.lists.length ?? 0)}
      title="Saved Lists"
    >
      <IndexTable
        columns={listIndexColumns()}
        dataSource={lists.data?.lists ?? []}
        loading={lists.isLoading || lists.isFetching}
        locale={{ emptyText: "No saved lists" }}
        onActivateRow={(list) =>
          navigateToAtlasRoute({
            kind: "list",
            slug: list.slug,
            selectedRecordKey: null,
          })
        }
        rowKey={(list) => list.list_key}
      />
    </EntityIndexPage>
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
      title: "Items",
      dataIndex: "item_count",
      align: "right",
      width: 96,
      render: (value: bigint) => value.toLocaleString(),
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
        <PaneIconLink
          href={listEditPath(list.slug)}
          icon={<Pencil size={15} />}
          label={`Edit ${list.name}`}
          onClick={(event) => {
            stopIndexRowAction(event);
            if (!shouldHandleAtlasRouteClick(event)) {
              return;
            }
            event.preventDefault();
            navigateToAtlasRoute({ kind: "listEdit", slug: list.slug });
          }}
          title="Edit"
        />
      ),
    },
  ];
}
