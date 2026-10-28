import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
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
import { useEffect, useMemo, useRef } from "react";
import { deleteSavedList, getSavedList, updateSavedList } from "../../api/atlasApi";
import type { UpdateSavedListRequest } from "../../generated/atlas";
import { normalizeOptionalText, normalizeTags, savedListTagOptions } from "./listUtils";
import { useSavedLists } from "./savedListQueries";
import { formatDate } from "../../shared/ui/formatDate";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
type ListEditViewProps = {
  route: { slug: string };
};
type EditListFormValues = {
  name: string;
  description?: string;
  tags?: string[];
};
export function ListEditView({ route }: ListEditViewProps) {
  const [form] = Form.useForm<EditListFormValues>();
  const queryClient = useQueryClient();
  const lists = useSavedLists();
  const list = useQuery({
    queryKey: ["saved-list", route.slug],
    queryFn: () => getSavedList(route.slug),
  });
  usePageTitle(list.data?.list.name ?? "List");
  const tagOptions = useMemo(
    () =>
      savedListTagOptions(lists.data?.lists ?? []).map((tag) => ({
        label: tag,
        value: tag,
      })),
    [lists.data?.lists],
  );
  const update = useMutation({
    mutationFn: (request: UpdateSavedListRequest) => updateSavedList(request),
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      await queryClient.invalidateQueries({ queryKey: ["saved-list", route.slug] });
      await queryClient.invalidateQueries({ queryKey: ["saved-list", view.list.slug] });
      navigateDestination(
        listDestination({
          slug: view.list.slug,
          selectedRecordKey: null,
        }),
      );
    },
  });
  const deleteListMutation = useMutation({
    mutationFn: () => deleteSavedList(list.data!.list.list_key),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      navigateDestination(listsDestination());
    },
  });
  const initializedKey = useRef<string | null>(null);
  useEffect(() => {
    if (!list.data) {
      return;
    }
    const values = {
      name: list.data.list.name,
      description: list.data.list.description ?? undefined,
      tags: list.data.list.tags,
    };
    if (initializedKey.current !== list.data.list.list_key) {
      form.resetFields();
      form.setFieldsValue(values);
      form.setFields(
        (Object.keys(values) as Array<keyof EditListFormValues>).map((name) => ({
          name,
          touched: false,
        })),
      );
      initializedKey.current = list.data.list.list_key;
    } else {
      for (const field of Object.keys(values) as Array<keyof EditListFormValues>) {
        if (!form.isFieldTouched(field)) form.setFieldValue(field, values[field]);
      }
    }
  }, [form, list.data]);
  return (
    <main className="list-edit-view">
      <section className="list-edit-view__header">
        <Typography.Link
          className="list-edit-view__back-link"
          href={destinationHref(
            listDestination({ slug: route.slug, selectedRecordKey: null }),
          )}
          onClick={(event) => {
            if (!isPrimaryActivation(event)) return;
            event.preventDefault();
            navigateDestination(
              listDestination({
                slug: route.slug,
                selectedRecordKey: null,
              }),
            );
          }}
        >
          <ArrowLeft size={14} aria-hidden="true" />
          Back to list
        </Typography.Link>
        <div>
          <h2>Edit list</h2>
          {list.data && <p>{list.data.list.name}</p>}
        </div>
      </section>
      <section className="list-edit-view__body">
        {list.isLoading ? (
          <div className="detail-empty">Loading list</div>
        ) : !list.data ? (
          <div className="detail-empty">
            {list.error ? "List unavailable" : "List not found"}
          </div>
        ) : (
          <div className="list-edit-view__form">
            <Form
              form={form}
              layout="vertical"
              onFinish={(values) =>
                update.mutate({
                  list_key: list.data!.list.list_key,
                  slug: list.data!.list.slug,
                  name: values.name,
                  description: normalizeOptionalText(values.description),
                  tags: normalizeTags(values.tags),
                })
              }
            >
              <Form.Item
                label="Name"
                name="name"
                validateFirst
                rules={[
                  { required: true, whitespace: true, message: "Name is required." },
                ]}
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
              <div className="list-edit-view__actions">
                <Button htmlType="submit" loading={update.isPending} type="primary">
                  Save
                </Button>
                <Button
                  href={destinationHref(
                    listDestination({
                      slug: list.data.list.slug,
                      selectedRecordKey: null,
                    }),
                  )}
                  onClick={(event) => {
                    if (!isPrimaryActivation(event)) {
                      return;
                    }
                    event.preventDefault();
                    navigateDestination(
                      listDestination({
                        slug: list.data!.list.slug,
                        selectedRecordKey: null,
                      }),
                    );
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
              <DangerActionButton
                confirmTitle={`Delete ${list.data.list.name}?`}
                confirmContent="This permanently deletes the list and its saved membership."
                confirmOkText="Delete list"
                icon={<Trash2 size={16} />}
                loading={deleteListMutation.isPending}
                onConfirm={() => deleteListMutation.mutateAsync()}
              >
                Delete List
              </DangerActionButton>
            </div>
          </div>
        )}
        {list.error && (
          <>
            <ErrorNotice error={list.error} title="List unavailable" />
            <Button loading={list.isFetching} onClick={() => void list.refetch()}>
              Retry list
            </Button>
          </>
        )}
        {update.error && (
          <ErrorNotice
            error={update.error}
            title="List changes could not be confirmed"
          />
        )}
      </section>
    </main>
  );
}
