import {
  encounterParticipant,
  renderEncounter as render,
} from "./encounterTestFixtures";
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
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
  const onPreview = vi.fn().mockResolvedValue({ consequences: [] });
  const current = encounterParticipant({
    participant_kind: "creature",
    side: "enemy",
    track_dying: false,
    revision: 0n,
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
  });
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
      onPreviewCondition={onPreview}
      current={current}
      encounterRevision={14n}
      disabled={false}
      pending={false}
      addDisabled={false}
      conditionDefinitions={definitions}
      damageTypes={[
        { value: "fire", label: "Fire" },
        { value: "cold", label: "Cold" },
      ]}
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
        parameters: {
          kind: "persistent_damage",
          formula: "3d6",
          damage_type: "fire",
          multiplier: "double",
          recovery: { kind: "specified", dc: 12n },
        },
        origin: current.conditions[0].origin,
        duration: current.conditions[0].duration,
      }),
    ),
  );
  expect(screen.getByRole("textbox", { name: "Damage formula" })).toHaveValue("3d6");
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Damage type" }));
  fireEvent.click(
    await screen.findByText("Cold", { selector: ".ant-select-item-option-content" }),
  );
  await waitFor(() =>
    expect(onPreview).toHaveBeenLastCalledWith(
      "actor",
      expect.objectContaining({
        parameters: expect.objectContaining({ damage_type: "cold" }),
      }),
    ),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(onUpdate).toHaveBeenLastCalledWith(
      "actor",
      expect.objectContaining({
        parameters: {
          kind: "persistent_damage",
          formula: "3d6",
          damage_type: "cold",
          multiplier: "double",
          recovery: { kind: "specified", dc: 12n },
        },
        origin: current.conditions[0].origin,
        duration: current.conditions[0].duration,
      }),
    ),
  );
});

it("shows linked persistent sources without disclosure and dims the backend-superseded source", async () => {
  const condition = (id: bigint, formula: string) => ({
    condition_id: id,
    name: `Source ${id}`,
    condition_key: "conditionitems:persistent",
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
    immunity_note: "Immune to fire",
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
  const recoveryButton = screen.getByRole("button", { name: "Roll Fire recovery" });
  expect(recoveryButton).toBeVisible();
  expect(recoveryButton).toHaveTextContent("");
  expect(recoveryButton.closest(".encounter-condition-actions")).not.toBeNull();
  expect(recoveryButton.closest(".encounter-condition-row")).toContainElement(
    screen.getByRole("link", { name: "Source 1 · 1d6 Fire" }),
  );
  expect(screen.queryByText(/null|undefined|Result/)).not.toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Source 1 · 1d6 Fire" })).toBeVisible();
  const superseded = screen
    .getByRole("link", { name: "Source 2 · 2d6 Fire" })
    .closest(".encounter-condition-row")!;
  expect(superseded).toHaveClass("encounter-condition-row--superseded");
  expect(within(superseded as HTMLElement).getByText("Superseded")).toBeVisible();
  for (const row of document.querySelectorAll(".encounter-condition-row")) {
    expect(within(row as HTMLElement).getByText("Immune to fire")).toBeVisible();
  }
  expect(screen.getByRole("button", { name: "Edit Source 1" })).toBeVisible();
  expect(screen.queryByRole("button", { name: /Persistent Damage/ })).toBeNull();
  fireEvent.click(await screen.findByRole("button", { name: "Remove Source 2" }));
  expect(onRemove).toHaveBeenCalledWith("actor", 2n, 8n);
  rerender(
    <EncounterConditionControls
      {...props}
      current={{
        ...current,
        persistent_damage_groups: [
          { ...current.persistent_damage_groups[0], selected_source_id: 2n },
        ],
      }}
    />,
  );
  expect(
    screen
      .getByRole("link", { name: "Source 2 · 2d6 Fire" })
      .closest(".encounter-condition-row"),
  ).not.toHaveClass("encounter-condition-row--superseded");
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
  expect(screen.getByRole("link", { name: "Source 1 · 1d6 ×2 Fire" })).toHaveAttribute(
    "href",
    "/records/conditionitems:persistent",
  );
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

it("shows a detail-free persistent condition without an internal Reminder label or empty metadata", () => {
  const current = encounterParticipant({
    conditions: [
      {
        condition_id: 9n,
        condition_key: "conditionitems:example",
        name: "Persistent Damage",
        created_at: "",
        updated_at: "",
        scope: { kind: "everyone" },
        extra_action: { kind: "unspecified" },
        immunity_override: false,
        parameters: {
          kind: "persistent_damage_reminder",
          formula: null,
          damage_type: null,
          multiplier: "normal",
          recovery: { kind: "standard" },
        },
      },
    ],
  });
  render(
    <EncounterConditionControls
      current={current}
      encounterRef="encounter"
      encounterRevision={1n}
      pending={false}
      disabled={false}
      addDisabled={false}
      conditionDefinitions={[]}
      damageTypes={[]}
      participants={[current]}
      onPreviewCondition={vi.fn()}
      onAddCondition={vi.fn()}
      onUpdateCondition={vi.fn()}
      onRemoveCondition={vi.fn()}
      onReference={vi.fn()}
    />,
  );
  const row = screen
    .getByRole("link", { name: "Persistent Damage" })
    .closest(".encounter-condition-row");
  expect(screen.queryByText(/Reminder/)).toBeNull();
  expect(row?.querySelector(".encounter-condition-meta")).toBeNull();
});

it("keeps inherited conditions read-only and preserves distinct rules and exact source links", () => {
  const source = {
    record_key: "actors:aaaaaaaaaaaaaaaa",
    embedded_path: [{ collection: "/items", identity: { Stable: "bbbbbbbbbbbbbbbb" } }],
    field: null,
    passage: null,
    source_fingerprint: "source-fingerprint",
  };
  const current = encounterParticipant({
    turn_resources: {
      turn_key: null,
      round_number: null,
      regular_total: null,
      regular_remaining: null,
      extra_total: null,
      extra_remaining: null,
      extra_uses: [],
      reactions_total: null,
      reactions_remaining: null,
      start_results: [],
      pending_reason: null,
      can_edit_actions: false,
      can_spend_actions: { available: false },
      can_spend_reactions: { available: true },
      forecast: {
        label: "Actions",
        base_value: 3n,
        adjusted_value: 2n,
        segments: [],
        adjustments: [],
        suppressed_adjustments: [],
      },
      inherent_conditions: {
        entries: [
          {
            name: "Slowed",
            has_value: true,
            value: 1n,
            condition_ref: "conditionitems:xYTAsEpcJE1Ccni3",
            source_name: "Slow",
            source,
          },
        ],
        unavailable_reason: null,
      },
    },
  });
  const onReference = vi.fn(),
    onRemove = vi.fn(),
    onUpdate = vi.fn();
  render(
    <EncounterConditionControls
      current={current}
      encounterRef="encounter"
      encounterRevision={1n}
      disabled={false}
      pending={false}
      addDisabled={false}
      conditionDefinitions={[]}
      participants={[current]}
      onAddCondition={vi.fn()}
      onPreviewCondition={vi.fn()}
      onUpdateCondition={onUpdate}
      onRemoveCondition={onRemove}
      onReference={onReference}
    />,
  );
  expect(screen.queryByText("No conditions")).toBeNull();
  expect(screen.getByText(/^Inherent/)).toBeVisible();
  expect(
    screen.queryByRole("button", { name: /Edit Slowed|Remove Slowed/ }),
  ).toBeNull();
  expect(screen.queryByRole("spinbutton")).toBeNull();
  const rule = screen.getByRole("link", { name: "Slowed 1" });
  fireEvent.click(rule);
  expect(onReference).toHaveBeenCalledWith("conditionitems:xYTAsEpcJE1Ccni3", rule);
  onReference.mockClear();
  const provenance = screen.getByRole("link", { name: "Slow" });
  expect(provenance).toHaveAttribute(
    "href",
    expect.stringContaining("records/actors:aaaaaaaaaaaaaaaa"),
  );
  fireEvent.click(provenance, { ctrlKey: true });
  expect(onReference).not.toHaveBeenCalled();
  fireEvent.click(provenance);
  expect(onReference).toHaveBeenCalledWith(source.record_key, provenance, source);
  expect(onRemove).not.toHaveBeenCalled();
  expect(onUpdate).not.toHaveBeenCalled();
});
