import { Info } from "lucide-react";
import type { CheckDetailsView } from "../../generated/atlas";
import { ValueExplanation } from "../ui/ValueExplanation";
import { TraitBadge } from "./TraitBadge";

export function CheckDetails({
  label,
  details,
}: {
  label: string;
  details: CheckDetailsView;
}) {
  return (
    <ValueExplanation
      iconOnly
      label={`Details for ${label}`}
      title={label}
      content={
        <div>
          <strong>Traits</strong>
          <div className="check-details__traits">
            {details.traits.map((trait) => (
              <TraitBadge key={trait.value} trait={trait} />
            ))}
          </div>
        </div>
      }
    >
      <Info size={14} aria-hidden="true" />
    </ValueExplanation>
  );
}
