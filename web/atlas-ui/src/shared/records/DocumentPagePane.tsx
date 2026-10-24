import type { ReactNode } from "react";

export function DocumentPagePane({
  identity,
  children,
  layout = "document",
}: {
  identity?: ReactNode;
  children: ReactNode;
  layout?: "document" | "creature";
}) {
  return (
    <section className={`record-view__pane record-view__pane--${layout}`}>
      {identity}
      {children}
    </section>
  );
}
