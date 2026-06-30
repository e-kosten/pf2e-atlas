import { Button, Checkbox, Tag } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Pencil, Plus } from "lucide-react";
import { useMemo, useState } from "react";
import type { SavedListSummaryView } from "../../generated/atlas";
import {
  listEditPath,
  listPath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "../../app/routes";
import {
  IndexTable,
  RowTitleLink,
  stopIndexRowAction,
} from "../../shared/ui/tables/IndexTable";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { CreateListModal } from "./CreateListModal";
import { useSavedLists } from "./savedListQueries";
import { formatDate, listCountLabel, savedListTagOptions } from "./listUtils";

type ListIndexViewProps = {
  route: Extract<AtlasRoute, { kind: "lists" }>;
};

export function ListIndexView(_props: ListIndexViewProps) {
  const [createOpen, setCreateOpen] = useState(false);
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const lists = useSavedLists();
  const allLists = useMemo(() => lists.data?.lists ?? [], [lists.data?.lists]);
  const tagOptions = useMemo(() => savedListTagOptions(allLists), [allLists]);
  const filteredLists = useMemo(
    () => filterListsByTags(allLists, selectedTags),
    [allLists, selectedTags],
  );

  return (
    <EntityIndexPage
      actions={
        <div className="list-index-view__actions">
          {tagOptions.length > 0 && (
            <Checkbox.Group
              className="list-index-view__tag-filter"
              onChange={(values) => setSelectedTags(values.map(String))}
              options={tagOptions.map((tag) => ({ label: tag, value: tag }))}
              value={selectedTags}
            />
          )}
          <Button
            icon={<Plus size={16} />}
            onClick={() => setCreateOpen(true)}
            type="primary"
          >
            New List
          </Button>
        </div>
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
      summary={listCountLabel(filteredLists.length)}
      title="Saved Lists"
    >
      <IndexTable
        columns={listIndexColumns()}
        dataSource={filteredLists}
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

function filterListsByTags(
  lists: SavedListSummaryView[],
  selectedTags: string[],
): SavedListSummaryView[] {
  if (selectedTags.length === 0) {
    return lists;
  }
  const selected = new Set(selectedTags);
  return lists.filter((list) => list.tags.some((tag) => selected.has(tag)));
}

function listIndexColumns(): ColumnsType<SavedListSummaryView> {
  return [
    {
      title: "List",
      render: (_, list) => (
        <RowTitleLink
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
          subtitle={list.slug}
          title={list.name}
        />
      ),
    },
    {
      title: "Description",
      dataIndex: "description",
      render: (value) => value ?? "",
    },
    {
      title: "Tags",
      dataIndex: "tags",
      render: (tags: string[]) => (
        <div className="list-index-view__tags">
          {tags.map((tag) => (
            <Tag key={tag}>{tag}</Tag>
          ))}
        </div>
      ),
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
