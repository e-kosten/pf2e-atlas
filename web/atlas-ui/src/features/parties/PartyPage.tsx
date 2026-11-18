import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Button, Checkbox, Dropdown, Form, Input, InputNumber, Popover } from "antd";
import { Ellipsis } from "lucide-react";
import { useId, useState } from "react";
import {
  AtlasApiError,
  deleteParty,
  getParties,
  updateParty,
} from "../../api/atlasApi";
import { AtlasLink } from "../../app/AtlasLink";
import { characterDestination, partiesDestination } from "../../app/destinations";
import type { PartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { AddCharacter } from "./AddCharacter";
import { CharacterEditor } from "./CharacterEditor";
import { refreshCharacters, useCharacters } from "./characterQueries";
import { memberCount, PartyDirectory } from "./PartiesPage";
import { PartyNavigation } from "./PartyNavigation";
import { useParties } from "./useParties";

function membershipInputs(members: PartyView["members"]) {
  return members.map(({ character_key, character_revision }) => ({
    character_key,
    expected_character_revision: character_revision,
  }));
}

function PartySettings({
  party: initialParty,
  onClose,
}: {
  party: PartyView;
  onClose: () => void;
}) {
  const [party, setParty] = useState(initialParty);
  const [form] = Form.useForm<{ name: string; level?: number }>();
  const id = useId();
  const [closing, setClosing] = useState(false);
  const [bulk, setBulk] = useState(false);
  const [selected, setSelected] = useState(party.members.map((m) => m.character_key));
  const affected = party.members.filter((m) => selected.includes(m.character_key));
  const characters = useCharacters(bulk);
  const paused = useSavedWritesPaused();
  const client = useQueryClient();
  const save = useMutation({
    retry: false,
    mutationFn: ({ name, level }: { name: string; level?: number }) =>
      updateParty(party.party_key, {
        expected_revision: party.revision,
        name,
        members: membershipInputs(party.members),
        level_updates: bulk
          ? affected.map((m) => ({
              character_key: m.character_key,
              expected_character_revision: m.character_revision,
              level,
            }))
          : [],
      }),
    onSuccess: async () => {
      await refreshCharacters(client);
      setClosing(true);
    },
  });
  const stale = save.error instanceof AtlasApiError && save.error.status === 409;
  const review = useMutation({
    retry: false,
    mutationFn: async () => {
      const [index] = await Promise.all([
        getParties(),
        bulk ? characters.refetch({ throwOnError: true }) : Promise.resolve(),
      ]);
      const latest = index.parties.find((item) => item.party_key === party.party_key);
      if (!latest) throw new Error("This party was deleted.");
      return latest;
    },
    onSuccess: (latest) => {
      if (!form.isFieldTouched("name")) form.setFieldValue("name", latest.name);
      setSelected((keys) =>
        keys.filter((key) => latest.members.some((m) => m.character_key === key)),
      );
      setParty(latest);
      save.reset();
    },
  });
  const close = () => setClosing(true);
  const busy = paused || save.isPending || review.isPending;
  const canSave =
    !busy && !stale && (!bulk || (characters.data && affected.length > 0));
  return (
    <AtlasModal
      open={!closing}
      afterClose={onClose}
      onCancel={close}
      title="Party settings"
      okText="Save changes"
      onOk={() => form.submit()}
      confirmLoading={save.isPending}
      closable={!save.isPending}
      keyboard={!save.isPending}
      mask={{ closable: !save.isPending }}
      okButtonProps={{
        disabled: !canSave,
      }}
      cancelButtonProps={{ disabled: save.isPending }}
      confirmDiscard={{ when: () => form.isFieldsTouched() || bulk, onDiscard: close }}
    >
      {save.error && (
        <ErrorNotice
          title="Party could not be saved"
          error={save.error}
          action={
            stale && (
              <Button loading={review.isPending} onClick={() => review.mutate()}>
                Review latest changes
              </Button>
            )
          }
        />
      )}
      {review.error && (
        <ErrorNotice title="Latest party changes could not load" error={review.error} />
      )}
      {review.data && !stale && (
        <section aria-label="Latest saved party">
          <h3>Latest saved party</h3>
          <p>{review.data.name}</p>
          {review.data.members.length ? (
            <ul>
              {review.data.members.map((member) => (
                <li key={member.character_key}>
                  {member.name} ·{" "}
                  {member.level == null ? "No level" : `Level ${member.level}`}
                </li>
              ))}
            </ul>
          ) : (
            <p>No characters in this party.</p>
          )}
        </section>
      )}
      <Form
        form={form}
        name={id}
        layout="vertical"
        initialValues={{ name: party.name }}
        disabled={busy}
        onFinish={(v) => {
          if (canSave) save.mutate(v);
        }}
      >
        <Form.Item
          label="Party name"
          name="name"
          rules={[{ required: true, whitespace: true, message: "Enter a party name." }]}
        >
          <Input maxLength={255} />
        </Form.Item>
        {(party.members.length > 0 || bulk) && (
          <Checkbox checked={bulk} onChange={(e) => setBulk(e.target.checked)}>
            Set character levels
          </Checkbox>
        )}
        {bulk && (
          <>
            <Form.Item
              name="level"
              label="Level"
              rules={[
                {
                  required: true,
                  type: "integer",
                  min: 1,
                  max: 20,
                  message: "Use a level from 1 to 20.",
                },
              ]}
            >
              <InputNumber min={1} max={20} />
            </Form.Item>
            <fieldset className="party-level-choices">
              <legend>Update these characters</legend>
              {characters.error && (
                <ErrorNotice
                  title="Memberships could not load"
                  error={characters.error}
                  action={
                    <Button onClick={() => void characters.refetch()}>Retry</Button>
                  }
                />
              )}
              {party.members.map((m) => {
                const others = characters.data?.characters
                  .find((c) => c.character_key === m.character_key)
                  ?.parties.filter((p) => p.party_key !== party.party_key);
                return (
                  <Checkbox
                    key={m.character_key}
                    checked={selected.includes(m.character_key)}
                    onChange={(e) =>
                      setSelected(
                        e.target.checked
                          ? [...selected, m.character_key]
                          : selected.filter((k) => k !== m.character_key),
                      )
                    }
                  >
                    {m.name}
                    {others?.length
                      ? ` · Also updates ${others.map((p) => p.name).join(", ")}`
                      : ""}
                  </Checkbox>
                );
              })}
            </fieldset>
          </>
        )}
      </Form>
    </AtlasModal>
  );
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
  const [settings, setSettings] = useState<PartyView>();
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
                    setSettings(party);
                  }}
                >
                  Party settings
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
      {settings && (
        <PartySettings party={settings} onClose={() => setSettings(undefined)} />
      )}
      {dialog}
    </section>
  );
}
