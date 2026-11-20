import { Button, ConfigProvider, Select, Space, Typography } from "antd";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";
import {
  getCharacter,
  getEncounterPcReference,
  savePcAbilityLink,
  setEncounterPcReference,
} from "../../api/atlasApi";
import type {
  EncounterPcAssociation,
  PcAbilitySelection,
  PcAbilityView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { AtlasDrawer } from "../../shared/ui/overlays/AtlasDrawer";
import { refreshCharacters, useCharacters } from "./characterQueries";
import { PcAbilityBrowser, type AbilityNavigation } from "./PcAbilityBrowser";

type CharacterLocation = { kind: "character"; characterKey: string };
type EncounterLocation = {
  kind: "encounter";
  encounterKey: string;
  participantKey: string;
  revision: number;
};
type Location = CharacterLocation | EncounterLocation;

const allPcReferences = ["pc-reference"] as const;
const getPopupContainer = (trigger?: HTMLElement) =>
  trigger?.closest<HTMLElement>(".pc-character-drawer-root") ?? document.body;
export function CharacterReference({
  location,
  name,
  opener,
  onClose,
  onRefresh,
  navigation,
  mode = "drawer",
}: {
  location: Location;
  name: string;
  opener?: HTMLElement | null;
  onClose?: () => void;
  mode?: "page" | "drawer";
  navigation?: AbilityNavigation;
  onRefresh?: () => void;
}) {
  const client = useQueryClient();
  const titleId = useId();
  const paused = useSavedWritesPaused();
  const [closing, setClosing] = useState(false);
  const [editingAssociation, setEditingAssociation] = useState(false);
  const [memberKey, setMemberKey] = useState<string>();
  const reference = useQuery({
    queryKey: resultReferenceKey(location),
    queryFn: async ({ signal }) =>
      location.kind === "character"
        ? getCharacter(location.characterKey, signal)
        : ((
            await getEncounterPcReference(
              location.encounterKey,
              location.participantKey,
              signal,
            )
          ).reference ?? null),
    retry: false,
    meta: referenceQueryMeta,
  });
  const characters = useCharacters(location.kind === "encounter" && editingAssociation);
  const save = useMutation({
    retry: false,
    mutationFn: ({
      ability,
      selection,
    }: {
      ability: PcAbilityView;
      selection: PcAbilitySelection;
    }) => {
      const target = reference.data;
      if (!target) throw new Error("Character is unavailable.");
      return savePcAbilityLink(target.character_key, ability.ability_key, {
        expected_character_revision: target.character_revision,
        selection,
      });
    },
    onSuccess: async (saved) => {
      client.setQueryData(resultReferenceKey(location), saved);
      await refreshCharacters(client);
    },
    onError: () => {
      void reference.refetch();
    },
  });
  const attach = useMutation({
    retry: false,
    mutationFn: ({
      encounter,
      association,
    }: {
      encounter: EncounterLocation;
      association: EncounterPcAssociation;
    }) =>
      setEncounterPcReference(encounter.encounterKey, encounter.participantKey, {
        expected_encounter_revision: encounter.revision,
        association,
      }),
    onSuccess: async (_receipt, { encounter }) => {
      await Promise.all([
        client.invalidateQueries({ queryKey: allPcReferences }),
        client.invalidateQueries({ queryKey: ["encounter", encounter.encounterKey] }),
      ]);
      setEditingAssociation(false);
    },
    onError: (_error, { encounter }) => {
      void client.invalidateQueries({
        queryKey: ["encounter", encounter.encounterKey],
      });
    },
  });

  const doc = reference.data;
  const busy = save.isPending || attach.isPending;
  const associationOptions = characters.data?.characters.map((character) => ({
    value: character.character_key,
    label: character.name,
  }));
  const content = (
    <>
      {reference.error && (
        <ErrorNotice
          error={reference.error}
          title="Could not load PC reference"
          action={<Button onClick={() => void reference.refetch()}>Retry</Button>}
        />
      )}
      {attach.error && (
        <ErrorNotice error={attach.error} title="Could not update character link" />
      )}
      <PcAbilityBrowser
        key={doc?.character_key}
        navigation={navigation}
        doc={doc}
        link={save}
        blocked={paused || attach.isPending}
        toolbar={
          <>
            {reference.isPending && <p role="status">Loading PC reference…</p>}
            {location.kind === "encounter" && (doc || doc === null) && (
              <Space wrap>
                {doc && (
                  <Typography.Text type="secondary">
                    {doc.name}’s character reference
                  </Typography.Text>
                )}
                <Button
                  disabled={paused || busy}
                  onClick={() => {
                    setMemberKey(doc?.character_key);
                    setEditingAssociation(!editingAssociation);
                  }}
                >
                  {doc ? "Change character" : "Attach to character"}
                </Button>
                {doc && (
                  <Button
                    disabled={paused || busy}
                    onClick={() =>
                      attach.mutate({
                        encounter: location,
                        association: { kind: "detach" },
                      })
                    }
                  >
                    Detach
                  </Button>
                )}
              </Space>
            )}
            {location.kind === "encounter" && editingAssociation && (
              <>
                {characters.error && (
                  <ErrorNotice
                    error={characters.error}
                    title="Could not load characters"
                  />
                )}
                <Space wrap>
                  <Select
                    aria-label="Character"
                    placeholder="Choose a character"
                    options={associationOptions}
                    value={memberKey}
                    onChange={setMemberKey}
                    loading={characters.isPending}
                    style={{ minWidth: "14rem" }}
                  />
                  <Button
                    type="primary"
                    disabled={paused || !memberKey || busy}
                    onClick={() => {
                      if (memberKey)
                        attach.mutate({
                          encounter: location,
                          association: { kind: "attach", character_key: memberKey },
                        });
                    }}
                  >
                    Attach
                  </Button>
                </Space>
              </>
            )}
            {doc && (
              <>
                {(mode === "drawer" || onRefresh) && (
                  <Space wrap>
                    {mode === "drawer" && doc.level !== undefined && (
                      <Typography.Text type="secondary">
                        Level {doc.level}
                      </Typography.Text>
                    )}
                    {onRefresh && (
                      <Button disabled={paused} onClick={onRefresh}>
                        {doc.imported
                          ? "Refresh from Pathbuilder"
                          : "Import from Pathbuilder"}
                      </Button>
                    )}
                  </Space>
                )}
              </>
            )}
          </>
        }
      />
    </>
  );
  return mode === "page" ? (
    content
  ) : (
    <AtlasDrawer
      open={!closing}
      aria-labelledby={titleId}
      onClose={() => setClosing(true)}
      closable={!busy}
      keyboard={!busy}
      mask={{ closable: !busy }}
      afterOpenChange={(open) => {
        if (!open) {
          onClose?.();
          opener?.focus({ preventScroll: true });
        }
      }}
      size="min(1080px, 100vw)"
      title={<span id={titleId}>{doc?.name ?? name}</span>}
      focusable={{ focusTriggerAfterClose: false }}
      classNames={{ root: "pc-character-drawer-root" }}
      styles={{
        root: { overflow: "hidden", pointerEvents: "auto" },
        body: {
          display: "flex",
          flexDirection: "column",
          gap: 12,
          minHeight: 0,
          overflow: "hidden",
        },
      }}
      drawerRender={(content) => (
        <ConfigProvider getPopupContainer={getPopupContainer}>{content}</ConfigProvider>
      )}
    >
      {content}
    </AtlasDrawer>
  );
}

function resultReferenceKey(location: Location) {
  return location.kind === "character"
    ? ["character", location.characterKey]
    : [...allPcReferences, "encounter", location.encounterKey, location.participantKey];
}

export function PcCharacterDrawer(
  props: Omit<React.ComponentProps<typeof CharacterReference>, "mode">,
) {
  return <CharacterReference {...props} mode="drawer" />;
}
