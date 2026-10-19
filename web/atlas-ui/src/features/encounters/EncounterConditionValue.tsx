import { Form, InputNumber } from "antd";
import { useRef, useState } from "react";

export function EncounterConditionValue({
  name,
  value,
  disabled,
  onCommit,
}: {
  name: string;
  value: bigint | undefined;
  disabled: boolean;
  onCommit: (value: bigint) => Promise<boolean>;
}) {
  const [draft, setDraft] = useState<number | null | undefined>(undefined);
  const [error, setError] = useState<string>();
  const inFlight = useRef(false);
  const shown =
    draft === undefined ? (value === undefined ? null : Number(value)) : draft;
  const commit = async (input: number | null) => {
    if (disabled || inFlight.current) return;
    if (input === null || !Number.isSafeInteger(input) || input < 0) {
      setError("Enter a whole number, zero or greater.");
      return;
    }
    setError(undefined);
    if (BigInt(input) === value) return;
    inFlight.current = true;
    try {
      if (await onCommit(BigInt(input))) setDraft(undefined);
    } catch {
      /* The mutation owner supplies the server error beside the row. */
    } finally {
      inFlight.current = false;
    }
  };
  return (
    <Form.Item
      style={{ marginBottom: 0 }}
      validateStatus={error ? "error" : undefined}
      help={error}
    >
      <InputNumber
        mode="spinner"
        style={{ width: 144 }}
        aria-label={`${name} value`}
        disabled={disabled}
        step={1}
        value={shown}
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
      />
    </Form.Item>
  );
}
