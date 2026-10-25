import { atlasRouter } from "../../app/router";
import {
  navigateDestination,
  destinationHref,
  isPrimaryActivation,
} from "../../app/navigation";
import { listDestination, listEditDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { ListTagSummary } from "./ListTagSummary";
import { useQueryClient, type QueryClient } from "@tanstack/react-query";
import { ChoicePicker } from "../../shared/ui/pickers/ChoicePicker";
import { Alert, Button, Input, Typography } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Pencil, Plus, Search } from "lucide-react";
import { useLayoutEffect, useMemo, useRef, useState } from "react";
import type { SavedListSummaryView } from "../../generated/atlas";
import {
  IndexTable,
  RowTitleLink,
  stopIndexRowAction,
} from "../../shared/ui/tables/IndexTable";
import { PaneIconLink } from "../../shared/ui/actions/PaneAction";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { CreateListModal } from "./CreateListModal";
import { useSavedLists } from "./savedListQueries";
import { listCountLabel, savedListTagOptions } from "./listUtils";
import { formatDate } from "../../shared/ui/formatDate";
type ListIndexViewProps = {
  route: Record<string, never>;
};
type IndexState = {
  name: string;
  include: string[];
  exclude: string[];
  scroll: number;
  documentScroll: number;
};
// Browser-session collection state survives detail/edit navigation, scoped to this app client.
const indexPositions = new WeakMap<QueryClient, IndexState>();
export function ListIndexView(_props: ListIndexViewProps) {
  usePageTitle("Lists");
  const client = useQueryClient();
  const [state, setState] = useState<IndexState>(
    () =>
      indexPositions.get(client) ?? {
        name: "",
        include: [],
        exclude: [],
        scroll: 0,
        documentScroll: 0,
      },
  );
  const scrollRef = useRef<HTMLElement>(null);
  const restored = useRef(false);
  function changeState(next: IndexState) {
    indexPositions.set(client, next);
    setState(next);
    if (scrollRef.current) scrollRef.current.scrollTop = next.scroll;
    if (window.scrollY !== next.documentScroll) window.scrollTo(0, next.documentScroll);
  }
  const [tagsOpen, setTagsOpen] = useState(false);
  const [tagSearch, setTagSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const selectedTags = [...state.include, ...state.exclude];
  const lists = useSavedLists();
  const allLists = useMemo(() => lists.data?.lists ?? [], [lists.data?.lists]);
  const tagOptions = useMemo(() => savedListTagOptions(allLists), [allLists]);
  const filteredLists = useMemo(() => filterLists(allLists, state), [allLists, state]);
  useLayoutEffect(() => {
    if (lists.data && scrollRef.current && !restored.current) {
      const restore = () => {
        if (scrollRef.current) scrollRef.current.scrollTop = state.scroll;
        if (window.scrollY !== state.documentScroll)
          window.scrollTo(0, state.documentScroll);
      };
      restore();
      const frame = requestAnimationFrame(restore);
      restored.current = true;
      return () => cancelAnimationFrame(frame);
    }
  }, [lists.data, state.scroll, state.documentScroll]);
  useLayoutEffect(() => {
    const snapshot = () =>
      indexPositions.set(client, {
        ...state,
        scroll: scrollRef.current?.scrollTop ?? 0,
        documentScroll: window.scrollY,
      });
    return atlasRouter.subscribe("onBeforeNavigate", snapshot);
  }, [client, state]);
  return (
    <EntityIndexPage
      scrollRef={scrollRef}
      onScroll={(event) => {
        const next = {
          ...state,
          scroll: event.currentTarget.scrollTop,
          documentScroll: window.scrollY,
        };
        indexPositions.set(client, next);
      }}
      toolbar={
        <PrimaryToolbar className="list-index-view__actions">
          <Input
            className="list-index-view__search"
            aria-label="Search saved lists"
            placeholder="Search lists…"
            prefix={<Search size={16} />}
            allowClear
            value={state.name}
            onChange={(event) =>
              changeState({
                ...state,
                name: event.target.value,
                scroll: 0,
                documentScroll: 0,
              })
            }
          />
          {(tagOptions.length > 0 || selectedTags.length > 0) && (
            <ChoicePicker
              id="saved-list-tags"
              label="Tags"
              choices={tagOptions.map((tag) => ({ value: tag, label: tag }))}
              selected={selectedTags}
              stateOf={(tag) =>
                state.include.includes(tag)
                  ? "include"
                  : state.exclude.includes(tag)
                    ? "exclude"
                    : "neutral"
              }
              onChange={(tag, next) =>
                changeState({
                  ...state,
                  include:
                    next === "include"
                      ? [...state.include.filter((value) => value !== tag), tag]
                      : state.include.filter((value) => value !== tag),
                  exclude:
                    next === "exclude"
                      ? [...state.exclude.filter((value) => value !== tag), tag]
                      : state.exclude.filter((value) => value !== tag),
                  scroll: 0,
                  documentScroll: 0,
                })
              }
              open={tagsOpen}
              onOpenChange={(open) => {
                setTagsOpen(open);
                if (!open) setTagSearch("");
              }}
              text={tagSearch}
              onTextChange={setTagSearch}
            />
          )}
          <Button
            icon={<Plus size={16} />}
            onClick={(event) => {
              event.currentTarget.focus({ preventScroll: true });
              setCreateOpen(true);
            }}
            type="primary"
          >
            New list
          </Button>
        </PrimaryToolbar>
      }
      className="list-index-view"
      overlays={
        <CreateListModal
          open={createOpen}
          onCancel={() => setCreateOpen(false)}
          onCreated={(slug) => {
            setCreateOpen(false);
            navigateDestination(
              listDestination({
                slug,
                selectedRecordKey: null,
              }),
            );
          }}
        />
      }
      summary={lists.data ? listCountLabel(filteredLists.length) : undefined}
      title="Saved Lists"
    >
      {lists.error && (
        <Alert
          type="error"
          title={
            lists.data ? "Saved lists could not refresh" : "Saved lists could not load"
          }
          description={lists.error.message}
          action={
            <Button loading={lists.isFetching} onClick={() => void lists.refetch()}>
              Retry lists
            </Button>
          }
        />
      )}
      <IndexTable
        columns={listIndexColumns()}
        scroll={{ x: 880 }}
        dataSource={filteredLists}
        loading={lists.isLoading || lists.isFetching}
        locale={{
          emptyText: lists.isLoading
            ? "Loading saved lists…"
            : lists.error && !lists.data
              ? "Saved lists unavailable"
              : selectedTags.length || state.name.trim()
                ? "No lists match these filters"
                : "No saved lists. Create a list to collect records.",
        }}
        onActivateRow={(list) =>
          navigateDestination(
            listDestination({
              slug: list.slug,
              selectedRecordKey: null,
            }),
          )
        }
        rowKey={(list) => list.list_key}
      />
    </EntityIndexPage>
  );
}
function filterLists(
  lists: SavedListSummaryView[],
  state: IndexState,
): SavedListSummaryView[] {
  const included = state.include;
  const excluded = new Set(state.exclude);
  const name = state.name.trim().toLocaleLowerCase();
  return lists.filter(
    (list) =>
      list.name.toLocaleLowerCase().includes(name) &&
      (!included.length || list.tags.some((tag) => included.includes(tag))) &&
      !list.tags.some((tag) => excluded.has(tag)),
  );
}
function listIndexColumns(): ColumnsType<SavedListSummaryView> {
  const columns: ColumnsType<SavedListSummaryView> = [
    {
      title: "List",
      minWidth: 240,
      width: 280,
      render: (_, list) => (
        <RowTitleLink
          href={destinationHref(
            listDestination({ slug: list.slug, selectedRecordKey: null }),
          )}
          onClick={(event) => {
            if (!isPrimaryActivation(event)) {
              return;
            }
            event.preventDefault();
            navigateDestination(
              listDestination({
                slug: list.slug,
                selectedRecordKey: null,
              }),
            );
          }}
          subtitle={list.slug}
          title={list.name}
        />
      ),
    },
    {
      title: "Description",
      minWidth: 280,
      dataIndex: "description",
      render: (value: string | null) =>
        value ? (
          <Typography.Paragraph
            className="list-index-view__description"
            ellipsis={{ rows: 2 }}
            title={value}
          >
            {value}
          </Typography.Paragraph>
        ) : (
          ""
        ),
    },
    {
      title: "Tags",
      minWidth: 160,
      width: 240,
      render: (_, list) => (
        <ListTagSummary tags={list.tags} label={list.name + " tags"} />
      ),
    },
    {
      title: "Items",
      dataIndex: "item_count",
      align: "right",
      width: 96,
      render: (value: number) => value.toLocaleString(),
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
          href={destinationHref(listEditDestination({ slug: list.slug }))}
          icon={<Pencil size={15} />}
          label={`Edit ${list.name}`}
          onClick={(event) => {
            stopIndexRowAction(event);
            if (!isPrimaryActivation(event)) {
              return;
            }
            event.preventDefault();
            navigateDestination(
              listEditDestination({
                slug: list.slug,
              }),
            );
          }}
          title="Edit"
        />
      ),
    },
  ];
  return columns.map((column) => ({
    ...column,
    onCell: () => ({ style: { verticalAlign: "top" } }),
  }));
}
