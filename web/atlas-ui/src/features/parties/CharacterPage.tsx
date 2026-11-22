import { Button } from "antd";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useLayoutEffect, useRef, useState } from "react";
import { deleteCharacter } from "../../api/atlasApi";
import { PcAbilityBrowser, type AbilityNavigation } from "./PcAbilityBrowser";
import { charactersDestination } from "../../app/destinations";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { EntityActionsMenu } from "../../shared/ui/actions/EntityActionsMenu";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import { SavedCharacterSheet } from "./CharacterSheet";
import { CharacterEditor } from "./CharacterEditor";
import { CharacterMemberships } from "./CharactersPage";
import {
  characterQueryKey,
  refreshCharacters,
  useCharacter,
  useSavedAbilityLink,
} from "./characterQueries";
import { CharacterSectionNav } from "./CharacterSectionNav";
import { PathbuilderImport } from "./PathbuilderImport";
import type { PcReferenceView } from "../../generated/atlas";

const shortCharacterHeight = 520;
const shortCharacterWorkspace = `(max-height: ${shortCharacterHeight}px)`;

type CharacterScrollPosition = { pane: HTMLElement; offset: number };

function useCharacterWorkspaceScroll() {
  const workspace = useRef<HTMLDivElement>(null);
  const [scrollPage, setScrollPage] = useState(
    () => window.matchMedia(shortCharacterWorkspace).matches,
  );
  const handoff = useRef<CharacterScrollPosition | undefined>(undefined);
  const pagePosition = useRef<CharacterScrollPosition | undefined>(undefined);

  useLayoutEffect(() => {
    const root = workspace.current;
    const page = root?.closest<HTMLElement>(".atlas-app__content");
    if (!root || !page) return;
    const workspaceRoot = root;
    const pageScroller = page;
    const media = window.matchMedia(shortCharacterWorkspace);

    function currentPane() {
      const panes = Array.from(
        workspaceRoot.querySelectorAll<HTMLElement>(
          ".character-sheet__pane, .reference-results__scroll, .reference-reader__scroll",
        ),
      ).filter((pane) => pane.getClientRects().length > 0);
      return (
        panes.find((candidate) => candidate.contains(document.activeElement)) ??
        panes.find((candidate) => candidate.scrollTop > 0) ??
        panes[panes.length - 1]
      );
    }

    function rememberPagePosition() {
      // The browser may clamp the outer scroll position before the resize event.
      if (
        window.innerHeight > shortCharacterHeight ||
        !workspaceRoot.hasAttribute("data-scroll-page")
      )
        return;
      const previousPane = pagePosition.current?.pane;
      const pane = previousPane?.getClientRects().length ? previousPane : currentPane();
      if (pane) {
        pagePosition.current = {
          pane,
          offset: Math.max(
            0,
            pageScroller.getBoundingClientRect().top - pane.getBoundingClientRect().top,
          ),
        };
      }
    }

    function resize(event: MediaQueryListEvent) {
      const pane = currentPane();
      handoff.current = event.matches
        ? pane && { pane, offset: pane.scrollTop }
        : pagePosition.current;
      setScrollPage(event.matches);
    }

    media.addEventListener("change", resize);
    pageScroller.addEventListener("scroll", rememberPagePosition, { passive: true });
    setScrollPage(media.matches);
    return () => {
      media.removeEventListener("change", resize);
      pageScroller.removeEventListener("scroll", rememberPagePosition);
    };
  }, []);

  useLayoutEffect(() => {
    const position = handoff.current;
    handoff.current = undefined;
    const page = workspace.current?.closest<HTMLElement>(".atlas-app__content");
    if (!position?.pane.isConnected || !page) return;
    if (scrollPage) {
      pagePosition.current = position;
      if (position.offset > 0) {
        page.scrollTop +=
          position.pane.getBoundingClientRect().top -
          page.getBoundingClientRect().top +
          position.offset;
      }
    } else {
      pagePosition.current = undefined;
      if (position.pane.getClientRects().length)
        position.pane.scrollTop = position.offset;
    }
  }, [scrollPage]);

  return { workspace, scrollPage };
}

export function CharacterPage({
  characterKey,
  navigation,
}: {
  characterKey: string;
  navigation: AbilityNavigation;
}) {
  const query = useCharacter(characterKey);
  const { workspace, scrollPage } = useCharacterWorkspaceScroll();
  usePageTitle(query.data?.name ?? "Character");
  return (
    <div
      ref={workspace}
      className="directory-page character-workspace"
      data-scroll-page={scrollPage || undefined}
    >
      <CharacterSectionNav current="All characters" />
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
  const deletion = useConfirmDangerAction();
  const [edit, setEdit] = useState<{
    mode: "edit" | "import";
    baseline: PcReferenceView;
    opener: HTMLElement | null;
  }>();
  function show(mode: "edit" | "import", opener: HTMLElement | null = actions.current) {
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
          {character.parties.length > 0 && (
            <div className="character-secondary">
              Parties <CharacterMemberships parties={character.parties} />
            </div>
          )}
        </div>
        <EntityActionsMenu
          label="Character actions"
          trigger={actions}
          items={[
            { key: "edit", label: "Edit name and level", disabled: paused },
            { key: "import", label: "New import from Pathbuilder", disabled: paused },
            {
              key: "delete",
              label: "Delete character",
              danger: true,
              disabled: paused,
            },
          ]}
          onAction={(key, opener) => {
            if (key === "edit" || key === "import") {
              show(key, opener);
            } else {
              deletion.confirm({
                title: `Delete ${character.name}?`,
                okText: "Delete character",
                content:
                  "This removes the character from all parties and detaches its live rules from encounters. Encounter participants keep their names, levels, HP and initiative.",
                onConfirm: async () => {
                  await deleteCharacter(character.character_key, {
                    expected_revision: character.character_revision,
                  });
                  await navigate(charactersDestination());
                  await refreshCharacters(client);
                },
              });
            }
          }}
        />
      </header>
      <SavedCharacterSheet
        character={character}
        headingLevel={character.level}
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
      {deletion.dialog}
      {edit?.mode === "edit" && (
        <CharacterEditor
          character={edit.baseline}
          onClose={closeEditor}
          onSaved={closeEditor}
        />
      )}
      {edit?.mode === "import" && (
        <PathbuilderImport character={edit.baseline} onClose={closeEditor} />
      )}
    </>
  );
}
