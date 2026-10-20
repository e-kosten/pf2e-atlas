import { Alert, Button, Checkbox, Form, Select, Space } from "antd";
import { useState } from "react";
import type { DetectionEventView } from "../../generated/atlas";
type Seek = Extract<DetectionEventView, { kind: "seek" }>;
export function EncounterDetectionEventForm({
  kind,
  subjectName,
  prior,
  disabled,
  changed,
  onReload,
  onApply,
}: {
  kind: "seek" | "became_invisible";
  subjectName: string;
  prior: Seek["prior"] | undefined;
  disabled: boolean;
  changed: boolean;
  onReload: () => void;
  onApply: (event: DetectionEventView) => Promise<void>;
}) {
  const [error, setError] = useState<string>();
  const [saving, setSaving] = useState(false);
  return (
    <>
      {changed && (
        <Alert
          type="warning"
          title="Encounter changed"
          action={
            <Button disabled={disabled || saving} onClick={onReload}>
              Reload relationship
            </Button>
          }
        />
      )}
      {error && (
        <Alert type="error" title="Detection event failed" description={error} />
      )}
      <Form
        layout="vertical"
        disabled={disabled || saving || changed}
        initialValues={{ prior, canObserve: false }}
        onFinish={async (values: {
          prior?: Seek["prior"];
          outcome?: Seek["outcome"];
          observedBefore?: string;
          canObserve: boolean;
        }) => {
          setError(undefined);
          setSaving(true);
          try {
            if (kind === "seek") {
              if (!values.prior || !values.outcome) return;
              await onApply({
                kind,
                outcome: values.outcome,
                prior: values.prior,
                can_observe: values.canObserve,
              });
            } else
              await onApply({
                kind,
                observed_before: values.observedBefore === "yes",
                can_observe: values.canObserve,
              });
          } catch (value) {
            setError(
              value instanceof Error
                ? value.message
                : "Could not record detection event.",
            );
          } finally {
            setSaving(false);
          }
        }}
      >
        {kind === "seek" ? (
          <>
            <Form.Item
              name="prior"
              label="Before Seek"
              rules={[{ required: true, message: "Choose the prior detection state." }]}
            >
              <Select
                aria-label="Before Seek"
                options={[
                  { value: "observed", label: "Observed" },
                  { value: "hidden", label: "Hidden" },
                  { value: "undetected", label: "Undetected" },
                ]}
              />
            </Form.Item>
            <Form.Item
              name="outcome"
              label="Seek result"
              rules={[{ required: true, message: "Choose the Seek result." }]}
            >
              <Select
                aria-label="Seek result"
                options={[
                  { value: "critical_success", label: "Critical success" },
                  { value: "success", label: "Success" },
                  { value: "failure", label: "Failure" },
                  { value: "critical_failure", label: "Critical failure" },
                ]}
              />
            </Form.Item>
          </>
        ) : (
          <Form.Item
            name="observedBefore"
            label="Was visible before?"
            rules={[
              { required: true, message: "Choose whether the subject was visible." },
            ]}
          >
            <Select
              aria-label="Was visible before"
              options={[
                { value: "yes", label: "Yes" },
                { value: "no", label: "No" },
              ]}
            />
          </Form.Item>
        )}
        <Form.Item name="canObserve" valuePropName="checked">
          <Checkbox>Can observe {subjectName}</Checkbox>
        </Form.Item>
        <Space>
          <Button
            type="primary"
            htmlType="submit"
            disabled={disabled || changed}
            loading={saving}
          >
            Record
          </Button>
        </Space>
      </Form>
    </>
  );
}
