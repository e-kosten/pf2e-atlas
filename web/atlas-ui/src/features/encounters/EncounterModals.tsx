import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { encounterStatusLabels } from "./encounterStatus";
import { EncounterModal } from "./EncounterModal";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Input, InputNumber, Popconfirm, Select } from "antd";
import { useEffect, useId, useRef } from "react";
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
  roundDurationSeconds: number;
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
  const formId = useId();
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
    <EncounterModal
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
        name={formId}
        disabled={mutation.isPending}
        form={form}
        layout="vertical"
        onFinish={(v) => {
          if (!mutation.isPending) mutation.mutate(v);
        }}
      >
        {mutation.error && (
          <ErrorNotice error={mutation.error} title="Encounter could not be created" />
        )}
        <Form.Item name="name" label="Name" rules={[{ required: true }]}>
          <Input />
        </Form.Item>
      </Form>
    </EncounterModal>
  );
}

export function EditEncounterModal({
  encounter,
  encounterNote,
  open,
  onCancel,
  onDelete,
  onSave,
  disabled = false,
  pending = false,
  error,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  open: boolean;
  onCancel: () => void;
  onDelete?: () => Promise<unknown>;
  onSave: (encounter: UpdateEncounterRequest) => Promise<unknown>;
  disabled?: boolean;
  pending?: boolean;
  error?: string;
}) {
  const formId = useId();
  return (
    <EncounterModal
      title="Settings"
      open={open}
      destroyOnHidden
      onCancel={onCancel}
      footer={
        <EncounterEditActions
          name={encounter.name}
          formId={formId}
          onCancel={onCancel}
          onDelete={onDelete}
          disabled={disabled}
          pending={pending}
        />
      }
      closable={!pending}
      keyboard={!pending}
      mask={{ closable: !pending }}
    >
      <EditEncounterForm
        externalFormId={formId}
        hideActions
        encounter={encounter}
        disabled={disabled}
        pending={pending}
        error={error}
        encounterNote={encounterNote}
        onCancel={onCancel}
        onDelete={onDelete}
        onSave={onSave}
      />
    </EncounterModal>
  );
}

export function EditEncounterForm({
  encounter,
  encounterNote,
  onCancel,
  onDelete,
  onSave,
  disabled = false,
  pending = false,
  error,
  externalFormId,
  hideActions = false,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  onCancel?: () => void;
  onDelete?: () => Promise<unknown>;
  onSave: (encounter: UpdateEncounterRequest) => Promise<unknown>;
  disabled?: boolean;
  pending?: boolean;
  error?: string;
  externalFormId?: string;
  hideActions?: boolean;
}) {
  const generatedFormId = useId();
  const formId = externalFormId ?? generatedFormId;
  const [form] = Form.useForm<EditEncounterFormValues>();
  const initializedKey = useRef<string | null>(null);
  useEffect(() => {
    const values = {
      name: encounter.name,
      roundDurationSeconds: Number(encounter.round_duration_seconds),
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
      name={formId}
      id={formId}
      disabled={disabled}
      form={form}
      layout="vertical"
      onFinish={async (values) => {
        if (disabled) return;
        try {
          await onSave({
            encounter_key: encounter.encounter_key,
            ...(form.isFieldTouched("roundDurationSeconds")
              ? { round_duration_seconds: BigInt(values.roundDurationSeconds) }
              : {}),
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
        <ErrorNotice
          error={error}
          title="Encounter change could not be confirmed"
          description={error}
        />
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
      <Form.Item
        name="roundDurationSeconds"
        label="Seconds per round"
        rules={[
          { required: true, type: "number", min: 1, max: 86400, transform: Number },
          {
            validator: (_, value) =>
              Number.isSafeInteger(value)
                ? Promise.resolve()
                : Promise.reject(new Error("Enter a whole number of seconds.")),
          },
        ]}
      >
        <InputNumber min={1} max={86400} precision={0} />
      </Form.Item>
      <Form.Item name="status" label="Status" rules={[{ required: true }]}>
        <Select<EncounterStatusView>
          options={["draft", "running", "complete", "archived"].map((value) => ({
            value: value as EncounterStatusView,
            label: encounterStatusLabels[value as EncounterStatusView],
          }))}
        />
      </Form.Item>
      {!hideActions && (
        <EncounterEditActions
          name={encounter.name}
          formId={formId}
          onCancel={onCancel}
          onDelete={onDelete}
          disabled={disabled}
          pending={pending}
        />
      )}
    </Form>
  );
}

function EncounterEditActions({
  name,
  formId,
  onCancel,
  onDelete,
  disabled,
  pending,
}: {
  name: string;
  formId: string;
  onCancel?: () => void;
  onDelete?: () => Promise<unknown>;
  disabled: boolean;
  pending: boolean;
}) {
  return (
    <div className="encounter-actions">
      {onCancel ? (
        <Button disabled={pending} onClick={onCancel}>
          Cancel
        </Button>
      ) : null}
      {onDelete ? (
        <Popconfirm
          title={`Delete ${name}?`}
          description="This permanently deletes the encounter."
          placement="topLeft"
          okText="Delete"
          cancelText="Cancel"
          okButtonProps={{ danger: true, loading: pending }}
          cancelButtonProps={{ disabled: pending }}
          disabled={disabled}
          onConfirm={onDelete}
        >
          <Button disabled={disabled} danger>
            Delete
          </Button>
        </Popconfirm>
      ) : null}
      <Button
        disabled={disabled}
        loading={pending}
        type="primary"
        htmlType="submit"
        form={formId}
      >
        Save
      </Button>
    </div>
  );
}
