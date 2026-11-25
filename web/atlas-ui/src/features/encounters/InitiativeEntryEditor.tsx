import { Alert, Button, Form, Input, InputNumber, type GetRef } from "antd";
import { useEffectEvent, useLayoutEffect, useRef, useState } from "react";
import type { AddEncounterInitiativeEntryRequest } from "../../generated/atlas";
import { MarkdownDocument } from "../../shared/documents/MarkdownDocument";
import { MarkdownEditor } from "../../shared/documents/MarkdownEditor";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { EncounterModal } from "./EncounterModal";

export type InitiativeEntryFields = Omit<
  AddEncounterInitiativeEntryRequest,
  "expected_revision" | "request_id"
>;

function textValue(note?: string) {
  return note?.trim() ? note : "";
}

export function InitiativeEntryEditor({
  initial,
  disabled,
  pending,
  error,
  conflict,
  onSave,
  onClose,
  getDismissFocus,
}: {
  initial?: InitiativeEntryFields;
  disabled: boolean;
  pending: boolean;
  error?: string;
  conflict?: { latest: InitiativeEntryFields; keepDraft: () => void };
  onSave: (values: InitiativeEntryFields) => Promise<boolean>;
  onClose: () => void;
  getDismissFocus: () => { opener: HTMLElement | null; pane: HTMLElement | null };
}) {
  const [form] = Form.useForm<InitiativeEntryFields>();
  const formElement = useRef<GetRef<typeof Form<InitiativeEntryFields>>>(null);
  const baseline = useRef(initial);
  const [saved, setSaved] = useState(false);
  const hasDraft = () => {
    const values = form.getFieldsValue();
    const original = baseline.current;
    return (
      textValue(values.display_name) !== textValue(original?.display_name) ||
      (values.initiative ?? null) !== (original?.initiative ?? null) ||
      textValue(values.note) !== textValue(original?.note)
    );
  };
  const close = () => {
    restoreOverlayFocus(() => {
      const { opener, pane } = getDismissFocus();
      return (
        [opener, pane, pane?.closest<HTMLElement>("#atlas-main")].find(
          (element) =>
            element?.isConnected &&
            !element.matches(":disabled") &&
            !element.closest("[inert]") &&
            element.getClientRects().length > 0,
        ) ?? null
      );
    }, formElement.current?.nativeElement?.closest<HTMLElement>('[role="dialog"]'));
    onClose();
  };
  const closeAfterSave = useEffectEvent(close);
  useLayoutEffect(() => {
    // The mutation can resolve before the parent enables the opener again.
    if (saved && !pending) closeAfterSave();
  }, [saved, pending]);
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
      focusable={{ focusTriggerAfterClose: false }}
      keyboard={!disabled}
      mask={{ closable: false }}
      confirmDiscard={{ when: hasDraft, onDiscard: close }}
      onCancel={close}
      onOk={() => form.submit()}
    >
      <Form<InitiativeEntryFields>
        ref={formElement}
        form={form}
        layout="vertical"
        className="initiative-entry-editor"
        initialValues={initial}
        disabled={disabled}
        onFinish={async (values) => {
          try {
            if (!conflict && (await onSave(values))) setSaved(true);
          } catch {
            // The encounter mutation owner reports the failure.
          }
        }}
      >
        <>
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
        </>
        <Form.Item name="note" label="Notes" style={{ marginBottom: 0 }}>
          <MarkdownEditor label="Notes" disabled={disabled} />
        </Form.Item>
      </Form>
    </EncounterModal>
  );
}
