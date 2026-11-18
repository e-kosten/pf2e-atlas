import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Input, List, Select, Space } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import {
  addEncounterCharacter,
  getCharacters,
  getEncounter,
  getParties,
  updateParty,
} from "../../api/atlasApi";
import type { CharacterSummaryView, PartyView } from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters, useCharacters } from "./characterQueries";
import { useParties } from "./useParties";

export function CharacterPicker({
  party,
  encounter,
  onClose,
  onSaved,
}: {
  party?: PartyView;
  encounter?: { encounterKey: string; revision: number };
  onClose: () => void;
  onSaved?: () => void;
}) {
  const characters = useCharacters();
  const parties = useParties(Boolean(encounter));
  const [query, setQuery] = useState("");
  const [partyFilter, setPartyFilter] = useState<string>();
  const [closing, setClosing] = useState(false);
  const [requestId] = useState(() => crypto.randomUUID());
  const client = useQueryClient();
  const paused = useSavedWritesPaused();
  const [reviewedParty, setReviewedParty] = useState(party);
  const [revision, setRevision] = useState(encounter?.revision);
  const submitted = useRef<{
    character: CharacterSummaryView;
    party?: PartyView;
    revision?: number;
  } | null>(null);
  const review = useMutation({
    retry: false,
    mutationFn: async () => {
      const index = await getCharacters();
      client.setQueryData(["characters"], index);
      const currentParty = party
        ? (await getParties()).parties.find(
            (item) => item.party_key === party.party_key,
          )
        : undefined;
      if (party && !currentParty) throw new Error("This party is no longer available.");
      const currentEncounter = encounter
        ? await getEncounter(encounter.encounterKey)
        : undefined;
      return { party: currentParty, revision: currentEncounter?.revision };
    },
    onSuccess: (latest) => {
      if (
        latest.party?.members.some(
          (member) => member.character_key === save.variables?.character_key,
        )
      ) {
        setClosing(true);
        onSaved?.();
      }
      setReviewedParty(latest.party);
      setRevision(latest.revision);
      submitted.current = null;
      save.reset();
    },
  });
  const save = useMutation({
    retry: false,
    mutationFn: async (character: CharacterSummaryView) => {
      submitted.current ??= { character, party: reviewedParty, revision };
      const checked = submitted.current;
      character = checked.character;
      if (encounter)
        return addEncounterCharacter(encounter.encounterKey, {
          request_id: requestId,
          expected_encounter_revision: checked.revision!,
          source: {
            kind: "existing",
            character_key: character.character_key,
            expected_character_revision: character.revision,
          },
        });
      if (checked.party)
        return updateParty(checked.party.party_key, {
          name: checked.party.name,
          expected_revision: checked.party.revision,
          members: [
            ...checked.party.members.map((member) => ({
              character_key: member.character_key,
              expected_character_revision: member.character_revision,
            })),
            {
              character_key: character.character_key,
              expected_character_revision: character.revision,
            },
          ],
        });
    },
    onError: (error) => {
      if (!isUnconfirmedMutation(error)) submitted.current = null;
      void refreshCharacters(client);
    },
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (saved && "available" in saved && !saved.available) return;
      setClosing(true);
      onSaved?.();
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));
  const visible = characters.data?.characters.filter(
    (character) =>
      character.name.toLocaleLowerCase().includes(query.trim().toLocaleLowerCase()) &&
      (!partyFilter || character.parties.some((p) => p.party_key === partyFilter)) &&
      !reviewedParty?.members.some(
        (member) => member.character_key === character.character_key,
      ),
  );
  return (
    <AtlasModal
      open={!closing}
      title="Choose a character"
      onCancel={() => setClosing(true)}
      afterClose={onClose}
      closable={!save.isPending}
      keyboard={!save.isPending}
      mask={{ closable: !save.isPending }}
      footer={
        <Button disabled={save.isPending} onClick={() => setClosing(true)}>
          Cancel
        </Button>
      }
    >
      <Space orientation="vertical" className="character-import">
        <Input
          aria-label="Find a character"
          placeholder="Find a character"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          allowClear
        />
        {encounter && (
          <Select
            aria-label="Filter by party"
            placeholder="All parties"
            allowClear
            value={partyFilter}
            onChange={setPartyFilter}
            options={parties.data?.parties.map((p) => ({
              value: p.party_key,
              label: p.name,
            }))}
          />
        )}
        {characters.error && (
          <ErrorNotice
            error={characters.error}
            title="Characters could not load"
            action={<Button onClick={() => void characters.refetch()}>Retry</Button>}
          />
        )}
        {save.error && (
          <ErrorNotice
            error={save.error}
            title="Character could not be added"
            action={
              canReview || (!encounter && uncertain) ? (
                <Button loading={review.isPending} onClick={() => review.mutate()}>
                  Review latest changes
                </Button>
              ) : undefined
            }
          />
        )}
        {save.data && "available" in save.data && !save.data.available && (
          <p role="status">
            The participant from this request was removed. Close this picker to start a
            new addition.
          </p>
        )}
        {review.error && (
          <ErrorNotice title="Current values could not load" error={review.error} />
        )}
        {review.data && (
          <p role="status">Latest character and roster values reviewed.</p>
        )}
        <List
          loading={characters.isPending}
          dataSource={visible}
          locale={{ emptyText: "No matching characters." }}
          renderItem={(character) => (
            <List.Item>
              <div>
                {character.name}
                {character.level !== undefined && (
                  <span className="character-secondary">Level {character.level}</span>
                )}
              </div>
              <Button
                disabled={
                  paused ||
                  save.isPending ||
                  canReview ||
                  review.isPending ||
                  (uncertain &&
                    save.variables?.character_key !== character.character_key)
                }
                loading={
                  save.isPending &&
                  save.variables?.character_key === character.character_key
                }
                onClick={() => save.mutate(character)}
              >
                {uncertain ? "Retry to confirm" : "Add"}
              </Button>
            </List.Item>
          )}
        />
      </Space>
    </AtlasModal>
  );
}
