import type { RecordSummaryView } from "../../generated/atlas";
import { RecordEditionBadge } from "./RecordEditionBadge";
import { RecordRarity } from "./RecordRarity";
import { TraitBadge } from "./TraitBadge";

export function RecordIdentityBadges({
  identity,
  linked = true,
  className = "badge-row",
}: {
  identity?: Partial<Pick<RecordSummaryView, "rarity" | "edition" | "traits">>;
  linked?: boolean;
  className?: string;
}) {
  const traits = identity?.traits?.filter((trait) => trait.kind !== "rarity") ?? [];
  const rarity = identity?.rarity?.label.trim() ? identity.rarity : undefined;
  if (!rarity && !identity?.edition && !traits.length) return null;

  return (
    <div className={className}>
      <RecordRarity rarity={rarity} linked={linked} />
      <RecordEditionBadge edition={identity?.edition} />
      {traits.map((trait) => (
        <TraitBadge
          key={`${trait.kind}-${trait.value}`}
          trait={trait}
          linked={linked}
        />
      ))}
    </div>
  );
}
