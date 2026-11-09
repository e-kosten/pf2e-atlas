import { Button, Tooltip, type ButtonProps, type TooltipProps } from "antd";

export function EncounterRowAction({
  tooltip,
  hideTooltip,
  tooltipPlacement,
  ...buttonProps
}: Omit<ButtonProps, "type" | "size"> & {
  tooltip: string;
  hideTooltip?: boolean;
  tooltipPlacement?: TooltipProps["placement"];
}) {
  return (
    <Tooltip
      title={tooltip}
      placement={tooltipPlacement}
      trigger={["hover", "focus"]}
      open={hideTooltip ? false : undefined}
    >
      <Button {...buttonProps} type="text" size="small" />
    </Tooltip>
  );
}
