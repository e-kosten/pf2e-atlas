import { isUnconfirmedMutation } from "../../api/apiError";
import { isSharedFailure } from "../../shared/feedback/errorMessages";
import { Button, Dropdown, type MenuProps } from "antd";
import { ChevronDown } from "lucide-react";
import { useRef, useState, type SyntheticEvent } from "react";
import type { AddEncounterPartyRequest, PartyView } from "../../generated/atlas";
import { AtlasLink } from "../../app/AtlasLink";
import { partiesDestination } from "../../app/destinations";
import { useParties } from "../parties/useParties";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";

export type EncounterPcAddMode = "existing" | "manual";

export function EncounterAddMenu({
  onAdd,
  onAddCreature,
  onAddPc,
  onAddInitiativeEntry,
  disabled,
  encounterRevision,
}: {
  onAdd: (request: AddEncounterPartyRequest) => Promise<void>;
  onAddCreature: () => void;
  onAddPc: (mode: EncounterPcAddMode) => void;
  onAddInitiativeEntry?: (opener: HTMLElement | null) => void;
  disabled: boolean;
  encounterRevision?: number;
}) {
  const [open, setOpen] = useState(false);
  const [adding, setAdding] = useState(false);
  const parties = useParties(open);
  const inFlight = useRef(false);
  const trigger = useRef<HTMLElement | null>(null);
  const pendingRequest = useRef<AddEncounterPartyRequest | null>(null);
  const add = async (party: PartyView) => {
    if (disabled || inFlight.current || party.members.length === 0) return;
    inFlight.current = true;
    setAdding(true);
    setOpen(false);
    try {
      if (pendingRequest.current?.party_key !== party.party_key)
        pendingRequest.current = null;
      pendingRequest.current ??= {
        request_id: crypto.randomUUID(),
        party_key: party.party_key,
        expected_revision: party.revision,
        expected_encounter_revision: encounterRevision!,
        characters: party.members.map((member) => ({
          character_key: member.character_key,
          expected_revision: member.character_revision,
        })),
      };
      await onAdd(pendingRequest.current);
      pendingRequest.current = null;
    } catch (error) {
      if (!isUnconfirmedMutation(error)) pendingRequest.current = null;
      // Keep uncertain writes tied to their original request for a safe retry.
    } finally {
      inFlight.current = false;
      setAdding(false);
    }
  };
  const items: MenuProps["items"] =
    parties.data?.parties.map((party) => ({
      key: party.party_key,
      disabled:
        disabled ||
        adding ||
        encounterRevision === undefined ||
        parties.isFetching ||
        party.members.length === 0,
      label: (
        <div className="party-menu-entry">
          <span>{party.name}</span>
          <span className="party-menu-entry__members">
            {party.members.map((m) => m.name).join(", ") || "No PCs"}
          </span>
        </div>
      ),
      onClick: () => void add(party),
    })) ?? [];
  if (parties.isLoading)
    items.push({
      key: "loading",
      disabled: true,
      label: <LoadingStatus label="Loading parties…" />,
    });
  if (parties.error && !isSharedFailure(parties.error))
    items.push({
      key: "retry",
      label: "Parties could not load. Retry",
      onClick: () => {
        void parties.refetch();
      },
    });
  if (parties.data?.parties.length === 0 && !parties.error)
    items.push({
      key: "create",
      label: <AtlasLink {...partiesDestination()}>Create party…</AtlasLink>,
    });
  const openEditor = (
    event: SyntheticEvent,
    show: (opener: HTMLElement | null) => void,
  ) => {
    // Enter can activate the dialog's newly focused close button.
    event.preventDefault();
    setOpen(false);
    trigger.current?.focus();
    show(trigger.current);
  };
  const menuItems: MenuProps["items"] = [
    {
      key: "creature",
      label: "Add creature or hazard…",
      disabled: disabled || adding,
      onClick: ({ domEvent }) => openEditor(domEvent, onAddCreature),
    },
    ...(onAddInitiativeEntry
      ? [
          {
            key: "initiative-entry",
            label: "Add custom turn…",
            disabled: disabled || adding,
            onClick: ({ domEvent }: { domEvent: SyntheticEvent }) =>
              openEditor(domEvent, onAddInitiativeEntry),
          },
        ]
      : []),
    { type: "divider" },
    {
      key: "pc",
      type: "group",
      label: "Add PC",
      children: [
        {
          key: "existing-pc",
          label: "Existing character",
          disabled: disabled || adding,
          onClick: ({ domEvent }) => openEditor(domEvent, () => onAddPc("existing")),
        },
        {
          key: "manual-pc",
          label: "Enter manually",
          disabled: disabled || adding,
          onClick: ({ domEvent }) => openEditor(domEvent, () => onAddPc("manual")),
        },
      ],
    },
    { type: "divider" },
    { key: "parties", type: "group", label: "Add party", children: items },
  ];
  return (
    <>
      <Dropdown
        trigger={["click"]}
        open={open && !disabled && !adding}
        onOpenChange={setOpen}
        menu={{ items: menuItems, "aria-label": "Add participants" }}
      >
        <Button
          ref={(node) => {
            trigger.current = node;
          }}
          disabled={disabled || adding}
          loading={adding}
          aria-label="Add participants"
          aria-haspopup="menu"
          aria-expanded={open && !disabled && !adding}
        >
          Add <ChevronDown size={14} aria-hidden="true" />
        </Button>
      </Dropdown>
    </>
  );
}
