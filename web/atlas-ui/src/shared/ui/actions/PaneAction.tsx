import { Button } from "antd";

export function PaneIconButton({
  disabled,
  icon,
  label,
  onClick,
  title = label,
}: {
  disabled?: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  title?: string;
}) {
  return (
    <Button
      aria-label={label}
      className="pane-toggle"
      disabled={disabled}
      icon={icon}
      onClick={onClick}
      title={title}
      type="text"
    />
  );
}

export function PaneIconLink({
  href,
  icon,
  label,
  onClick,
  title = label,
}: {
  href: string;
  icon: React.ReactNode;
  label: string;
  onClick?: React.MouseEventHandler<HTMLAnchorElement>;
  title?: string;
}) {
  return (
    <Button
      aria-label={label}
      className="pane-toggle"
      href={href}
      icon={icon}
      onClick={onClick}
      title={title}
      type="text"
    />
  );
}
