import type { ReactNode } from "react";
import "../../styles/ui/actor-encounter-layout.css";

export type ActorEncounterContent = {
  durability: ReactNode;
  conditions: ReactNode;
  participantKey: string;
};

/** Shared running layout; callers retain ownership of rules and live controls. */
export function ActorEncounterLayout({
  durability,
  runtime,
  children,
}: {
  durability?: ReactNode;
  runtime: ReactNode;
  children?: ReactNode;
}) {
  if (!runtime)
    return (
      <>
        {durability}
        {children}
      </>
    );
  return (
    <div className="actor-encounter-host">
      <div
        className={`actor-encounter-layout${children ? " actor-encounter-layout--with-rules" : ""}`}
      >
        {durability && (
          <div className="actor-encounter-layout__durability">{durability}</div>
        )}
        <aside className="actor-encounter-layout__runtime">{runtime}</aside>
        {children && <div className="actor-encounter-layout__rules">{children}</div>}
      </div>
    </div>
  );
}
