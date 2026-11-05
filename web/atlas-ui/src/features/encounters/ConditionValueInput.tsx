import { Button, InputNumber, Space, Tooltip } from "antd";
import { Minus, Plus, X } from "lucide-react";
export function ConditionValueInput({
  value,
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
    value === 1 && onRemove ? (removeLabel ?? `Remove ${name}`) : `Decrease ${name}`;
  const increaseLabel = `Increase ${name}`;
  return (
    <Space.Compact size={size}>
      <Tooltip title={decreaseLabel} trigger={["hover", "focus"]}>
        <Button
          disabled={disabled || readOnly || (value === 1 && !onRemove) || value == null}
          icon={value === 1 && onRemove ? <X size={14} /> : <Minus size={14} />}
          aria-label={decreaseLabel}
          onClick={() => {
            if (value === 1 && onRemove) onRemove();
            else if (value != null) {
              onChange?.(Math.max(1, value - 1));
              onStep?.(Math.max(1, value - 1));
            }
          }}
        />
      </Tooltip>
      <InputNumber
        aria-label={name}
        min={1}
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
