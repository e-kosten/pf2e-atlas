import { ExternalLink, Pencil, Plus, Trash2 } from "lucide-react";
import {
  keepPreviousData,
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { Button, Form, Input, Modal, Select, Table } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useEffect, useMemo, useState } from "react";
import type {
  CreateSavedListRequest,
  FilterEditorView,
  FilterValueListView,
  SavedListItemView,
  SavedListSummaryView,
  UpdateSavedListRequest,
} from "../generated/atlas";
import {
  createSavedList,
  deleteSavedList,
  discoverFilterEditor,
  discoverFilterValues,
  filterSavedList,
  getRecordDetail,
  getSavedList,
  removeSavedListItem,
  updateSavedList,
} from "../api/atlasApi";
import {
  buildBasicFilter,
  buildSavedListFilterDiscoveryContext,
  DEFAULT_SEARCH_STATE,
  encodeSearchExecutionState,
  type SearchFormState,
} from "../state/searchState";
import { AntFilterControls } from "./ant/AntFilters";
import type { FilterPanelState } from "./filterControls";
import { RecordPresentation } from "./recordPresentation";
import {
  listEditPath,
  listPath,
  navigateToAtlasRoute,
  recordPath,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "./routes";
import { useSavedLists } from "./savedListQueries";
import { WorkspaceLayout } from "./WorkspaceLayout";

type ListIndexViewProps = {
  route: Extract<AtlasRoute, { kind: "lists" }>;
};

type ListDetailViewProps = {
  route: Extract<AtlasRoute, { kind: "list" }>;
};

type ListEditViewProps = {
  route: Extract<AtlasRoute, { kind: "listEdit" }>;
};

type CreateListFormValues = {
  name: string;
};

type EditListFormValues = {
  name: string;
  description?: string;
};

const LIST_WORKSPACE_WIDTH_SPECS = {
  filter: { defaultWidth: 280, minWidth: 240 },
  results: { defaultWidth: 420, minWidth: 280 },
  detail: { defaultWidth: 640, minWidth: 360 },
};
const LIST_SEARCH_REQUEST_DEBOUNCE_MS = 300;

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

export function ListDetailView({ route }: ListDetailViewProps) {
  const queryClient = useQueryClient();
  const lists = useSavedLists();
  const [filters, setFilters] = useState<SearchFormState>(DEFAULT_SEARCH_STATE);
  const activeFilters = useDebouncedSearchFilters(filters);
  const filterToken = useMemo(
    () => encodeSearchExecutionState(activeFilters),
    [activeFilters],
  );
  const filterDiscovery = useSavedListFilterDiscovery(route.slug, filters);
  const list = useQuery({
    queryKey: ["saved-list", route.slug, filterToken],
    placeholderData: keepPreviousData,
    queryFn: () => {
      const query = listSearchQuery(activeFilters);
      return filterSavedList({
        list_ref: route.slug,
        ...(query ? { query } : {}),
        filter: buildBasicFilter(activeFilters),
      });
    },
  });
  const selectedItem = list.data?.items.find(
    (item) => item.record_key === route.selectedRecordKey,
  );
  const detail = useRecordDetail(
    selectedItem?.status === "unresolved" ? null : route.selectedRecordKey,
  );
  const removeItem = useMutation({
    mutationFn: (recordKey: string) =>
      removeSavedListItem({ list_ref: route.slug, record_ref: recordKey }),
    onSuccess: async (_view, recordKey) => {
      await queryClient.invalidateQueries({ queryKey: ["saved-list", route.slug] });
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      if (recordKey === route.selectedRecordKey) {
        navigateToAtlasRoute({
          kind: "list",
          slug: route.slug,
          selectedRecordKey: null,
        });
      }
    },
  });

  return (
    <WorkspaceLayout
      filter={
        <ListInfoPane
          currentSlug={route.slug}
          filterState={{
            search: filters,
            setSearch: setFilters,
            filterEditor: filterDiscovery.filterEditor,
            filterValuesByField: filterDiscovery.filterValuesByField,
            filterDiscoveryLoading: filterDiscovery.loading,
            errorMessage: filterDiscovery.errorMessage,
          }}
          list={list.data?.list}
          lists={lists.data?.lists ?? []}
          loading={list.isLoading}
          listsLoading={lists.isLoading || lists.isFetching}
          onSelectList={(slug) =>
            navigateToAtlasRoute({ kind: "list", slug, selectedRecordKey: null })
          }
        />
      }
      results={
        <ListItemsPane
          items={list.data?.items ?? []}
          loading={list.isLoading || list.isFetching}
          removingKey={
            removeItem.isPending && typeof removeItem.variables === "string"
              ? removeItem.variables
              : null
          }
          selectedRecordKey={route.selectedRecordKey}
          onRemove={(recordKey) => removeItem.mutate(recordKey)}
          onSelect={(recordKey) =>
            navigateToAtlasRoute({
              kind: "list",
              slug: route.slug,
              selectedRecordKey: recordKey,
            })
          }
        />
      }
      selectedRecordKey={route.selectedRecordKey}
      labels={{ filter: "List", results: "Items", detail: "Detail" }}
      sizing="detail-focus"
      widthSpecs={LIST_WORKSPACE_WIDTH_SPECS}
      detailHeaderActions={
        route.selectedRecordKey ? (
          <a
            aria-label="Open full page"
            className="pane-toggle"
            href={recordPath(route.selectedRecordKey)}
            onClick={(event) => {
              if (!shouldHandleAtlasRouteClick(event)) {
                return;
              }
              event.preventDefault();
              navigateToAtlasRoute({
                kind: "record",
                recordKey: route.selectedRecordKey!,
              });
            }}
            title="Open full page"
          >
            <ExternalLink size={16} />
          </a>
        ) : null
      }
      detail={
        <section className="detail-panel">
          {selectedItem?.status === "unresolved" ? (
            <div className="detail-empty">This saved record is unresolved.</div>
          ) : (
            <RecordPresentation
              detail={detail.data}
              loading={detail.isLoading || detail.isFetching}
              onReference={(recordKey) =>
                navigateToAtlasRoute({
                  kind: "list",
                  slug: route.slug,
                  selectedRecordKey: recordKey,
                })
              }
            />
          )}
          {list.error && <InlineError message={list.error.message} />}
          {detail.error && <InlineError message={detail.error.message} />}
          {removeItem.error && <InlineError message={removeItem.error.message} />}
        </section>
      }
    />
  );
}

export function ListEditView({ route }: ListEditViewProps) {
  const [form] = Form.useForm<EditListFormValues>();
  const queryClient = useQueryClient();
  const list = useQuery({
    queryKey: ["saved-list", route.slug],
    queryFn: () => getSavedList(route.slug),
  });
  const update = useMutation({
    mutationFn: (request: UpdateSavedListRequest) => updateSavedList(request),
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      await queryClient.invalidateQueries({ queryKey: ["saved-list", route.slug] });
      await queryClient.invalidateQueries({ queryKey: ["saved-list", view.list.slug] });
      navigateToAtlasRoute({
        kind: "list",
        slug: view.list.slug,
        selectedRecordKey: null,
      });
    },
  });
  const deleteListMutation = useMutation({
    mutationFn: () => deleteSavedList(list.data!.list.list_key),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      navigateToAtlasRoute({ kind: "lists" });
    },
  });

  useEffect(() => {
    if (!list.data) {
      return;
    }
    form.setFieldsValue({
      name: list.data.list.name,
      description: list.data.list.description ?? undefined,
    });
  }, [form, list.data]);

  return (
    <main className="list-edit-view">
      <section className="list-index-view__toolbar">
        <div>
          <h2>Edit List</h2>
          {list.data && <p>{list.data.list.name}</p>}
        </div>
      </section>
      <section className="list-edit-view__body">
        {list.isLoading ? (
          <div className="detail-empty">Loading list</div>
        ) : !list.data ? (
          <div className="detail-empty">List not found</div>
        ) : (
          <div className="list-edit-view__form">
            <Form
              form={form}
              layout="vertical"
              onFinish={(values) =>
                update.mutate({
                  list_key: list.data!.list.list_key,
                  slug: slugify(values.name),
                  name: values.name,
                  description: normalizeOptionalText(values.description),
                })
              }
            >
              <Form.Item
                label="Name"
                name="name"
                rules={[
                  { required: true },
                  {
                    validator: (_, value: string | undefined) =>
                      value === undefined || slugify(value).length > 0
                        ? Promise.resolve()
                        : Promise.reject(
                            new Error("Use at least one letter or number."),
                          ),
                  },
                ]}
              >
                <Input className="list-edit-view__name" />
              </Form.Item>
              <Form.Item label="Description" name="description">
                <Input.TextArea autoSize={{ minRows: 3, maxRows: 6 }} />
              </Form.Item>
              <div className="list-edit-view__actions">
                <Button htmlType="submit" loading={update.isPending} type="primary">
                  Save
                </Button>
                <Button
                  href={listPath(list.data.list.slug)}
                  onClick={(event) => {
                    if (!shouldHandleAtlasRouteClick(event)) {
                      return;
                    }
                    event.preventDefault();
                    navigateToAtlasRoute({
                      kind: "list",
                      slug: list.data!.list.slug,
                      selectedRecordKey: null,
                    });
                  }}
                >
                  Done
                </Button>
              </div>
            </Form>
            <dl className="list-edit-view__meta">
              <div>
                <dt>Created</dt>
                <dd>{formatDate(list.data.list.created_at)}</dd>
              </div>
              <div>
                <dt>Updated</dt>
                <dd>{formatDate(list.data.list.updated_at)}</dd>
              </div>
            </dl>
            <div className="list-edit-view__danger">
              <Button
                danger
                icon={<Trash2 size={16} />}
                loading={deleteListMutation.isPending}
                onClick={() => deleteListMutation.mutate()}
              >
                Delete List
              </Button>
            </div>
          </div>
        )}
        {list.error && <InlineError message={list.error.message} />}
        {update.error && <InlineError message={update.error.message} />}
        {deleteListMutation.error && (
          <InlineError message={deleteListMutation.error.message} />
        )}
      </section>
    </main>
  );
}

function ListInfoPane({
  currentSlug,
  filterState,
  list,
  lists,
  listsLoading,
  loading,
  onSelectList,
}: {
  currentSlug: string;
  filterState: FilterPanelState;
  list: SavedListSummaryView | undefined;
  lists: SavedListSummaryView[];
  listsLoading: boolean;
  loading: boolean;
  onSelectList: (slug: string) => void;
}) {
  if (loading) {
    return <div className="list-info-pane">Loading list</div>;
  }
  if (!list) {
    return <div className="list-info-pane">List not found</div>;
  }
  const listOptions = lists.some((listOption) => listOption.slug === list.slug)
    ? lists
    : [list, ...lists];
  return (
    <section className="list-info-pane">
      <Select
        aria-label="Selected list"
        className="list-info-pane__select"
        loading={listsLoading}
        onChange={onSelectList}
        options={listOptions.map((listOption) => ({
          label: listOption.name,
          value: listOption.slug,
        }))}
        value={currentSlug}
      />
      <div>
        <h2>{list.name}</h2>
      </div>
      {list.description && (
        <p className="list-info-pane__description">{list.description}</p>
      )}
      <Button
        href={listEditPath(list.slug)}
        icon={<Pencil size={16} />}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) {
            return;
          }
          event.preventDefault();
          navigateToAtlasRoute({ kind: "listEdit", slug: list.slug });
        }}
      >
        Edit
      </Button>
      <AntFilterControls
        filterState={filterState}
        includeResultOptions={false}
        includeSearch
      />
    </section>
  );
}

function useSavedListFilterDiscovery(
  listRef: string,
  filters: SearchFormState,
): {
  filterEditor: FilterEditorView | undefined;
  filterValuesByField: Record<string, FilterValueListView | undefined>;
  loading: boolean;
  errorMessage: string | null;
} {
  const queryClient = useQueryClient();
  const filterToken = useMemo(() => encodeSearchExecutionState(filters), [filters]);
  const context = useMemo(
    () => buildSavedListFilterDiscoveryContext(listRef, filters),
    [listRef, filters],
  );
  const filterEditorQuery = useQuery({
    queryKey: [
      "saved-list-filter-editor",
      listRef,
      filterToken,
      filters.visibleFilterIds,
    ],
    queryFn: () =>
      discoverFilterEditor({
        context,
        selected_field_ids: filters.visibleFilterIds,
      }),
  });
  const valueFieldIds = useMemo(() => {
    const fields = (filterEditorQuery.data?.groups ?? []).flatMap(
      (group) => group.fields,
    );
    const visibleFields = new Set(filters.visibleFilterIds);
    const hiddenFields = new Set(filters.hiddenFilterIds);
    return fields
      .filter(
        (field) =>
          field.applicability === "applicable" &&
          field.supports_counts &&
          (field.placement === "always_visible" ||
            visibleFields.has(field.id) ||
            (field.placement === "initially_visible" && !hiddenFields.has(field.id))),
      )
      .map((field) => field.id);
  }, [filterEditorQuery.data, filters.hiddenFilterIds, filters.visibleFilterIds]);
  const filterValueQueries = useQueries({
    queries: valueFieldIds.map((fieldId) => ({
      queryKey: ["saved-list-filter-values", listRef, filterToken, fieldId],
      enabled: !filterEditorQuery.isPlaceholderData,
      placeholderData: () =>
        queryClient.getQueryData<FilterValueListView>([
          "saved-list-filter-values",
          listRef,
          filterToken,
          fieldId,
        ]),
      queryFn: () =>
        discoverFilterValues({
          context,
          field_id: fieldId,
        }),
    })),
  });
  const filterValuesByField = useMemo(() => {
    const pairs = valueFieldIds.map((fieldId, index) => [
      fieldId,
      filterValueQueries[index]?.data,
    ]);
    return Object.fromEntries(pairs);
  }, [filterValueQueries, valueFieldIds]);
  const errorMessage =
    filterEditorQuery.error?.message ??
    filterValueQueries.find((query) => query.error)?.error?.message ??
    null;
  return {
    filterEditor: filterEditorQuery.data,
    filterValuesByField,
    loading:
      filterEditorQuery.isLoading ||
      filterEditorQuery.isFetching ||
      filterValueQueries.some((query) => query.isLoading || query.isFetching),
    errorMessage,
  };
}

function listSearchQuery(filters: SearchFormState): string | undefined {
  const query = filters.query.trim();
  return filters.mode === "text_search" && query.length > 0 ? query : undefined;
}

function useDebouncedSearchFilters(filters: SearchFormState): SearchFormState {
  const [activeFilters, setActiveFilters] = useState(filters);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setActiveFilters(filters);
    }, LIST_SEARCH_REQUEST_DEBOUNCE_MS);
    return () => window.clearTimeout(timeout);
  }, [filters]);

  return activeFilters;
}

function ListItemsPane({
  items,
  loading,
  onRemove,
  onSelect,
  removingKey,
  selectedRecordKey,
}: {
  items: SavedListItemView[];
  loading: boolean;
  onRemove: (recordKey: string) => void;
  onSelect: (recordKey: string) => void;
  removingKey: string | null;
  selectedRecordKey: string | null;
}) {
  const columns: ColumnsType<SavedListItemView> = [
    {
      title: "Record",
      render: (_, item) => (
        <button
          className="row-link"
          disabled={item.status === "unresolved"}
          onClick={() => onSelect(item.record_key)}
          type="button"
        >
          <span>{item.record?.title ?? item.snapshot.title}</span>
        </button>
      ),
    },
    {
      title: "Kind",
      width: 120,
      render: (_, item) => item.record?.kind_label ?? item.snapshot.kind ?? "",
    },
    {
      title: "",
      width: 52,
      render: (_, item) => (
        <button
          aria-label={`Remove ${item.snapshot.title}`}
          className="pane-toggle"
          disabled={removingKey === item.record_key}
          onClick={() => onRemove(item.record_key)}
          title="Remove"
          type="button"
        >
          <Trash2 size={15} />
        </button>
      ),
    },
  ];

  return (
    <section className="results-panel">
      <div className="results-scroll">
        <Table
          columns={columns}
          dataSource={items}
          loading={loading}
          locale={{ emptyText: "No saved records" }}
          pagination={false}
          rowClassName={(item) =>
            item.record_key === selectedRecordKey
              ? "result-row result-row--active"
              : "result-row"
          }
          rowKey={(item) => item.record_key}
          size="middle"
        />
      </div>
    </section>
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

function CreateListModal({
  onCancel,
  onCreated,
  open,
}: {
  onCancel: () => void;
  onCreated: (slug: string) => void;
  open: boolean;
}) {
  const [form] = Form.useForm<CreateListFormValues>();
  const queryClient = useQueryClient();
  const create = useMutation({
    mutationFn: (request: CreateSavedListRequest) => createSavedList(request),
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      onCreated(view.list.slug);
      form.resetFields();
    },
  });

  return (
    <Modal
      confirmLoading={create.isPending}
      okText="Create"
      onCancel={onCancel}
      onOk={() => form.submit()}
      open={open}
      title="New List"
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={(values) =>
          create.mutate({
            slug: slugify(values.name),
            name: values.name,
          })
        }
      >
        <Form.Item
          label="Name"
          name="name"
          rules={[
            { required: true },
            {
              validator: (_, value: string | undefined) =>
                value === undefined || slugify(value).length > 0
                  ? Promise.resolve()
                  : Promise.reject(new Error("Use at least one letter or number.")),
            },
          ]}
        >
          <Input />
        </Form.Item>
      </Form>
      {create.error && <InlineError message={create.error.message} />}
    </Modal>
  );
}

function useRecordDetail(recordKey: string | null) {
  return useQuery({
    queryKey: ["record-detail", recordKey],
    queryFn: () => getRecordDetail(recordKey!),
    enabled: recordKey !== null,
  });
}

function InlineError({ message }: { message: string }) {
  return <div className="error-banner">{message}</div>;
}

function listCountLabel(count: number): string {
  return count === 1 ? "1 saved list" : `${count.toLocaleString()} saved lists`;
}

function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function normalizeOptionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}
