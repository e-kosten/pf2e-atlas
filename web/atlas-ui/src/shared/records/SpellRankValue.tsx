import type { ReactNode } from "react";
import type { SpellRankChangeView } from "../../generated/atlas";
import { ValueExplanation } from "../ui/ValueExplanation";

/** A verified change relative to this copy's base, not complete coverage. */
export function SpellRankValue({
  change,
  label,
  children,
}: {
  change?: SpellRankChangeView;
  label: string;
  children: ReactNode;
}) {
  if (!change) return <>{children}</>;
  return (
    <ValueExplanation
      label={`Heightening details for ${label}`}
      content={<>Base rank: {change.base_value}</>}
    >
      {children}
    </ValueExplanation>
  );
}
