import { Button, Form, Input } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useId, useState } from "react";
import { createParty } from "../../api/atlasApi";
import { AtlasLink } from "../../app/AtlasLink";
import { partyDestination } from "../../app/destinations";
import type { PartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";
import { PartyNavigation } from "./PartyNavigation";
import { useParties } from "./useParties";

function NewParty({ onClose }: { onClose: () => void }) {
  const [form] = Form.useForm<{ name: string }>();
  const id = useId();
  const [closing, setClosing] = useState(false);
  const client = useQueryClient();
  const navigate = useNavigate();
  const paused = useSavedWritesPaused();
  const save = useMutation({
    retry: false,
    mutationFn: ({ name }: { name: string }) => createParty({ name, members: [] }),
    onSuccess: async ({ party_key }) => {
      await refreshCharacters(client);
      setClosing(true);
      await navigate(partyDestination(party_key));
    },
  });
  const close = () => setClosing(true);
  return (
    <AtlasModal
      open={!closing}
      afterClose={onClose}
      title="New party"
      okText="Create party"
      onOk={() => form.submit()}
      onCancel={close}
      confirmDiscard={{ when: () => form.isFieldsTouched(), onDiscard: close }}
      confirmLoading={save.isPending}
      closable={!save.isPending}
      keyboard={!save.isPending}
      mask={{ closable: !save.isPending }}
      okButtonProps={{ disabled: paused }}
      cancelButtonProps={{ disabled: save.isPending }}
    >
      {save.error && (
        <ErrorNotice title="Party could not be saved" error={save.error} />
      )}
      <Form
        form={form}
        name={id}
        layout="vertical"
        disabled={paused || save.isPending}
        onFinish={(values) => save.mutate(values)}
      >
        <Form.Item
          name="name"
          label="Party name"
          rules={[{ required: true, whitespace: true, message: "Enter a party name." }]}
        >
          <Input maxLength={255} />
        </Form.Item>
      </Form>
    </AtlasModal>
  );
}

export function PartyDirectory({
  parties,
  selected,
}: {
  parties: PartyView[];
  selected?: string;
}) {
  const [creating, setCreating] = useState(false);
  const paused = useSavedWritesPaused();
  const Heading = selected ? "h2" : "h1";
  return (
    <section
      className={
        selected ? "party-directory party-directory--aside" : "party-directory"
      }
    >
      <div className="party-page-heading">
        <Heading>Parties</Heading>
        <Button
          type={selected ? "default" : "primary"}
          disabled={paused}
          onClick={(event) => {
            event.currentTarget.focus({ preventScroll: true });
            setCreating(true);
          }}
        >
          New party
        </Button>
      </div>
      {parties.length ? (
        <ul>
          {parties.map((party) => (
            <li key={party.party_key}>
              <AtlasLink
                {...partyDestination(party.party_key)}
                aria-current={party.party_key === selected ? "page" : undefined}
              >
                <strong>{party.name}</strong>
                <span>
                  {selected
                    ? memberCount(party.members.length)
                    : party.members.map((member) => member.name).join(", ") ||
                      "No characters"}
                </span>
              </AtlasLink>
            </li>
          ))}
        </ul>
      ) : (
        <p>No parties yet.</p>
      )}
      {creating && <NewParty onClose={() => setCreating(false)} />}
    </section>
  );
}

export function memberCount(count: number) {
  return count === 1 ? "1 character" : `${count} characters`;
}

export function PartiesPage() {
  usePageTitle("Parties");
  const query = useParties();
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
      {query.isPending && <p role="status">Loading parties…</p>}
      {query.data && <PartyDirectory parties={query.data.parties} />}
    </div>
  );
}
