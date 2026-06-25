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
    <button
      aria-label={label}
      className="pane-toggle"
      disabled={disabled}
      onClick={onClick}
      title={title}
      type="button"
    >
      {icon}
    </button>
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
    <a
      aria-label={label}
      className="pane-toggle"
      href={href}
      onClick={onClick}
      title={title}
    >
      {icon}
    </a>
  );
}
