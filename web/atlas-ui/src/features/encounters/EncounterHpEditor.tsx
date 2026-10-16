import { Alert, Button, Form, Input, Select, Space } from "antd";
import { useEffect, useId, type ReactNode } from "react";
import type {
  EncounterHpAction,
  EncounterParticipantView,
} from "../../generated/atlas";

type HpTarget = "current" | "temporary" | "maximum";
type HpValues = { target: HpTarget; input: string };
const labels: Record<HpTarget, string> = {
  current: "Set current HP",
  temporary: "Set temporary HP",
  maximum: "Override maximum",
};
function initialInput(current: EncounterParticipantView, target: HpTarget) {
  return (
    (target === "current"
      ? current.current_hp
      : target === "temporary"
        ? current.temporary_hp
        : current.max_hp
    )?.toString() ?? ""
  );
}

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
  const target: HpTarget = Form.useWatch("target", form) ?? "current";
  useEffect(() => {
    if (!form.isFieldTouched("input"))
      form.setFields([
        { name: "input", value: initialInput(current, target), touched: false },
      ]);
  }, [current, target, form]);
  const apply = async (action: EncounterHpAction) => {
    if (disabled) return;
    try {
      onCommitted(await onAction(action));
    } catch {
      /* Preserve raw formula and show the mutation owner's error. */
    }
  };
  return (
    <>
      {recovery}
      {error && (
        <Alert type="error" title="HP could not be updated" description={error} />
      )}
      <Form
        name={name}
        form={form}
        disabled={disabled}
        layout="vertical"
        initialValues={{ target: "current", input: initialInput(current, "current") }}
        onFinish={(values) => {
          const input = values.input ?? "";
          void apply(
            values.target === "temporary"
              ? { kind: "set_temporary", input }
              : {
                  kind: values.target === "current" ? "set_current" : "set_maximum",
                  input: input.trim() ? input : null,
                },
          );
        }}
      >
        <Form.Item name="target" label="Edit">
          <Select
            aria-label="HP field"
            options={[
              { value: "current", label: "Current HP" },
              { value: "temporary", label: "Temporary HP" },
              { value: "maximum", label: "Maximum HP" },
            ]}
            onChange={(value: HpTarget) =>
              form.setFields([
                {
                  name: "input",
                  value: initialInput(current, value),
                  touched: false,
                  errors: [],
                },
              ])
            }
          />
        </Form.Item>
        <Form.Item
          name="input"
          label={
            target === "current"
              ? "Current HP"
              : target === "temporary"
                ? "Temporary HP"
                : "Maximum HP"
          }
          rules={
            target === "temporary"
              ? [
                  {
                    required: true,
                    whitespace: true,
                    message: "Enter temporary HP, or 0 to clear it.",
                  },
                ]
              : []
          }
          extra={target === "temporary" ? undefined : "Leave blank if unknown."}
        >
          <Input
            aria-label={
              target === "current"
                ? "HP"
                : target === "temporary"
                  ? "Temp HP"
                  : "Maximum HP"
            }
          />
        </Form.Item>
        {target === "maximum" && (
          <p className="muted">
            {current.max_hp === undefined
              ? "Maximum unavailable"
              : current.hp_origin === "explicit"
                ? "Manual maximum"
                : "Creature default"}
          </p>
        )}
        <Space wrap>
          <Button
            type="primary"
            htmlType="submit"
            loading={pending}
            disabled={disabled}
          >
            {labels[target]}
          </Button>
          {target === "maximum" &&
            current.record_key &&
            current.hp_origin === "explicit" && (
              <Button
                disabled={disabled}
                onClick={() => void apply({ kind: "use_derived_maximum" })}
              >
                Use creature default
              </Button>
            )}
          <Button disabled={pending} onClick={onCancel}>
            Cancel
          </Button>
        </Space>
      </Form>
    </>
  );
}
