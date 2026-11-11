import type { ReactNode } from "react";

/** Shared title strip for the saved-list and encounter workbenches. */
export function WorkspaceContextBar({
  title,
  parentLink,
  actions,
}: {
  title?: string;
  parentLink?: ReactNode;
  actions?: ReactNode;
}) {
  return (
    <header className="workspace-context-bar">
      {parentLink && <div>{parentLink}</div>}
      <div className="workspace-context-bar__main">
        <div className="workspace-context-bar__identity">
          {title && <h2>{title}</h2>}
        </div>
        {actions && <div className="workspace-context-bar__actions">{actions}</div>}
      </div>
    </header>
  );
}
