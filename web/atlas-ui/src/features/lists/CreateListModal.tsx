import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Form, Input, Modal } from "antd";
import { createSavedList } from "../../api/atlasApi";
import type { CreateSavedListRequest } from "../../generated/atlas";
import { InlineError, slugify } from "./listUtils";

type CreateListFormValues = {
  name: string;
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
