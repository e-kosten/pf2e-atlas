import { ArrowLeft, Plus } from "lucide-react";
import { DirectoryHeader } from "../../shared/ui/pages/EntityIndexPage";
import { CharacterSectionNav } from "./CharacterSectionNav";
import { Button, Form, Input } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useId, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import scrollIntoView from "scroll-into-view-if-needed";
import { createParty } from "../../api/atlasApi";
import { AtlasLink } from "../../app/AtlasLink";
import { useDesktopTab } from "../../app/DesktopTabContext";
import { partiesDestination, partyDestination } from "../../app/destinations";
import type { PartyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { DraftDiscardDialog, useDraftNavigation } from "./CharacterDraftNavigation";
import { refreshCharacters } from "./characterQueries";
import { useParties } from "./useParties";

function NewParty({ onClose }: { onClose: () => void }) {
  const [form] = Form.useForm<{ name: string }>();
  const id = useId();
  const [partyName, setPartyName] = useState("");
  const [closing, setClosing] = useState(false);
  const client = useQueryClient();
  const navigate = useNavigate();
  const paused = useSavedWritesPaused();
  const inFlight = useRef<Promise<unknown> | null>(null);
  const committed = useRef(false);
  const dirty = () => !committed.current && Boolean(form.getFieldValue("name")?.trim());
  const { blocker, waiting } = useDraftNavigation(dirty, () =>
    committed.current ? null : inFlight.current,
  );
  const save = useMutation({
    retry: false,
    mutationFn: ({ name }: { name: string }) =>
      createParty({ name: name.trim(), members: [] }),
    onSuccess: async ({ party_key }) => {
      await refreshCharacters(client);
      committed.current = true;
      setClosing(true);
      if (waiting.current === 0) await navigate(partyDestination(party_key));
    },
  });
  const close = () => setClosing(true);
  function submit(values: { name: string }) {
    if (paused || inFlight.current) return;
    inFlight.current = save
      .mutateAsync(values)
      .catch(() => undefined)
      .finally(() => {
        inFlight.current = null;
      });
  }
  return (
    <>
      <AtlasModal
        open={!closing}
        afterClose={onClose}
        title="New party"
        okText="Create party"
        onOk={() => form.submit()}
        onCancel={close}
        confirmDiscard={{ when: dirty, onDiscard: close }}
        confirmLoading={save.isPending}
        closable={!save.isPending}
        keyboard={!save.isPending}
        mask={{ closable: !save.isPending }}
        okButtonProps={{ disabled: paused || !partyName?.trim() }}
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
          onValuesChange={(_changed, all) => setPartyName(all.name)}
          onFinish={submit}
        >
          <Form.Item
            name="name"
            label="Party name"
            rules={[
              { required: true, whitespace: true, message: "Enter a party name." },
            ]}
          >
            <Input maxLength={255} />
          </Form.Item>
        </Form>
      </AtlasModal>
      <DraftDiscardDialog
        open={blocker.status === "blocked"}
        title="Discard new party?"
        description="The party name you entered will be discarded."
        onKeep={() => blocker.reset?.()}
        onDiscard={() => blocker.proceed?.()}
      />
    </>
  );
}

function NewPartyAction() {
  const [creating, setCreating] = useState(false);
  const paused = useSavedWritesPaused();
  return (
    <>
      <Button
        type="primary"
        icon={<Plus size={16} />}
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
        focused.closest(
          ".party-directory-back, .parties-workspace-page .directory-header__heading",
        )
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
    <main
      className={`directory-page parties-workspace-page${selected ? " parties-workspace-page--selected" : ""}`}
    >
      <DirectoryHeader
        title="Parties"
        actions={<NewPartyAction />}
        navigation={<CharacterSectionNav current="Parties" />}
      />
      {selected && (
        <AtlasLink
          {...partiesDestination()}
          resetScroll={false}
          className="party-directory-back"
        >
          <ArrowLeft size={14} aria-hidden="true" />
          Back to parties
        </AtlasLink>
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
    </main>
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
