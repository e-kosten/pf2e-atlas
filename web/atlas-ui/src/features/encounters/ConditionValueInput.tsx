import { Button, InputNumber, Space, Tooltip } from "antd";
import { Minus, Plus, X } from "lucide-react";
export function ConditionValueInput({
  value,
  minimum = 1,
  onChange,
  onRemove,
  removeLabel,
  disabled,
  readOnly = false,
  size = "middle",
  name,
  onBlur,
  onKeyDown,
  onStep,
}: {
  minimum?: number;
  value?: number | null;
  onChange?: (value: number | null) => void;
  onRemove?: () => void;
  removeLabel?: string;
  disabled?: boolean;
  readOnly?: boolean;
  size?: "small" | "middle";
  name: string;
  onStep?: (value: number) => void;
  onBlur?: () => void;
  onKeyDown?: React.KeyboardEventHandler<HTMLInputElement>;
}) {
  const decreaseLabel =
    minimum === 1 && value === 1 && onRemove
      ? (removeLabel ?? `Remove ${name}`)
      : `Decrease ${name}`;
  const increaseLabel = `Increase ${name}`;
  return (
    <Space.Compact size={size}>
      <Tooltip title={decreaseLabel} trigger={["hover", "focus"]}>
        <Button
          disabled={
            disabled ||
            readOnly ||
            (minimum === 1 && value === 1 && !onRemove) ||
            value == null ||
            (minimum === 0 && value === 0)
          }
          icon={
            minimum === 1 && value === 1 && onRemove ? (
              <X size={14} />
            ) : (
              <Minus size={14} />
            )
          }
          aria-label={decreaseLabel}
          onClick={() => {
            if (minimum === 1 && value === 1 && onRemove) onRemove();
            else if (value != null) {
              onChange?.(Math.max(minimum, value - 1));
              onStep?.(Math.max(minimum, value - 1));
            }
          }}
        />
      </Tooltip>
      <InputNumber
        aria-label={name}
        min={minimum === 0 ? undefined : minimum}
        changeOnBlur={minimum !== 0}
        placeholder={minimum === 0 ? "—" : undefined}
        controls={false}
        value={value}
        disabled={disabled && !readOnly}
        readOnly={readOnly}
        onChange={onChange}
        onBlur={onBlur}
        onKeyDown={onKeyDown}
        style={{ width: size === "small" ? 44 : 64 }}
        styles={{ input: { textAlign: "center" } }}
      />
      <Tooltip title={increaseLabel} trigger={["hover", "focus"]}>
        <Button
          disabled={disabled || readOnly}
          icon={<Plus size={14} />}
          aria-label={increaseLabel}
          onClick={() => {
            onChange?.((value ?? 0) + 1);
            onStep?.((value ?? 0) + 1);
          }}
        />
      </Tooltip>
    </Space.Compact>
  );
}
