import { Button, Tooltip } from "antd";
import type { ComponentPropsWithRef, ReactNode } from "react";

type PaneActionProps = Omit<
  ComponentPropsWithRef<typeof Button>,
  "aria-label" | "children" | "icon" | "type"
> & {
  icon: ReactNode;
  label: string;
  tooltip?: boolean;
  outlined?: boolean;
};

export function PaneIconButton({
  className,
  icon,
  label,
  title = label,
  tooltip = true,
  outlined = false,
  ...buttonProps
}: Omit<PaneActionProps, "href"> & {
  onClick: NonNullable<PaneActionProps["onClick"]>;
}) {
  const action = (
    <Button
      {...buttonProps}
      aria-label={label}
      className={["pane-toggle", className].filter(Boolean).join(" ")}
      icon={icon}
      title={tooltip ? undefined : title}
      type={outlined ? "default" : "text"}
    />
  );
  return tooltip ? (
    <Tooltip title={title} trigger={["hover", "focus"]}>
      {action}
    </Tooltip>
  ) : (
    action
  );
}

export function PaneIconLink({
  className,
  icon,
  label,
  title = label,
  tooltip = true,
  outlined = false,
  ...buttonProps
}: PaneActionProps & {
  href: string;
}) {
  const action = (
    <Button
      {...buttonProps}
      aria-label={label}
      className={["pane-toggle", className].filter(Boolean).join(" ")}
      icon={icon}
      title={tooltip ? undefined : title}
      type={outlined ? "default" : "text"}
    />
  );
  return tooltip ? (
    <Tooltip title={title} trigger={["hover", "focus"]}>
      {action}
    </Tooltip>
  ) : (
    action
  );
}
