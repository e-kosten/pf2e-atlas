import { AtlasLink } from "../../app/AtlasLink";
import { charactersDestination, partiesDestination } from "../../app/destinations";

const directories = [
  { label: "All characters", destination: charactersDestination() },
  { label: "Parties", destination: partiesDestination() },
] as const;

export function CharacterSectionNav({
  current,
}: {
  current: (typeof directories)[number]["label"];
}) {
  return (
    <nav className="character-section-nav" aria-label="Character pages">
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
