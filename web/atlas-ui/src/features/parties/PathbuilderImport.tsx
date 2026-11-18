import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Input, List, Space, Typography } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useRef, useState } from "react";
import {
  addEncounterCharacter,
  getEncounter,
  getParties,
  previewPathbuilderPc,
  savePathbuilderPc,
} from "../../api/atlasApi";
import type {
  CharacterMutationView,
  EncounterCharacterMutationView,
  PcReferenceView,
  PartyView,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";

export function PathbuilderImport({
  party,
  character,
  encounter,
  onClose,
  onSaved,
  onEncounterAdded,
}: {
  party?: PartyView;
  character?: PcReferenceView;
  encounter?: { encounterKey: string; revision: number };
  onClose: () => void;
  onSaved?: (key?: string) => void;
  onEncounterAdded?: (result: { characterKey: string; participantKey: string }) => void;
}) {
  const textareaId = useId();
  const [requestId] = useState(() => crypto.randomUUID());
  const [text, setText] = useState("");
  const [reviewing, setReviewing] = useState(false);
  const [closing, setClosing] = useState(false);
  const paused = useSavedWritesPaused();
  const client = useQueryClient();
  const preview = useMutation({
    mutationFn: async (json: string) => {
      const view = await previewPathbuilderPc(json, character?.character_key);
      const target = party
        ? (await getParties()).parties.find(
            (item) => item.party_key === party.party_key,
          )
        : undefined;
      if (party && !target)
        throw new Error("This party is no longer available. Your export is retained.");
      return {
        view,
        party: target,
        encounterRevision: encounter
          ? (await getEncounter(encounter.encounterKey)).revision
          : undefined,
      };
    },
    onSuccess: () => setReviewing(true),
    retry: false,
  });
  const submitted = useRef<NonNullable<typeof preview.data> | null>(null);
  const save = useMutation({
    retry: false,
    mutationFn: async (
      latest: NonNullable<typeof preview.data>,
    ): Promise<CharacterMutationView | EncounterCharacterMutationView> => {
      submitted.current ??= latest;
      const reviewed = submitted.current;
      if (encounter) {
        const added = await addEncounterCharacter(encounter.encounterKey, {
          request_id: requestId,
          expected_encounter_revision: reviewed.encounterRevision!,
          source: {
            kind: "pathbuilder",
            json: text,
            expected_artifact_generation: reviewed.view.artifact_generation,
          },
        });
        return added;
      }
      return savePathbuilderPc({
        request_id: requestId,
        json: text,
        character_key: character?.character_key,
        expected_character_revision: reviewed.view.character_revision,
        party: reviewed.party
          ? {
              party_key: reviewed.party.party_key,
              expected_revision: reviewed.party.revision,
            }
          : undefined,
        expected_artifact_generation: reviewed.view.artifact_generation,
      });
    },
    onError: (error) => {
      if (!isUnconfirmedMutation(error)) submitted.current = null;
      void refreshCharacters(client);
    },
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (!saved.available) return;
      setClosing(true);
      if ("participant_key" in saved)
        onEncounterAdded?.({
          characterKey: saved.character_key,
          participantKey: saved.participant_key,
        });
      onSaved?.(saved.character_key);
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));
  const busy = preview.isPending || save.isPending;
  const draft = preview.data?.view;
  const commitLabel = character
    ? "Refresh character"
    : encounter
      ? "Add to encounter"
      : party
        ? "Add to party"
        : "Add character";
  return (
    <AtlasModal
      open={!closing}
      title={character ? `Refresh ${character.name}` : "Import from Pathbuilder"}
      onCancel={() => setClosing(true)}
      confirmDiscard={{
        when: () => Boolean(text.trim()),
        onDiscard: () => setClosing(true),
      }}
      afterClose={onClose}
      confirmLoading={busy}
      cancelButtonProps={{ disabled: busy }}
      closable={!busy}
      keyboard={!busy}
      mask={{ closable: !busy }}
      footer={(_, { CancelBtn }) =>
        reviewing ? (
          <Space>
            <CancelBtn />
            <Button
              disabled={busy || uncertain}
              onClick={() => {
                setReviewing(false);
                save.reset();
              }}
            >
              Edit import
            </Button>
            <Button
              type="primary"
              loading={save.isPending}
              disabled={paused || busy || canReview}
              onClick={() => {
                if (preview.data) save.mutate(preview.data);
              }}
            >
              {uncertain ? "Retry to confirm" : commitLabel}
            </Button>
          </Space>
        ) : (
          <Space>
            <CancelBtn />
            <Button
              type="primary"
              disabled={!text.trim() || busy}
              loading={preview.isPending}
              onClick={() => {
                save.reset();
                preview.mutate(text);
              }}
            >
              Continue
            </Button>
          </Space>
        )
      }
    >
      <Space orientation="vertical" size="middle" className="character-import">
        {!reviewing && (
          <div>
            <label htmlFor={textareaId}>Pathbuilder JSON</label>
            <Input.TextArea
              id={textareaId}
              value={text}
              rows={8}
              autoComplete="off"
              spellCheck={false}
              disabled={busy}
              onChange={(event) => {
                setText(event.target.value);
                preview.reset();
                save.reset();
              }}
            />
          </div>
        )}
        {preview.error && (
          <ErrorNotice error={preview.error} title="Could not review character" />
        )}
        {save.error && (
          <ErrorNotice
            error={save.error}
            title="Could not save character"
            action={
              canReview ? (
                <Button
                  loading={preview.isPending}
                  onClick={() => {
                    save.reset();
                    submitted.current = null;
                    preview.mutate(text);
                  }}
                >
                  Review latest changes
                </Button>
              ) : undefined
            }
          />
        )}
        {save.data && !save.data.available && (
          <p role="status">
            The character created by this request was deleted. Close this import to
            start a new one.
          </p>
        )}
        {reviewing && draft && (
          <section aria-label="Import review">
            <Typography.Title level={4}>
              {draft.name} · Level {draft.level}
            </Typography.Title>
            {character &&
              (draft.previous_name !== draft.name ||
                draft.previous_level !== draft.level) && (
                <p>
                  Current character {draft.previous_name ?? character.name}
                  {draft.previous_level !== undefined &&
                    ` · Level ${draft.previous_level}`}
                </p>
              )}
            {draft.retained_manual_count > 0 && (
              <p>
                {draft.retained_manual_count} saved{" "}
                {draft.retained_manual_count === 1 ? "link retained" : "links retained"}
              </p>
            )}
            <p>
              {draft.resolved_count} linked · {draft.unresolved_count} need lookup
            </p>
            <div className="character-import__groups">
              {draft.sections.map((section) => (
                <section key={section.kind}>
                  <h3>
                    {section.label} <span>{section.abilities.length}</span>
                  </h3>
                  <List
                    size="small"
                    dataSource={section.abilities}
                    renderItem={(ability) => (
                      <List.Item key={ability.ability_key}>
                        <div>
                          <span>
                            {ability.label}
                            {ability.choice && ` (${ability.choice})`}
                          </span>
                          {ability.record_title &&
                            ability.record_title.trim().toLocaleLowerCase() !==
                              ability.label.trim().toLocaleLowerCase() && (
                              <span className="character-secondary">
                                {ability.record_title}
                              </span>
                            )}
                        </div>
                        {!ability.record_title && (
                          <span className="character-secondary">
                            {ability.ambiguous ? "Choose a rule" : "Find rule"}
                          </span>
                        )}
                      </List.Item>
                    )}
                  />
                </section>
              ))}
            </div>
          </section>
        )}
      </Space>
    </AtlasModal>
  );
}
