import { isUnconfirmedMutation } from "../../api/apiError";
import { isSharedFailure } from "../../shared/feedback/errorMessages";
import { Button, Dropdown, type MenuProps } from "antd";
import { ChevronDown } from "lucide-react";
import { useRef, useState } from "react";
import type { AddEncounterPartyRequest, PartyView } from "../../generated/atlas";
import { AtlasLink } from "../../app/AtlasLink";
import { partiesDestination } from "../../app/destinations";
import { useParties } from "../parties/useParties";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";

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
  onAddPc: () => void;
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
  const menuItems: MenuProps["items"] = [
    {
      key: "creature",
      label: "Add creature or hazard…",
      disabled: disabled || adding,
      onClick: () => {
        setOpen(false);
        trigger.current?.focus();
        onAddCreature();
      },
    },
    {
      key: "pc",
      label: "Add PC…",
      disabled: disabled || adding,
      onClick: () => {
        setOpen(false);
        trigger.current?.focus();
        onAddPc();
      },
    },
    ...(onAddInitiativeEntry
      ? [
          {
            key: "initiative-entry",
            label: "Add custom turn…",
            disabled: disabled || adding,
            onClick: () => {
              setOpen(false);
              trigger.current?.focus();
              onAddInitiativeEntry(trigger.current);
            },
          },
        ]
      : []),
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
