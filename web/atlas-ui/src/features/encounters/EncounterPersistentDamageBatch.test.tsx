import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type {
  EncounterParticipantView,
  PersistentDamageResultView,
} from "../../generated/atlas";
import { EncounterPersistentDamageBatch } from "./EncounterPersistentDamageBatch";
const api = vi.hoisted(() => ({ prepareEncounterPersistentDamage: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
beforeEach(() => {
  api.prepareEncounterPersistentDamage.mockImplementation(async (_ref, request) => ({
    complete: request.groups.length === 2,
    recovery: null,
    damage: {
      encounter_key: "encounter",
      request_id: request.request_id,
      total: request.groups.reduce(
        (sum: bigint, g: { damage: { kind: string; amount?: bigint } }) =>
          sum + (g.damage.amount ?? 0n),
        0n,
      ),
      hardness_absorbed: 0n,
      groups: request.groups.map(
        (g: { episode_id: string; damage: { kind: string; amount?: bigint } }) => ({
          episode_id: g.episode_id,
          damage_type: g.episode_id,
          source_condition_id: g.episode_id === "fire" ? 7n : 9n,
          roll: null,
          amount: g.damage.amount ?? 0n,
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
  track_dying: true,
  mortality: "alive",
  base_death_threshold: 4n,
  knockout_options: [],
  effective_max_hp: { state: "value", value: 20, adjustment: null },
  persistent_damage_groups: [
    {
      episode_id: "fire",
      damage_type: "fire",
      selected_source_id: 7n,
      sources: [
        {
          condition_id: 7n,
          formula: "1d6",
          roll_preview: { kind: "dice", sides: [6] },
          multiplier: "normal",
          recovery_dc: 15n,
        },
      ],
    },
    {
      episode_id: "acid",
      damage_type: "acid",
      selected_source_id: 9n,
      sources: [
        {
          condition_id: 8n,
          formula: "1d8",
          roll_preview: { kind: "dice", sides: [8] },
          multiplier: "normal",
          recovery_dc: 15n,
        },
        {
          condition_id: 9n,
          formula: "1d4",
          roll_preview: { kind: "dice", sides: [4] },
          multiplier: "normal",
          recovery_dc: 15n,
        },
      ],
    },
  ],
};
const result: PersistentDamageResultView = {
  encounter_key: "encounter",
  request_id: "receipt",
  total: 8n,
  hardness_absorbed: 0n,
  groups: [
    {
      episode_id: "fire",
      damage_type: "fire",
      source_condition_id: 7n,
      roll: null,
      amount: 5n,
      table_resolved: true,
    },
    {
      episode_id: "acid",
      damage_type: "acid",
      source_condition_id: 9n,
      roll: null,
      amount: 3n,
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
        encounterRevision={6n}
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
  const submit = await screen.findByRole("button", { name: "Apply 8 damage" });
  fireEvent.focus(submit);
  expect(await screen.findByRole("tooltip")).toHaveTextContent(
    "Effects triggered by taking damage, such as increasing Dying, happen once.",
  );
  fireEvent.blur(submit);
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
  fireEvent.click(await screen.findByRole("button", { name: "Apply 8 damage" }));
  await screen.findByText("Connection lost");
  expect(screen.getByRole("spinbutton", { name: "Fire damage amount" })).toHaveValue(
    "5",
  );
  fireEvent.click(await screen.findByRole("button", { name: "Apply 8 damage" }));
  await waitFor(() => expect(onApply).toHaveBeenCalledTimes(2));
  expect(onApply.mock.calls[0][0]).toEqual(onApply.mock.calls[1][0]);
  expect(onApply).toHaveBeenLastCalledWith({
    boundary_key: null,
    expected_encounter_revision: 6n,
    participant_key: "actor",
    artifact_generation: "generation",
    recoveries: [],
    groups: [
      {
        episode_id: "fire",
        damage: { kind: "table_result", amount: 5n },
      },
      { episode_id: "acid", damage: { kind: "table_result", amount: 3n } },
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
  expect(await screen.findByText("Fire: 5")).toBeVisible();
  expect(screen.getByText("Acid: 3")).toBeVisible();
  expect(screen.queryByRole("button", { name: /^Apply (?:\d+ )?damage$/ })).toBeNull();
});

it("uses table results for the whole form when one protection requires a table ruling", async () => {
  const onApply = vi.fn().mockResolvedValue(result);
  render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      onRecover={vi.fn()}
      participant={participant}
      encounterRevision={6n}
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
          { episode_id: "fire", damage: { kind: "table_result", amount: 5n } },
          { episode_id: "acid", damage: { kind: "table_result", amount: 3n } },
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
      encounterRevision={6n}
      generation="generation"
      damageApplied
      savedResult={{ ...result, total: 3n, hardness_absorbed: 5n }}
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
  expect(screen.getByText("Fire: 5")).toBeVisible();
  expect(screen.queryByText("Fire: 5 damage applied")).not.toBeInTheDocument();
});

it("shows immunity before damage and retains the recorded reason when reference advice changes", async () => {
  const source = {
    ...participant,
    persistent_damage_groups: [participant.persistent_damage_groups[0]],
  };
  const protection = { immune: true, weakness: 0n, resistance: 0n };
  const onApply = vi.fn().mockResolvedValue({
    encounter_key: "encounter",
    request_id: "immune",
    groups: [
      {
        episode_id: "fire",
        damage_type: "fire",
        source_condition_id: 7n,
        roll: null,
        amount: 0n,
        table_resolved: false,
        protection,
      },
    ],
    total: 0n,
    hardness_absorbed: 0n,
  });
  const props = {
    participant: source,
    encounterRevision: 6n,
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
    damage: {
      encounter_key: "encounter",
      request_id: request.request_id,
      groups: [
        {
          episode_id: "fire",
          damage_type: "fire",
          source_condition_id: 7n,
          amount: 0n,
          roll: null,
          table_resolved: false,
          protection,
        },
      ],
      total: 0n,
      hardness_absorbed: 0n,
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
        fire: { kind: "known", protection: { ...protection, immune: false } },
      }}
    />,
  );
  expect(screen.getByText("Immune to fire")).toBeVisible();
});

it("explains unresolved protections and starts with an editable table amount", async () => {
  const onApply = vi.fn().mockResolvedValue({ ...result, total: 3n });
  render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      onRecover={vi.fn()}
      participant={{
        ...participant,
        persistent_damage_groups: [participant.persistent_damage_groups[0]],
      }}
      encounterRevision={6n}
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
    damage: { ...result, total: 3n },
  });
  fireEvent.click(await screen.findByRole("button", { name: "Apply 3 damage" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        groups: [
          {
            episode_id: "fire",
            damage: { kind: "table_result", amount: 3n },
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
      recovery: request.recovery
        ? {
            encounter_key: "encounter",
            request_id: request.request_id,
            episode_id: request.recovery.episode_id,
            damage_type: "fire",
            dc: 0n,
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
        encounterRevision={6n}
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
  fireEvent.click(screen.getByText("Table results"));
  fireEvent.change(
    await screen.findByRole("spinbutton", { name: "Fire damage amount" }),
    { target: { value: "5" } },
  );
  fireEvent.click(await screen.findByRole("button", { name: "Roll Fire recovery" }));
  fireEvent.click(await screen.findByRole("button", { name: /^Roll$/ }));
  await waitFor(() =>
    expect(screen.getAllByText("Clears after Apply").length).toBeGreaterThan(0),
  );
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
      { kind: "prepared", episode_id: "fire", request_id: expect.any(String) },
    ],
    groups: [
      { episode_id: "fire", damage: { kind: "table_result", amount: 5n } },
      { episode_id: "acid", damage: { kind: "table_result", amount: 3n } },
    ],
  });
});

it("shows the backend's refreshed protections and amount while retaining prepared dice identity", async () => {
  api.prepareEncounterPersistentDamage.mockImplementation(async (_ref, request) => {
    const refreshed = request.groups[0]?.damage.kind === "prepared";
    return {
      complete: request.groups.length === 1,
      recovery: null,
      damage: {
        encounter_key: "encounter",
        request_id: request.request_id,
        total: refreshed ? 0n : 5n,
        hardness_absorbed: 0n,
        groups: request.groups.length
          ? [
              {
                episode_id: "fire",
                damage_type: "fire",
                source_condition_id: 7n,
                amount: refreshed ? 0n : 5n,
                roll: null,
                table_resolved: false,
                protection: { immune: refreshed, weakness: 0n, resistance: 0n },
              },
            ]
          : [],
      },
    };
  });
  render(
    <EncounterPersistentDamageBatch
      encounterRef="encounter"
      participant={{
        ...participant,
        persistent_damage_groups: [participant.persistent_damage_groups[0]],
      }}
      encounterRevision={6n}
      generation="generation"
      damageApplied={false}
      damageTypes={[{ value: "fire", label: "Fire" }]}
      disabled={false}
      pending={false}
      onApply={vi.fn()}
      onRecover={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire damage" }));
  expect(await screen.findByText("Immune to fire")).toBeVisible();
  expect(screen.getByRole("button", { name: "Apply 0 damage" })).toBeEnabled();
  expect(screen.queryByText("5 damage")).toBeNull();
});
