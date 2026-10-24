import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type {
  EncounterParticipantView,
  PersistentDamageResultView,
} from "../../generated/atlas";
import { EncounterPersistentDamageBatch } from "./EncounterPersistentDamageBatch";
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
        { condition_id: 7n, formula: "1d6", multiplier: "normal", recovery_dc: 15n },
      ],
    },
    {
      episode_id: "acid",
      damage_type: "acid",
      selected_source_id: 9n,
      sources: [
        { condition_id: 8n, formula: "1d8", multiplier: "normal", recovery_dc: 15n },
        { condition_id: 9n, formula: "1d4", multiplier: "normal", recovery_dc: 15n },
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
  fireEvent.click(screen.getByRole("button", { name: /^Apply (?:\d+ )?damage$/ }));
  await screen.findByText("Connection lost");
  expect(screen.getByRole("spinbutton", { name: "Fire damage amount" })).toHaveValue(
    "5",
  );
  fireEvent.click(screen.getByRole("button", { name: /^Apply (?:\d+ )?damage$/ }));
  await waitFor(() => expect(onApply).toHaveBeenCalledTimes(2));
  expect(onApply.mock.calls[0][0]).toEqual(onApply.mock.calls[1][0]);
  expect(onApply).toHaveBeenLastCalledWith({
    boundary_key: null,
    expected_encounter_revision: 6n,
    participant_key: "actor",
    artifact_generation: "generation",
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
  fireEvent.click(screen.getByRole("button", { name: "Apply damage" }));
  await waitFor(() => expect(screen.getByText("Enter a whole number.")).toBeVisible());
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
      {...props}
      resolutions={{ fire: { kind: "known", protection } }}
    />,
  );
  expect(screen.getByText("Immune to fire")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Roll & apply damage" }));
  await screen.findByText("Total: 0 damage applied");
  rerender(
    <EncounterPersistentDamageBatch
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
  fireEvent.click(screen.getByRole("button", { name: /^Apply (?:\d+ )?damage$/ }));
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
