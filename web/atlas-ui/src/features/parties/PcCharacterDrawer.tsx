import { Button, ConfigProvider, Select, Space, Typography } from "antd";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";
import { getEncounterPcReference, setEncounterPcReference } from "../../api/atlasApi";
import type { EncounterPcAssociation } from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { AtlasDrawer } from "../../shared/ui/overlays/AtlasDrawer";
import { useCharacters, useSavedAbilityLink } from "./characterQueries";
import { SavedCharacterSheet } from "./CharacterSheet";
import { PcAbilityBrowser } from "./PcAbilityBrowser";

type EncounterLocation = {
  encounterKey: string;
  participantKey: string;
  revision: number;
};

const allPcReferences = ["pc-reference"] as const;
const getPopupContainer = (trigger?: HTMLElement) =>
  trigger?.closest<HTMLElement>(".pc-character-drawer-root") ?? document.body;
export function PcCharacterDrawer({
  location,
  name,
  opener,
  onClose,
}: {
  location: EncounterLocation;
  name: string;
  opener?: HTMLElement | null;
  onClose?: () => void;
}) {
  const client = useQueryClient();
  const titleId = useId();
  const paused = useSavedWritesPaused();
  const [closing, setClosing] = useState(false);
  const [editingAssociation, setEditingAssociation] = useState(false);
  const [memberKey, setMemberKey] = useState<string>();
  const queryKey = [
    ...allPcReferences,
    "encounter",
    location.encounterKey,
    location.participantKey,
  ];
  const reference = useQuery({
    queryKey,
    queryFn: async ({ signal }) =>
      (
        await getEncounterPcReference(
          location.encounterKey,
          location.participantKey,
          signal,
        )
      ).reference ?? null,
    retry: false,
    meta: referenceQueryMeta,
  });
  const doc = reference.data;
  const characters = useCharacters(editingAssociation);
  const save = useSavedAbilityLink(doc, queryKey);
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

  const busy = save.isPending || attach.isPending;
  const associationOptions = characters.data?.characters.map((character) => ({
    value: character.character_key,
    label: [
      character.name,
      character.level !== undefined && `Level ${character.level}`,
      ...character.parties.map((party) => party.name),
    ]
      .filter(Boolean)
      .join(" · "),
  }));
  return (
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
      title={
        <span id={titleId} className="pc-character-drawer__title">
          {doc?.name ?? name}
        </span>
      }
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
      <div className="pc-character-drawer__association">
        {reference.isPending && <LoadingStatus label="Loading PC reference…" />}
        {doc !== undefined && (
          <Space wrap>
            {doc?.level !== undefined && (
              <Typography.Text type="secondary">Level {doc.level}</Typography.Text>
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
        {editingAssociation && (
          <>
            {characters.error && (
              <ErrorNotice error={characters.error} title="Could not load characters" />
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
      </div>
      {doc && (
        <SavedCharacterSheet
          key={doc.character_key}
          character={doc}
          defaultTab="abilities"
          headingLevel={doc.level}
          refreshCharacter={async () =>
            (await reference.refetch()).data?.character_revision
          }
          active={!closing}
          abilities={(active) => (
            <PcAbilityBrowser
              active={active}
              revision={doc.character_revision}
              doc={doc}
              link={save}
              blocked={paused || attach.isPending}
            />
          )}
        />
      )}
    </AtlasDrawer>
  );
}
