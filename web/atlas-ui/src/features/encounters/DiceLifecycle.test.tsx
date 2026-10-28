import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  AtlasApiError,
  getEncounterStrikeOptions,
  getInitiativeOptions,
  rollEncounterStrike,
  previewEncounterStrikeContext,
} from "../../api/atlasApi";
import { activityFixture, knownFact } from "../../test/fixtures";
import type {
  EncounterParticipantView,
  EncounterStrikeOptionsView,
  InitiativeRollView,
  RollInitiativeRequest,
  EncounterStrikeRollView,
  RollEncounterStrikeRequest,
} from "../../generated/atlas";
import { EncounterStrikeControl } from "./EncounterStrikeControl";
import { useEncounterStrike } from "./useEncounterStrike";
import type { useEncounterMutations } from "./useEncounterMutations";
import { InitiativePopover, InitiativeRollModal } from "./InitiativeRollModal";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getEncounterStrikeOptions: vi.fn(),
  getInitiativeOptions: vi.fn(),
  rollEncounterStrike: vi.fn(),
  previewEncounterStrikeContext: vi.fn(),
}));
const participant: EncounterParticipantView = {
  knockout_options: [],
  effective_max_hp: { state: "value", value: 20, adjustment: null },
  persistent_damage_groups: [],
  track_dying: false,
  mortality: "alive",
  base_death_threshold: 4n,
  participant_key: "p",
  display_name: "Goblin",
  participant_kind: "creature",
  revision: 1n,
  hp_origin: "explicit",
  variant_origin: "explicit",
  participant_variant: "normal",
  status: "active",
  position: 1n,
  side: "enemy",
  initiative_order: 1n,
  temporary_hp: 0n,
  defeated: false,
  hazard_disabled: false,
  cast_unavailable_reason: null,
  hidden: false,
  note_hint: null,
  conditions: [],
};
const outcome = {
  formula: "1d20+9",
  total: 19,
  expression: { kind: "number" as const, value: 19 },
};
function wrapper({ children }: { children: React.ReactNode }) {
  return (
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      {children}
    </QueryClientProvider>
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  vi.spyOn(window, "matchMedia").mockImplementation(
    (media) =>
      ({
        matches: true,
        media,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      }) as unknown as MediaQueryList,
  );
  vi.mocked(getEncounterStrikeOptions).mockResolvedValue({
    applied: [],
    conditional: [],
    expected_revision: 1n,
    expected_participant_revision: 1n,
    title: "Jaws",
    attacks: activityFixture().attack_options,
    unavailable: null,
    damage_available: true,
    damage_unavailable: null,
  });
  vi.mocked(getInitiativeOptions).mockResolvedValue({
    encounter_key: "e",
    artifact_generation: "generation",
    expected_revision: 1n,
    participants: [
      {
        participant_key: "p",
        expected_revision: 1n,
        statistics: [{ key: "perception", label: "Perception", modifier: 9 }],
      },
    ],
  });
});
afterEach(() => vi.restoreAllMocks());

it("moves keyboard focus into a damage result and restores its exact opener", async () => {
  vi.mocked(rollEncounterStrike).mockImplementation(async (_key, request) =>
    strikeReceipt(request),
  );
  renderStrike();
  const trigger = screen.getByRole("button", { name: "Roll Jaws damage" });
  act(() => trigger.focus());
  // jsdom does not synthesize the native button click from Enter; send that activation.
  fireEvent.click(trigger);
  const popup = await screen.findByRole("dialog", { name: "Jaws · Damage" });
  await waitFor(() => expect(popup).toHaveFocus());
  await screen.findByRole("button", { name: "Reroll" });
  expect(popup).toHaveFocus(); // Async result must not send focus back behind the popup.
  fireEvent.click(within(popup).getByRole("button", { name: "Close" }));
  await waitFor(() => expect(trigger).toHaveFocus());
  fireEvent.click(trigger);
  await screen.findByRole("button", { name: "Reroll" });
  expect(rollEncounterStrike).toHaveBeenCalledTimes(1);
});

it("waits for initiative controls before opening and permits cancelling a pending read", async () => {
  const options = await getInitiativeOptions("e");
  let resolve!: (value: typeof options) => void;
  vi.mocked(getInitiativeOptions).mockImplementation(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  render(
    <InitiativePopover
      participant={participant}
      encounterKey="e"
      onRoll={vi.fn()}
      onUpdate={vi.fn()}
      disabled={false}
    />,
    { wrapper },
  );
  const trigger = screen.getByRole("button", { name: "Goblin initiative, not rolled" });
  fireEvent.click(trigger);
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  fireEvent.keyDown(document, { key: "Escape" });
  await act(async () => resolve(options));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  fireEvent.click(trigger);
  const popup = await screen.findByRole("dialog", { name: "Goblin · Initiative" });
  expect(within(popup).getByRole("combobox")).toHaveAccessibleName(
    "Goblin initiative statistic",
  );
});

it("moves focus into initiative controls and restores the roster opener on dismissal", async () => {
  render(
    <InitiativePopover
      participant={participant}
      encounterKey="e"
      onRoll={vi.fn()}
      onUpdate={vi.fn()}
      disabled={false}
    />,
    { wrapper },
  );
  const trigger = screen.getByRole("button", { name: "Goblin initiative, not rolled" });
  act(() => trigger.focus());
  fireEvent.click(trigger);
  const popup = await screen.findByRole("dialog", { name: "Goblin · Initiative" });
  await waitFor(() => expect(popup).toHaveFocus());
  await within(popup).findByRole("button", { name: "Close" });
  fireEvent.keyDown(popup, { key: "Escape", code: "Escape" });
  await waitFor(() => expect(trigger).toHaveFocus());
});
function strikeReceipt(request: RollEncounterStrikeRequest): EncounterStrikeRollView {
  return {
    request_id: request.request_id,
    encounter_key: "e",
    participant_key: "p",
    title: "Jaws",
    attack: null,
    attack_modifier: null,
    damage: [
      {
        id: "primary",
        label: "Piercing",
        formula: "1d6",
        damage_type: "piercing",
        category: "normal",
        outcome,
        unavailable: null,
      },
    ],
    damage_totals: [],
    damage_total: outcome.total,
    persistent_total: null,
    damage_minimum_adjustment: 0,
    persistent_minimum_adjustment: 0,
  };
}
function renderStrike() {
  return render(
    <EncounterStrikeControl
      encounterKey="e"
      generation="generation"
      participant={participant}
      activity={{
        ...activityFixture(),
        damage: knownFact([
          {
            id: "primary",
            formula: knownFact("1d6"),
            damage_type: knownFact("piercing"),
            kinds: knownFact(["damage"]),
            category: knownFact("normal"),
            materials: knownFact([]),
            apply_modifier: knownFact(false),
          },
        ]),
      }}
      onBeginStrike={vi.fn()}
      disabled={false}
    />,
    { wrapper },
  );
}
it("damage roll makes a fresh attempt after an uncertain response and on reroll", async () => {
  vi.mocked(rollEncounterStrike)
    .mockRejectedValueOnce(new TypeError("Network interrupted"))
    .mockImplementation(async (_key, request) => strikeReceipt(request));
  renderStrike();
  fireEvent.click(screen.getByRole("button", { name: "Roll Jaws damage" }));
  fireEvent.click(await screen.findByRole("button", { name: "Roll again" }));
  await screen.findByRole("button", { name: "Reroll" });
  const calls = vi.mocked(rollEncounterStrike).mock.calls;
  expect(calls[1][1].request_id).not.toBe(calls[0][1].request_id);
  fireEvent.click(screen.getByRole("button", { name: "Reroll" }));
  await waitFor(() => expect(rollEncounterStrike).toHaveBeenCalledTimes(3));
  expect(calls[2][1].request_id).not.toBe(calls[0][1].request_id);
});
it.each([400, 409])(
  "damage roll exposes a definite %s rejection and permits a new request",
  async (status) => {
    vi.mocked(rollEncounterStrike)
      .mockRejectedValueOnce(new AtlasApiError(status, "Review the current attack."))
      .mockImplementation(async (_key, request) => strikeReceipt(request));
    renderStrike();
    fireEvent.click(screen.getByRole("button", { name: "Roll Jaws damage" }));
    await screen.findByText("Review the current attack.");
    expect(screen.queryByRole("button", { name: "Retry" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Roll again" }));
    await screen.findByRole("button", { name: "Reroll" });
    const calls = vi.mocked(rollEncounterStrike).mock.calls;
    expect(calls[1][1].request_id).not.toBe(calls[0][1].request_id);
  },
);
it("retains the displayed damage after a failed reroll and loses it on unmount", async () => {
  vi.mocked(rollEncounterStrike)
    .mockImplementationOnce(async (_key, request) => strikeReceipt(request))
    .mockRejectedValueOnce(new TypeError("Connection interrupted"));
  const mounted = renderStrike();
  fireEvent.click(screen.getByRole("button", { name: "Roll Jaws damage" }));
  await screen.findByRole("button", { name: "Reroll" });
  const shown = screen.getByText(`Damage: ${outcome.total}`).textContent;
  fireEvent.click(screen.getByRole("button", { name: "Reroll" }));
  await screen.findByRole("button", { name: "Roll again" });
  expect(screen.getByText(shown!)).toBeInTheDocument();
  mounted.unmount();
  renderStrike();
  expect(screen.queryByText(shown!)).not.toBeInTheDocument();
});

function renderInitiative(
  onRoll: (request: RollInitiativeRequest) => Promise<InitiativeRollView>,
) {
  render(
    <InitiativeRollModal
      encounterKey="e"
      participants={[participant]}
      onClose={vi.fn()}
      onRoll={onRoll}
      disabled={false}
      onSetTotal={vi.fn()}
    />,
    { wrapper },
  );
}
it("initiative retries the same request, then makes a fresh deliberate reroll", async () => {
  const onRoll = vi
    .fn<(request: RollInitiativeRequest) => Promise<InitiativeRollView>>()
    .mockRejectedValueOnce(new TypeError("Network interrupted"))
    .mockImplementation(async (request) => ({
      request_id: request.request_id,
      encounter_key: "e",
      rolls: [
        { participant_key: "p", display_name: "Goblin", basis: "Perception", outcome },
      ],
    }));
  renderInitiative(onRoll);
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  fireEvent.click(await screen.findByRole("button", { name: "Retry" }));
  await screen.findByRole("button", { name: "Reroll" });
  expect(onRoll.mock.calls[1][0]).toEqual(onRoll.mock.calls[0][0]);
  expect(onRoll.mock.calls[0][0].artifact_generation).toBe("generation");
  fireEvent.click(screen.getByRole("button", { name: "Reroll" }));
  await waitFor(() => expect(onRoll).toHaveBeenCalledTimes(3));
  expect(onRoll.mock.calls[2][0].request_id).not.toBe(
    onRoll.mock.calls[0][0].request_id,
  );
});
it("initiative stops resubmission after a typed revision conflict", async () => {
  const onRoll = vi.fn().mockRejectedValue(
    new AtlasApiError(409, "Changed", {
      code: "encounter_participant_conflict",
      message: "Changed",
    }),
  );
  renderInitiative(onRoll);
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await screen.findByText("Encounter changed. Close and reopen initiative.");
  expect(screen.queryByRole("button", { name: "Retry" })).not.toBeInTheDocument();
  expect(onRoll).toHaveBeenCalledTimes(1);
});

it("attack rolls immediately with selected MAP, makes fresh attempts, and context opens without rolling", async () => {
  const beginContext = vi.fn();
  const renderContext = vi.fn(() => <div>Target questions</div>);
  vi.mocked(rollEncounterStrike)
    .mockRejectedValueOnce(new TypeError("Connection interrupted"))
    .mockImplementation(async (_key, request) => ({
      ...strikeReceipt(request),
      attack: outcome,
      damage: [],
    }));
  render(
    <EncounterStrikeControl
      encounterKey="e"
      generation="generation"
      participant={participant}
      activity={activityFixture()}
      onBeginStrike={beginContext}
      renderStrikeContext={renderContext}
      disabled={false}
    />,
    { wrapper },
  );
  const choice = activityFixture().attack_options[1];
  fireEvent.click(
    screen.getByRole("button", {
      name: `Strike Jaws attack ${choice.modifier >= 0 ? "+" : ""}${choice.modifier}`,
    }),
  );
  expect(screen.getByRole("button", { name: "Target & checks" })).toBeDisabled();
  fireEvent.click(await screen.findByRole("button", { name: "Roll again" }));
  await screen.findByRole("button", { name: "Reroll" });
  const calls = vi.mocked(rollEncounterStrike).mock.calls;
  expect(calls[0][1]).toMatchObject({
    attack_number: choice.attack_number,
    roll_kind: "attack",
  });
  expect(calls[1][1].request_id).not.toBe(calls[0][1].request_id);
  fireEvent.click(screen.getByRole("button", { name: "Target & checks" }));
  expect(beginContext).toHaveBeenCalledWith(
    expect.objectContaining({ rollId: calls[1][1].request_id }),
  );
  expect(screen.getByText("Target questions")).toBeInTheDocument();
  expect(rollEncounterStrike).toHaveBeenCalledTimes(2);
  fireEvent.click(screen.getByRole("button", { name: "Back to roll" }));
  expect(rollEncounterStrike).toHaveBeenCalledTimes(2);
});

function StrikeContextHarness() {
  const strike = useEncounterStrike({
    encounterRef: "e",
    revision: 1n,
    generation: "generation",
    participants: [participant],
    mutations: { pending: false, refreshError: null } as ReturnType<
      typeof useEncounterMutations
    >,
  });
  return (
    <EncounterStrikeControl
      encounterKey="e"
      generation="generation"
      participant={participant}
      activity={activityFixture()}
      disabled={false}
      onBeginStrike={strike.launch}
      renderStrikeContext={strike.renderContext}
    />
  );
}

it("reopens retained attacks in the result view after switching context owners", async () => {
  const options = await getEncounterStrikeOptions("e", {
    participant_key: "p",
    reference: {
      artifact_generation: "generation",
      selection: {
        record_key: "creatures:actor",
        embedded_path: [],
        fields: [],
        passage: null,
        source_fingerprint: null,
      },
    },
  });
  vi.mocked(previewEncounterStrikeContext).mockResolvedValue({
    options,
    targeting_check: null,
    action: { issues: [], confused: false, disruption: null, targeting: null },
  });
  vi.mocked(rollEncounterStrike).mockImplementation(async (_key, request) => ({
    ...strikeReceipt(request),
    attack: outcome,
    attack_modifier: 9,
    damage: [],
  }));
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <StrikeContextHarness />
    </ConfigProvider>,
    { wrapper },
  );
  const choices = activityFixture().attack_options.slice(0, 2);
  const trigger = (index: number) =>
    screen.getByRole("button", {
      name: `Strike Jaws attack ${choices[index].modifier >= 0 ? "+" : ""}${choices[index].modifier}`,
    });
  for (const index of [0, 1]) {
    fireEvent.click(trigger(index));
    const dialog = await screen.findByRole("dialog", { name: "Jaws · Attack" });
    await within(dialog).findByRole("button", { name: "Reroll" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Target & checks" }));
    await within(dialog).findByRole("combobox", { name: "Strike target" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  }
  fireEvent.click(trigger(0));
  const reopened = await screen.findByRole("dialog", { name: "Jaws · Attack" });
  fireEvent.click(within(reopened).getByRole("button", { name: "Target & checks" }));
  await within(reopened).findByRole("combobox", { name: "Strike target" });
  expect(rollEncounterStrike).toHaveBeenCalledTimes(2);
});

it("retains the displayed attack's adjustment hints until another roll succeeds", async () => {
  const base: EncounterStrikeOptionsView = {
    applied: [],
    conditional: [{ source: "hidden", label: "Hidden", reason: "Check detection" }],
    expected_revision: 1n,
    expected_participant_revision: 1n,
    title: "Jaws",
    attacks: activityFixture().attack_options,
    unavailable: null,
    damage_available: true,
    damage_unavailable: null,
  };
  const changed: EncounterStrikeOptionsView = {
    ...base,
    expected_revision: 2n,
    expected_participant_revision: 2n,
    applied: [{ source: "frightened", label: "Frightened", reason: "-2 attack" }],
    conditional: [],
    attacks: base.attacks.map((a) => ({ ...a, modifier: a.modifier - 2 })),
  };
  vi.mocked(getEncounterStrikeOptions)
    .mockResolvedValueOnce(base)
    .mockResolvedValue(changed);
  vi.mocked(rollEncounterStrike)
    .mockImplementationOnce(async (_key, request) => ({
      ...strikeReceipt(request),
      attack: outcome,
      attack_modifier: 9,
      damage: [],
    }))
    .mockRejectedValueOnce(new TypeError("Lost response"))
    .mockImplementationOnce(async (_key, request) => ({
      ...strikeReceipt(request),
      attack: {
        formula: "1d20+7",
        total: 17,
        expression: { kind: "number", value: 17 },
      },
      attack_modifier: 7,
      damage: [],
    }));
  render(
    <EncounterStrikeControl
      encounterKey="e"
      generation="generation"
      participant={participant}
      activity={activityFixture()}
      disabled={false}
      onBeginStrike={vi.fn()}
    />,
    { wrapper },
  );
  const choice = activityFixture().attack_options[0];
  fireEvent.click(
    screen.getByRole("button", {
      name: `Strike Jaws attack ${choice.modifier >= 0 ? "+" : ""}${choice.modifier}`,
    }),
  );
  const dialog = await screen.findByRole("dialog", { name: "Jaws · Attack" });
  await within(dialog).findByRole("button", { name: "Reroll" });
  fireEvent.click(within(dialog).getByRole("button", { name: "Reroll" }));
  await within(dialog).findByRole("button", { name: "Roll again" });
  expect(
    within(dialog).getByRole("status", { name: "1d20+9: 19" }),
  ).toBeInTheDocument();
  expect(
    within(dialog).queryByText("Applied · Frightened · -2 attack"),
  ).not.toBeInTheDocument();
  expect(
    within(dialog).getByText("Context · Hidden · Check detection"),
  ).toBeInTheDocument();
  fireEvent.click(within(dialog).getByRole("button", { name: "Roll again" }));
  await within(dialog).findByRole("status", { name: "1d20+7: 17" });
  expect(
    within(dialog).getByText("Applied · Frightened · -2 attack"),
  ).toBeInTheDocument();
  expect(
    within(dialog).queryByText("Context · Hidden · Check detection"),
  ).not.toBeInTheDocument();
});
