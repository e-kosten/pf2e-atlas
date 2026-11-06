import type { ReactNode } from "react";

/** Shared browse toolbar frame with a naturally wrapping content row. */
export function BrowsePaneHeader({
  className,
  children,
}: {
  className: string;
  children: ReactNode;
}) {
  return (
    <div className={`browse-pane-header ${className}`}>
      <div className="browse-pane-header__content">{children}</div>
    </div>
  );
}
