import { Button } from "antd";
import { useNavigate } from "@tanstack/react-router";
import { AtlasLink } from "../../app/AtlasLink";
import { characterDestination, partyDestination } from "../../app/destinations";
import type { CharacterPartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { DirectoryHeader } from "../../shared/ui/pages/EntityIndexPage";
import { EncounterDirectoryNav } from "../encounters/EncounterDirectoryNav";
import { AddCharacter } from "./AddCharacter";
import { useCharacters } from "./characterQueries";

export function CharacterMemberships({ parties }: { parties: CharacterPartyView[] }) {
  return parties.length ? (
    <ul className="character-memberships" aria-label="Party memberships">
      {parties.map((party) => (
        <li key={party.party_key}>
          <AtlasLink {...partyDestination(party.party_key)}>{party.name}</AtlasLink>
        </li>
      ))}
    </ul>
  ) : (
    <>No parties</>
  );
}

export function CharactersPage() {
  usePageTitle("Characters");
  const query = useCharacters();
  const navigate = useNavigate();
  return (
    <main className="directory-page">
      <DirectoryHeader
        title="Characters"
        navigation={<EncounterDirectoryNav current="Characters" />}
        actions={
          <AddCharacter onSaved={(key) => void navigate(characterDestination(key))} />
        }
      />
      {query.error && (
        <ErrorNotice
          title="Characters could not load"
          error={query.error}
          action={<Button onClick={() => void query.refetch()}>Retry</Button>}
        />
      )}
      {query.isPending && <LoadingStatus label="Loading characters…" />}
      {query.data &&
        (query.data.characters.length ? (
          <table className="party-roster">
            <thead>
              <tr>
                <th>Character</th>
                <th>Level</th>
                <th>Parties</th>
              </tr>
            </thead>
            <tbody>
              {query.data.characters.map((character) => (
                <tr key={character.character_key}>
                  <td>
                    <AtlasLink {...characterDestination(character.character_key)}>
                      {character.name}
                    </AtlasLink>
                  </td>
                  <td>{character.level ?? "—"}</td>
                  <td>
                    <CharacterMemberships parties={character.parties} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p>No saved characters yet.</p>
        ))}
    </main>
  );
}
