import { Button } from "antd";
import type { ComponentPropsWithRef, ReactNode } from "react";

type PaneActionProps = Omit<
  ComponentPropsWithRef<typeof Button>,
  "aria-label" | "children" | "icon" | "type"
> & {
  icon: ReactNode;
  label: string;
};

export function PaneIconButton({
  className,
  icon,
  label,
  title = label,
  ...buttonProps
}: Omit<PaneActionProps, "href"> & {
  onClick: NonNullable<PaneActionProps["onClick"]>;
}) {
  return (
    <Button
      {...buttonProps}
      aria-label={label}
      className={["pane-toggle", className].filter(Boolean).join(" ")}
      icon={icon}
      title={title}
      type="text"
    />
  );
}

export function PaneIconLink({
  className,
  icon,
  label,
  title = label,
  ...buttonProps
}: PaneActionProps & {
  href: string;
}) {
  return (
    <Button
      {...buttonProps}
      aria-label={label}
      className={["pane-toggle", className].filter(Boolean).join(" ")}
      icon={icon}
      title={title}
      type="text"
    />
  );
}
