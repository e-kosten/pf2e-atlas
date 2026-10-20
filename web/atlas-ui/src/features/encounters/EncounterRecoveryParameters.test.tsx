import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { EncounterConditionControls } from "./EncounterConditionControls";
import type {
  EncounterConditionDefinitionView,
  EncounterParticipantView,
} from "../../generated/atlas";
const actor: EncounterParticipantView = {
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
  track_dying: false,
  mortality: "alive",
  base_death_threshold: 4n,
  persistent_damage_groups: [],
  conditions: [],
};
const definition: EncounterConditionDefinitionView = {
  condition_ref: "conditionitems:effect",
  name: "Source effect",
  editor_kind: "frightened",
  scope_label: null,
  supports_extra_action: false,
  supports_duration_instead_of_value: false,
  eligibility: { kind: "available" },
  has_value: true,
  automation_level: "automated",
  applies_to: [],
  categories: [],
};
function setup(
  parameters: EncounterParticipantView["conditions"][number]["parameters"],
  editorKind: EncounterConditionDefinitionView["editor_kind"],
  onRemove = vi.fn(),
) {
  const current = {
    ...actor,
    conditions: [
      {
        condition_id: 7n,
        condition_key: "conditionitems:effect",
        name: "Source effect",
        value: 1n,
        scope: { kind: "everyone" as const },
        extra_action: { kind: "unspecified" as const },
        immunity_override: false,
        parameters,
        created_at: "now",
        updated_at: "now",
      },
    ],
  };
  const onUpdate = vi.fn().mockRejectedValue(new Error("Connection lost"));
  render(
    <EncounterConditionControls
      encounterRef="encounter"
      encounterRevision={4n}
      current={current}
      participants={[current]}
      disabled={false}
      pending={false}
      addDisabled={false}
      conditionDefinitions={[{ ...definition, editor_kind: editorKind }]}
      onAddCondition={vi.fn()}
      onUpdateCondition={onUpdate}
      onRemoveCondition={onRemove}
      onReference={vi.fn()}
      onPreviewCondition={vi.fn().mockResolvedValue({ consequences: [] })}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Edit Source effect" }));
  return onUpdate;
}
it("edits typed recovery behavior and floor while preserving the source condition value", async () => {
  const onUpdate = setup(
    { kind: "frightened", recovery: { kind: "held" } },
    "frightened",
  );
  fireEvent.mouseDown(
    await screen.findByRole("combobox", { name: "Frightened recovery" }),
  );
  fireEvent.click(
    await screen.findByText("End of turn", {
      selector: ".ant-select-item-option-content",
    }),
  );
  fireEvent.change(
    await screen.findByRole("spinbutton", { name: "Frightened minimum" }),
    { target: { value: "2" } },
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(onUpdate).toHaveBeenCalledWith(
      "actor",
      expect.objectContaining({
        value: 1n,
        parameters: { kind: "frightened", recovery: { kind: "end_turn", minimum: 2n } },
      }),
    ),
  );
});
it("retains a failed typed broken-threshold draft without deriving it from HP", async () => {
  const onUpdate = setup({ kind: "broken", threshold: 6n }, "broken");
  const threshold = await screen.findByRole("spinbutton", { name: "Broken threshold" });
  fireEvent.change(threshold, { target: { value: "8" } });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(onUpdate).toHaveBeenCalledWith(
      "actor",
      expect.objectContaining({ parameters: { kind: "broken", threshold: 8n } }),
    ),
  );
  expect(threshold).toHaveValue("8");
});

it("uses the decrement remove action at one without a zero draft", async () => {
  const onRemove = vi.fn().mockResolvedValue(false);
  const onUpdate = setup({ kind: "none" }, "ordinary", onRemove);
  const dialog = screen.getByRole("dialog");
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Remove Condition value" }),
  );
  await waitFor(() =>
    expect(onRemove).toHaveBeenCalledExactlyOnceWith("actor", 7n, 4n),
  );
  expect(onUpdate).not.toHaveBeenCalled();
});

it("keeps waking normal unless the source restricts it", async () => {
  const onUpdate = setup({ kind: "unconscious", waking: "normal" }, "unconscious");
  const restriction = await screen.findByRole("checkbox", {
    name: "Wake automatically",
  });
  expect(restriction).toBeChecked();
  const info = screen.getByRole("button", { name: "About automatic waking" });
  fireEvent.focus(info);
  expect(await screen.findByRole("tooltip")).toHaveTextContent(
    "When healing, damage, or another normal waking trigger applies.",
  );
  fireEvent.click(info);
  expect(restriction).toBeChecked();
  fireEvent.click(restriction);
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() =>
    expect(onUpdate).toHaveBeenCalledWith(
      "actor",
      expect.objectContaining({
        parameters: { kind: "unconscious", waking: "source_controlled" },
      }),
    ),
  );
});
