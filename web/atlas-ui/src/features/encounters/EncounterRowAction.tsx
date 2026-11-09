import { Button, Tooltip, type ButtonProps } from "antd";

export function EncounterRowAction({
  tooltip,
  hideTooltip,
  ...buttonProps
}: Omit<ButtonProps, "type" | "size"> & {
  tooltip: string;
  hideTooltip?: boolean;
}) {
  return (
    <Tooltip
      title={tooltip}
      trigger={["hover", "focus"]}
      open={hideTooltip ? false : undefined}
    >
      <Button {...buttonProps} type="text" size="small" />
    </Tooltip>
  );
}
