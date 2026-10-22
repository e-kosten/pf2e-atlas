import { Button, InputNumber, Space } from "antd";
import { Minus, Plus, X } from "lucide-react";
export function ConditionValueInput({
  value,
  onChange,
  onRemove,
  removeLabel,
  disabled,
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
  name: string;
  onStep?: (value: number) => void;
  onBlur?: () => void;
  onKeyDown?: React.KeyboardEventHandler<HTMLInputElement>;
}) {
  return (
    <Space.Compact>
      <Button
        disabled={disabled || (value === 1 && !onRemove) || value == null}
        icon={value === 1 && onRemove ? <X size={14} /> : <Minus size={14} />}
        aria-label={
          value === 1 && onRemove
            ? (removeLabel ?? `Remove ${name}`)
            : `Decrease ${name}`
        }
        onClick={() => {
          if (value === 1 && onRemove) onRemove();
          else if (value != null) {
            onChange?.(Math.max(1, value - 1));
            onStep?.(Math.max(1, value - 1));
          }
        }}
      />
      <InputNumber
        aria-label={name}
        min={1}
        controls={false}
        value={value}
        disabled={disabled}
        onChange={onChange}
        onBlur={onBlur}
        onKeyDown={onKeyDown}
        style={{ width: 64 }}
        styles={{ input: { textAlign: "center" } }}
      />
      <Button
        disabled={disabled}
        icon={<Plus size={14} />}
        aria-label={`Increase ${name}`}
        onClick={() => {
          onChange?.((value ?? 0) + 1);
          onStep?.((value ?? 0) + 1);
        }}
      />
    </Space.Compact>
  );
}
