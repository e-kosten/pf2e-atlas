import { Alert, Button, Form, Input, Modal, Select, Space } from "antd";
import { useEffect, useId, useState, type ReactNode } from "react";
import type {
  EncounterParticipantView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { optionalBigIntInput, participantUpdate } from "./participantEdits";

type ParticipantValues = {
  name: string;
  initiative: string;
  side: EncounterParticipantView["side"];
  note: string;
};

export function EncounterParticipantSettings({
  participant,
  disabled,
  pending,
  error,
  recovery,
  onUpdate,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onUpdate: (request: UpdateEncounterParticipantRequest) => Promise<boolean>;
}) {
  const [open, setOpen] = useState(false);
  const [committed, setCommitted] = useState(false);
  return (
    <>
      <Button
        size="small"
        disabled={disabled}
        onClick={() => {
          setCommitted(false);
          setOpen(true);
        }}
      >
        Participant settings
      </Button>
      <Modal
        title="Participant settings"
        open={open && !(committed && !disabled)}
        destroyOnHidden
        footer={null}
        onCancel={() => setOpen(false)}
        closable={!pending}
        keyboard={!pending}
        mask={{ closable: !pending }}
      >
        <ParticipantSettingsForm
          participant={participant}
          disabled={disabled}
          pending={pending}
          error={error}
          recovery={recovery}
          onCancel={() => setOpen(false)}
          onSave={async (request) => {
            const refreshed = await onUpdate(request);
            if (refreshed) setOpen(false);
            else setCommitted(true);
          }}
        />
      </Modal>
    </>
  );
}

function ParticipantSettingsForm({
  participant,
  disabled,
  pending,
  error,
  recovery,
  onCancel,
  onSave,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onCancel: () => void;
  onSave: (request: UpdateEncounterParticipantRequest) => Promise<void>;
}) {
  const name = useId();
  const [form] = Form.useForm<ParticipantValues>();
  useEffect(() => {
    const values: ParticipantValues = {
      name: participant.display_name,
      initiative: participant.initiative?.toString() ?? "",
      side: participant.side,
      note: participant.note ?? "",
    };
    for (const field of Object.keys(values) as Array<keyof ParticipantValues>) {
      if (!form.isFieldTouched(field))
        form.setFields([{ name: field, value: values[field], touched: false }]);
    }
  }, [form, participant]);
  return (
    <>
      {recovery}
      {error && (
        <Alert
          type="error"
          title="Participant could not be saved"
          description={error}
        />
      )}
      <Form
        name={name}
        form={form}
        layout="vertical"
        disabled={disabled}
        onFinish={async (values) => {
          if (disabled) return;
          const initiative = optionalBigIntInput(values.initiative ?? "");
          if (initiative === null) return;
          try {
            await onSave(
              participantUpdate(participant, {
                display_name: values.name,
                initiative,
                side: values.side,
                note: values.note || undefined,
              }),
            );
          } catch {
            /* The mutation owner displays the error; retain the draft. */
          }
        }}
      >
        <Form.Item
          name="name"
          label="Name"
          rules={[
            { required: true, whitespace: true, message: "Enter a participant name." },
          ]}
        >
          <Input aria-label="Participant name" />
        </Form.Item>
        <Form.Item
          name="initiative"
          label="Initiative"
          rules={[
            {
              validator: (_rule, value: string | undefined) =>
                optionalBigIntInput(value ?? "") === null
                  ? Promise.reject(new Error("Enter a whole number or leave blank."))
                  : Promise.resolve(),
            },
          ]}
        >
          <Input aria-label="Participant initiative" inputMode="numeric" />
        </Form.Item>
        <Form.Item name="side" label="Side" rules={[{ required: true }]}>
          <Select
            aria-label="Participant side"
            options={["pc", "ally", "enemy", "neutral", "hazard"].map((value) => ({
              value,
              label: value,
            }))}
          />
        </Form.Item>
        <Form.Item name="note" label="Notes">
          <Input.TextArea
            aria-label="Participant notes"
            autoSize={{ minRows: 3, maxRows: 8 }}
          />
        </Form.Item>
        <Space wrap>
          <Button
            type="primary"
            htmlType="submit"
            loading={pending}
            disabled={disabled}
          >
            Save
          </Button>
          <Button disabled={pending} onClick={onCancel}>
            Cancel
          </Button>
        </Space>
      </Form>
    </>
  );
}
