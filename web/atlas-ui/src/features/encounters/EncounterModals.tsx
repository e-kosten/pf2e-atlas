import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { SavedEditConflictReview } from "../../shared/feedback/SavedEditConflictReview";
import {
  useEncounterSettingsDraft,
  type EncounterSettingsValues,
} from "./useEncounterSettingsDraft";
import { encounterStatusLabels } from "./encounterStatus";
import { EncounterModal } from "./EncounterModal";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Input, InputNumber, Select } from "antd";
import type { FormInstance } from "antd";
import {
  useId,
  useImperativeHandle,
  useRef,
  useState,
  type ReactNode,
  type Ref,
} from "react";
import { formatDate } from "../../shared/ui/formatDate";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { createEncounter } from "../../api/atlasApi";
import type {
  EncounterDetailView,
  EncounterStatusView,
  EncounterSummaryView,
  UpdateEncounterRequest,
} from "../../generated/atlas";

type CreateEncounterForm = {
  name: string;
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
  onRefreshConflict,
  disabled = false,
  pending = false,
  error,
  feedback,
  afterClose,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  open: boolean;
  onCancel: () => void;
  onDelete?: () => Promise<unknown>;
  onSave: (encounter: UpdateEncounterRequest) => Promise<unknown>;
  onRefreshConflict: () => Promise<EncounterDetailView | undefined>;
  disabled?: boolean;
  pending?: boolean;
  error?: string;
  feedback?: ReactNode;
  afterClose?: () => void;
}) {
  const formId = useId();
  const hasDraft = useRef<() => boolean>(null);
  const [saveBlocked, setSaveBlocked] = useState(false);
  const cancel = () => {
    setSaveBlocked(false);
    onCancel();
  };
  return (
    <EncounterModal
      title="Settings"
      open={open}
      destroyOnHidden
      onCancel={cancel}
      afterClose={afterClose}
      confirmDiscard={{ when: () => hasDraft.current?.() ?? false, onDiscard: cancel }}
      confirmLoading={pending}
      cancelButtonProps={{ disabled: pending }}
      footer={(_, { CancelBtn }) => (
        <EncounterEditActions
          name={encounter.name}
          formId={formId}
          cancelButton={<CancelBtn />}
          onDelete={onDelete}
          disabled={disabled}
          pending={pending}
          saveBlocked={saveBlocked}
        />
      )}
      closable={!pending}
      keyboard={!pending}
      mask={{ closable: !pending }}
    >
      {feedback}
      <>
        <EditEncounterForm
          key={encounter.encounter_key}
          externalFormId={formId}
          hideActions
          draftCheck={hasDraft}
          onConflictChange={setSaveBlocked}
          onRefreshConflict={onRefreshConflict}
          encounter={encounter}
          disabled={disabled}
          pending={pending}
          error={error}
          encounterNote={encounterNote}
          onCancel={onCancel}
          onDelete={onDelete}
          onSave={onSave}
        />
        <div className="encounter-settings__dates">
          <span>Created {formatDate(encounter.created_at)}</span>
          <span>Updated {formatDate(encounter.updated_at)}</span>
        </div>
      </>
    </EncounterModal>
  );
}

export function EditEncounterForm({
  encounter,
  encounterNote,
  onCancel,
  onDelete,
  onSave,
  onRefreshConflict,
  disabled = false,
  pending = false,
  error,
  externalFormId,
  onConflictChange,
  hideActions = false,
  draftCheck,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  onCancel?: () => void;
  onDelete?: () => Promise<unknown>;
  onSave: (encounter: UpdateEncounterRequest) => Promise<unknown>;
  onRefreshConflict: () => Promise<EncounterDetailView | undefined>;
  disabled?: boolean;
  pending?: boolean;
  error?: string;
  externalFormId?: string;
  onConflictChange?: (blocked: boolean) => void;
  hideActions?: boolean;
  draftCheck?: Ref<() => boolean>;
}) {
  const generatedFormId = useId();
  const formId = externalFormId ?? generatedFormId;
  const [form] = Form.useForm<EncounterSettingsValues>();
  const draft = useEncounterSettingsDraft({
    encounter,
    encounterNote,
    form,
    onSave,
    onRefreshConflict,
    onConflictChange,
  });
  useImperativeHandle(draftCheck, () => draft.hasChanges, [draft.hasChanges]);
  return (
    <>
      {draft.conflict && <EncounterSettingsConflict draft={draft} form={form} />}
      <Form
        name={formId}
        id={formId}
        disabled={disabled}
        form={form}
        layout="vertical"
        onFinish={(values) => {
          if (!disabled) void draft.submit(values);
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
          name="round_duration_seconds"
          label="Round duration"
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
          <InputNumber
            min={1}
            max={86400}
            precision={0}
            suffix={<span>seconds</span>}
            aria-describedby={`${formId}-duration-unit`}
            style={{ width: 144 }}
          />
        </Form.Item>
        <span id={`${formId}-duration-unit`} hidden>
          In seconds
        </span>
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
            saveBlocked={!!draft.conflict}
          />
        )}
      </Form>
    </>
  );
}

function EncounterSettingsConflict({
  draft,
  form,
}: {
  draft: ReturnType<typeof useEncounterSettingsDraft>;
  form: FormInstance<EncounterSettingsValues>;
}) {
  Form.useWatch(
    (values: EncounterSettingsValues) =>
      draft.conflict?.fields.map((field) => values[field]),
    form,
  );
  return (
    <SavedEditConflictReview
      fields={draft.reviewFields()}
      loading={draft.conflict?.loading}
      refreshFailed={!draft.conflict?.loading && !draft.conflict?.saved}
      onRetry={draft.retryConflict}
    />
  );
}

function EncounterEditActions({
  name,
  formId,
  onCancel,
  cancelButton,
  onDelete,
  disabled,
  pending,
  saveBlocked,
}: {
  name: string;
  formId: string;
  onCancel?: () => void;
  cancelButton?: ReactNode;
  onDelete?: () => Promise<unknown>;
  disabled: boolean;
  pending: boolean;
  saveBlocked: boolean;
}) {
  return (
    <div className="encounter-actions">
      {cancelButton ??
        (onCancel ? (
          <Button disabled={pending} onClick={onCancel}>
            Cancel
          </Button>
        ) : null)}
      {onDelete ? (
        <DangerActionButton
          confirmTitle={`Delete ${name}?`}
          confirmContent="This permanently deletes the encounter."
          confirmOkText="Delete"
          disabled={disabled}
          onConfirm={onDelete}
        >
          Delete
        </DangerActionButton>
      ) : null}
      <Button
        disabled={disabled || saveBlocked}
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
