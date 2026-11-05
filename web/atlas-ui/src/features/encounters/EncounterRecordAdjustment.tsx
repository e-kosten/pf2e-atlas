import { Form, Select } from "antd";

export function EncounterRecordAdjustment({
  loading = false,
  disabled = false,
}: {
  loading?: boolean;
  disabled?: boolean;
}) {
  return (
    <Form.Item
      className="encounter-add-adjustment"
      name="adjustment"
      label="Adjustment"
    >
      <Select
        aria-label="Adjustment"
        styles={{ content: { textAlign: "start" } }}
        placeholder={loading ? "Loading…" : "Unavailable"}
        disabled={disabled}
        loading={loading}
        options={[
          { value: "normal", label: "Normal" },
          { value: "elite", label: "Elite" },
          { value: "weak", label: "Weak" },
        ]}
      />
    </Form.Item>
  );
}
