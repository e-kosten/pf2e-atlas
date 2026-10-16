import type { ReactNode } from "react";

/** Shared primary control layout; sizes inherit the root Ant configuration. */
export function PrimaryToolbar({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={["primary-toolbar", className].filter(Boolean).join(" ")}>
      {children}
    </div>
  );
}
