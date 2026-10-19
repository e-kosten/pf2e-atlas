import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { App } from "antd";
import {
  EncounterSpellControls,
  EncounterSpellResources,
  SharedResourceControl,
} from "./EncounterSpellControls";
import { detailFixture, spellFixture } from "../../test/fixtures";
import type { EncounterParticipantView } from "../../generated/atlas";

it("disables casting at JSON numeric zero and keeps the count visible", () => {
  const detail = detailFixture("actors:caster");
  detail.presentation.body = { kind: "spell", value: spellFixture() };
  detail.selected.spell_rank = 3;
  // Exercise the actual HTTP representation, not bigint-only test fixtures.
  const participant = JSON.parse(
    JSON.stringify({
      spellcasting: {
        resources: [
          {
            key: "rank3",
            entry_id: "entry",
            label: "Rank 3 slots",
            scope: { kind: "rank", rank: 3 },
            capacity: 4,
            remaining: 0,
            can_undo: true,
          },
        ],
        spells: [
          {
            navigation: detail.selected,
            entry_id: "entry",
            title: "Fireball",
            expended: null,
            ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
          },
        ],
      },
    }),
  ) as EncounterParticipantView;
  const onAction = vi.fn();
  const { rerender } = render(
    <App>
      <EncounterSpellControls
        participant={participant}
        detail={detail}
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>,
  );
  expect(screen.getByRole("button", { name: "Cast" })).toBeDisabled();
  expect(screen.getByLabelText("Rank 3 slots: 0 of 4 remaining")).toHaveTextContent(
    "0/4",
  );
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  expect(onAction).not.toHaveBeenCalled();
  participant.spellcasting!.resources[0].remaining = 4n;
  rerender(
    <App>
      <EncounterSpellControls
        participant={participant}
        detail={detail}
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>,
  );
  expect(screen.getByRole("button", { name: "Cast" })).toBeEnabled();
  expect(screen.getByLabelText("Rank 3 slots: 4 of 4 remaining")).toHaveTextContent(
    "4/4",
  );
});

it("keeps edits and Restore all local until Done, and Cancel discards them", async () => {
  const participant = resourceParticipant();
  const onAction = vi.fn().mockResolvedValue(true);
  render(
    <App>
      <EncounterSpellResources
        participant={participant}
        entryId="entry"
        title="Arcane spells"
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>,
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Manage Arcane spells resources" }),
  );
  fireEvent.change(screen.getByRole("spinbutton", { name: "Rank 3 slots remaining" }), {
    target: { value: "1" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Restore all" }));
  expect(
    screen.getByRole("spinbutton", { name: "Rank 3 slots remaining" }),
  ).toHaveValue("4");
  expect(onAction).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  fireEvent.click(
    screen.getByRole("button", { name: "Manage Arcane spells resources" }),
  );
  expect(
    screen.getByRole("spinbutton", { name: "Rank 3 slots remaining" }),
  ).toHaveValue("2");
  fireEvent.change(screen.getByRole("spinbutton", { name: "Rank 3 slots remaining" }), {
    target: { value: "1" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Done" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledWith(
      {
        kind: "set_entry_resources",
        entry_id: "entry",
        remaining: { rank3: 1n, rank4: 3n },
      },
      7n,
    ),
  );
});

it("preserves a draft and prevents overwrite when the participant revision changes", () => {
  const participant = resourceParticipant();
  const onAction = vi.fn();
  const view = (p: EncounterParticipantView) => (
    <App>
      <EncounterSpellResources
        participant={p}
        entryId="entry"
        title="Arcane spells"
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>
  );
  const { rerender } = render(view(participant));
  fireEvent.click(
    screen.getByRole("button", { name: "Manage Arcane spells resources" }),
  );
  fireEvent.change(screen.getByRole("spinbutton", { name: "Rank 3 slots remaining" }), {
    target: { value: "1" },
  });
  rerender(view({ ...participant, revision: 8n }));
  expect(
    screen.getByRole("spinbutton", { name: "Rank 3 slots remaining" }),
  ).toHaveValue("1");
  expect(screen.getByRole("button", { name: "Done" })).toBeDisabled();
  expect(screen.getByRole("alert")).toHaveTextContent("Your edits have not been saved");
  expect(onAction).not.toHaveBeenCalled();
});

function resourceParticipant() {
  return {
    revision: 7n,
    participant_key: "caster",
    position: 1n,
    initiative_order: 1n,
    display_name: "Caster",
    hp_origin: "derived",
    variant_origin: "default_unadjusted",
    participant_kind: "creature",
    side: "enemy",
    participant_variant: "normal",
    record_key: "actors:caster",
    status: "active",
    temporary_hp: 0n,
    defeated: false,
    hidden: false,
    note_hint: "",
    conditions: [],
    spellcasting: {
      spells: [],
      resources: [
        {
          key: "rank3",
          entry_id: "entry",
          label: "Rank 3 slots",
          scope: { kind: "rank", rank: 3 },
          capacity: 4n,
          remaining: 2n,
          can_undo: false,
        },
        {
          key: "rank4",
          entry_id: "entry",
          label: "Rank 4 slots",
          scope: { kind: "rank", rank: 4 },
          capacity: 3n,
          remaining: 3n,
          can_undo: false,
        },
      ],
    },
  } as EncounterParticipantView;
}

it("restores a shared rank directly while preserving other pools", async () => {
  const participant = resourceParticipant();
  const onAction = vi.fn().mockResolvedValue(true);
  render(
    <App>
      <SharedResourceControl
        participant={participant}
        resource={participant.spellcasting!.resources[0]}
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>,
  );
  fireEvent.click(
    screen.getByRole("button", { name: "Rank 3 slots: 2 of 4 remaining" }),
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Restore one Rank 3 slots" }),
  );
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledWith(
      {
        kind: "set_entry_resources",
        entry_id: "entry",
        remaining: { rank3: 3n, rank4: 3n },
      },
      7n,
    ),
  );
  fireEvent.click(screen.getByRole("button", { name: "Restore all" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenLastCalledWith(
      {
        kind: "set_entry_resources",
        entry_id: "entry",
        remaining: { rank3: 4n, rank4: 3n },
      },
      7n,
    ),
  );
});
