import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { Button, Form, Input, Space } from "antd";
import { useEffect, useId, type ReactNode } from "react";
import type {
  EncounterHpAction,
  EncounterParticipantView,
} from "../../generated/atlas";

type HpValues = { current: string; maximum: string; temporary: string };

export function EncounterHpEditor({
  current,
  disabled,
  pending,
  error,
  recovery,
  onAction,
  onCancel,
  onCommitted,
}: {
  current: EncounterParticipantView;
  disabled: boolean;
  pending: boolean;
  error?: string;
  recovery?: ReactNode;
  onAction: (action: EncounterHpAction) => Promise<boolean>;
  onCancel: () => void;
  onCommitted: (refreshed: boolean) => void;
}) {
  const name = useId();
  const [form] = Form.useForm<HpValues>();
  useEffect(() => {
    const values: HpValues = {
      current: current.current_hp?.toString() ?? "",
      maximum: current.max_hp?.toString() ?? "",
      temporary: current.temporary_hp.toString(),
    };
    for (const field of Object.keys(values) as Array<keyof HpValues>) {
      if (!form.isFieldTouched(field))
        form.setFields([{ name: field, value: values[field], touched: false }]);
    }
  }, [current, form]);
  const apply = async (action: EncounterHpAction) => {
    if (disabled) return;
    try {
      onCommitted(await onAction(action));
    } catch {
      /* The owner displays errors and keeps the entered values available. */
    }
  };
  return (
    <>
      {recovery}
      {error && (
        <ErrorNotice
          error={error}
          title="HP could not be updated"
          description={error}
        />
      )}
      <Form
        name={name}
        form={form}
        disabled={disabled}
        layout="vertical"
        onFinish={(values) => {
          if (disabled) return;
          if (!form.isFieldsTouched()) {
            onCancel();
            return;
          }
          const optional = (input: string | undefined) =>
            input?.trim() ? input : null;
          void apply({
            kind: "set_values",
            ...(form.isFieldTouched("current")
              ? { current: { input: optional(values.current) } }
              : {}),
            ...(form.isFieldTouched("maximum")
              ? { maximum: { input: optional(values.maximum) } }
              : {}),
            ...(form.isFieldTouched("temporary")
              ? { temporary: values.temporary }
              : {}),
          });
        }}
      >
        <Form.Item name="current" label="Current HP">
          <Input aria-label="Current HP" placeholder="Unknown" />
        </Form.Item>
        <Form.Item
          name="maximum"
          label="Maximum HP"
          extra={
            current.record_key && current.hp_origin === "explicit" ? (
              <Button
                type="link"
                size="small"
                disabled={disabled}
                onClick={() => void apply({ kind: "use_derived_maximum" })}
              >
                Use creature default
              </Button>
            ) : undefined
          }
        >
          <Input aria-label="Maximum HP" placeholder="Unknown" />
        </Form.Item>
        <Form.Item name="temporary" label="Temporary HP">
          <Input aria-label="Temporary HP" />
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
