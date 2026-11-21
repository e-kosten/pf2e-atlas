import { Button } from "antd";
import { useNavigate } from "@tanstack/react-router";
import { AtlasLink } from "../../app/AtlasLink";
import { characterDestination, partyDestination } from "../../app/destinations";
import type { CharacterPartyView, CharacterSummaryView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { IndexTable } from "../../shared/ui/tables/IndexTable";
import type { TableProps } from "antd";
import { CharacterSectionNav } from "./CharacterSectionNav";
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

function CharacterName({ character }: { character: CharacterSummaryView }) {
  return (
    <AtlasLink
      className="index-row-link__title"
      {...characterDestination(character.character_key)}
    >
      {character.name}
    </AtlasLink>
  );
}

const columns: TableProps<CharacterSummaryView>["columns"] = [
  {
    title: "Character",
    key: "name",
    render: (_, character) => <CharacterName character={character} />,
  },
  {
    title: "Level",
    dataIndex: "level",
    width: 88,
    render: (level?: number) => level ?? "—",
  },
  {
    title: "Parties",
    key: "parties",
    width: "35%",
    render: (_, character) => <CharacterMemberships parties={character.parties} />,
  },
];

const compactColumns: NonNullable<TableProps<CharacterSummaryView>["columns"]> = [
  {
    title: "Character",
    key: "name",
    render: (_, character) => (
      <div className="index-row-summary">
        <CharacterName character={character} />
        <div className="index-row-summary__facts">
          {character.level != null && <span>Level {character.level}</span>}
          <CharacterMemberships parties={character.parties} />
        </div>
      </div>
    ),
  },
];

export function CharactersPage() {
  usePageTitle("Characters");
  const query = useCharacters();
  const navigate = useNavigate();
  return (
    <EntityIndexPage
      className="characters-index-view"
      title="Characters"
      navigation={<CharacterSectionNav current="All characters" />}
      actions={
        <AddCharacter onSaved={(key) => void navigate(characterDestination(key))} />
      }
    >
      {query.error && (
        <ErrorNotice
          title={
            query.data ? "Characters could not refresh" : "Characters could not load"
          }
          error={query.error}
          action={<Button onClick={() => void query.refetch()}>Retry</Button>}
        />
      )}
      <IndexTable
        rowKey={(character) => character.character_key}
        onActivateRow={(character) =>
          void navigate(characterDestination(character.character_key))
        }
        columns={columns}
        compact={{ below: 520, columns: compactColumns }}
        dataSource={query.data?.characters ?? []}
        loading={query.isFetching}
        locale={{
          emptyText:
            query.isPending || (query.error && !query.data)
              ? null
              : "No saved characters yet. Add a PC to get started.",
        }}
      />
    </EntityIndexPage>
  );
}
