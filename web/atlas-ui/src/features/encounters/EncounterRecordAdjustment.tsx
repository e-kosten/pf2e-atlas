import { Form, Select } from "antd";

export function EncounterRecordAdjustment({ loading = false }: { loading?: boolean }) {
  return (
    <Form.Item
      className="encounter-add-adjustment"
      name="adjustment"
      label="Adjustment"
    >
      <Select
        aria-label="Adjustment"
        styles={{ content: { textAlign: "start" } }}
        placeholder="Loading…"
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
