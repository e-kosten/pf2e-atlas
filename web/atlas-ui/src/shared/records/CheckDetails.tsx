import type { CheckDetailsView } from "../../generated/atlas";
import { ValueExplanation } from "../ui/ValueExplanation";
import { TraitBadge } from "./TraitBadge";

export function CheckDetails({
  label,
  details,
  iconOnly,
}: {
  label: string;
  details: CheckDetailsView;
  iconOnly: boolean;
}) {
  return (
    <ValueExplanation
      iconOnly={iconOnly}
      label={iconOnly ? `Details for ${label}` : "Check details"}
      title={label}
      content={
        <div className="check-details__traits">
          {details.traits.map((trait) => (
            <TraitBadge key={trait.value} trait={trait} />
          ))}
        </div>
      }
    >
      {label}
    </ValueExplanation>
  );
}
