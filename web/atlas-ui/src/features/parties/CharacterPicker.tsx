import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Input, List, Select, Space } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  addEncounterCharacter,
  getCharacters,
  getEncounter,
  getParties,
  updateParty,
} from "../../api/atlasApi";
import type {
  AddEncounterCharacterRequest,
  CharacterSummaryView,
  PartyView,
  UpdatePartyRequest,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters, useCharacters } from "./characterQueries";
import { useParties } from "./useParties";

type CharacterAddition = { characterKey: string } & (
  | { encounterKey: string; request: AddEncounterCharacterRequest }
  | { partyKey: string; request: UpdatePartyRequest }
);

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
          (member) => member.character_key === save.variables?.characterKey,
        )
      ) {
        setClosing(true);
        onSaved?.();
      }
      setReviewedParty(latest.party);
      setRevision(latest.revision);
      save.reset();
    },
  });
  const save = useMutation({
    retry: false,
    mutationFn: async (addition: CharacterAddition) =>
      "encounterKey" in addition
        ? addEncounterCharacter(addition.encounterKey, addition.request)
        : updateParty(addition.partyKey, addition.request),
    onError: () => void refreshCharacters(client),
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (saved && "available" in saved && !saved.available) return;
      setClosing(true);
      onSaved?.();
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));
  function add(character: CharacterSummaryView) {
    if (uncertain && save.variables) {
      save.mutate(save.variables);
      return;
    }
    const characterKey = character.character_key;
    if (encounter) {
      save.mutate({
        characterKey,
        encounterKey: encounter.encounterKey,
        request: {
          request_id: requestId,
          expected_encounter_revision: revision!,
          source: {
            kind: "existing",
            character_key: characterKey,
            expected_character_revision: character.revision,
          },
        },
      });
    } else if (reviewedParty) {
      save.mutate({
        characterKey,
        partyKey: reviewedParty.party_key,
        request: {
          name: reviewedParty.name,
          expected_revision: reviewedParty.revision,
          members: [
            ...reviewedParty.members.map((member) => ({
              character_key: member.character_key,
              expected_character_revision: member.character_revision,
            })),
            {
              character_key: characterKey,
              expected_character_revision: character.revision,
            },
          ],
        },
      });
    }
  }
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
        {characters.isPending && <LoadingStatus label="Loading characters…" />}
        {characters.data && (
          <List
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
                      save.variables?.characterKey !== character.character_key)
                  }
                  loading={
                    save.isPending &&
                    save.variables?.characterKey === character.character_key
                  }
                  onClick={() => add(character)}
                >
                  {uncertain ? "Retry to confirm" : "Add"}
                </Button>
              </List.Item>
            )}
          />
        )}
      </Space>
    </AtlasModal>
  );
}
