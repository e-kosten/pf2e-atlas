import { EncounterModal } from "./EncounterModal";
import {
  Alert,
  Button,
  Checkbox,
  Form,
  Input,
  InputNumber,
  Select,
  Segmented,
  Space,
} from "antd";
import { useEffect, useId, useState, type ReactNode } from "react";
import type {
  EncounterParticipantView,
  UpdateEncounterParticipantRequest,
} from "../../generated/atlas";
import { Pencil } from "lucide-react";
import { EncounterOriginalRecord } from "./EncounterOriginalRecord";
import { optionalBigIntInput, participantUpdate } from "./participantEdits";

type ParticipantValues = {
  name: string;
  initiative: string;
  side: EncounterParticipantView["side"];
  adjustment: EncounterParticipantView["participant_variant"];
  note: string;
};

export function EncounterParticipantSettings({
  participant,
  disabled,
  pending,
  error,
  recovery,
  onUpdate,
  onLifeSettings,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onUpdate: (request: UpdateEncounterParticipantRequest) => Promise<boolean>;
  onLifeSettings?: (
    settings: Omit<
      import("../../generated/atlas").SetEncounterLifeSettingsRequest,
      "request_id" | "expected_encounter_revision"
    >,
  ) => Promise<boolean>;
}) {
  const formId = useId();
  const [open, setOpen] = useState(false);
  const [committed, setCommitted] = useState(false);
  const [savingLifeSettings, setSavingLifeSettings] = useState(false);
  return (
    <>
      <Button
        size="small"
        type="text"
        aria-label="Participant settings"
        title="Participant settings"
        icon={<Pencil size={16} aria-hidden="true" />}
        disabled={disabled}
        onClick={() => {
          setCommitted(false);
          setOpen(true);
        }}
      />
      <EncounterModal
        title="Participant settings"
        open={open && !(committed && !disabled)}
        destroyOnHidden
        footer={
          <Space>
            <Button disabled={pending} onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button
              type="primary"
              htmlType="submit"
              form={formId}
              loading={pending && !savingLifeSettings}
              disabled={disabled}
            >
              Save
            </Button>
          </Space>
        }
        onCancel={() => setOpen(false)}
        closable={!pending}
        keyboard={!pending}
        mask={{ closable: !pending }}
      >
        <ParticipantSettingsForm
          formId={formId}
          participant={participant}
          disabled={disabled && !savingLifeSettings}
          pending={pending && !savingLifeSettings}
          error={error}
          recovery={recovery}
          lifeSettings={
            participant.participant_kind !== "hazard" && onLifeSettings ? (
              <LifeSettings
                participant={participant}
                disabled={disabled || pending}
                onSave={async (settings) => {
                  setSavingLifeSettings(true);
                  try {
                    return await onLifeSettings(settings);
                  } finally {
                    setSavingLifeSettings(false);
                  }
                }}
              />
            ) : undefined
          }
          onSave={async (request) => {
            if (disabled || pending) return;
            const refreshed = await onUpdate(request);
            if (refreshed) setOpen(false);
            else setCommitted(true);
          }}
        />
      </EncounterModal>
    </>
  );
}

function ParticipantSettingsForm({
  participant,
  disabled,
  pending,
  error,
  recovery,
  lifeSettings,
  formId,
  onSave,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  lifeSettings?: ReactNode;
  formId: string;
  onSave: (request: UpdateEncounterParticipantRequest) => Promise<void>;
}) {
  const name = useId();
  const [form] = Form.useForm<ParticipantValues>();
  useEffect(() => {
    const values: ParticipantValues = {
      name: participant.display_name,
      initiative: participant.initiative?.toString() ?? "",
      side: participant.side,
      adjustment: participant.participant_variant,
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
        id={formId}
        form={form}
        layout="vertical"
        disabled={disabled || pending}
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
                ...(form.isFieldTouched("adjustment")
                  ? { participant_variant: values.adjustment }
                  : {}),
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
        {participant.participant_kind === "creature" && (
          <Form.Item name="adjustment" label="Adjustment">
            <Select
              aria-label="Adjustment"
              options={[
                { value: "normal", label: "Normal" },
                { value: "elite", label: "Elite" },
                { value: "weak", label: "Weak" },
              ]}
            />
          </Form.Item>
        )}
        <Form.Item name="note" label="Notes">
          <Input.TextArea
            aria-label="Participant notes"
            autoSize={{ minRows: 3, maxRows: 8 }}
          />
        </Form.Item>
        {lifeSettings}
        {participant.record_key && (
          <EncounterOriginalRecord recordKey={participant.record_key} />
        )}
      </Form>
    </>
  );
}

function LifeSettings({
  participant,
  disabled,
  onSave,
}: {
  participant: EncounterParticipantView;
  disabled: boolean;
  onSave: NonNullable<
    Parameters<typeof EncounterParticipantSettings>[0]["onLifeSettings"]
  >;
}) {
  const [threshold, setThreshold] = useState<number | null>(
    Number(participant.base_death_threshold),
  );
  const [error, setError] = useState<string>();
  const [pendingMortality, setPendingMortality] =
    useState<EncounterParticipantView["mortality"]>();
  const save = async (settings: Parameters<typeof onSave>[0]) => {
    setError(undefined);
    try {
      await onSave(settings);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not save settings.");
    }
  };
  return (
    <section className="encounter-life-settings" aria-label="Dying settings">
      {error && <Alert type="error" title={error} />}
      <Checkbox
        checked={participant.track_dying}
        disabled={disabled}
        onChange={(e) => void save({ track_dying: e.target.checked })}
      >
        Track dying
      </Checkbox>
      {participant.track_dying && (
        <div className="encounter-life-settings__fields">
          <Form.Item label="Death threshold" style={{ marginBottom: 0 }}>
            <InputNumber
              style={{ width: "100%" }}
              aria-label="Death threshold"
              disabled={disabled}
              min={1}
              precision={0}
              value={threshold}
              onChange={setThreshold}
              onBlur={() => {
                if (
                  threshold != null &&
                  threshold > 0 &&
                  Number.isSafeInteger(threshold) &&
                  threshold !== Number(participant.base_death_threshold)
                )
                  void save({
                    track_dying: true,
                    base_death_threshold: BigInt(threshold),
                  });
              }}
            />
          </Form.Item>
        </div>
      )}
      <Segmented<EncounterParticipantView["mortality"]>
        className="encounter-life-settings__status"
        aria-label="Life status"
        disabled={disabled}
        value={pendingMortality ?? participant.mortality}
        options={[
          { value: "alive", label: "Alive" },
          { value: "dead", label: "Dead" },
        ]}
        onChange={async (mortality) => {
          setPendingMortality(mortality);
          try {
            await save({ track_dying: participant.track_dying, mortality });
          } finally {
            setPendingMortality(undefined);
          }
        }}
      />
    </section>
  );
}
