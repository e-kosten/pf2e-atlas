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
  onArchive,
  open,
  onCancel,
  onComplete,
  onDelete,
  onSave,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  onArchive?: () => void;
  open: boolean;
  onCancel: () => void;
  onComplete?: () => void;
  onDelete?: () => void;
  onSave: (encounter: UpdateEncounterRequest) => void;
}) {
  return (
    <Modal title="Edit encounter" open={open} onCancel={onCancel} footer={null}>
      <EditEncounterForm
        encounter={encounter}
        encounterNote={encounterNote}
        onArchive={onArchive}
        onCancel={onCancel}
        onComplete={onComplete}
        onDelete={onDelete}
        onSave={onSave}
      />
    </Modal>
  );
}

export function EditEncounterForm({
  encounter,
  encounterNote,
  onArchive,
  onCancel,
  onComplete,
  onDelete,
  onSave,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  onArchive?: () => void;
  onCancel?: () => void;
  onComplete?: () => void;
  onDelete?: () => void;
  onSave: (encounter: UpdateEncounterRequest) => void;
}) {
  const [form] = Form.useForm<EditEncounterFormValues>();
  useEffect(() => {
    form.setFieldsValue({
      name: encounter.name,
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
          slug: encounter.slug,
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
        {onComplete ? <Button onClick={onComplete}>Complete</Button> : null}
        {onArchive ? <Button onClick={onArchive}>Archive</Button> : null}
        {onDelete ? (
          <Button danger onClick={onDelete}>
            Delete
          </Button>
        ) : null}
        <Button type="primary" onClick={() => form.submit()}>
          Save
        </Button>
      </div>
    </Form>
  );
}
