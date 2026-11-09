import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type {
  EncounterParticipantView,
  PersistentDamageResultView,
} from "../../generated/atlas";
import { EncounterPersistentDamageBatch } from "./EncounterPersistentDamageBatch";
const draftContext = {
  encounter_key: "encounter",
  participant_key: "actor",
  boundary_key: null,
  round_number: 1,
  record_key: null,
  artifact_generation: "generation",
};
const fireRoll = {
  formula: "1d6",
  total: 5,
  expression: {
    kind: "dice" as const,
    dice: [{ sides: 6, value: 5, kept: true }],
    total: 5,
  },
};
const fireDraft = { context: draftContext, source_condition_id: 7, roll: fireRoll };
const issuedFireDraft = { episode_id: "fire", draft: fireDraft };
const api = vi.hoisted(() => ({ prepareEncounterPersistentDamage: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
afterEach(() => vi.restoreAllMocks());
beforeEach(() => {
  vi.spyOn(window, "matchMedia").mockImplementation((query) => ({
    media: query,
    matches: query.includes("prefers-reduced-motion"),
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
  api.prepareEncounterPersistentDamage.mockImplementation(async (_ref, request) => ({
    complete: request.groups.length === 2,
    recovery: null,
    damage_draft: null,
    recovery_draft: null,
    damage: {
      encounter_key: "encounter",
      request_id: request.request_id,
      total: request.groups.reduce(
        (sum: number, g: { damage: { kind: string; amount?: number } }) =>
          sum + (g.damage.amount ?? 0),
        0,
      ),
      hardness_absorbed: 0,
      groups: request.groups.map(
        (g: { episode_id: string; damage: { kind: string; amount?: number } }) => ({
          episode_id: g.episode_id,
          damage_type: g.episode_id,
          source_condition_id: g.episode_id === "fire" ? 7 : 9,
          roll: null,
          amount: g.damage.amount ?? 0,
          table_resolved: g.damage.kind === "table_result",
        }),
      ),
    },
  }));
});
const participant: EncounterParticipantView = {
  participant_key: "actor",
  display_name: "Actor",
  participant_kind: "creature",
  revision: 1,
  hp_origin: "explicit",
  variant_origin: "explicit",
  participant_variant: "normal",
  status: "active",
  position: 1,
  side: "enemy",
  initiative_order: 1,
  temporary_hp: 0,
  defeated: false,
  hazard_disabled: false,
  cast_unavailable_reason: null,
  hidden: false,
  note_hint: null,
  conditions: [],
  track_dying: true,
  mortality: "alive",
  base_death_threshold: 4,
  manual_trackers: [],
  knockout_options: [],
  effective_max_hp: { state: "value", value: 20, adjustment: null },
  persistent_damage_groups: [
    {
      episode_id: "fire",
      damage_type: "fire",
      selected_source_id: 7,
      sources: [
        {
          condition_id: 7,
          formula: "1d6",
          roll_preview: { kind: "dice", sides: [6] },
          multiplier: "normal",
          recovery_dc: 15,
        },
      ],
    },
    {
      episode_id: "acid",
      damage_type: "acid",
      selected_source_id: 9,
      sources: [
        {
          condition_id: 8,
          formula: "1d8",
          roll_preview: { kind: "dice", sides: [8] },
          multiplier: "normal",
          recovery_dc: 15,
        },
        {
          condition_id: 9,
          formula: "1d4",
          roll_preview: { kind: "dice", sides: [4] },
          multiplier: "normal",
          recovery_dc: 15,
        },
      ],
    },
  ],
};
const result: PersistentDamageResultView = {
  encounter_key: "encounter",
  request_id: "receipt",
  total: 8,
  hardness_absorbed: 0,
  groups: [
    {
      episode_id: "fire",
      damage_type: "fire",
      source_condition_id: 7,
      roll: null,
      amount: 5,
      table_resolved: true,
    },
    {
      episode_id: "acid",
      damage_type: "acid",
      source_condition_id: 9,
      roll: null,
      amount: 3,
      table_resolved: true,
    },
  ],
};
it("submits all damage types together and retains table totals across an uncertain retry", async () => {
  const onApply = vi
    .fn()
    .mockRejectedValueOnce(new Error("Connection lost"))
    .mockResolvedValue(result);
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentDamageBatch
        encounterRef="encounter"
        onRecover={vi.fn()}
        participant={participant}
        encounterRevision={6}
        generation="generation"
        damageApplied={false}
        damageTypes={[
          { value: "fire", label: "Fire" },
          { value: "acid", label: "Acid" },
        ]}
        disabled={false}
        pending={false}
        onApply={onApply}
      />
    </ConfigProvider>,
  );
  expect(screen.getByLabelText("Unrolled d6")).toBeVisible();
  expect(screen.getByLabelText("Unrolled d4")).toBeVisible();
  expect(screen.queryByLabelText("Unrolled d8")).toBeNull();
  expect(screen.queryByLabelText("Rolling d6")).toBeNull();
  expect(onApply).not.toHaveBeenCalled();
  fireEvent.click(
    within(screen.getByLabelText("Persistent damage resolution")).getByText(
      "Table results",
    ),
  );
  fireEvent.change(
    await screen.findByRole("spinbutton", { name: "Fire damage amount" }),
    { target: { value: "5" } },
  );
  fireEvent.change(screen.getByRole("spinbutton", { name: "Acid damage amount" }), {
    target: { value: "3" },
  });
  expect(screen.getAllByText("Table results")).toHaveLength(1);
  expect(await screen.findByRole("button", { name: "Apply 8 damage" })).toBeEnabled();
  expect(screen.getByText("Damage applies as one event.")).toBeVisible();
  fireEvent.click(screen.getByText("Roll in Atlas"));
  await waitFor(() =>
    expect(screen.queryByRole("spinbutton", { name: "Fire damage amount" })).toBeNull(),
  );
  fireEvent.click(screen.getByText("Table results"));
  expect(
    await screen.findByRole("spinbutton", { name: "Fire damage amount" }),
  ).toHaveValue("5");
  expect(screen.getByRole("spinbutton", { name: "Acid damage amount" })).toHaveValue(
    "3",
  );
  expect(screen.queryByRole("combobox", { name: "Acid damage source" })).toBeNull();
  const amount = screen.getByRole("spinbutton", { name: "Fire damage amount" });
  fireEvent.focus(amount);
  expect(await screen.findByRole("tooltip")).toHaveTextContent("After IWR");
  fireEvent.blur(amount);
  fireEvent.click(await screen.findByRole("button", { name: "Apply 8 damage" }));
  await screen.findByText("Could not apply persistent damage.");
  expect(screen.getByRole("spinbutton", { name: "Fire damage amount" })).toHaveValue(
    "5",
  );
  fireEvent.click(await screen.findByRole("button", { name: "Apply 8 damage" }));
  await waitFor(() => expect(onApply).toHaveBeenCalledTimes(2));
  expect(onApply.mock.calls[0][0]).toEqual(onApply.mock.calls[1][0]);
  expect(onApply).toHaveBeenLastCalledWith({
    boundary_key: null,
    expected_encounter_revision: 6,
    participant_key: "actor",
    artifact_generation: "generation",
    recoveries: [],
    groups: [
      {
        episode_id: "fire",
        damage: { kind: "table_result", amount: 5 },
      },
      { episode_id: "acid", damage: { kind: "table_result", amount: 3 } },
    ],
    context: {
      lethality: null,
      critical: null,
      enemy_source: null,
      responsible_participant_key: null,
      base_death_threshold: null,
      keep_current_initiative: null,
      attack_or_spell_damage: null,
      natural_rest_healing: null,
      keep_upright: null,
    },
  });
  expect(await screen.findByText("Damage: 5 applied")).toBeVisible();
  expect(screen.getByText("Damage: 3 applied")).toBeVisible();
  expect(screen.queryByRole("button", { name: /^Apply (?:\d+ )?damage$/ })).toBeNull();
});

it("retains an automatic recovery result after its effect clears without inventing dice", async () => {
  const onRecover = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentDamageBatch
        encounterRef="encounter"
        participant={{ ...participant, persistent_damage_groups: [] }}
        displayGroups={[participant.persistent_damage_groups[0]]}
        encounterRevision={7}
        generation="generation"
        damageApplied={true}
        savedResult={result}
        recoveryResults={[
          {
            encounter_key: "encounter",
            request_id: "automatic-recovery",
            episode_id: "fire",
            damage_type: "fire",
            dc: 0,
            success: true,
            roll: null,
          },
        ]}
        damageTypes={[{ value: "fire", label: "Fire" }]}
        disabled={false}
        pending={false}
        onApply={vi.fn()}
        onRecover={onRecover}
      />
    </ConfigProvider>,
  );
  expect(screen.getByText("Effect cleared")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "View Fire recovery" }));
  await waitFor(() => expect(screen.getByText("Recovered · DC 0")).toBeVisible());
  expect(screen.queryByRole("button", { name: "Roll" })).toBeNull();
  expect(screen.queryByLabelText(/(?:Unrolled|Rolling) d/)).toBeNull();
  expect(onRecover).not.toHaveBeenCalled();
});

it("uses table results for the whole form when one protection requires a table ruling", async () => {
  const onApply = vi.fn().mockResolvedValue(result);
  render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      onRecover={vi.fn()}
      participant={participant}
      encounterRevision={6}
      generation="generation"
      damageApplied={false}
      damageTypes={[
        { value: "fire", label: "Fire" },
        { value: "acid", label: "Acid" },
      ]}
      disabled={false}
      pending={false}
      onApply={onApply}
      resolutions={{ fire: { kind: "table", reason: "Resolve silver at the table." } }}
    />,
  );
  expect(screen.getByText("Resolve silver at the table.")).toBeVisible();
  expect(screen.getByRole("radio", { name: "Roll in Atlas" })).toBeDisabled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Fire damage amount" }), {
    target: { value: "5" },
  });
  expect(screen.getByRole("button", { name: "Apply damage" })).toBeDisabled();
  expect(onApply).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Acid damage amount" }), {
    target: { value: "3" },
  });
  fireEvent.click(await screen.findByRole("button", { name: "Apply 8 damage" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        groups: [
          { episode_id: "fire", damage: { kind: "table_result", amount: 5 } },
          { episode_id: "acid", damage: { kind: "table_result", amount: 3 } },
        ],
      }),
    ),
  );
});

it("shows the server total after shared Hardness without claiming per-type totals were all applied", () => {
  render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      onRecover={vi.fn()}
      participant={participant}
      encounterRevision={6}
      generation="generation"
      damageApplied
      savedResult={{ ...result, total: 3, hardness_absorbed: 5 }}
      damageTypes={[
        { value: "fire", label: "Fire" },
        { value: "acid", label: "Acid" },
      ]}
      disabled={false}
      pending={false}
      onApply={vi.fn()}
    />,
  );
  expect(screen.getByText(/Total: 3 damage applied/)).toBeVisible();
  expect(screen.getByText(/Hardness absorbed 5/)).toBeVisible();
  expect(screen.getByText("Damage: 5 applied")).toBeVisible();
  expect(screen.queryByText("Fire: 5 damage applied")).not.toBeInTheDocument();
});

it("keeps current protections as editing advice without attributing them to saved table damage", async () => {
  const onApply = vi.fn().mockResolvedValue(result);
  const props = {
    encounterRef: "encounter",
    participant,
    encounterRevision: 6,
    generation: "generation",
    damageApplied: false,
    damageTypes: [
      { value: "fire", label: "Fire" },
      { value: "acid", label: "Acid" },
    ],
    disabled: false,
    pending: false,
    onApply,
    onRecover: vi.fn(),
  };
  const { rerender } = render(
    <EncounterPersistentDamageBatch
      {...props}
      resolutions={{
        fire: {
          kind: "known",
          protection: { immune: true, weakness: 0, resistance: 0 },
        },
        acid: {
          kind: "known",
          protection: { immune: false, weakness: 3, resistance: 0 },
        },
      }}
    />,
  );
  fireEvent.click(screen.getByText("Table results"));
  expect(screen.getByText("Immune to fire")).toBeVisible();
  expect(screen.getByText("Weakness 3")).toBeVisible();
  fireEvent.change(
    await screen.findByRole("spinbutton", { name: "Fire damage amount" }),
    { target: { value: "5" } },
  );
  fireEvent.change(screen.getByRole("spinbutton", { name: "Acid damage amount" }), {
    target: { value: "3" },
  });
  fireEvent.click(await screen.findByRole("button", { name: "Apply 8 damage" }));
  expect(await screen.findByText("Damage: 5 applied")).toBeVisible();
  expect(screen.getByText("Damage: 3 applied")).toBeVisible();
  expect(screen.queryByText("Immune to fire")).toBeNull();
  expect(screen.queryByText("Weakness 3")).toBeNull();
  rerender(
    <EncounterPersistentDamageBatch
      {...props}
      damageApplied
      savedResult={result}
      resolutions={{
        fire: {
          kind: "known",
          protection: { immune: false, weakness: 0, resistance: 9 },
        },
        acid: {
          kind: "known",
          protection: { immune: true, weakness: 0, resistance: 0 },
        },
      }}
    />,
  );
  expect(screen.getByText("Total: 8 damage applied")).toBeVisible();
  expect(screen.queryByText("Resistance 9")).toBeNull();
  expect(screen.queryByText("Immune to acid")).toBeNull();
});

it("shows immunity before damage and retains the recorded reason when reference advice changes", async () => {
  const source = {
    ...participant,
    persistent_damage_groups: [participant.persistent_damage_groups[0]],
  };
  const protection = { immune: true, weakness: 0, resistance: 0 };
  const onApply = vi.fn().mockResolvedValue({
    encounter_key: "encounter",
    request_id: "immune",
    groups: [
      {
        episode_id: "fire",
        damage_type: "fire",
        source_condition_id: 7,
        roll: null,
        amount: 0,
        table_resolved: false,
        protection,
      },
    ],
    total: 0,
    hardness_absorbed: 0,
  });
  const props = {
    participant: source,
    encounterRevision: 6,
    generation: "generation",
    damageApplied: false,
    damageTypes: [{ value: "fire", label: "Fire" }],
    disabled: false,
    pending: false,
    onApply,
  };
  const { rerender } = render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      onRecover={vi.fn()}
      {...props}
      resolutions={{ fire: { kind: "known", protection } }}
    />,
  );
  expect(screen.getByText("Immune to fire")).toBeVisible();
  api.prepareEncounterPersistentDamage.mockImplementation(async (_ref, request) => ({
    complete: true,
    recovery: null,
    damage_draft: request.groups.some(
      ({ damage }: { damage: { kind: string } }) => damage.kind === "roll",
    )
      ? issuedFireDraft
      : null,
    recovery_draft: null,
    damage: {
      encounter_key: "encounter",
      request_id: request.request_id,
      groups: [
        {
          episode_id: "fire",
          damage_type: "fire",
          source_condition_id: 7,
          amount: 0,
          roll: fireRoll,
          table_resolved: false,
          protection,
        },
      ],
      total: 0,
      hardness_absorbed: 0,
    },
  }));
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire damage" }));
  fireEvent.click(await screen.findByRole("button", { name: "Apply 0 damage" }));
  await screen.findByText("Total: 0 damage applied");
  rerender(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      onRecover={vi.fn()}
      {...props}
      resolutions={{
        fire: {
          kind: "known",
          protection: { immune: false, weakness: 9, resistance: 0 },
        },
      }}
    />,
  );
  expect(screen.getByText("Immune to fire")).toBeVisible();
  expect(screen.queryByText("Weakness 9")).toBeNull();
});

it("explains unresolved protections and starts with an editable table amount", async () => {
  const onApply = vi.fn().mockResolvedValue({ ...result, total: 3 });
  render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      onRecover={vi.fn()}
      participant={{
        ...participant,
        persistent_damage_groups: [participant.persistent_damage_groups[0]],
      }}
      encounterRevision={6}
      generation="generation"
      damageApplied={false}
      damageTypes={[{ value: "fire", label: "Fire" }]}
      disabled={false}
      pending={false}
      onApply={onApply}
      resolutions={{ fire: { kind: "table", reason: "Resolve silver at the table." } }}
    />,
  );
  expect(screen.getByText("Resolve silver at the table.")).toBeVisible();
  const amount = await screen.findByRole("spinbutton", { name: "Fire damage amount" });
  fireEvent.change(amount, { target: { value: "3" } });
  api.prepareEncounterPersistentDamage.mockResolvedValue({
    complete: true,
    recovery: null,
    damage_draft: null,
    recovery_draft: null,
    damage: { ...result, total: 3 },
  });
  fireEvent.click(await screen.findByRole("button", { name: "Apply 3 damage" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        groups: [
          {
            episode_id: "fire",
            damage: { kind: "table_result", amount: 3 },
          },
        ],
      }),
    ),
  );
});

it("prepares recovery after one type while another amount is unresolved, then applies both once", async () => {
  const original = api.prepareEncounterPersistentDamage.getMockImplementation()!;
  api.prepareEncounterPersistentDamage.mockImplementation(async (ref, request) => {
    const preview = await original(ref, request);
    return {
      ...preview,
      recovery_draft: request.recovery
        ? {
            context: draftContext,
            source_condition_id: 7,
            dc: 0,
            dc_override: 0,
            roll: null,
          }
        : null,
      recovery: request.recovery
        ? {
            encounter_key: "encounter",
            request_id: request.request_id,
            episode_id: request.recovery.episode_id,
            damage_type: "fire",
            dc: 0,
            roll: null,
            success: true,
          }
        : null,
    };
  });
  const onApply = vi.fn().mockResolvedValue(result);
  const onRecover = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentDamageBatch
        encounterRef="encounter"
        participant={participant}
        encounterRevision={6}
        generation="generation"
        damageApplied={false}
        damageTypes={[
          { value: "fire", label: "Fire" },
          { value: "acid", label: "Acid" },
        ]}
        disabled={false}
        pending={false}
        onApply={onApply}
        onRecover={onRecover}
      />
    </ConfigProvider>,
  );
  expect(await screen.findByText(/Fire, Acid not rolled$/)).toBeVisible();
  fireEvent.click(screen.getByText("Table results"));
  expect(await screen.findByText(/Fire, Acid not entered$/)).toBeVisible();
  fireEvent.change(
    await screen.findByRole("spinbutton", { name: "Fire damage amount" }),
    { target: { value: "5" } },
  );
  await waitFor(() =>
    expect(
      screen.getByText(
        (_text, node) =>
          node?.tagName === "SPAN" &&
          node.textContent === "Total: 5 damage · Acid not entered",
      ),
    ).toBeVisible(),
  );
  fireEvent.click(await screen.findByRole("button", { name: "Roll Fire recovery" }));
  fireEvent.click(await screen.findByRole("button", { name: /^Roll$/ }));
  await waitFor(() =>
    expect(screen.getAllByText("Clears after Apply").length).toBeGreaterThan(0),
  );
  expect(screen.getByText("Clear Fire")).toBeVisible();
  expect(onApply).not.toHaveBeenCalled();
  expect(onRecover).not.toHaveBeenCalled();
  expect(screen.getByRole("button", { name: "Apply damage" })).toBeDisabled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Acid damage amount" }), {
    target: { value: "3" },
  });
  fireEvent.click(await screen.findByRole("button", { name: "Apply 8 damage" }));
  await waitFor(() => expect(onApply).toHaveBeenCalledOnce());
  expect(onApply.mock.calls[0][0]).toMatchObject({
    recoveries: [
      {
        kind: "prepared",
        episode_id: "fire",
        draft: { context: draftContext, source_condition_id: 7, dc: 0 },
      },
    ],
    groups: [
      { episode_id: "fire", damage: { kind: "table_result", amount: 5 } },
      { episode_id: "acid", damage: { kind: "table_result", amount: 3 } },
    ],
  });
});

it("shows the backend's refreshed protections and amount while retaining prepared dice identity", async () => {
  api.prepareEncounterPersistentDamage.mockImplementation(async (_ref, request) => {
    const refreshed = request.groups[0]?.damage.kind === "prepared";
    return {
      complete: request.groups.length === 1,
      recovery: null,
      damage_draft: request.groups[0]?.damage.kind === "roll" ? issuedFireDraft : null,
      recovery_draft: null,
      damage: {
        encounter_key: "encounter",
        request_id: request.request_id,
        total: refreshed ? 0 : 5,
        hardness_absorbed: 0,
        groups: request.groups.length
          ? [
              {
                episode_id: "fire",
                damage_type: "fire",
                source_condition_id: 7,
                amount: refreshed ? 0 : 5,
                roll: fireRoll,
                table_resolved: false,
                protection: { immune: refreshed, weakness: 0, resistance: 0 },
              },
            ]
          : [],
      },
    };
  });
  const onApply = vi.fn().mockResolvedValue({ ...result, total: 0 });
  render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      participant={{
        ...participant,
        persistent_damage_groups: [participant.persistent_damage_groups[0]],
      }}
      encounterRevision={6}
      generation="generation"
      damageApplied={false}
      damageTypes={[{ value: "fire", label: "Fire" }]}
      disabled={false}
      pending={false}
      onApply={onApply}
      onRecover={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire damage" }));
  expect(await screen.findByText("Immune to fire")).toBeVisible();
  expect(screen.getByRole("button", { name: "Apply 0 damage" })).toBeEnabled();
  expect(screen.queryByText("5 damage")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Apply 0 damage" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        groups: [
          { episode_id: "fire", damage: { kind: "prepared", draft: fireDraft } },
        ],
      }),
    ),
  );
});

it.each([null, { ...issuedFireDraft, episode_id: "acid" }])(
  "does not stage damage when the issued draft is missing or for another episode",
  async (damageDraft) => {
    const original = api.prepareEncounterPersistentDamage.getMockImplementation()!;
    api.prepareEncounterPersistentDamage.mockImplementation(async (ref, request) => {
      const preview = await original(ref, request);
      if (request.groups[0]?.damage.kind !== "roll") return preview;
      return {
        ...preview,
        complete: true,
        damage_draft: damageDraft,
        damage: {
          ...preview.damage,
          total: fireRoll.total,
          groups: [
            { ...preview.damage.groups[0], roll: fireRoll, amount: fireRoll.total },
          ],
        },
      };
    });
    const onApply = vi.fn();
    render(
      <EncounterPersistentDamageBatch
        encounterRef="encounter"
        participant={{
          ...participant,
          persistent_damage_groups: [participant.persistent_damage_groups[0]],
        }}
        encounterRevision={6}
        generation="generation"
        damageApplied={false}
        damageTypes={[{ value: "fire", label: "Fire" }]}
        disabled={false}
        pending={false}
        onApply={onApply}
        onRecover={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Roll Fire damage" }));
    expect(await screen.findByText("Couldn’t confirm the roll.")).toBeVisible();
    expect(screen.getByRole("button", { name: "Roll Fire damage" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Apply damage" })).toBeDisabled();
    expect(onApply).not.toHaveBeenCalled();
  },
);

it("keeps post-Apply recovery and Clear usable with the refreshed revision", async () => {
  const onRecover = vi.fn().mockResolvedValue({
    encounter_key: "encounter",
    request_id: "clear",
    episode_id: "fire",
    damage_type: "fire",
    dc: null,
    roll: null,
    success: true,
  });
  const props = {
    encounterRef: "encounter",
    participant,
    encounterRevision: 6,
    generation: "generation",
    damageApplied: false,
    damageTypes: [
      { value: "fire", label: "Fire" },
      { value: "acid", label: "Acid" },
    ],
    disabled: false,
    pending: false,
    onApply: vi.fn(),
    onRecover,
  };
  const { rerender } = render(<EncounterPersistentDamageBatch {...props} />);
  rerender(
    <EncounterPersistentDamageBatch
      {...props}
      encounterRevision={8}
      damageApplied
      savedResult={result}
      episodes={[
        {
          episode_id: "fire",
          damage_resolution: { kind: "table", reason: "At table" },
          end_turn_recovery_available: true,
          end_turn_recovery_unavailable_reason: null,
          end_turn_roll_available: true,
          end_turn_roll_unavailable_reason: null,
        },
      ]}
    />,
  );
  expect(screen.getByRole("button", { name: "Roll Fire recovery" })).toBeEnabled();
  const clear = screen.getByRole("button", { name: "Clear fire effect" });
  expect(clear).toBeEnabled();
  fireEvent.click(clear);
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith(
      expect.objectContaining({
        resolution: "clear",
        episode_id: "fire",
        expected_encounter_revision: 8,
      }),
    ),
  );
});
