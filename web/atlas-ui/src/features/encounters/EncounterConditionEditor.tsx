import { Alert, Button, Collapse, Form, Input, InputNumber, Select, Space } from "antd";
import { useEffect, useId, type ReactNode } from "react";
import type {
  EncounterConditionDefinitionView,
  EncounterParticipantConditionView,
  EncounterParticipantView,
} from "../../generated/atlas";

type Duration = NonNullable<EncounterParticipantConditionView["duration"]>;
export type ConditionFormValues = {
  conditionRef?: string;
  value?: number | null;
  durationAmount?: number | null;
  durationUnit: Duration["unit"];
  timingParticipant?: string;
  timingEvent?: "start" | "end";
  sourceParticipantKey?: string;
  note?: string;
};
const durationFields: Array<keyof ConditionFormValues> = [
  "durationAmount",
  "durationUnit",
  "timingParticipant",
  "timingEvent",
];
export const integerRule = (required: boolean) => ({
  validator: (_rule: unknown, value: number | null | undefined) => {
    if (value === null || value === undefined)
      return required
        ? Promise.reject(new Error("Enter a whole number."))
        : Promise.resolve();
    return Number.isSafeInteger(value) && value >= 0
      ? Promise.resolve()
      : Promise.reject(new Error("Enter a whole number, zero or greater."));
  },
});

export function conditionDuration(values: ConditionFormValues): Duration | undefined {
  if (values.durationAmount === null || values.durationAmount === undefined)
    return undefined;
  return {
    amount: BigInt(values.durationAmount),
    unit: values.durationUnit,
    timing:
      values.durationUnit === "rounds" && values.timingParticipant && values.timingEvent
        ? { participant_key: values.timingParticipant, event: values.timingEvent }
        : null,
  };
}

export function EncounterConditionEditor({
  condition,
  definitions,
  participants,
  disabled,
  pending,
  error,
  recovery,
  onSave,
  onCancel,
  onRemove,
  onReload,
}: {
  condition?: EncounterParticipantConditionView;
  definitions: EncounterConditionDefinitionView[];
  participants: EncounterParticipantView[];
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onSave: (values: ConditionFormValues, durationEdited: boolean) => Promise<void>;
  onCancel: () => void;
  onRemove?: () => void;
  onReload?: () => void;
}) {
  const name = useId();
  const [form] = Form.useForm<ConditionFormValues>();
  const conditionRef = Form.useWatch("conditionRef", form);
  const definition = definitions.find(
    (d) => d.condition_ref === (condition?.condition_key ?? conditionRef),
  );
  const hasValue = definition?.has_value ?? condition?.value !== undefined;
  const durationAmount = Form.useWatch("durationAmount", form);
  const durationUnit = Form.useWatch("durationUnit", form) ?? "rounds";
  const isRounds =
    durationAmount !== undefined &&
    durationAmount !== null &&
    durationUnit === "rounds";
  useEffect(() => {
    if (!condition) return;
    const values: ConditionFormValues = {
      conditionRef: condition.condition_key,
      value: condition.value === undefined ? undefined : Number(condition.value),
      durationAmount:
        condition.duration === undefined
          ? undefined
          : Number(condition.duration.amount),
      durationUnit: condition.duration?.unit ?? "rounds",
      timingParticipant: condition.duration?.timing?.participant_key,
      timingEvent: condition.duration?.timing?.event,
      sourceParticipantKey: condition.source_participant_key,
      note: condition.note,
    };
    for (const field of Object.keys(values) as Array<keyof ConditionFormValues>) {
      if (!form.isFieldTouched(field))
        form.setFields([{ name: field, value: values[field], touched: false }]);
    }
  }, [condition, form]);
  const timingRule = {
    validator: (_rule: unknown, value: string | undefined) => {
      const unchangedUnresolved =
        condition?.duration?.unit === "rounds" &&
        condition.duration.timing === null &&
        !form.isFieldsTouched(durationFields);
      return !isRounds || value || unchangedUnresolved
        ? Promise.resolve()
        : Promise.reject(new Error("Choose when these rounds count down."));
    },
  };
  return (
    <>
      {onReload && (
        <Alert
          type="warning"
          title="Encounter changed"
          description="Reload this condition to review its current values before saving. This replaces your unsaved changes."
          action={
            <Button disabled={pending} onClick={onReload}>
              Reload condition
            </Button>
          }
        />
      )}
      {recovery}
      {error && (
        <Alert type="error" title="Condition action failed" description={error} />
      )}
      <Form
        name={name}
        form={form}
        layout="vertical"
        disabled={disabled}
        initialValues={{
          durationUnit: condition?.duration?.unit ?? "rounds",
          durationAmount: condition?.duration
            ? Number(condition.duration.amount)
            : undefined,
          timingParticipant: condition?.duration?.timing?.participant_key,
          timingEvent: condition?.duration?.timing?.event,
          value: condition?.value === undefined ? undefined : Number(condition.value),
          conditionRef: condition?.condition_key,
          sourceParticipantKey: condition?.source_participant_key,
          note: condition?.note,
        }}
        onFinish={async (values) => {
          if (disabled || onReload) return;
          try {
            await onSave(values, form.isFieldsTouched(durationFields));
          } catch {
            /* Keep the draft; the feature mutation owner supplies feedback. */
          }
        }}
      >
        {!condition && (
          <Form.Item
            name="conditionRef"
            label="Condition"
            rules={[{ required: true, message: "Choose a condition." }]}
          >
            <Select
              aria-label="Add condition"
              showSearch={{ optionFilterProp: "label" }}
              options={definitions.map((d) => ({
                label: d.name,
                value: d.condition_ref,
              }))}
              onChange={(ref) => {
                const selected = definitions.find((d) => d.condition_ref === ref);
                form.setFields([
                  {
                    name: "value",
                    value: selected?.has_value
                      ? Number(selected.default_value ?? 1n)
                      : undefined,
                    errors: [],
                  },
                ]);
              }}
            />
          </Form.Item>
        )}
        {hasValue && (
          <Form.Item name="value" label="Value" rules={[integerRule(true)]}>
            <InputNumber
              mode="spinner"
              style={{ width: 144 }}
              aria-label="Condition value"
              step={1}
            />
          </Form.Item>
        )}
        <Collapse
          ghost
          defaultActiveKey={
            condition?.duration || condition?.source_participant_key || condition?.note
              ? ["details"]
              : []
          }
          items={[
            {
              key: "details",
              label: "Optional details",
              children: (
                <>
                  <div className="encounter-duration-fields">
                    <Form.Item
                      name="durationAmount"
                      label="Duration"
                      rules={[integerRule(false)]}
                    >
                      <InputNumber aria-label="Duration amount" step={1} />
                    </Form.Item>
                    <Form.Item name="durationUnit" label="Unit">
                      <Select
                        aria-label="Duration unit"
                        options={["rounds", "minutes", "hours", "days"].map(
                          (value) => ({ value, label: value }),
                        )}
                      />
                    </Form.Item>
                  </div>
                  {isRounds ? (
                    <>
                      <Form.Item
                        name="timingParticipant"
                        label="Whose turn"
                        rules={[timingRule]}
                      >
                        <Select
                          aria-label="Duration turn"
                          allowClear
                          options={participants.map((p) => ({
                            value: p.participant_key,
                            label: p.display_name,
                          }))}
                        />
                      </Form.Item>
                      <Form.Item
                        name="timingEvent"
                        label="Count down at"
                        rules={[timingRule]}
                      >
                        <Select
                          aria-label="Duration timing"
                          allowClear
                          options={[
                            { value: "start", label: "Start of turn" },
                            { value: "end", label: "End of turn" },
                          ]}
                        />
                      </Form.Item>
                      {condition?.duration?.unit === "rounds" &&
                        condition.duration.timing === null && (
                          <p className="muted">
                            Timing is not configured. Choose a turn to start tracking
                            rounds.
                          </p>
                        )}
                    </>
                  ) : durationAmount !== undefined && durationAmount !== null ? (
                    <p className="muted">Reminder only; tracked at the table.</p>
                  ) : null}
                  <Form.Item name="sourceParticipantKey" label="Source">
                    <Select
                      aria-label="Source"
                      allowClear
                      options={participants.map((p) => ({
                        value: p.participant_key,
                        label: p.display_name,
                      }))}
                    />
                  </Form.Item>
                  <Form.Item name="note" label="Note">
                    <Input.TextArea
                      aria-label="Condition note"
                      autoSize={{ minRows: 2, maxRows: 8 }}
                    />
                  </Form.Item>
                </>
              ),
            },
          ]}
        />
        <Space wrap>
          <Button
            type="primary"
            htmlType="submit"
            disabled={disabled || !!onReload}
            loading={pending}
          >
            {condition ? "Save" : "Add"}
          </Button>
          <Button disabled={pending} onClick={onCancel}>
            Cancel
          </Button>
          {onRemove && (
            <Button danger disabled={disabled} onClick={onRemove}>
              Remove condition
            </Button>
          )}
        </Space>
      </Form>
    </>
  );
}
