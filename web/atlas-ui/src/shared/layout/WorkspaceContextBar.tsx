import type { ReactNode } from "react";

/** Shared title strip for the saved-list and encounter workbenches. */
export function WorkspaceContextBar({
  title,
  leading,
  actions,
}: {
  title?: string;
  leading?: ReactNode;
  actions?: ReactNode;
}) {
  return (
    <header className="workspace-context-bar">
      <div className="workspace-context-bar__identity">
        {leading}
        {title && <h2>{title}</h2>}
      </div>
      {actions && <div className="workspace-context-bar__actions">{actions}</div>}
    </header>
  );
}
