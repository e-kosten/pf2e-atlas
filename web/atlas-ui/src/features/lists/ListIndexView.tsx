import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { atlasRouter } from "../../app/router";
import { useRouterState } from "@tanstack/react-router";
import { listIndexSearch, type ListIndexInput } from "../../app/routeSearch";
import {
  navigateDestination,
  destinationHref,
  isPrimaryActivation,
} from "../../app/navigation";
import { listDestination, listsDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { PrimaryToolbar } from "../../shared/ui/PrimaryToolbar";
import { ListTagSummary } from "./ListTagSummary";
import { useQueryClient, type QueryClient } from "@tanstack/react-query";
import { ChoicePicker } from "../../shared/ui/pickers/ChoicePicker";
import { Button, Input, Typography } from "antd";
import type { ColumnsType } from "antd/es/table";
import { Pencil, Plus, Search } from "lucide-react";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { SavedListSummaryView } from "../../generated/atlas";
import {
  IndexTable,
  RowTitleLink,
  stopIndexRowAction,
} from "../../shared/ui/tables/IndexTable";
import { PaneIconButton } from "../../shared/ui/actions/PaneAction";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { EditListModal } from "./EditListModal";
import { useListSettingsOpening } from "./useListSettingsOpening";
import { SettingsOpeningError } from "../../shared/ui/overlays/SettingsOpeningError";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { CreateListModal } from "./CreateListModal";
import { useSavedLists } from "./savedListQueries";
import { listCountLabel, savedListTagOptions } from "./listUtils";
import { formatDate } from "../../shared/ui/formatDate";
type ListIndexViewProps = {
  route: Record<string, never>;
};
type IndexPosition = {
  scroll: number;
  documentScroll: number;
};
const indexPositions = new WeakMap<QueryClient, Map<string, IndexPosition>>();
export function ListIndexView(_props: ListIndexViewProps) {
  usePageTitle("Lists");
  const client = useQueryClient();
  const search = useRouterState({
    router: atlasRouter,
    select: (state) => state.location.search,
  });
  const state = listIndexSearch(search);
  const [nameDraft, setNameDraft] = useState(state.query);
  const previousQuery = useRef(state.query);
  useEffect(() => {
    if (previousQuery.current === state.query) return;
    previousQuery.current = state.query;
    setNameDraft((draft) => (draft.trim() === state.query ? draft : state.query));
  }, [state.query]);
  const positionKey = JSON.stringify(state);
  const positions = useMemo(() => {
    let saved = indexPositions.get(client);
    if (!saved) {
      saved = new Map();
      indexPositions.set(client, saved);
    }
    return saved;
  }, [client]);
  const scrollRef = useRef<HTMLElement>(null);
  const restored = useRef<string | null>(null);
  function changeState(next: ListIndexInput) {
    positions.clear();
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
    if (window.scrollY !== 0) window.scrollTo(0, 0);
    void atlasRouter.navigate({
      ...listsDestination(next),
      replace: true,
      resetScroll: false,
    });
  }
  const [tagsOpen, setTagsOpen] = useState(false);
  const [tagSearch, setTagSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const paused = useSavedWritesPaused();
  const editing = useListSettingsOpening();
  const newList = useRef<HTMLButtonElement>(null);
  const selectedTags = [...state.include, ...state.exclude];
  const lists = useSavedLists();
  const allLists = useMemo(() => lists.data?.lists ?? [], [lists.data?.lists]);
  const tagOptions = useMemo(() => savedListTagOptions(allLists), [allLists]);
  const filteredLists = useMemo(() => filterLists(allLists, state), [allLists, state]);
  useLayoutEffect(() => {
    if (lists.data && scrollRef.current && restored.current !== positionKey) {
      const saved = positions.get(positionKey) ?? { scroll: 0, documentScroll: 0 };
      const restore = () => {
        if (scrollRef.current) scrollRef.current.scrollTop = saved.scroll;
        if (window.scrollY !== saved.documentScroll)
          window.scrollTo(0, saved.documentScroll);
      };
      restore();
      const frame = requestAnimationFrame(restore);
      restored.current = positionKey;
      return () => cancelAnimationFrame(frame);
    }
  }, [lists.data, positions, positionKey]);
  useLayoutEffect(() => {
    const snapshot = () =>
      positions.set(positionKey, {
        scroll: scrollRef.current?.scrollTop ?? 0,
        documentScroll: window.scrollY,
      });
    return atlasRouter.subscribe("onBeforeNavigate", snapshot);
  }, [positions, positionKey]);
  return (
    <EntityIndexPage
      actions={
        <Button
          disabled={paused}
          ref={newList}
          icon={<Plus size={16} />}
          onClick={(event) => {
            event.currentTarget.focus({ preventScroll: true });
            setCreateOpen(true);
          }}
          type="primary"
        >
          New list
        </Button>
      }
      scrollRef={scrollRef}
      onScroll={(event) => {
        positions.set(positionKey, {
          scroll: event.currentTarget.scrollTop,
          documentScroll: window.scrollY,
        });
      }}
      toolbar={
        <PrimaryToolbar className="list-index-view__filters">
          <Input
            className="list-index-view__search"
            aria-label="Search saved lists"
            placeholder="Search lists…"
            prefix={<Search size={16} />}
            allowClear
            value={nameDraft}
            onChange={(event) => {
              setNameDraft(event.target.value);
              changeState({
                ...state,
                query: event.target.value,
              });
            }}
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
        </PrimaryToolbar>
      }
      className="list-index-view"
      overlays={
        <>
          {editing.data && editing.ref && (
            <EditListModal
              listRef={editing.ref}
              initialDetail={editing.data}
              open={editing.open}
              onClose={editing.close}
              onDeleted={editing.close}
              afterClose={() => {
                editing.clear();
                restoreOverlayFocus(
                  editing.opener?.isConnected ? editing.opener : newList.current,
                  undefined,
                );
              }}
            />
          )}
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
        </>
      }
      summary={lists.data ? listCountLabel(filteredLists.length) : undefined}
      title="Saved Lists"
    >
      {lists.error && (
        <ErrorNotice
          error={lists.error}
          title={
            lists.data ? "Saved lists could not refresh" : "Saved lists could not load"
          }
          action={
            <Button loading={lists.isFetching} onClick={() => void lists.refetch()}>
              Retry lists
            </Button>
          }
        />
      )}
      <SettingsOpeningError
        error={editing.error}
        title="List settings could not load"
        onRetry={() => {
          editing.opener?.focus({ preventScroll: true });
          editing.retry();
        }}
      />
      <IndexTable
        columns={listIndexColumns(
          (list, opener) => {
            opener.focus({ preventScroll: true });

            editing.begin(list.list_key, opener);
          },
          editing.preparing ? editing.ref : undefined,
          paused,
        )}
        scroll={{ x: 880 }}
        dataSource={filteredLists}
        loading={lists.isLoading || lists.isFetching}
        locale={{
          emptyText: lists.isLoading
            ? null
            : lists.error && !lists.data
              ? "Saved lists unavailable"
              : selectedTags.length || state.query.trim()
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
  state: ReturnType<typeof listIndexSearch>,
): SavedListSummaryView[] {
  const included = state.include;
  const excluded = new Set(state.exclude);
  const name = state.query.trim().toLocaleLowerCase();
  return lists.filter(
    (list) =>
      list.name.toLocaleLowerCase().includes(name) &&
      (!included.length || list.tags.some((tag) => included.includes(tag))) &&
      !list.tags.some((tag) => excluded.has(tag)),
  );
}
function listIndexColumns(
  onEdit: (list: SavedListSummaryView, opener: HTMLElement) => void,
  openingKey?: string,
  paused = false,
): ColumnsType<SavedListSummaryView> {
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
        <PaneIconButton
          icon={<Pencil size={15} />}
          label={`Edit ${list.name}`}
          disabled={paused}
          aria-busy={openingKey === list.list_key}
          loading={openingKey === list.list_key ? { delay: 200 } : false}
          onClick={(event) => {
            stopIndexRowAction(event);
            onEdit(list, event.currentTarget);
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
