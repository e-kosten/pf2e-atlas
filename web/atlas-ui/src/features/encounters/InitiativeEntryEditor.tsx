import { Alert, Button, Form, Input, InputNumber } from "antd";
import { useTabCloseGuard } from "../../app/DesktopTabContext";
import type { AddEncounterInitiativeEntryRequest } from "../../generated/atlas";
import { MarkdownDocument } from "../../shared/documents/MarkdownDocument";
import { MarkdownEditor } from "../../shared/documents/MarkdownEditor";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { EncounterModal } from "./EncounterModal";

export type InitiativeEntryFields = Omit<
  AddEncounterInitiativeEntryRequest,
  "expected_revision" | "request_id"
>;

export function InitiativeEntryEditor({
  initial,
  disabled,
  error,
  conflict,
  onSave,
  onClose,
}: {
  initial?: InitiativeEntryFields;
  disabled: boolean;
  error?: string;
  conflict?: { latest: InitiativeEntryFields; keepDraft: () => void };
  onSave: (values: InitiativeEntryFields) => Promise<boolean>;
  onClose: () => void;
}) {
  const [form] = Form.useForm<InitiativeEntryFields>();
  useTabCloseGuard(() =>
    disabled ? "pending" : form.isFieldsTouched() ? "dirty" : undefined,
  );
  return (
    <EncounterModal
      open
      width={800}
      title={initial ? "Edit custom turn" : "Add custom turn"}
      okText={initial ? "Save" : "Add"}
      confirmLoading={disabled}
      okButtonProps={{ disabled: !!conflict }}
      cancelButtonProps={{ disabled }}
      closable={!disabled}
      keyboard={!disabled}
      mask={{ closable: false }}
      onCancel={onClose}
      onOk={() => form.submit()}
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={initial}
        disabled={disabled}
        onFinish={async (values) => {
          try {
            if (!conflict && (await onSave(values))) onClose();
          } catch {
            // The encounter mutation owner reports the failure.
          }
        }}
      >
        {conflict && (
          <Alert
            type="warning"
            title="This custom turn changed while you were editing"
            description={
              <>
                <p>
                  Saved custom turn: {conflict.latest.display_name} (initiative{" "}
                  {conflict.latest.initiative})
                </p>
                {conflict.latest.note && (
                  <MarkdownDocument source={conflict.latest.note} />
                )}
                <Button onClick={conflict.keepDraft}>Keep my draft</Button>
              </>
            }
          />
        )}
        {!conflict && error && (
          <ErrorNotice error={error} title="Custom turn could not be saved" />
        )}
        <div className="initiative-entry-editor__fields">
          <Form.Item
            name="display_name"
            label="Name"
            rules={[{ required: true, whitespace: true }]}
          >
            <Input />
          </Form.Item>
          <Form.Item
            name="initiative"
            label="Initiative"
            rules={[
              { required: true },
              {
                type: "integer",
                min: Number.MIN_SAFE_INTEGER,
                max: Number.MAX_SAFE_INTEGER,
                message: "Enter a whole number.",
              },
            ]}
          >
            <InputNumber />
          </Form.Item>
        </div>
        <Form.Item name="note" label="Notes">
          <MarkdownEditor disabled={disabled} roomy />
        </Form.Item>
      </Form>
    </EncounterModal>
  );
}
