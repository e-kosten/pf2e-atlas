import { Form, InputNumber, Segmented, Select, Space } from "antd";
import type { FormInstance } from "antd";
import type { LifecycleCheckInputView } from "../../generated/atlas";

export type LifecycleCheckValues = {
  checkMode?: "roll" | "degree" | "total" | "decline";
  degree?: "critical_failure" | "failure" | "success" | "critical_success";
  total?: number;
  natural?: number | null;
};
export function lifecycleCheck(values: LifecycleCheckValues): LifecycleCheckInputView {
  if (values.checkMode === "degree") {
    if (!values.degree) throw new Error("Choose the check result.");
    return { kind: "table_degree", degree: values.degree };
  }
  if (values.checkMode === "total") {
    if (!Number.isSafeInteger(values.total)) throw new Error("Enter the check total.");
    return {
      kind: "table_result",
      total: BigInt(values.total!),
      natural: values.natural == null ? null : BigInt(values.natural),
    };
  }
  return { kind: "roll" };
}
export function LifecycleCheckFields({
  form,
  allowDecline = false,
}: {
  form: FormInstance;
  allowDecline?: boolean;
}) {
  const mode = Form.useWatch("checkMode", form) ?? "roll";
  return (
    <>
      <Form.Item name="checkMode">
        <Segmented
          aria-label="Check result"
          options={[
            { value: "roll", label: "Roll in Atlas" },
            { value: "degree", label: "Table outcome" },
            { value: "total", label: "Table total" },
            ...(allowDecline ? [{ value: "decline", label: "Decline check" }] : []),
          ]}
        />
      </Form.Item>
      {mode === "degree" && (
        <Form.Item
          name="degree"
          label="Outcome"
          rules={[{ required: true, message: "Choose the outcome." }]}
        >
          <Select
            aria-label="Check outcome"
            options={[
              { value: "critical_success", label: "Critical success" },
              { value: "success", label: "Success" },
              { value: "failure", label: "Failure" },
              { value: "critical_failure", label: "Critical failure" },
            ]}
          />
        </Form.Item>
      )}
      {mode === "total" && (
        <Space wrap>
          <Form.Item
            name="total"
            label="Total"
            rules={[
              {
                validator: (_rule, value) =>
                  Number.isSafeInteger(value)
                    ? Promise.resolve()
                    : Promise.reject(new Error("Enter a whole number.")),
              },
            ]}
          >
            <InputNumber aria-label="Check total" />
          </Form.Item>
          <Form.Item
            name="natural"
            label="Natural d20"
            rules={[
              {
                validator: (_rule, value) =>
                  value == null ||
                  (Number.isSafeInteger(value) && value >= 1 && value <= 20)
                    ? Promise.resolve()
                    : Promise.reject(new Error("Enter 1–20.")),
              },
            ]}
          >
            <InputNumber aria-label="Natural d20" min={1} max={20} />
          </Form.Item>
        </Space>
      )}
    </>
  );
}
