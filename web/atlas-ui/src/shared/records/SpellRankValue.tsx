import { InfoCircleOutlined } from "@ant-design/icons";
import type { ReactNode } from "react";
import type { SpellRankChangeView } from "../../generated/atlas";
import { ValueExplanation } from "../ui/ValueExplanation";

/** A verified change relative to this copy's base, not complete coverage. */
export function SpellRankValue({
  change,
  label,
  children,
  interactiveContent = false,
}: {
  change?: SpellRankChangeView;
  label: string;
  children: ReactNode;
  interactiveContent?: boolean;
}) {
  if (!change) return <>{children}</>;
  const explanation = (
    <ValueExplanation
      label={`Heightening details for ${label}`}
      content={<>Base rank: {change.base_value}</>}
      iconOnly={interactiveContent}
    >
      {interactiveContent ? <InfoCircleOutlined /> : children}
    </ValueExplanation>
  );
  return interactiveContent ? (
    <>
      {children} {explanation}
    </>
  ) : (
    explanation
  );
}
