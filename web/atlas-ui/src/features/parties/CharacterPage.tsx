import { Button, Popover } from "antd";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { deleteCharacter } from "../../api/atlasApi";
import { charactersDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { CharacterEditor } from "./CharacterEditor";
import { CharacterMemberships } from "./CharactersPage";
import { refreshCharacters, useCharacter } from "./characterQueries";
import { PartyNavigation } from "./PartyNavigation";
import { CharacterReference } from "./PcCharacterDrawer";
import { PathbuilderImport } from "./PathbuilderImport";
import type { PcReferenceView } from "../../generated/atlas";

export function CharacterPage({ characterKey }: { characterKey: string }) {
  const query = useCharacter(characterKey);
  usePageTitle(query.data?.name ?? "Character");
  return (
    <div className="party-page">
      <PartyNavigation active="characters" />
      {query.error && (
        <ErrorNotice
          title="Character could not load"
          error={query.error}
          action={<Button onClick={() => void query.refetch()}>Retry</Button>}
        />
      )}
      {query.isPending && <p role="status">Loading character…</p>}
      {query.data && <CharacterDocument key={characterKey} character={query.data} />}
      {query.data === null && <p>Character unavailable.</p>}
    </div>
  );
}

function CharacterDocument({ character }: { character: PcReferenceView }) {
  const paused = useSavedWritesPaused();
  const navigate = useNavigate();
  const client = useQueryClient();
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<{
    mode: "edit" | "import";
    baseline: PcReferenceView;
  }>();
  function show(mode: "edit" | "import") {
    setOpen(false);
    setEdit({ mode, baseline: character });
  }
  return (
    <>
      <header className="party-page-heading character-heading">
        <div>
          <h1>
            {character.name}{" "}
            {character.level != null && <small>Level {character.level}</small>}
          </h1>
          <div className="character-secondary">
            {character.parties.length > 0 && "Parties "}
            <CharacterMemberships parties={character.parties} />
          </div>
        </div>
        <Popover
          trigger="click"
          placement="bottomRight"
          open={open}
          onOpenChange={setOpen}
          content={
            <div className="party-context-actions">
              <Button type="text" disabled={paused} onClick={() => show("edit")}>
                Edit name and level
              </Button>
              <Button type="text" disabled={paused} onClick={() => show("import")}>
                {character.imported
                  ? "Refresh from Pathbuilder"
                  : "Import from Pathbuilder"}
              </Button>
              <DangerActionButton
                type="text"
                disabled={paused}
                confirmTitle={`Delete ${character.name}?`}
                confirmOkText="Delete character"
                confirmContent="This removes the character from all parties and detaches its live rules from encounters. Encounter participants keep their names, levels, HP and initiative."
                onConfirm={async () => {
                  await deleteCharacter(character.character_key, {
                    expected_revision: character.character_revision,
                  });
                  await navigate(charactersDestination());
                  await refreshCharacters(client);
                }}
              >
                Delete character
              </DangerActionButton>
            </div>
          }
        >
          <Button aria-expanded={open}>Actions</Button>
        </Popover>
      </header>
      <CharacterReference
        mode="page"
        location={{ kind: "character", characterKey: character.character_key }}
        name={character.name}
      />
      {edit?.mode === "edit" && (
        <CharacterEditor
          character={edit.baseline}
          onClose={() => setEdit(undefined)}
          onSaved={() => setEdit(undefined)}
        />
      )}
      {edit?.mode === "import" && (
        <PathbuilderImport
          character={edit.baseline}
          onClose={() => setEdit(undefined)}
          onSaved={() => setEdit(undefined)}
        />
      )}
    </>
  );
}
