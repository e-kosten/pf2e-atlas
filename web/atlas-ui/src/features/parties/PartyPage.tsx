import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Button, Dropdown, Popover } from "antd";
import { Ellipsis } from "lucide-react";
import { useRef, useState } from "react";
import { deleteParty, updateParty } from "../../api/atlasApi";
import { AtlasLink } from "../../app/AtlasLink";
import { characterDestination, partiesDestination } from "../../app/destinations";
import type { PartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import { AddCharacter } from "./AddCharacter";
import { CharacterEditor } from "./CharacterEditor";
import { refreshCharacters } from "./characterQueries";
import { memberCount, PartyDirectory } from "./PartiesPage";
import { PartyNavigation } from "./PartyNavigation";
import { RenameParty, SetPartyLevels } from "./PartyEditors";
import { useParties } from "./useParties";

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
  return (
    <div className="party-page">
      <PartyNavigation active="parties" />
      {query.error && (
        <ErrorNotice
          title="Parties could not load"
          error={query.error}
          action={<Button onClick={() => void query.refetch()}>Retry</Button>}
        />
      )}
      {query.isPending && <p role="status">Loading party…</p>}
      {query.data && (
        <div className="party-workspace">
          <PartyDirectory selected={partyKey} parties={query.data.parties} />
          {party ? (
            <PartyRoster key={partyKey} party={party} />
          ) : (
            <p>Party unavailable.</p>
          )}
        </div>
      )}
    </div>
  );
}

function PartyRoster({ party }: { party: PartyView }) {
  const navigate = useNavigate();
  const client = useQueryClient();
  const paused = useSavedWritesPaused();
  const [editing, setEditing] = useState<{
    kind: "rename" | "levels";
    party: PartyView;
  }>();
  const actions = useRef<HTMLButtonElement>(null);
  const [manual, setManual] = useState<PartyView>();
  const [open, setOpen] = useState(false);
  const { confirm, dialog } = useConfirmDangerAction();
  const change = useMutation({
    retry: false,
    mutationFn: (members: PartyView["members"]) =>
      updateParty(party.party_key, {
        name: party.name,
        expected_revision: party.revision,
        members: membershipInputs(members),
        level_updates: [],
      }),
    onSuccess: () => refreshCharacters(client),
  });
  const busy = paused || change.isPending;
  return (
    <section className="party-roster-pane">
      <header className="party-page-heading">
        <div>
          <h1>{party.name}</h1>
          <span className="character-secondary">
            {memberCount(party.members.length)}
          </span>
        </div>
        <div className="party-heading-actions">
          <AddCharacter
            party={party}
            onManual={() => setManual(party)}
            onSaved={(key) => void navigate(characterDestination(key))}
          />
          <Popover
            trigger="click"
            placement="bottomRight"
            open={open}
            onOpenChange={setOpen}
            content={
              <div className="party-context-actions">
                <Button
                  type="text"
                  disabled={busy}
                  onClick={() => {
                    setOpen(false);
                    actions.current?.focus({ preventScroll: true });
                    setEditing({ kind: "rename", party });
                  }}
                >
                  Rename party
                </Button>
                <Button
                  type="text"
                  disabled={busy || party.members.length === 0}
                  onClick={() => {
                    setOpen(false);
                    actions.current?.focus({ preventScroll: true });
                    setEditing({ kind: "levels", party });
                  }}
                >
                  Set character levels…
                </Button>
                <DangerActionButton
                  type="text"
                  disabled={busy}
                  confirmTitle={`Delete ${party.name}?`}
                  confirmOkText="Delete party"
                  confirmContent="The characters stay saved. PCs already in encounters keep their live rules and combat state."
                  onConfirm={async () => {
                    await deleteParty(party.party_key, party.revision);
                    await navigate(partiesDestination());
                    await refreshCharacters(client);
                  }}
                >
                  Delete party
                </DangerActionButton>
              </div>
            }
          >
            <Button
              icon={<Ellipsis size={16} />}
              ref={actions}
              aria-label="Party actions"
              aria-expanded={open}
            />
          </Popover>
        </div>
      </header>
      {change.error && (
        <ErrorNotice title="Roster could not be updated" error={change.error} />
      )}
      {manual && (
        <CharacterEditor
          inline
          party={manual}
          onClose={() => setManual(undefined)}
          onSaved={() => setManual(undefined)}
        />
      )}
      {party.members.length ? (
        <table className="party-roster">
          <thead>
            <tr>
              <th>Character</th>
              <th>Level</th>
              <th>Reference</th>
              <th aria-label="Actions" />
            </tr>
          </thead>
          <tbody>
            {party.members.map((member, index) => (
              <tr key={member.character_key}>
                <td>
                  <AtlasLink {...characterDestination(member.character_key)}>
                    {member.name}
                  </AtlasLink>
                </td>
                <td>{member.level ?? "—"}</td>
                <td>{member.imported ? "Pathbuilder" : "Manual"}</td>
                <td>
                  <Dropdown
                    trigger={["click"]}
                    menu={{
                      items: [
                        { key: "up", label: "Move up", disabled: index === 0 },
                        {
                          key: "down",
                          label: "Move down",
                          disabled: index === party.members.length - 1,
                        },
                        { key: "remove", label: "Remove from party", danger: true },
                      ],
                      onClick: ({ key }) => {
                        const members = [...party.members];
                        members.splice(index, 1);
                        if (key === "remove") {
                          const revision = party.revision;
                          confirm({
                            title: `Remove ${member.name} from ${party.name}?`,
                            content:
                              "The character stays saved and keeps its encounter links.",
                            okText: "Remove from party",
                            onConfirm: async () => {
                              await updateParty(party.party_key, {
                                expected_revision: revision,
                                name: party.name,
                                members: membershipInputs(members),
                                level_updates: [],
                              });
                              await refreshCharacters(client);
                            },
                          });
                        } else {
                          members.splice(
                            key === "up" ? index - 1 : index + 1,
                            0,
                            member,
                          );
                          change.mutate(members);
                        }
                      },
                    }}
                  >
                    <Button
                      type="text"
                      disabled={busy}
                      aria-label={`Actions for ${member.name}`}
                      icon={<Ellipsis size={16} />}
                    />
                  </Dropdown>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <p>No characters in this party yet.</p>
      )}
      {editing?.kind === "rename" && (
        <RenameParty party={editing.party} onClose={() => setEditing(undefined)} />
      )}
      {editing?.kind === "levels" && (
        <SetPartyLevels party={editing.party} onClose={() => setEditing(undefined)} />
      )}
      {dialog}
    </section>
  );
}
