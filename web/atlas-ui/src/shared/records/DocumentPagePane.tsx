import type { ReactNode } from "react";
import { PaneFrame } from "../layout/PaneLayout";

export function DocumentPagePane({
  actions,
  children,
  title = "Reference",
}: {
  actions?: ReactNode;
  children: ReactNode;
  title?: string;
}) {
  return (
    <PaneFrame className="record-view__pane" headerActions={actions} label={title}>
      {children}
    </PaneFrame>
  );
}
