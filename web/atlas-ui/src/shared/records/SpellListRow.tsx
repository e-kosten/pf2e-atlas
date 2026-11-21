import type { ReactNode } from "react";

export function SpellListRow({
  label,
  children,
}: {
  label: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="actor-casting__row">
      <strong>{label}</strong>
      <span>{children}</span>
    </div>
  );
}
