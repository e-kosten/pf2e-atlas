import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { encounterCastingReminders } from "../../api/atlasApi";
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  encounterCastingReminders: vi.fn(),
}));
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
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
        encounterRevision={1n}
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
        encounterRevision={1n}
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
    knockout_options: [],
    effective_max_hp: { state: "value", value: 20, adjustment: null },
    persistent_damage_groups: [],
    track_dying: false,
    mortality: "alive",
    base_death_threshold: 4n,
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
    hazard_disabled: false,
    cast_unavailable_reason: null,
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

it("casts directly once in the existing surface without a second resource action", async () => {
  const detail = detailFixture("actors:caster");
  detail.presentation.body = { kind: "spell", value: spellFixture() };
  detail.selected.spell_rank = 3;
  const participant = resourceParticipant();
  participant.spellcasting!.spells = [
    {
      navigation: detail.selected,
      entry_id: "entry",
      title: "Fireball",
      expended: null,
      ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
    },
  ];
  const onAction = vi.fn().mockResolvedValue(true);
  render(
    <App>
      <EncounterSpellControls
        encounterRevision={1n}
        participant={participant}
        detail={detail}
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledExactlyOnceWith({
      kind: "cast",
      selection: detail.selected,
    }),
  );
  expect(screen.queryByText("Record past use")).toBeNull();
  expect(screen.queryByRole("dialog")).toBeNull();
});

it("uses catalog identity only when the displayed reference lacks it", async () => {
  const detail = detailFixture("actors:caster");
  detail.presentation.body = { kind: "spell", value: spellFixture() };
  detail.selected.spell_rank = 3;
  detail.selected.source_fingerprint = null;
  const canonical = { ...detail.selected, source_fingerprint: "current-source" };
  const participant = resourceParticipant();
  participant.spellcasting!.spells = [
    {
      navigation: canonical,
      entry_id: "entry",
      title: "Fireball",
      expended: null,
      ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
    },
  ];
  const onAction = vi.fn().mockResolvedValue(true);
  const view = render(
    <App>
      <EncounterSpellControls
        encounterRevision={1n}
        participant={participant}
        detail={detail}
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenCalledWith({ kind: "cast", selection: canonical }),
  );
  const stale = {
    ...detail,
    selected: { ...detail.selected, source_fingerprint: "stale-display" },
  };
  view.rerender(
    <App>
      <EncounterSpellControls
        encounterRevision={1n}
        participant={participant}
        detail={stale}
        disabled={false}
        pending={false}
        onAction={onAction}
      />
    </App>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenLastCalledWith({
      kind: "cast",
      selection: { ...canonical, source_fingerprint: "stale-display" },
    }),
  );
});

it("retries unavailable reminders without disabling direct Cast and renders only advisories", async () => {
  const detail = detailFixture("actors:caster");
  detail.presentation.body = { kind: "spell", value: spellFixture() };
  detail.selected.spell_rank = 3;
  const participant = resourceParticipant();
  participant.spellcasting!.spells = [
    {
      navigation: detail.selected,
      entry_id: "entry",
      title: "Fireball",
      expended: null,
      ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
    },
  ];
  vi.mocked(encounterCastingReminders)
    .mockRejectedValueOnce(new Error("Unavailable"))
    .mockResolvedValueOnce({
      advisories: [
        { source: "condition", label: "Check", reason: "DC 8 flat check" },
        {
          source: "control",
          label: "Controlled",
          reason: "The controller chooses your actions.",
        },
      ],
    });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = render(
    <QueryClientProvider client={client}>
      <App>
        <EncounterSpellControls
          encounterRevision={1n}
          encounterRef="encounter"
          participant={participant}
          detail={detail}
          disabled={false}
          pending={false}
          onAction={vi.fn()}
        />
      </App>
    </QueryClientProvider>,
  );
  expect(await screen.findByText("Reminders unavailable")).toBeVisible();
  expect(screen.getByRole("button", { name: "Cast" })).toBeEnabled();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByText("Check · DC 8 flat check")).toBeVisible();
  expect(
    screen.getByText("Controlled · The controller chooses your actions").parentElement,
  ).toBe(screen.getByText("Check · DC 8 flat check").parentElement);
  expect(screen.queryByText("Choose target geometry")).toBeNull();
  view.unmount();
  client.clear();
});

it("retains exact-spell reminders through use-count refresh and shows errors without losing the previous reminder", async () => {
  const detail = detailFixture("actors:caster");
  detail.presentation.body = { kind: "spell", value: spellFixture() };
  detail.selected.spell_rank = 3;
  const participant = resourceParticipant();
  participant.ability_states = [
    {
      label: "Daily ability",
      navigation: detail.selected,
      rule_ordinal: null,
      state: { kind: "uses", remaining: 2n, maximum: 3n, interval: "day" },
    },
  ];
  participant.spellcasting!.spells = [
    {
      navigation: detail.selected,
      entry_id: "entry",
      title: "Fireball",
      expended: null,
      ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
    },
  ];
  const initial: Awaited<ReturnType<typeof encounterCastingReminders>> = {
    advisories: [
      {
        source: "condition",
        label: "Stupefied",
        reason: "DC 6 flat check or the spell is disrupted",
      },
    ],
  };
  let reject: (reason: Error) => void = () => {};
  vi.mocked(encounterCastingReminders)
    .mockReset()
    .mockResolvedValueOnce(initial)
    .mockImplementationOnce(
      () =>
        new Promise((_resolve, fail) => {
          reject = fail;
        }),
    )
    .mockResolvedValueOnce({ ...initial, advisories: [] });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = (current: typeof participant) => (
    <QueryClientProvider client={client}>
      <App>
        <EncounterSpellControls
          encounterRevision={current.revision}
          encounterRef="encounter"
          participant={current}
          detail={detail}
          disabled={false}
          pending={false}
          onAction={vi.fn()}
        />
      </App>
    </QueryClientProvider>
  );
  const { rerender } = render(view(participant));
  const label = "Stupefied · DC 6 flat check or the spell is disrupted";
  const reminder = await screen.findByText(label);
  rerender(
    view({
      ...participant,
      revision: participant.revision + 1n,
      ability_states: [
        {
          ...participant.ability_states[0],
          state: { kind: "uses", remaining: 1n, maximum: 3n, interval: "day" },
        },
      ],
    }),
  );
  await waitFor(() => expect(encounterCastingReminders).toHaveBeenCalledTimes(2));
  expect(screen.getByText(label)).toBe(reminder);
  await act(async () => reject(new Error("Unavailable")));
  expect(await screen.findByText("Reminders unavailable")).toBeVisible();
  expect(screen.getByText(label)).toBe(reminder);
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await waitFor(() => expect(screen.queryByText(label)).toBeNull());
  client.clear();
});

it("retires a stale casting DC after a source option changes even when replacement advice fails", async () => {
  const detail = detailFixture("actors:caster");
  detail.presentation.body = { kind: "spell", value: spellFixture() };
  detail.selected.spell_rank = 3;
  const participant = resourceParticipant();
  participant.spellcasting!.spells = [
    {
      navigation: detail.selected,
      entry_id: "entry",
      title: "Fireball",
      expended: null,
      ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
    },
  ];
  const option = {
    label: "Incurable Curse",
    navigation: {
      ...detail.selected,
      embedded_path: [
        { collection: "/items", identity: { Stable: "curse00000000001" } },
      ],
      source_fingerprint: "source",
    },
    rule_ordinal: 0,
    state: {
      kind: "option",
      always_active: true,
      active: true,
      selection: "Stupefied",
      choices: [
        { value: "Stupefied", label: "Stupefied", consequence: null },
        { value: "Clumsy", label: "Clumsy", consequence: null },
      ],
    },
  } satisfies NonNullable<EncounterParticipantView["ability_states"]>[number];
  participant.ability_states = [option];
  let rejectRefresh: (reason: Error) => void = () => {};
  vi.mocked(encounterCastingReminders)
    .mockReset()
    .mockResolvedValueOnce({
      advisories: [
        {
          source: "source-condition:curse",
          label: "Stupefied",
          reason: "DC 6 flat check or the spell is disrupted",
        },
      ],
    })
    .mockImplementationOnce(
      () =>
        new Promise((_resolve, reject) => {
          rejectRefresh = reject;
        }),
    );
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = (current: typeof participant) => (
    <QueryClientProvider client={client}>
      <App>
        <EncounterSpellControls
          encounterRevision={current.revision}
          encounterRef="encounter"
          participant={current}
          detail={detail}
          disabled={false}
          pending={false}
          onAction={vi.fn()}
        />
      </App>
    </QueryClientProvider>
  );
  const { rerender, unmount } = render(view(participant));
  const oldText = "Stupefied · DC 6 flat check or the spell is disrupted";
  await screen.findByText(oldText);
  const cast = screen.getByRole("button", { name: "Cast" });
  rerender(
    view({
      ...participant,
      revision: participant.revision + 1n,
      ability_states: [
        {
          ...option,
          state: { ...option.state, kind: "option", selection: "Clumsy" },
        },
      ],
    }),
  );
  expect(screen.queryByText(oldText)).toBeNull();
  await waitFor(() => expect(encounterCastingReminders).toHaveBeenCalledTimes(2));
  await act(async () => rejectRefresh(new Error("Unavailable")));
  expect(await screen.findByText("Reminders unavailable")).toBeVisible();
  expect(screen.queryByText(oldText)).toBeNull();
  expect(screen.getByRole("button", { name: "Cast" })).toBe(cast);
  expect(cast).toBeEnabled();
  unmount();
  client.clear();
});

it("refreshes condition reminders on the encounter revision without changing the Cast anchor", async () => {
  const detail = detailFixture("actors:caster");
  detail.presentation.body = { kind: "spell", value: spellFixture() };
  detail.selected.spell_rank = 3;
  const participant = resourceParticipant();
  const grabbed: EncounterParticipantView["conditions"][number] = {
    condition_id: 1n,
    name: "Grabbed",
    scope: { kind: "everyone" },
    extra_action: { kind: "unspecified" },
    immunity_override: false,
    parameters: { kind: "none" },
    created_at: "now",
    updated_at: "now",
  };
  participant.conditions = [
    grabbed,
    { ...grabbed, condition_id: 2n, name: "Stupefied", value: 1n },
  ];
  participant.spellcasting!.spells = [
    {
      navigation: detail.selected,
      entry_id: "entry",
      title: "Fireball",
      expended: null,
      ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
    },
  ];
  type Reminders = Awaited<ReturnType<typeof encounterCastingReminders>>;
  let finishRefresh: (result: Reminders) => void = () => {};
  vi.mocked(encounterCastingReminders)
    .mockReset()
    .mockResolvedValueOnce({
      advisories: [
        {
          source: "condition",
          label: "Grabbed, Stupefied",
          reason: "DC 6 flat check or the spell is disrupted",
        },
      ],
    })
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finishRefresh = resolve;
        }),
    );
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = (current: typeof participant, revision: bigint) => (
    <QueryClientProvider client={client}>
      <App>
        <EncounterSpellControls
          encounterRevision={revision}
          encounterRef="encounter"
          participant={current}
          detail={detail}
          disabled={false}
          pending={false}
          onAction={vi.fn()}
        />
      </App>
    </QueryClientProvider>
  );
  const { rerender, unmount } = render(view(participant, 1n));
  const oldText = "Grabbed, Stupefied · DC 6 flat check or the spell is disrupted";
  await screen.findByText(oldText);
  const castButton = screen.getByRole("button", { name: "Cast" });
  // Removing a condition changes the encounter, while the participant revision stays 7.
  rerender(view({ ...participant, conditions: [grabbed] }, 2n));
  expect(screen.queryByText(oldText)).toBeNull();
  await waitFor(() => expect(encounterCastingReminders).toHaveBeenCalledTimes(2));
  await act(async () =>
    finishRefresh({
      advisories: [
        {
          source: "condition",
          label: "Grabbed",
          reason: "DC 5 flat check or the spell is disrupted",
        },
      ],
    }),
  );
  expect(
    await screen.findByText("Grabbed · DC 5 flat check or the spell is disrupted"),
  ).toBeVisible();
  expect(screen.getByRole("button", { name: "Cast" })).toBe(castButton);
  expect(castButton).toBeEnabled();
  unmount();
  client.clear();
});

it.each(["participant", "selection", "generation"])(
  "never retains a reminder across a changed %s",
  async (changed) => {
    const detail = detailFixture("actors:caster");
    detail.presentation.body = { kind: "spell", value: spellFixture() };
    detail.selected.spell_rank = 3;
    const participant = resourceParticipant();
    const spell = {
      navigation: detail.selected,
      entry_id: "entry",
      title: "Fireball",
      expended: null,
      ranks: [{ rank: 3, resource_key: "rank3", unavailable_reason: null }],
    };
    participant.spellcasting!.spells = [spell];
    const initial: Awaited<ReturnType<typeof encounterCastingReminders>> = {
      advisories: [
        { source: "condition", label: "Old", reason: "Old participant reminder" },
      ],
    };
    vi.mocked(encounterCastingReminders)
      .mockReset()
      .mockResolvedValueOnce(initial)
      .mockImplementationOnce(() => new Promise(() => {}));
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const view = (current: typeof participant, document: typeof detail) => (
      <QueryClientProvider client={client}>
        <App>
          <EncounterSpellControls
            encounterRevision={1n}
            encounterRef="encounter"
            participant={current}
            detail={document}
            disabled={false}
            pending={false}
            onAction={vi.fn()}
          />
        </App>
      </QueryClientProvider>
    );
    const { rerender, unmount } = render(view(participant, detail));
    await screen.findByText("Old · Old participant reminder");
    const selected =
      changed === "selection"
        ? { ...detail.selected, source_fingerprint: "new-copy" }
        : detail.selected;
    const nextDetail = {
      ...detail,
      selected,
      artifact_generation:
        changed === "generation" ? "new-generation" : detail.artifact_generation,
    };
    const nextParticipant = {
      ...participant,
      participant_key:
        changed === "participant" ? "new-participant" : participant.participant_key,
      spellcasting: {
        ...participant.spellcasting!,
        spells: [{ ...spell, navigation: selected }],
      },
    };
    rerender(view(nextParticipant, nextDetail));
    expect(screen.queryByText("Old · Old participant reminder")).toBeNull();
    unmount();
    client.clear();
  },
);

it("keeps a known saved outcome inside the resource editor after its refresh fails", async () => {
  const participant = resourceParticipant();
  const props = {
    participant,
    resource: participant.spellcasting!.resources[0],
    disabled: false,
    pending: false,
    onAction: vi.fn().mockResolvedValue(false),
  };
  const { rerender } = render(<SharedResourceControl {...props} />);
  fireEvent.click(
    screen.getByRole("button", { name: "Rank 3 slots: 2 of 4 remaining" }),
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Restore one Rank 3 slots" }),
  );
  await screen.findByText("Resource change could not be confirmed");
  rerender(
    <SharedResourceControl
      {...props}
      recovery={<span role="status">Saved, but the encounter could not refresh</span>}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent(
    "Saved, but the encounter could not refresh",
  );
  expect(screen.queryByText("Resource change could not be confirmed")).toBeNull();
  expect(props.onAction).toHaveBeenCalledOnce();
});
