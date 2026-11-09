import {
  navigateDestination,
  destinationHref,
  isPrimaryActivation,
} from "../../app/navigation";
import { listDestination, listsDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Input, Select, Typography } from "antd";
import { ArrowLeft, Trash2 } from "lucide-react";
import { useLayoutEffect, useMemo, useRef, useState } from "react";
import { deleteSavedList, getSavedList, updateSavedList } from "../../api/atlasApi";
import type {
  SavedListDetailView,
  SavedListSummaryView,
  UpdateSavedListRequest,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { SavedEditConflictReview } from "../../shared/feedback/SavedEditConflictReview";
import { savedEditConflictFields } from "../../shared/feedback/savedEditConflict";
import { normalizeOptionalText, normalizeTags, savedListTagOptions } from "./listUtils";
import { invalidateSavedListQueries, useSavedLists } from "./savedListQueries";
import { formatDate } from "../../shared/ui/formatDate";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";

type ListEditViewProps = {
  route: { slug: string };
};

type EditListFormValues = {
  name: string;
  description: string;
  tags: string[];
};

type Edits = Pick<UpdateSavedListRequest, "name" | "description" | "tags">;
type Field = keyof Edits;
const fields = ["name", "description", "tags"] as const;
const labels: Record<Field, string> = {
  name: "Name",
  description: "Description",
  tags: "Tags",
};

type Review = {
  fields: Field[];
} & (
  { status: "loading" | "failed" } | { status: "ready"; saved: SavedListSummaryView }
);

function listQuery(listRef: string) {
  return {
    queryKey: ["saved-list", listRef],
    queryFn: () => getSavedList(listRef),
  };
}

function sameTags(left: string[], right: string[]) {
  const normalized = normalizeTags(right);
  return (
    left.length === normalized.length && left.every((tag, i) => tag === normalized[i])
  );
}

export function ListEditView({ route }: ListEditViewProps) {
  return <ListEditorPage key={route.slug} routeSlug={route.slug} />;
}

function ListEditorPage({ routeSlug }: { routeSlug: string }) {
  const [resolved, setResolved] = useState<{
    detail: SavedListDetailView;
    updatedAt: number;
  }>();
  const list = useQuery({
    ...listQuery(resolved?.detail.list.list_key ?? routeSlug),
    initialData: resolved?.detail,
    initialDataUpdatedAt: resolved?.updatedAt,
    refetchOnMount: resolved ? undefined : "always",
  });
  if (!resolved && list.isFetchedAfterMount && list.isSuccess) {
    setResolved({ detail: list.data, updatedAt: list.dataUpdatedAt });
  }
  const saved = resolved ? list.data?.list : undefined;
  const loading = list.isLoading || (!resolved && list.isFetching);
  const returnTo = listDestination({
    slug: saved?.slug ?? routeSlug,
    selectedRecordKey: null,
  });
  usePageTitle(saved?.name ?? "List");
  return (
    <main className="list-edit-view">
      <section className="list-edit-view__header">
        <Typography.Link
          className="list-edit-view__back-link"
          href={destinationHref(returnTo)}
          onClick={(event) => {
            if (!isPrimaryActivation(event)) return;
            event.preventDefault();
            navigateDestination(returnTo);
          }}
        >
          <ArrowLeft size={14} aria-hidden="true" />
          Back to list
        </Typography.Link>
        <div>
          <h2>Edit list</h2>
          {saved && <p>{saved.name}</p>}
        </div>
      </section>
      <section className="list-edit-view__body">
        {saved ? (
          <ListEditForm
            key={saved.list_key}
            saved={saved}
            routeSlug={routeSlug}
            readError={list.error}
            fetching={list.isFetching}
            retry={() => void list.refetch()}
          />
        ) : loading ? (
          <LoadingStatus className="detail-empty" label="Loading list…" />
        ) : list.error ? (
          <ErrorNotice
            error={list.error}
            title="List unavailable"
            action={
              <Button loading={list.isFetching} onClick={() => void list.refetch()}>
                Retry list
              </Button>
            }
          />
        ) : (
          <div className="detail-empty">List not found</div>
        )}
      </section>
    </main>
  );
}

function ListEditForm({
  saved,
  routeSlug,
  readError,
  fetching,
  retry,
}: {
  saved: SavedListSummaryView;
  routeSlug: string;
  readError: Error | null;
  fetching: boolean;
  retry: () => void;
}) {
  const [form] = Form.useForm<EditListFormValues>();
  const client = useQueryClient();
  const lists = useSavedLists();
  const [edits, setEdits] = useState<Edits>({});
  const [review, setReview] = useState<Review>();
  const shown = useRef(saved);
  const nameDirty = !!edits.name;
  const descriptionDirty = !!edits.description;
  const tagsDirty = !!edits.tags;
  useLayoutEffect(() => {
    shown.current = {
      ...saved,
      name: nameDirty ? shown.current.name : saved.name,
      description: descriptionDirty ? shown.current.description : saved.description,
      tags: tagsDirty ? shown.current.tags : saved.tags,
    };
    form.setFieldsValue({
      ...(!nameDirty && { name: saved.name }),
      ...(!descriptionDirty && { description: saved.description ?? "" }),
      ...(!tagsDirty && { tags: saved.tags }),
    });
  }, [descriptionDirty, form, nameDirty, saved, tagsDirty]);
  const tagOptions = useMemo(
    () =>
      savedListTagOptions(lists.data?.lists ?? []).map((tag) => ({
        label: tag,
        value: tag,
      })),
    [lists.data?.lists],
  );

  async function refreshReview(conflicts: Field[]) {
    const loading: Review = { fields: conflicts, status: "loading" };
    setReview(loading);
    const query = listQuery(saved.list_key);
    try {
      await client.cancelQueries({ queryKey: query.queryKey, exact: true });
      const detail = await client.fetchQuery({ ...query, staleTime: 0, retry: false });
      setReview((current) =>
        current === loading
          ? { fields: conflicts, status: "ready", saved: detail.list }
          : current,
      );
    } catch {
      setReview((current) =>
        current === loading ? { fields: conflicts, status: "failed" } : current,
      );
    }
  }

  const update = useMutation({
    mutationFn: (request: UpdateSavedListRequest) =>
      updateSavedList(saved.list_key, request),
    retry: false,
    onSuccess: (view) =>
      invalidateSavedListQueries(client, [
        routeSlug,
        view.list.list_key,
        view.list.slug,
      ]),
  });
  const deleteListMutation = useMutation({
    mutationFn: () => deleteSavedList(saved.list_key),
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey: ["saved-lists"] });
      navigateDestination(listsDestination());
    },
  });

  function changeValues(changes: Partial<EditListFormValues>) {
    // Capture what the form showed even if a newer query response is now available.
    const baseline = shown.current;
    setEdits((previous) => {
      const next = { ...previous };
      if (changes.name !== undefined) {
        const expected = previous.name?.expected ?? baseline.name;
        next.name =
          changes.name === expected ? undefined : { expected, value: changes.name };
      }
      if (changes.description !== undefined) {
        const expected = previous.description
          ? previous.description.expected
          : (baseline.description ?? null);
        const value = normalizeOptionalText(changes.description) ?? null;
        const original = normalizeOptionalText(expected ?? undefined) ?? null;
        next.description = value === original ? undefined : { expected, value };
      }
      if (changes.tags !== undefined) {
        const expected = previous.tags?.expected ?? baseline.tags;
        const value = normalizeTags(changes.tags);
        next.tags = sameTags(value, expected) ? undefined : { expected, value };
      }
      return next;
    });
  }

  function resolveField(field: Field, useSaved: boolean, fresh: SavedListSummaryView) {
    setEdits((previous) => {
      const next = { ...previous };
      if (field === "name") {
        const edit = previous.name;
        next.name =
          useSaved || !edit || edit.value === fresh.name
            ? undefined
            : { ...edit, expected: fresh.name };
      } else if (field === "description") {
        const edit = previous.description;
        const expected = fresh.description ?? null;
        next.description =
          useSaved || !edit || edit.value === expected
            ? undefined
            : { ...edit, expected };
      } else {
        const edit = previous.tags;
        next.tags =
          useSaved || !edit || sameTags(edit.value, fresh.tags)
            ? undefined
            : { ...edit, expected: fresh.tags };
      }
      return next;
    });
    setReview((current) => {
      if (!current) return current;
      const remaining = current.fields.filter((candidate) => candidate !== field);
      return remaining.length ? { ...current, fields: remaining } : undefined;
    });
    update.reset();
    form.getFieldInstance(field)?.focus({ preventScroll: true });
  }

  const returnTo = listDestination({ slug: saved.slug, selectedRecordKey: null });
  return (
    <div className="list-edit-view__form">
      <Form
        form={form}
        layout="vertical"
        disabled={update.isPending}
        onValuesChange={changeValues}
        onFinish={() => {
          if (review || update.isPending) return;
          update.mutate(
            {
              ...(edits.name && { name: edits.name }),
              ...(edits.description && { description: edits.description }),
              ...(edits.tags && { tags: edits.tags }),
            },
            {
              onSuccess: (view) => {
                navigateDestination(
                  listDestination({ slug: view.list.slug, selectedRecordKey: null }),
                );
              },
              onError: (error) => {
                const conflict = savedEditConflictFields(error);
                const affected = fields.filter((field) => conflict?.includes(field));
                if (affected.length) void refreshReview(affected);
              },
            },
          );
        }}
      >
        <Form.Item
          label="Name"
          name="name"
          validateFirst
          rules={[{ required: true, whitespace: true, message: "Name is required." }]}
        >
          <Input />
        </Form.Item>
        <Form.Item label="Description" name="description">
          <Input.TextArea autoSize={{ minRows: 3, maxRows: 6 }} />
        </Form.Item>
        <Form.Item label="Tags" name="tags">
          <Select
            mode="tags"
            showSearch={{ optionFilterProp: "label" }}
            options={tagOptions}
            tokenSeparators={[","]}
          />
        </Form.Item>
        {review && (
          <SavedEditConflictReview
            fields={
              review.status === "ready"
                ? review.fields.map((field) => {
                    const fresh = review.saved;
                    const edit = edits[field];
                    const value = edit ? edit.value : saved[field];
                    return {
                      field,
                      label: labels[field],
                      saved: displaySetting(fresh[field]),
                      draft: displaySetting(value),
                      onUseSaved: () => resolveField(field, true, fresh),
                      onKeepMine: () => resolveField(field, false, fresh),
                    };
                  })
                : []
            }
            loading={review.status === "loading"}
            refreshFailed={review.status === "failed"}
            onRetry={() => void refreshReview(review.fields)}
          />
        )}
        <div className="list-edit-view__actions">
          <Button
            htmlType="submit"
            loading={update.isPending}
            disabled={!!review}
            type="primary"
          >
            Save
          </Button>
          <Button
            href={destinationHref(returnTo)}
            onClick={(event) => {
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              navigateDestination(returnTo);
            }}
          >
            Done
          </Button>
        </div>
      </Form>
      <dl className="list-edit-view__meta">
        <div>
          <dt>Created</dt>
          <dd>{formatDate(saved.created_at)}</dd>
        </div>
        <div>
          <dt>Updated</dt>
          <dd>{formatDate(saved.updated_at)}</dd>
        </div>
      </dl>
      <div className="list-edit-view__danger">
        <DangerActionButton
          confirmTitle={`Delete ${saved.name}?`}
          confirmContent="This permanently deletes the list and its saved membership."
          confirmOkText="Delete list"
          icon={<Trash2 size={16} />}
          loading={deleteListMutation.isPending}
          onConfirm={() => deleteListMutation.mutateAsync()}
        >
          Delete List
        </DangerActionButton>
      </div>
      {!review && readError && (
        <ErrorNotice
          error={readError}
          title="List unavailable"
          action={
            <Button loading={fetching} onClick={retry}>
              Retry list
            </Button>
          }
        />
      )}
      {!review && update.error && (
        <ErrorNotice error={update.error} title="List changes could not be confirmed" />
      )}
    </div>
  );
}

function displaySetting(value: string | string[] | null | undefined): string {
  return (Array.isArray(value) ? value.join(", ") : value) || "None";
}
