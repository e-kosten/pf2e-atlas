import { Tooltip, Button, Dropdown } from "antd";
import { ArrowDown, ArrowUp, SlidersHorizontal } from "lucide-react";
import type { RollMode } from "../../generated/atlas";

export function RollModeControl({
  label,
  value,
  onChange,
  disabled,
}: {
  label: string;
  value: RollMode;
  onChange: (value: RollMode) => void;
  disabled?: boolean;
}) {
  return (
    <Dropdown
      trigger={["click"]}
      disabled={disabled}
      menu={{
        selectable: true,
        selectedKeys: [value],
        items: [
          { key: "normal", label: "Normal" },
          { key: "fortune", label: "Fortune" },
          { key: "misfortune", label: "Misfortune" },
        ],
        onClick: ({ key }) => onChange(key as RollMode),
      }}
    >
      <Tooltip
        title={
          value === "normal"
            ? "Roll options"
            : value === "fortune"
              ? "Fortune"
              : "Misfortune"
        }
        trigger={["hover", "focus"]}
      >
        <Button
          size="small"
          disabled={disabled}
          type={value === "normal" ? "text" : "primary"}
          aria-label={`${label} roll options: ${value}`}
          icon={
            value === "normal" ? (
              <SlidersHorizontal size={14} />
            ) : value === "fortune" ? (
              <ArrowUp size={14} />
            ) : (
              <ArrowDown size={14} />
            )
          }
        />
      </Tooltip>
    </Dropdown>
  );
}
