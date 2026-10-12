import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Modal, Select } from "antd";
import { useEffect, useRef } from "react";
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
      confirmLoading={mutation.isPending}
      cancelButtonProps={{ disabled: mutation.isPending }}
      closable={!mutation.isPending}
      keyboard={!mutation.isPending}
      mask={{ closable: !mutation.isPending }}
      onOk={() => form.submit()}
    >
      <Form
        disabled={mutation.isPending}
        form={form}
        layout="vertical"
        onFinish={(v) => {
          if (!mutation.isPending) mutation.mutate(v);
        }}
      >
        {mutation.error && (
          <Alert
            type="error"
            title="Encounter could not be created"
            description={mutation.error.message}
          />
        )}
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
  disabled = false,
  pending = false,
  error,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  onArchive?: () => Promise<unknown>;
  open: boolean;
  onCancel: () => void;
  onComplete?: () => Promise<unknown>;
  onDelete?: () => void;
  onSave: (encounter: UpdateEncounterRequest) => Promise<unknown>;
  disabled?: boolean;
  pending?: boolean;
  error?: string;
}) {
  return (
    <Modal
      title="Edit encounter"
      open={open}
      destroyOnHidden
      onCancel={onCancel}
      footer={null}
      closable={!pending}
      keyboard={!pending}
      mask={{ closable: !pending }}
    >
      <EditEncounterForm
        encounter={encounter}
        disabled={disabled}
        pending={pending}
        error={error}
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
  disabled = false,
  pending = false,
  error,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  onArchive?: () => Promise<unknown>;
  onCancel?: () => void;
  onComplete?: () => Promise<unknown>;
  onDelete?: () => void;
  onSave: (encounter: UpdateEncounterRequest) => Promise<unknown>;
  disabled?: boolean;
  pending?: boolean;
  error?: string;
}) {
  const [form] = Form.useForm<EditEncounterFormValues>();
  const initializedKey = useRef<string | null>(null);
  useEffect(() => {
    const values = {
      name: encounter.name,
      description: encounter.description,
      note: encounterNote,
      status: encounter.status,
    };
    if (initializedKey.current !== encounter.encounter_key) {
      form.resetFields();
      form.setFieldsValue(values);
      form.setFields(
        (Object.keys(values) as Array<keyof EditEncounterFormValues>).map((name) => ({
          name,
          touched: false,
        })),
      );
      initializedKey.current = encounter.encounter_key;
    } else {
      for (const field of Object.keys(values) as Array<keyof EditEncounterFormValues>) {
        if (!form.isFieldTouched(field)) form.setFieldValue(field, values[field]);
      }
    }
  }, [encounter, encounterNote, form]);
  return (
    <Form
      disabled={disabled}
      form={form}
      layout="vertical"
      onFinish={async (values) => {
        if (disabled) return;
        try {
          await onSave({
            encounter_key: encounter.encounter_key,
            slug: encounter.slug,
            name: values.name,
            ...(values.description ? { description: values.description } : {}),
            ...(values.note ? { note: values.note } : {}),
            status: values.status,
          });
          form.setFields(
            (Object.keys(values) as Array<keyof EditEncounterFormValues>).map(
              (name) => ({ name, touched: false }),
            ),
          );
        } catch {
          /* The owner displays the failure; keep this draft. */
        }
      }}
    >
      {error && (
        <Alert type="error" title="Encounter action failed" description={error} />
      )}
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
        {onCancel ? (
          <Button disabled={pending} onClick={onCancel}>
            Cancel
          </Button>
        ) : null}
        {onComplete ? (
          <Button disabled={disabled} onClick={() => void onComplete().catch(() => {})}>
            Complete
          </Button>
        ) : null}
        {onArchive ? (
          <Button disabled={disabled} onClick={() => void onArchive().catch(() => {})}>
            Archive
          </Button>
        ) : null}
        {onDelete ? (
          <Button disabled={disabled} danger onClick={onDelete}>
            Delete
          </Button>
        ) : null}
        <Button
          disabled={disabled}
          loading={pending}
          type="primary"
          onClick={() => form.submit()}
        >
          Save
        </Button>
      </div>
    </Form>
  );
}
