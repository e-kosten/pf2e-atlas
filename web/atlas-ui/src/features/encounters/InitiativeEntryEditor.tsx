import { Alert, Button, Form, Input, InputNumber } from "antd";
import { useId, useLayoutEffect, useRef, useState } from "react";
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
  const [expanded, setExpanded] = useState(false);
  const noteId = useId();
  const expandButton = useRef<HTMLButtonElement>(null);
  const collapseLabel = useRef<HTMLSpanElement>(null);
  const restoreToggleFocus = useRef(false);
  useLayoutEffect(() => {
    if (expanded) {
      collapseLabel.current?.closest("button")?.focus({ preventScroll: true });
    } else if (restoreToggleFocus.current) {
      restoreToggleFocus.current = false;
      expandButton.current?.focus({ preventScroll: true });
    }
  }, [expanded]);
  useTabCloseGuard(() =>
    disabled ? "pending" : form.isFieldsTouched() ? "dirty" : undefined,
  );
  return (
    <EncounterModal
      open
      layout={expanded ? "document" : "form"}
      width={expanded ? "100%" : 800}
      title={expanded ? "Notes" : initial ? "Edit custom turn" : "Add custom turn"}
      footer={expanded ? null : undefined}
      okText={initial ? "Save" : "Add"}
      confirmLoading={disabled}
      okButtonProps={{ disabled: !!conflict }}
      cancelButtonProps={{ disabled }}
      closable={
        expanded
          ? {
              closeIcon: <span ref={collapseLabel}>Collapse editor</span>,
              "aria-label": "Collapse editor",
              disabled,
            }
          : !disabled
      }
      styles={
        expanded
          ? { close: { width: "auto", paddingInline: "var(--space-control)" } }
          : undefined
      }
      keyboard={!disabled}
      mask={{ closable: false }}
      onCancel={() => {
        if (expanded) setExpanded(false);
        else onClose();
      }}
      onOk={() => form.submit()}
    >
      <Form
        form={form}
        layout="vertical"
        className="initiative-entry-editor"
        data-expanded={expanded || undefined}
        initialValues={initial}
        disabled={disabled}
        onFinish={async (values) => {
          try {
            if (!expanded && !conflict && (await onSave(values))) onClose();
          } catch {
            // The encounter mutation owner reports the failure.
          }
        }}
      >
        <div hidden={expanded}>
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
        </div>
        <div className="initiative-entry-editor__notes-header" hidden={expanded}>
          <label htmlFor={noteId}>Notes</label>
          <Button
            ref={expandButton}
            size="small"
            type="text"
            disabled={disabled}
            onClick={() => {
              restoreToggleFocus.current = true;
              setExpanded(true);
            }}
          >
            Expand editor
          </Button>
        </div>
        <Form.Item name="note" noStyle>
          <MarkdownEditor
            id={noteId}
            label="Notes"
            disabled={disabled}
            sizing={expanded ? "fill" : "compact"}
          />
        </Form.Item>
      </Form>
    </EncounterModal>
  );
}
