import { Button, Form, Input } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useId, useLayoutEffect, useRef, useState } from "react";
import scrollIntoView from "scroll-into-view-if-needed";
import { createParty } from "../../api/atlasApi";
import { AtlasLink } from "../../app/AtlasLink";
import { partyDestination } from "../../app/destinations";
import type { PartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { EntityIndexPage } from "../../shared/ui/pages/EntityIndexPage";
import { IndexTable } from "../../shared/ui/tables/IndexTable";
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

function NewPartyAction({ primary = false }: { primary?: boolean }) {
  const [creating, setCreating] = useState(false);
  const paused = useSavedWritesPaused();
  return (
    <>
      <Button
        type={primary ? "primary" : "default"}
        disabled={paused}
        onClick={(event) => {
          event.currentTarget.focus({ preventScroll: true });
          setCreating(true);
        }}
      >
        New party
      </Button>
      {creating && <NewParty onClose={() => setCreating(false)} />}
    </>
  );
}

export function PartyDirectory({
  parties,
  selected,
}: {
  parties: PartyView[];
  selected: string;
}) {
  const current = useRef<HTMLLIElement>(null);
  useLayoutEffect(() => {
    const row = current.current;
    const directory = row?.parentElement;
    if (!row || !directory) return;
    const reveal = () => {
      if (directory.scrollWidth > directory.clientWidth) {
        scrollIntoView(row, {
          boundary: directory,
          block: "nearest",
          inline: "nearest",
          scrollMode: "if-needed",
        });
      }
    };
    reveal();
    const observer = new ResizeObserver(reveal);
    observer.observe(directory);
    return () => observer.disconnect();
  }, [selected, parties]);
  return (
    <section className="party-directory">
      <div className="party-page-heading">
        <h2>Parties</h2>
        <NewPartyAction />
      </div>
      {parties.length ? (
        <ul>
          {parties.map((party) => (
            <li
              key={party.party_key}
              ref={party.party_key === selected ? current : undefined}
            >
              <AtlasLink
                {...partyDestination(party.party_key)}
                aria-current={party.party_key === selected ? "page" : undefined}
              >
                <strong>{party.name}</strong>
                <span>{memberCount(party.members.length)}</span>
              </AtlasLink>
            </li>
          ))}
        </ul>
      ) : (
        <p>No parties yet.</p>
      )}
    </section>
  );
}

export function memberCount(count: number) {
  return count === 1 ? "1 character" : `${count} characters`;
}

export function PartiesPage() {
  usePageTitle("Parties");
  const query = useParties();
  const navigate = useNavigate();
  return (
    <EntityIndexPage
      className="parties-index"
      title="Parties"
      actions={<NewPartyAction primary />}
      toolbar={<PartyNavigation active="parties" />}
    >
      {query.error && (
        <ErrorNotice
          title="Parties could not load"
          error={query.error}
          action={<Button onClick={() => void query.refetch()}>Retry</Button>}
        />
      )}
      {query.isPending && <p role="status">Loading parties…</p>}
      {query.data && (
        <IndexTable<PartyView>
          dataSource={query.data.parties}
          rowKey={(party) => party.party_key}
          onActivateRow={(party) => void navigate(partyDestination(party.party_key))}
          locale={{ emptyText: "No parties yet." }}
          columns={[
            {
              title: "Party",
              width: "36%",
              render: (_, party) => (
                <AtlasLink {...partyDestination(party.party_key)}>
                  {party.name}
                </AtlasLink>
              ),
            },
            {
              title: "PCs",
              width: 72,
              render: (_, party) => party.members.length,
            },
            {
              title: "Characters",
              render: (_, party) =>
                party.members.map((member) => member.name).join(", ") || "—",
            },
          ]}
        />
      )}
    </EntityIndexPage>
  );
}
