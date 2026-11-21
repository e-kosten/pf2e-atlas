import { Button, Popover } from "antd";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Ellipsis } from "lucide-react";
import { useRef, useState } from "react";
import { deleteCharacter } from "../../api/atlasApi";
import { PcAbilityBrowser, type AbilityNavigation } from "./PcAbilityBrowser";
import { charactersDestination } from "../../app/destinations";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { SavedCharacterSheet } from "./CharacterSheet";
import { CharacterEditor } from "./CharacterEditor";
import { CharacterMemberships } from "./CharactersPage";
import {
  characterQueryKey,
  refreshCharacters,
  useCharacter,
  useSavedAbilityLink,
} from "./characterQueries";
import { EncounterDirectoryNav } from "../encounters/EncounterDirectoryNav";
import { PathbuilderImport } from "./PathbuilderImport";
import type { PcReferenceView } from "../../generated/atlas";

export function CharacterPage({
  characterKey,
  navigation,
}: {
  characterKey: string;
  navigation: AbilityNavigation;
}) {
  const query = useCharacter(characterKey);
  usePageTitle(query.data?.name ?? "Character");
  return (
    <div className="directory-page character-workspace">
      <EncounterDirectoryNav current="Characters" />
      <div className="character-workspace__document">
        {query.error && (
          <ErrorNotice
            title="Character could not load"
            error={query.error}
            action={<Button onClick={() => void query.refetch()}>Retry</Button>}
          />
        )}
        {query.isPending && <LoadingStatus label="Loading character…" />}
        {query.data && (
          <CharacterDocument
            key={characterKey}
            character={query.data}
            navigation={navigation}
            refreshCharacter={async () =>
              (await query.refetch()).data?.character_revision
            }
          />
        )}
        {query.data === null && <p>Character unavailable.</p>}
      </div>
    </div>
  );
}

function CharacterDocument({
  character,
  navigation,
  refreshCharacter,
}: {
  character: PcReferenceView;
  refreshCharacter: () => Promise<number | undefined>;
  navigation: AbilityNavigation;
}) {
  const actions = useRef<HTMLButtonElement>(null);
  const paused = useSavedWritesPaused();
  const link = useSavedAbilityLink(
    character,
    characterQueryKey(character.character_key),
  );
  const navigate = useNavigate();
  const client = useQueryClient();
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<{
    mode: "edit" | "import";
    baseline: PcReferenceView;
    opener: HTMLElement | null;
  }>();
  function show(mode: "edit" | "import", opener: HTMLElement | null = actions.current) {
    setOpen(false);
    opener?.focus({ preventScroll: true });
    setEdit({ mode, baseline: character, opener });
  }
  function closeEditor() {
    setEdit(undefined);
    restoreOverlayFocus(
      () => (edit?.opener?.isConnected ? edit.opener : actions.current),
      undefined,
    );
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
                New import from Pathbuilder
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
          <Button
            ref={actions}
            icon={<Ellipsis size={16} />}
            aria-label="Character actions"
            aria-expanded={open}
          />
        </Popover>
      </header>
      <SavedCharacterSheet
        character={character}
        defaultTab="stats"
        emptySheetAction={
          <Button
            type="primary"
            disabled={paused}
            onClick={(event) => show("import", event.currentTarget)}
          >
            Import from Pathbuilder
          </Button>
        }
        refreshCharacter={refreshCharacter}
        abilityRoute={{ active: Boolean(navigation.view), close: navigation.close }}
        abilities={(active) => (
          <PcAbilityBrowser
            navigation={navigation}
            doc={character}
            revision={character.character_revision}
            link={link}
            blocked={paused}
            active={active}
          />
        )}
      />
      {edit?.mode === "edit" && (
        <CharacterEditor
          character={edit.baseline}
          onClose={closeEditor}
          onSaved={closeEditor}
        />
      )}
      {edit?.mode === "import" && (
        <PathbuilderImport
          character={edit.baseline}
          onClose={closeEditor}
          onSaved={closeEditor}
        />
      )}
    </>
  );
}
