import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Input, Modal, Select } from "antd";
import { useEffect } from "react";
import { createEncounter } from "../../api/atlasApi";
import type {
  EncounterStatusView,
  EncounterSummaryView,
  UpdateEncounterRequest,
} from "../../generated/atlas";

type CreateEncounterForm = {
  name: string;
};

type EditEncounterFormValues = {
  name: string;
  slug: string;
  description?: string;
  note?: string;
  status: EncounterStatusView;
};

export function CreateEncounterModal({
  open,
  onCancel,
  onCreated,
}: {
  open: boolean;
  onCancel: () => void;
  onCreated: (slug: string) => void;
}) {
  const [form] = Form.useForm<CreateEncounterForm>();
  const queryClient = useQueryClient();
  const mutation = useMutation({
    mutationFn: (values: CreateEncounterForm) => createEncounter({ name: values.name }),
    onSuccess: async (view) => {
      await queryClient.invalidateQueries({ queryKey: ["encounters"] });
      form.resetFields();
      onCreated(view.encounter.slug);
    },
  });
  return (
    <Modal
      title="New encounter"
      open={open}
      onCancel={onCancel}
      onOk={() => form.submit()}
    >
      <Form form={form} layout="vertical" onFinish={(v) => mutation.mutate(v)}>
        <Form.Item name="name" label="Name" rules={[{ required: true }]}>
          <Input />
        </Form.Item>
      </Form>
    </Modal>
  );
}

export function EditEncounterModal({
  encounter,
  encounterNote,
  open,
  onCancel,
  onSave,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  open: boolean;
  onCancel: () => void;
  onSave: (encounter: UpdateEncounterRequest) => void;
}) {
  return (
    <Modal title="Edit encounter" open={open} onCancel={onCancel} footer={null}>
      <EditEncounterForm
        encounter={encounter}
        encounterNote={encounterNote}
        onCancel={onCancel}
        onSave={onSave}
      />
    </Modal>
  );
}

export function EditEncounterForm({
  encounter,
  encounterNote,
  onCancel,
  onSave,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  onCancel?: () => void;
  onSave: (encounter: UpdateEncounterRequest) => void;
}) {
  const [form] = Form.useForm<EditEncounterFormValues>();
  useEffect(() => {
    form.setFieldsValue({
      name: encounter.name,
      slug: encounter.slug,
      description: encounter.description,
      note: encounterNote,
      status: encounter.status,
    });
  }, [encounter, encounterNote, form]);
  return (
    <Form
      form={form}
      layout="vertical"
      onFinish={(values) =>
        onSave({
          encounter_key: encounter.encounter_key,
          slug: values.slug,
          name: values.name,
          ...(values.description ? { description: values.description } : {}),
          ...(values.note ? { note: values.note } : {}),
          status: values.status,
        })
      }
    >
      <Form.Item name="name" label="Name" rules={[{ required: true }]}>
        <Input />
      </Form.Item>
      <Form.Item name="slug" label="Slug" rules={[{ required: true }]}>
        <Input />
      </Form.Item>
      <Form.Item name="description" label="Description">
        <Input />
      </Form.Item>
      <Form.Item name="note" label="Note">
        <Input.TextArea />
      </Form.Item>
      <Form.Item name="status" label="Status" rules={[{ required: true }]}>
        <Select<EncounterStatusView>
          options={["draft", "running", "complete", "archived"].map((value) => ({
            value: value as EncounterStatusView,
            label: value,
          }))}
        />
      </Form.Item>
      <div className="encounter-actions">
        {onCancel ? <Button onClick={onCancel}>Cancel</Button> : null}
        <Button type="primary" onClick={() => form.submit()}>
          Save
        </Button>
      </div>
    </Form>
  );
}
