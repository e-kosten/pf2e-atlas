import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Flex, Input, List, Select } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState, type ComponentRef } from "react";
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
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { useCharacters } from "./characterQueries";
import { cacheParty, useParties } from "./useParties";

type CharacterAddition = { character: CharacterSummaryView } & (
  | { encounterKey: string; request: AddEncounterCharacterRequest }
  | { partyKey: string; request: UpdatePartyRequest }
);

export function CharacterPicker({
  party,
  encounter,
  onClose,
}: {
  party?: PartyView;
  encounter?: { encounterKey: string; revision: number };
  onClose: () => void;
}) {
  const search = useRef<ComponentRef<typeof Input>>(null);
  const list = useRef<HTMLDivElement>(null);
  const characters = useCharacters();
  const parties = useParties(Boolean(encounter));
  const [query, setQuery] = useState("");
  const [partyFilter, setPartyFilter] = useState<string>();
  const [closing, setClosing] = useState(false);
  const [status, setStatus] = useState("");
  const [recent, setRecent] = useState<Record<string, number>>({});
  const [needsEncounterRead, setNeedsEncounterRead] = useState(false);
  const client = useQueryClient();
  const paused = useSavedWritesPaused();
  const [reviewedParty, setReviewedParty] = useState(party);
  const [revision, setRevision] = useState(encounter?.revision);

  function focusResult(characterKey: string, moveToNext: boolean) {
    restoreOverlayFocus(() => {
      const rows = Array.from(
        list.current?.querySelectorAll<HTMLElement>("[data-picker-character]") ?? [],
      );
      const index = rows.findIndex(
        (row) => row.dataset.pickerCharacter === characterKey,
      );
      const candidates = moveToNext
        ? [...rows.slice(index + 1), ...rows.slice(0, index)]
        : rows.slice(index, index + 1);
      return (
        candidates
          .map((row) => row.querySelector<HTMLButtonElement>("button:not(:disabled)"))
          .find(Boolean) ??
        search.current?.input ??
        null
      );
    }, list.current);
  }

  const review = useMutation({
    retry: false,
    mutationFn: async (reason: "write-confirmed" | "review") => {
      if (reason === "write-confirmed" && encounter) {
        const queryKey = ["encounter", encounter.encounterKey];
        await client.cancelQueries({ queryKey, exact: true });
        const current = await client.fetchQuery({
          queryKey,
          queryFn: () => getEncounter(encounter.encounterKey),
          retry: false,
          staleTime: 0,
        });
        return { revision: current.revision };
      }
      const [index, partyIndex, current] = await Promise.all([
        getCharacters(),
        party ? getParties() : undefined,
        encounter ? getEncounter(encounter.encounterKey) : undefined,
      ]);
      client.setQueryData(["characters"], index);
      if (partyIndex) client.setQueryData(["parties"], partyIndex);
      if (current && encounter) {
        client.setQueryData(["encounter", encounter.encounterKey], current);
      }
      const currentParty = partyIndex?.parties.find(
        (item) => item.party_key === party?.party_key,
      );
      if (party && !currentParty) throw new Error("This party is no longer available.");
      return { party: currentParty, revision: current?.revision };
    },
    onSuccess: (latest, reason) => {
      if (latest.party) setReviewedParty(latest.party);
      if (latest.revision !== undefined) setRevision(latest.revision);
      setNeedsEncounterRead(false);
      const character = save.variables?.character;
      if (character) {
        const present = latest.party?.members.some(
          (member) => member.character_key === character.character_key,
        );
        if (reason === "review")
          setStatus(
            present
              ? `${character.name} is in the party.`
              : "Latest roster and characters loaded. Choose a character to add.",
          );
        focusResult(character.character_key, Boolean(present));
      }
      save.reset();
    },
  });
  const save = useMutation({
    retry: false,
    mutationFn: async (addition: CharacterAddition) =>
      "encounterKey" in addition
        ? addEncounterCharacter(addition.encounterKey, addition.request)
        : updateParty(addition.partyKey, addition.request),
    onSuccess: async (saved, { character }) => {
      if ("party_key" in saved) {
        setReviewedParty(saved);
        await cacheParty(client, saved);
        void Promise.all([
          client.invalidateQueries({ queryKey: ["characters"] }),
          client.invalidateQueries({
            queryKey: ["character", character.character_key],
          }),
          client.invalidateQueries({ queryKey: ["pc-reference"] }),
        ]);
        setStatus(`${character.name} added to ${saved.name}.`);
        focusResult(character.character_key, true);
        return;
      }
      setNeedsEncounterRead(true);
      if (saved.available) {
        setRecent((previous) => ({
          ...previous,
          [character.character_key]: (previous[character.character_key] ?? 0) + 1,
        }));
        setStatus(`${character.name} added to the encounter.`);
      } else {
        setStatus("The participant from this request was removed.");
      }
      void client.invalidateQueries({ queryKey: ["encounters"] });
      review.mutate("write-confirmed");
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const busy = save.isPending || review.isPending;
  function add(character: CharacterSummaryView) {
    if (busy || save.error || needsEncounterRead) return;
    setStatus("");
    const characterKey = character.character_key;
    if (encounter && revision !== undefined) {
      save.mutate({
        character,
        encounterKey: encounter.encounterKey,
        request: {
          request_id: crypto.randomUUID(),
          expected_encounter_revision: revision,
          source: {
            kind: "existing",
            character_key: characterKey,
            expected_character_revision: character.revision,
          },
        },
      });
    } else if (reviewedParty) {
      save.mutate({
        character,
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

  const members = new Set(reviewedParty?.members.map((member) => member.character_key));
  const partyChoices = encounter ? (parties.data?.parties ?? []) : [];
  const selectedParty = partyChoices.find((party) => party.party_key === partyFilter);
  const term = query.trim().toLocaleLowerCase();
  const visible = characters.data?.characters.filter(
    (character) =>
      [character.name, ...character.parties.map((party) => party.name)].some((label) =>
        label.toLocaleLowerCase().includes(term),
      ) &&
      (!selectedParty ||
        character.parties.some((party) => party.party_key === selectedParty.party_key)),
  );
  return (
    <AtlasModal
      open={!closing}
      title="Choose a character"
      onCancel={() => setClosing(true)}
      afterClose={onClose}
      closable={!busy}
      keyboard={!busy}
      confirmLoading={busy}
      mask={{ closable: !busy }}
      footer={
        <Flex align="center" justify="space-between" gap="small">
          <div className="character-picker__status">
            <div role="status">{status}</div>
            {save.isPending && !status && <LoadingStatus label="Adding character…" />}
          </div>
          <Button disabled={busy} onClick={() => setClosing(true)}>
            Done
          </Button>
        </Flex>
      }
    >
      <div className="character-picker">
        <Input
          ref={search}
          aria-label="Find a character"
          placeholder="Find a character"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          allowClear
        />
        {partyChoices.length > 0 && (
          <Select
            aria-label="Filter by party"
            placeholder="All parties"
            allowClear
            value={selectedParty?.party_key}
            onChange={setPartyFilter}
            options={partyChoices.map((party) => ({
              value: party.party_key,
              label: party.name,
            }))}
          />
        )}
        {parties.error && encounter && (
          <ErrorNotice
            error={parties.error}
            title="Parties could not load"
            action={<Button onClick={() => void parties.refetch()}>Retry</Button>}
          />
        )}
        {characters.error && (
          <ErrorNotice
            error={characters.error}
            title="Characters could not load"
            action={<Button onClick={() => void characters.refetch()}>Retry</Button>}
          />
        )}
        {save.error && !review.error && (
          <ErrorNotice
            error={save.error}
            title={
              uncertain
                ? "Character addition could not be confirmed"
                : "Character could not be added"
            }
            action={
              uncertain && encounter ? (
                <Button
                  disabled={paused || busy}
                  onClick={() => save.variables && save.mutate(save.variables)}
                >
                  Retry to confirm
                </Button>
              ) : (
                <Button disabled={busy} onClick={() => review.mutate("review")}>
                  Review latest changes
                </Button>
              )
            }
          />
        )}
        {review.error && (
          <ErrorNotice
            title={
              needsEncounterRead
                ? "Updated encounter could not load"
                : "Current roster could not load"
            }
            error={review.error}
            action={
              <Button
                disabled={busy}
                onClick={() =>
                  review.mutate(needsEncounterRead ? "write-confirmed" : "review")
                }
              >
                Retry
              </Button>
            }
          />
        )}
        {characters.isPending && <LoadingStatus label="Loading characters…" />}
        <div ref={list}>
          {characters.data && (
            <List
              dataSource={visible}
              rowKey={(character) => character.character_key}
              locale={{ emptyText: "No matching characters." }}
              renderItem={(character) => {
                const key = character.character_key;
                const inParty = members.has(key);
                const added = recent[key];
                const level =
                  character.level === undefined
                    ? "Level unset"
                    : `Level ${character.level}`;
                const memberships = character.parties
                  .map((party) => party.name)
                  .join(", ");
                const metadata = [
                  level,
                  memberships || "No parties",
                  character.imported ? "Pathbuilder" : "Manual",
                ].join(" · ");
                return (
                  <List.Item data-picker-character={key}>
                    <div className="character-picker__identity">
                      <strong>{character.name}</strong>
                      <span className="character-secondary">{metadata}</span>
                      {added && (
                        <span className="character-secondary">
                          Added {added} {added === 1 ? "time" : "times"}
                        </span>
                      )}
                    </div>
                    {inParty ? (
                      <span className="character-secondary">In party</span>
                    ) : (
                      <Button
                        aria-label={`Add ${character.name}, ${metadata}`}
                        disabled={
                          paused || busy || Boolean(save.error) || needsEncounterRead
                        }
                        onClick={() => add(character)}
                      >
                        Add
                      </Button>
                    )}
                  </List.Item>
                );
              }}
            />
          )}
        </div>
      </div>
    </AtlasModal>
  );
}
