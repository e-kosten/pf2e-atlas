import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type {
  EncounterConditionDefinitionView,
  EncounterParticipantView,
} from "../../generated/atlas";
vi.mock("../../api/atlasApi", () => ({
  previewEncounterCondition: vi.fn().mockResolvedValue({ consequences: [] }),
}));
import { EncounterConditionControls } from "./EncounterConditionControls";

it("edits typed persistent damage while preserving its episode, origin and independent duration", async () => {
  const onUpdate = vi.fn().mockRejectedValue(new Error("Try again"));
  const current: EncounterParticipantView = {
    cast_unavailable_reason: null,
    revision: 0n,
    hp_origin: "explicit",
    variant_origin: "explicit",
    participant_kind: "creature",
    participant_variant: "normal",
    status: "active",
    position: 1n,
    side: "enemy",
    initiative_order: 1n,
    temporary_hp: 0n,
    defeated: false,
    hazard_disabled: false,
    hidden: false,
    note_hint: null,
    persistent_damage_groups: [],
    track_dying: false,
    mortality: "alive",
    base_death_threshold: 4n,
    participant_key: "actor",
    display_name: "Actor",
    conditions: [
      {
        condition_id: 9n,
        condition_key: "conditionitems:example",
        name: "Burning source",
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        immunity_override: false,
        parameters: {
          kind: "persistent_damage",
          formula: "2d6",
          damage_type: "fire",
          multiplier: "double",
          recovery: { kind: "specified", dc: 12n },
          episode_id: "episode-fire",
        },
        origin: {
          application_id: "origin",
          grant_slot: "primary",
          participant_key: null,
          record_key: null,
          activity_id: null,
          artifact_generation: null,
        },
        duration: {
          amount: 2n,
          unit: "rounds",
          timing: { event: "end", participant_key: "actor" },
        },
        created_at: "now",
        updated_at: "now",
      },
    ],
  };
  const definitions: EncounterConditionDefinitionView[] = [
    {
      condition_ref: "conditionitems:example",
      name: "Burning source",
      editor_kind: "persistent_damage",
      scope_label: null,
      supports_extra_action: false,
      supports_duration_instead_of_value: false,
      eligibility: { kind: "available" },
      has_value: false,
      automation_level: "automated",
      applies_to: [],
      categories: [],
    },
  ];
  render(
    <EncounterConditionControls
      encounterRef="encounter"
      onPreviewCondition={vi.fn().mockResolvedValue({ consequences: [] })}
      current={current}
      encounterRevision={14n}
      disabled={false}
      pending={false}
      addDisabled={false}
      conditionDefinitions={definitions}
      damageTypes={[{ value: "fire", label: "Fire" }]}
      participants={[current]}
      onAddCondition={vi.fn()}
      onUpdateCondition={onUpdate}
      onRemoveCondition={vi.fn()}
      onReference={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Edit Burning source" }));
  expect(await screen.findByRole("textbox", { name: "Damage formula" })).toHaveValue(
    "2d6",
  );
  expect(screen.queryByRole("spinbutton", { name: "Condition value" })).toBeNull();
  expect(screen.getByRole("spinbutton", { name: "Flat check DC" })).toHaveValue("12");
  fireEvent.change(screen.getByRole("textbox", { name: "Damage formula" }), {
    target: { value: "3d6" },
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(onUpdate).toHaveBeenCalledWith(
      "actor",
      expect.objectContaining({
        expected_encounter_revision: 14n,
        event_intent: "apply",
        parameters: { ...current.conditions[0].parameters, formula: "3d6" },
        origin: current.conditions[0].origin,
        duration: current.conditions[0].duration,
      }),
    ),
  );
  expect(screen.getByRole("textbox", { name: "Damage formula" })).toHaveValue("3d6");
});

it("uses the backend-selected persistent source and keeps separate source controls inside its group", async () => {
  const condition = (id: bigint, formula: string) => ({
    condition_id: id,
    name: `Source ${id}`,
    scope: { kind: "everyone" },
    extra_action: { kind: "unspecified" },
    immunity_override: false,
    parameters: {
      kind: "persistent_damage",
      formula,
      damage_type: "fire",
      multiplier: "normal",
      recovery: { kind: "standard" },
      episode_id: "episode",
    },
    created_at: "now",
    updated_at: "now",
  });
  const group = {
    episode_id: "episode",
    damage_type: "fire",
    selected_source_id: 1n,
    sources: [
      { condition_id: 1n, formula: "1d6", multiplier: "normal", recovery_dc: 15n },
      { condition_id: 2n, formula: "2d6", multiplier: "normal", recovery_dc: 15n },
    ],
  };
  const current = {
    participant_key: "actor",
    display_name: "Actor",
    conditions: [condition(1n, "1d6"), condition(2n, "2d6")],
    persistent_damage_groups: [group],
  } as EncounterParticipantView;
  const onRemove = vi.fn().mockResolvedValue(true);
  const props = {
    encounterRef: "encounter",
    onPreviewCondition: vi.fn().mockResolvedValue({ consequences: [] }),
    current,
    encounterRevision: 8n,
    disabled: false,
    pending: false,
    addDisabled: false,
    conditionDefinitions: [],
    damageTypes: [{ value: "fire", label: "Fire" }],
    participants: [current],
    onAddCondition: vi.fn(),
    onUpdateCondition: vi.fn(),
    onRemoveCondition: onRemove,
    onReference: vi.fn(),
    onRecoverPersistent: vi.fn(),
  };
  const { rerender } = render(<EncounterConditionControls {...props} />);
  expect(screen.getByRole("button", { name: "Roll Fire recovery" })).toBeVisible();
  expect(screen.queryByText(/null|undefined|Result/)).not.toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: /Persistent Damage · 1d6 Fire · DC 15/ }),
  ).toBeVisible();
  expect(screen.queryByRole("button", { name: "Edit Source 1" })).toBeNull();
  fireEvent.click(
    screen.getByRole("button", { name: /Persistent Damage · 1d6 Fire · DC 15/ }),
  );
  fireEvent.click(await screen.findByRole("button", { name: "Remove Source 2" }));
  expect(onRemove).toHaveBeenCalledWith("actor", 2n, 8n);
  rerender(
    <EncounterConditionControls
      {...props}
      current={{
        ...current,
        persistent_damage_groups: [
          { ...current.persistent_damage_groups[0], selected_source_id: null },
        ],
      }}
    />,
  );
  expect(
    screen.getByRole("button", { name: /Persistent Damage · Fire · Choose source/ }),
  ).toBeVisible();
  rerender(
    <EncounterConditionControls
      {...props}
      current={{
        ...current,
        conditions: [
          {
            ...current.conditions[0],
            source_participant_key: "actor",
            note: "Burning note",
            duration: { amount: 2n, unit: "rounds", timing: null },
          },
        ],
        persistent_damage_groups: [
          { ...group, sources: [{ ...group.sources[0], multiplier: "double" }] },
        ],
      }}
    />,
  );
  expect(screen.getByText("Persistent Damage · 1d6 ×2 Fire · DC 15")).toBeVisible();
  expect(screen.getByText("Source: Actor")).toBeVisible();
  expect(screen.getByText("Burning note")).toBeVisible();
  expect(screen.getByText("2 rounds · timing not configured")).toBeVisible();
  expect(screen.getByRole("button", { name: "Edit Source 1" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Remove Source 1" }));
  expect(onRemove).toHaveBeenLastCalledWith("actor", 1n, 8n);
  expect(
    screen.queryByRole("dialog", { name: /Remove|Confirm/ }),
  ).not.toBeInTheDocument();
  // A successful roll clears every source while keeping the same popover mounted.
  props.onRecoverPersistent.mockImplementation(async () => {
    rerender(
      <EncounterConditionControls
        {...props}
        encounterRevision={9n}
        current={{ ...current, conditions: [], persistent_damage_groups: [] }}
      />,
    );
    return {
      encounter_key: "encounter",
      request_id: "recovery",
      episode_id: "episode",
      damage_type: "fire",
      dc: 15n,
      roll: null,
      table_result: 18n,
      success: true,
    };
  });
  rerender(<EncounterConditionControls {...props} />);
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire recovery" }));
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await screen.findByText("Recovered · DC 15");
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire recovery" }));
  await waitFor(() =>
    expect(
      screen.queryByRole("button", { name: "Roll Fire recovery" }),
    ).not.toBeInTheDocument(),
  );
});
