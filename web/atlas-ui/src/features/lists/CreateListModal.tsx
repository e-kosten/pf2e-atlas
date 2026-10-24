import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Modal, Select, Space, Typography } from "antd";
import { useMemo, useRef } from "react";
import { createSavedList } from "../../api/atlasApi";
import type { CreateSavedListRequest } from "../../generated/atlas";
import { normalizeTags, savedListTagOptions, slugify } from "./listUtils";
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
  const [form] = Form.useForm<CreateListFormValues>();
  const queryClient = useQueryClient();
  const inFlight = useRef(false);
  const lists = useSavedLists({ enabled: open });
  const tagOptions = useMemo(
    () =>
      savedListTagOptions(lists.data?.lists ?? []).map((tag) => ({
        label: tag,
        value: tag,
      })),
    [lists.data?.lists],
  );
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

  return (
    <Modal
      transitionName=""
      confirmLoading={create.isPending}
      cancelButtonProps={{ disabled: create.isPending }}
      closable={{ disabled: create.isPending }}
      keyboard={!create.isPending}
      mask={{ closable: !create.isPending }}
      okText="Create"
      onCancel={() => {
        if (!inFlight.current) onCancel();
      }}
      onOk={() => form.submit()}
      open={open}
      title="New List"
    >
      <Form
        form={form}
        layout="vertical"
        disabled={create.isPending}
        onFinish={(values) => {
          if (inFlight.current) return;
          inFlight.current = true;
          create.mutate({
            slug: slugify(values.name),
            name: values.name,
            tags: normalizeTags(values.tags),
          });
        }}
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
        <Form.Item
          label="Tags"
          name="tags"
          extra={
            lists.error && (
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
      {create.error && <Alert type="error" title={create.error.message} />}
    </Modal>
  );
}
