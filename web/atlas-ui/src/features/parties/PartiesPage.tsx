import { Button, Form, Input } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useId, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import scrollIntoView from "scroll-into-view-if-needed";
import { createParty } from "../../api/atlasApi";
import { AtlasButtonLink, AtlasLink } from "../../app/AtlasLink";
import { useDesktopTab } from "../../app/DesktopTabContext";
import {
  charactersDestination,
  partiesDestination,
  partyDestination,
} from "../../app/destinations";
import type { PartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";
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

function NewPartyAction() {
  const [creating, setCreating] = useState(false);
  const paused = useSavedWritesPaused();
  return (
    <>
      <Button
        type="primary"
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

function PartyDirectory({
  parties,
  selected,
  restorationId,
}: {
  parties: PartyView[];
  selected?: string;
  restorationId: string;
}) {
  const current = useRef<HTMLLIElement>(null);
  useLayoutEffect(() => {
    const row = current.current;
    const directory = row?.parentElement;
    if (!row || !directory) return;
    const reveal = () => {
      if (directory.clientWidth > 0) {
        scrollIntoView(row, {
          boundary: directory,
          block: "nearest",
          inline: "nearest",
          scrollMode: "if-needed",
        });
      }
    };
    reveal();
    let visible = directory.clientWidth > 0;
    const observer = new ResizeObserver(() => {
      const next = directory.clientWidth > 0;
      if (next && !visible) reveal();
      visible = next;
    });
    observer.observe(directory);
    return () => observer.disconnect();
  }, [selected]);
  return (
    <nav aria-label="Parties">
      <ul data-scroll-restoration-id={restorationId}>
        {parties.map((party) => (
          <li
            key={party.party_key}
            ref={party.party_key === selected ? current : undefined}
          >
            <AtlasLink
              {...partyDestination(party.party_key)}
              resetScroll={false}
              data-party-key={party.party_key}
              aria-current={party.party_key === selected ? "page" : undefined}
            >
              <strong>{party.name}</strong>
              <span>{memberCount(party.members.length)}</span>
            </AtlasLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}

export function memberCount(count: number) {
  return count === 1 ? "1 character" : `${count} characters`;
}

/** The parent route keeps this directory mounted while the roster changes. */
export function PartiesWorkspace({
  selected,
  children,
}: {
  selected?: string;
  children: ReactNode;
}) {
  const query = useParties();
  const tab = useDesktopTab();
  const active = tab?.active ?? true;
  const restorationId = tab ? `party-directory-${tab.id}` : "party-directory";
  const directory = useRef<HTMLElement>(null);
  const detail = useRef<HTMLDivElement>(null);
  const previousSelection = useRef(selected);
  useLayoutEffect(() => {
    const previous = previousSelection.current;
    previousSelection.current = selected;
    if (!active) return;
    const media = window.matchMedia("(max-width: 767px)");
    const focusDirectory = (key?: string) => {
      const target =
        Array.from(
          directory.current?.querySelectorAll<HTMLAnchorElement>("a[data-party-key]") ??
            [],
        ).find((link) => link.dataset.partyKey === key) ?? directory.current;
      target?.focus({ preventScroll: true });
    };
    const focusRoster = () => {
      const target =
        detail.current?.querySelector<HTMLElement>("[data-party-title]") ??
        detail.current;
      target?.focus({ preventScroll: true });
    };
    if (previous !== selected && media.matches) {
      if (selected) focusRoster();
      else focusDirectory(previous);
    }

    // CSS reflow can blur a newly hidden control before the media event arrives.
    let focused = document.activeElement;
    const rememberFocus = (event: FocusEvent) => {
      if (event.target instanceof Element) focused = event.target;
    };
    const recoverHiddenFocus = () => {
      if (!focused?.isConnected || focused.getClientRects().length > 0) return;
      if (
        directory.current?.contains(focused) ||
        detail.current?.contains(focused) ||
        focused.closest(".party-directory-back")
      ) {
        if (media.matches && selected) focusRoster();
        else focusDirectory(selected);
      }
    };
    document.addEventListener("focusin", rememberFocus);
    media.addEventListener("change", recoverHiddenFocus);
    return () => {
      document.removeEventListener("focusin", rememberFocus);
      media.removeEventListener("change", recoverHiddenFocus);
    };
  }, [selected, active]);

  return (
    <div
      className={`party-page parties-workspace-page${selected ? " parties-workspace-page--selected" : ""}`}
    >
      <header className="party-page-heading">
        <h1>Parties</h1>
        <AtlasButtonLink {...charactersDestination()}>Characters</AtlasButtonLink>
      </header>
      {selected && (
        <AtlasButtonLink
          {...partiesDestination()}
          resetScroll={false}
          className="party-directory-back"
        >
          Back to parties
        </AtlasButtonLink>
      )}
      {query.error && (
        <ErrorNotice
          title="Parties could not load"
          error={query.error}
          action={<Button onClick={() => void query.refetch()}>Retry</Button>}
        />
      )}
      <div className="party-workspace">
        <aside
          className="party-directory"
          ref={directory}
          tabIndex={-1}
          aria-label="Party list"
        >
          <NewPartyAction />
          {query.isPending && <LoadingStatus label="Loading parties…" />}
          {query.data &&
            (query.data.parties.length ? (
              <PartyDirectory
                selected={selected}
                parties={query.data.parties}
                restorationId={restorationId}
              />
            ) : (
              <p>No parties yet.</p>
            ))}
        </aside>
        <div className="party-workspace-detail" ref={detail} tabIndex={-1}>
          {query.isPending && selected && (
            <LoadingStatus className="party-roster-loading" label="Loading party…" />
          )}
          {query.data && children}
        </div>
      </div>
    </div>
  );
}

export function PartiesPage() {
  usePageTitle("Parties");
  const query = useParties();
  return (
    <section className="party-selection-prompt">
      <h2>
        {query.data?.parties.length ? "Select a party" : "Create your first party"}
      </h2>
      <p>
        {query.data?.parties.length
          ? "Choose a party to view its characters."
          : "Create a party to group your saved characters."}
      </p>
    </section>
  );
}
