import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Form, Input, Modal, Select } from "antd";
import { useMemo } from "react";
import { createSavedList } from "../../api/atlasApi";
import type { CreateSavedListRequest } from "../../generated/atlas";
import { InlineError, normalizeTags, savedListTagOptions, slugify } from "./listUtils";
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
            tags: normalizeTags(values.tags),
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
        <Form.Item label="Tags" name="tags">
          <Select
            mode="tags"
            optionFilterProp="label"
            options={tagOptions}
            showSearch
            tokenSeparators={[","]}
          />
        </Form.Item>
      </Form>
      {create.error && <InlineError message={create.error.message} />}
    </Modal>
  );
}
