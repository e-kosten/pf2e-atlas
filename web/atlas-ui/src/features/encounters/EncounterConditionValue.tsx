import { Form } from "antd";
import { ConditionValueInput } from "./ConditionValueInput";
import { useRef, useState } from "react";

export function EncounterConditionValue({
  name,
  value,
  disabled,
  onCommit,
  onClear,
}: {
  name: string;
  value: bigint | undefined;
  disabled: boolean;
  onCommit: (value: bigint) => Promise<boolean>;
  onClear?: () => Promise<boolean>;
}) {
  const [draft, setDraft] = useState<number | null | undefined>(undefined);
  const [error, setError] = useState<string>();
  const [saving, setSaving] = useState(false);
  const inFlight = useRef(false);
  const shown =
    draft === undefined ? (value === undefined ? null : Number(value)) : draft;
  const commit = async (input: number | null) => {
    if (disabled || inFlight.current) return;
    if (input === null || !Number.isSafeInteger(input) || input < 1) {
      setError("Enter a whole number, one or greater.");
      return;
    }
    setError(undefined);
    if (BigInt(input) === value) return;
    inFlight.current = true;
    setSaving(true);
    try {
      if (await onCommit(BigInt(input))) setDraft(undefined);
    } catch {
      /* The mutation owner supplies the server error beside the row. */
    } finally {
      inFlight.current = false;
      setSaving(false);
    }
  };
  return (
    <Form.Item
      style={{ marginBottom: 0 }}
      validateStatus={error ? "error" : undefined}
      help={error}
    >
      <ConditionValueInput
        size="small"
        name={`${name} value`}
        removeLabel={`Remove ${name}`}
        value={shown}
        disabled={disabled}
        readOnly={saving}
        onChange={(input) => {
          setDraft(input);
          setError(undefined);
        }}
        onStep={(input) => void commit(input)}
        onBlur={() => void commit(shown)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            void commit(shown);
          }
        }}
        onRemove={
          onClear
            ? async () => {
                if (disabled || inFlight.current) return;
                inFlight.current = true;
                setSaving(true);
                try {
                  if (await onClear()) setDraft(undefined);
                } catch {
                  /* Mutation owner reports failure. */
                } finally {
                  inFlight.current = false;
                  setSaving(false);
                }
              }
            : undefined
        }
      />
    </Form.Item>
  );
}
