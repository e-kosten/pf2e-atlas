import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { DndContext, DragOverlay } from "@dnd-kit/core";
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { Button, Tooltip } from "antd";
import { GripVertical, Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import { deleteParty, getParties, updateParty } from "../../api/atlasApi";
import { AtlasLink } from "../../app/AtlasLink";
import { characterDestination, partiesDestination } from "../../app/destinations";
import type { PartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { EntityActionsMenu } from "../../shared/ui/actions/EntityActionsMenu";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import { SortableRow, useSortableList } from "../../shared/ui/sortable/SortableList";
import { restoreOverlayFocus } from "../../shared/ui/overlays/restoreOverlayFocus";
import { AddCharacter } from "./AddCharacter";
import { CharacterEditor } from "./CharacterEditor";
import { refreshCharacters } from "./characterQueries";
import { memberCount } from "./PartiesPage";
import { RenameParty, SetPartyLevels } from "./PartyEditors";
import { cacheParty, useParties } from "./useParties";

function membershipInputs(members: PartyView["members"]) {
  return members.map(({ character_key, character_revision }) => ({
    character_key,
    expected_character_revision: character_revision,
  }));
}

export function PartyPage({ partyKey }: { partyKey: string }) {
  const query = useParties();
  const party = query.data?.parties.find((p) => p.party_key === partyKey);
  usePageTitle(party?.name ?? "Party");
  return party ? (
    <PartyRoster key={partyKey} party={party} />
  ) : query.data ? (
    <section className="party-selection-prompt">
      <h2 data-party-title tabIndex={-1}>
        Party unavailable
      </h2>
      <p>
        {query.data.parties.length
          ? "Choose another party from the list."
          : "Create a party to get started."}
      </p>
    </section>
  ) : null;
}

function PartyRoster({ party }: { party: PartyView }) {
  const navigate = useNavigate();
  const client = useQueryClient();
  const paused = useSavedWritesPaused();
  type PartyEdit = { kind: "rename" | "levels"; party: PartyView };
  const [editing, setEditing] = useState<PartyEdit>();
  const deletion = useConfirmDangerAction();
  const actions = useRef<HTMLButtonElement>(null);
  const [manual, setManual] = useState<PartyView>();
  const inlineArea = useRef<HTMLDivElement>(null);
  const addTrigger = useRef<HTMLButtonElement>(null);
  const [lastAdded, setLastAdded] = useState<string>();
  const added = party.members.find((member) => member.character_key === lastAdded);
  const change = useMutation({
    retry: false,
    mutationFn: (members: PartyView["members"]) =>
      updateParty(party.party_key, {
        name: party.name,
        expected_revision: party.revision,
        members: membershipInputs(members),
        level_updates: [],
      }),
    onSuccess: async (saved) => {
      await cacheParty(client, saved);
    },
  });
  const review = useMutation({
    retry: false,
    mutationFn: getParties,
    onSuccess: (latest) => {
      client.setQueryData(["parties"], latest);
      change.reset();
    },
  });
  const sortable = useSortableList({
    items: party.members,
    getKey: (member) => member.character_key,
    disabled:
      paused ||
      Boolean(manual) ||
      change.isPending ||
      Boolean(change.error) ||
      review.isPending,
    onReorder: async (key, target, placement) => {
      const member = party.members.find((member) => member.character_key === key);
      const ordered = party.members.filter((member) => member.character_key !== key);
      const index = ordered.findIndex((member) => member.character_key === target);
      if (!member || index < 0) return false;
      ordered.splice(placement === "after" ? index + 1 : index, 0, member);
      await change.mutateAsync(ordered);
      return true;
    },
  });
  const busy =
    paused ||
    sortable.saving ||
    Boolean(sortable.activeItem) ||
    review.isPending ||
    Boolean(manual);
  function closeManual(key?: string) {
    if (key) setLastAdded(key);
    restoreOverlayFocus(() => addTrigger.current, inlineArea.current);
    setManual(undefined);
  }
  return (
    <section className="party-roster-pane">
      <header className="party-page-heading">
        <div>
          <h2 data-party-title tabIndex={-1}>
            {party.name}
          </h2>
          <span className="character-secondary">
            {memberCount(party.members.length)}
          </span>
        </div>
        <div className="party-heading-actions">
          <AddCharacter
            party={party}
            onManual={() => setManual(party)}
            disabled={busy}
            triggerRef={addTrigger}
            onSaved={setLastAdded}
          />
          <EntityActionsMenu
            label="Party actions"
            trigger={actions}
            items={[
              { key: "rename", label: "Rename party", disabled: busy },
              {
                key: "levels",
                label: "Set character levels…",
                disabled: busy || party.members.length === 0,
              },
              { key: "delete", label: "Delete party", danger: true, disabled: busy },
            ]}
            onAction={(key) => {
              if (key === "rename" || key === "levels") {
                setEditing({ kind: key, party });
              } else {
                deletion.confirm({
                  title: `Delete ${party.name}?`,
                  okText: "Delete party",
                  content:
                    "The characters stay saved. PCs already in encounters keep their live rules and combat state.",
                  onConfirm: async () => {
                    await deleteParty(party.party_key, party.revision);
                    await navigate(partiesDestination());
                    await refreshCharacters(client);
                  },
                });
              }
            }}
          />
        </div>
      </header>
      {(review.error || change.error) && (
        <ErrorNotice
          title={
            review.error
              ? "Latest party could not load"
              : "Roster could not be reordered"
          }
          error={review.error ?? change.error}
          action={
            <Button loading={review.isPending} onClick={() => review.mutate()}>
              Review latest roster
            </Button>
          }
        />
      )}
      {added && (
        <p role="status">
          Added{" "}
          <AtlasLink {...characterDestination(added.character_key)}>
            {added.name}
          </AtlasLink>
        </p>
      )}
      {manual && (
        <div ref={inlineArea}>
          <CharacterEditor
            inline
            party={manual}
            onClose={() => closeManual()}
            onSaved={closeManual}
          />
        </div>
      )}
      {party.members.length ? (
        <DndContext
          {...sortable.context}
          accessibility={{
            announcements: {
              onDragStart: ({ active }) =>
                `Picked up ${party.members.find((member) => member.character_key === active.id)?.name ?? "character"}.`,
              onDragOver: ({ over }) =>
                over
                  ? `Position ${party.members.findIndex((member) => member.character_key === over.id) + 1}.`
                  : "Outside the roster.",
              onDragEnd: ({ over }) =>
                over
                  ? `Dropped at position ${party.members.findIndex((member) => member.character_key === over.id) + 1}.`
                  : "Reorder cancelled.",
              onDragCancel: () => "Reorder cancelled.",
            },
          }}
        >
          <table className="party-roster" aria-busy={sortable.saving}>
            <thead>
              <tr>
                <th scope="col">
                  <span className="sr-only">Order</span>
                </th>
                <th scope="col">Character</th>
                <th scope="col">Level</th>
                <th scope="col">Reference</th>
                <th scope="col">
                  <span className="sr-only">Actions</span>
                </th>
              </tr>
            </thead>
            <SortableContext
              items={sortable.items.map((member) => member.character_key)}
              strategy={verticalListSortingStrategy}
            >
              <tbody>
                {sortable.items.map((member) => (
                  <SortableRow
                    as="tr"
                    key={member.character_key}
                    id={member.character_key}
                    disabled={sortable.disabled || party.members.length < 2}
                  >
                    {({ attributes, listeners, setActivatorNodeRef }) => (
                      <>
                        <td>
                          {party.members.length > 1 && (
                            <Button
                              {...attributes}
                              {...listeners}
                              ref={setActivatorNodeRef}
                              type="text"
                              size="small"
                              disabled={sortable.disabled}
                              className="party-roster__drag-handle"
                              aria-label={`Reorder ${member.name}`}
                              icon={<GripVertical size={16} aria-hidden="true" />}
                            />
                          )}
                        </td>
                        <td>
                          <AtlasLink {...characterDestination(member.character_key)}>
                            {member.name}
                          </AtlasLink>
                        </td>
                        <td>{member.level ?? "—"}</td>
                        <td>{member.imported ? "Pathbuilder" : "Manual"}</td>
                        <td>
                          <Tooltip
                            title={`Remove ${member.name} from party`}
                            trigger={["hover", "focus"]}
                          >
                            <DangerActionButton
                              type="text"
                              icon={<Trash2 size={14} aria-hidden="true" />}
                              disabled={busy || Boolean(change.error)}
                              confirmTitle={`Remove ${member.name} from ${party.name}?`}
                              confirmContent="The character stays saved and keeps its encounter links."
                              confirmOkText="Remove from party"
                              aria-label={`Remove ${member.name} from party`}
                              onConfirm={async () => {
                                const saved = await updateParty(party.party_key, {
                                  expected_revision: party.revision,
                                  name: party.name,
                                  members: membershipInputs(
                                    party.members.filter(
                                      (item) =>
                                        item.character_key !== member.character_key,
                                    ),
                                  ),
                                  level_updates: [],
                                });
                                await cacheParty(client, saved);
                                void Promise.all([
                                  client.invalidateQueries({
                                    queryKey: ["characters"],
                                  }),
                                  client.invalidateQueries({
                                    queryKey: ["character", member.character_key],
                                  }),
                                  client.invalidateQueries({
                                    queryKey: ["pc-reference"],
                                  }),
                                ]);
                                restoreOverlayFocus(() => actions.current, undefined);
                              }}
                            />
                          </Tooltip>
                        </td>
                      </>
                    )}
                  </SortableRow>
                ))}
              </tbody>
            </SortableContext>
          </table>
          <DragOverlay>
            {sortable.activeItem && (
              <div className="party-roster__drag-preview">
                {sortable.activeItem.name}
              </div>
            )}
          </DragOverlay>
        </DndContext>
      ) : (
        <p>No characters in this party yet.</p>
      )}
      {deletion.dialog}
      {editing?.kind === "rename" && (
        <RenameParty party={editing.party} onClose={() => setEditing(undefined)} />
      )}
      {editing?.kind === "levels" && (
        <SetPartyLevels party={editing.party} onClose={() => setEditing(undefined)} />
      )}
    </section>
  );
}
