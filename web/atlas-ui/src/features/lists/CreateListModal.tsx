import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { isUnconfirmedMutation } from "../../api/apiError";
import { CheckListDirectory } from "./CheckListDirectory";
import { isSharedFailure } from "../../shared/feedback/errorMessages";
import { AtlasModal as Modal } from "../../shared/ui/overlays/AtlasModal";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Input, Select, Space, Typography } from "antd";
import { useMemo, useRef, useState } from "react";
import { createSavedList } from "../../api/atlasApi";
import type { CreateSavedListRequest } from "../../generated/atlas";
import { normalizeTags, savedListTagOptions } from "./listUtils";
import { useSavedLists } from "./savedListQueries";

type CreateListFormValues = {
  name: string;
  tags?: string[];
};

export function CreateListModal({
  onCancel,
  onCreated,
  open,
}: {
  onCancel: () => void;
  onCreated: (slug: string) => void;
  open: boolean;
}) {
  const paused = useSavedWritesPaused();
  const [form] = Form.useForm<CreateListFormValues>();
  const queryClient = useQueryClient();
  const inFlight = useRef(false);
  const lists = useSavedLists({ enabled: open, includeArchived: true });
  const tagOptions = useMemo(
    () =>
      savedListTagOptions(lists.data?.lists ?? []).map((tag) => ({
        label: tag,
        value: tag,
      })),
    [lists.data?.lists],
  );
  const [directoryChecked, setDirectoryChecked] = useState(false);
  const create = useMutation({
    mutationFn: (request: CreateSavedListRequest) => createSavedList(request),
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["saved-lists"] });
      onCreated(view.list.slug);
      form.resetFields();
    },
    onSettled: () => {
      inFlight.current = false;
    },
  });

  const uncertain = create.isError && isUnconfirmedMutation(create.error);
  const blocked = paused || (uncertain && !directoryChecked);
  return (
    <Modal
      transitionName=""
      confirmLoading={create.isPending}
      cancelButtonProps={{ disabled: create.isPending }}
      closable={{ disabled: create.isPending }}
      keyboard={!create.isPending}
      mask={{ closable: !create.isPending }}
      okText={uncertain ? "Create another list" : "Create list"}
      onCancel={() => {
        if (!inFlight.current) onCancel();
      }}
      okButtonProps={{ disabled: blocked }}
      onOk={() => {
        if (!blocked) form.submit();
      }}
      open={open}
      title="New list"
    >
      <Form
        form={form}
        layout="vertical"
        disabled={paused || create.isPending}
        onFinish={(values) => {
          if (inFlight.current || blocked) return;
          setDirectoryChecked(false);
          inFlight.current = true;
          create.mutate({
            name: values.name,
            tags: normalizeTags(values.tags),
          });
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
        <Form.Item
          label="Tags"
          name="tags"
          extra={
            lists.error &&
            !isSharedFailure(lists.error) && (
              <Space wrap>
                <Typography.Text type="secondary">
                  Tag suggestions could not load.
                </Typography.Text>
                <Button
                  size="small"
                  loading={lists.isFetching}
                  onClick={() => void lists.refetch()}
                >
                  Retry
                </Button>
              </Space>
            )
          }
        >
          <Select
            loading={lists.isFetching}
            mode="tags"
            showSearch={{ optionFilterProp: "label" }}
            options={tagOptions}
            tokenSeparators={[","]}
          />
        </Form.Item>
      </Form>
      {create.error && (
        <ErrorNotice
          error={create.error}
          outcome={uncertain ? "uncertain" : undefined}
          title={
            uncertain
              ? "List creation could not be confirmed"
              : "List could not be created"
          }
          description={
            uncertain
              ? "Check the list directory before creating another list."
              : undefined
          }
          action={
            uncertain && (
              <CheckListDirectory onChecked={() => setDirectoryChecked(true)} />
            )
          }
        />
      )}
    </Modal>
  );
}
