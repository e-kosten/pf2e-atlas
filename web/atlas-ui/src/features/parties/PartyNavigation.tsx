import { AtlasLink } from "../../app/AtlasLink";
import {
  charactersDestination,
  encountersDestination,
  partiesDestination,
} from "../../app/destinations";

export function PartyNavigation({ active }: { active: "parties" | "characters" }) {
  return (
    <nav className="party-navigation" aria-label="Character and party navigation">
      <AtlasLink {...encountersDestination()}>Encounters</AtlasLink>
      <AtlasLink
        {...partiesDestination()}
        aria-current={active === "parties" ? "page" : undefined}
      >
        Parties
      </AtlasLink>
      <AtlasLink
        {...charactersDestination()}
        aria-current={active === "characters" ? "page" : undefined}
      >
        Characters
      </AtlasLink>
    </nav>
  );
}
