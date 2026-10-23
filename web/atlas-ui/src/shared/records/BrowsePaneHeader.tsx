import type { ReactNode } from "react";

/** The browse host aligns these headers; each content row keeps its natural height. */
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
