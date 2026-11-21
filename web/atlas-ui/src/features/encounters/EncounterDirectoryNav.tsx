import { AtlasLink } from "../../app/AtlasLink";
import {
  charactersDestination,
  encountersDestination,
  partiesDestination,
} from "../../app/destinations";

const directories = [
  { label: "Encounters", destination: encountersDestination() },
  { label: "Parties", destination: partiesDestination() },
  { label: "Characters", destination: charactersDestination() },
] as const;

export function EncounterDirectoryNav({
  current,
}: {
  current: (typeof directories)[number]["label"];
}) {
  return (
    <nav className="directory-nav" aria-label="Encounter directories">
      {directories.map(({ label, destination }) => (
        <AtlasLink
          key={label}
          {...destination}
          aria-current={label === current ? "page" : undefined}
        >
          {label}
        </AtlasLink>
      ))}
    </nav>
  );
}
