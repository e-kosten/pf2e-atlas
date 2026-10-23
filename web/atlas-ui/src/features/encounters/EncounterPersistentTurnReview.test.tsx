import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import {
  encounterParticipant,
  renderEncounter as render,
} from "./encounterTestFixtures";
import { EncounterPersistentTurnReview } from "./EncounterPersistentTurnReview";
import type {
  EncounterParticipantConditionView,
  PersistentDamageGroupView,
} from "../../generated/atlas";
const api = vi.hoisted(() => ({ getEncounterPersistentReview: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
const fire: PersistentDamageGroupView = {
  episode_id: "fire",
  damage_type: "fire",
  selected_source_id: 2n,
  sources: [
    { condition_id: 2n, formula: "2d6", multiplier: "normal", recovery_dc: 15n },
    { condition_id: 1n, formula: "1d6", multiplier: "normal", recovery_dc: 15n },
  ],
};
const bleed: PersistentDamageGroupView = {
  episode_id: "bleed",
  damage_type: "bleed",
  selected_source_id: 3n,
  sources: [
    { condition_id: 3n, formula: "1d4", multiplier: "normal", recovery_dc: 15n },
  ],
};
const condition = (id: bigint, note: string): EncounterParticipantConditionView => ({
  condition_id: id,
  created_at: "",
  updated_at: "",
  name: "Persistent Damage",
  scope: { kind: "everyone" },
  extra_action: { kind: "unspecified" },
  immunity_override: false,
  parameters: {
    kind: "persistent_damage",
    formula: id === 1n ? "1d6" : "2d6",
    damage_type: "fire",
    multiplier: "normal",
    recovery: { kind: "standard" },
    episode_id: "fire",
  },
  note,
});
it("shows every fire source directly and ends fire from a table outcome without asking for a DC or damage", async () => {
  const review = {
    encounter_revision: 1n,
    damage_groups: [fire, bleed],
    round_number: 1n,
    damage_applied: false,
    damage_result: null,
    recovery_results: [],
    episodes: [fire, bleed].map((g) => ({
      episode_id: g.episode_id,
      end_turn_recovery_available: true,
      end_turn_roll_available: false,
      end_turn_recovery_unavailable_reason: null,
      end_turn_roll_unavailable_reason: "Record damage first",
    })),
  };
  api.getEncounterPersistentReview.mockResolvedValue(review);
  const onRecover = vi.fn().mockImplementation(async () => {
    api.getEncounterPersistentReview.mockResolvedValue({
      ...review,
      encounter_revision: 2n,
      damage_groups: [bleed],
    });
    return {
      encounter_key: "encounter",
      request_id: "clear",
      episode_id: "fire",
      damage_type: "fire",
      success: true,
      roll: null,
      dc: null,
    };
  });
  const onApply = vi.fn().mockResolvedValue({
    encounter_key: "encounter",
    request_id: "damage",
    groups: [],
    total: 3n,
    hardness_absorbed: 0n,
  });
  render(
    <EncounterPersistentTurnReview
      encounterRef="encounter"
      participant={encounterParticipant({
        persistent_damage_groups: [fire, bleed],
        conditions: [condition(1n, "Lingering flames"), condition(2n, "Dragon flame")],
      })}
      encounterRevision={1n}
      boundaryKey="outgoing"
      generation={null}
      damageTypes={[
        { value: "fire", label: "Fire" },
        { value: "bleed", label: "Bleed" },
      ]}
      disabled={false}
      pending={false}
      onApply={onApply}
      onRecover={onRecover}
    />,
  );
  const sources = await screen.findByRole("group", { name: "Fire sources" });
  expect(within(sources).getByText("1d6")).toBeVisible();
  expect(within(sources).getByText("2d6")).toBeVisible();
  expect(within(sources).getByText("Lingering flames")).toBeVisible();
  expect(within(sources).getByText("Dragon flame")).toBeVisible();
  expect(screen.queryByText("Additional sources")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "End fire damage" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledExactlyOnceWith({
      resolution: "clear",
      mode: "end_turn",
      boundary_key: "outgoing",
      expected_encounter_revision: 1n,
      participant_key: "actor",
      episode_id: "fire",
      dc_override: null,
    }),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "End fire damage" })).toBeDisabled(),
  );
  expect(screen.getByRole("button", { name: "End bleed damage" })).toBeEnabled();
  expect(onApply).not.toHaveBeenCalled();
  expect(screen.queryByRole("spinbutton", { name: "Recovery DC" })).toBeNull();
  fireEvent.click(
    within(screen.getByLabelText("Bleed damage result")).getByText("Table result"),
  );
  const amount = await screen.findByRole("spinbutton", { name: "Bleed damage amount" });
  expect(amount).toBeEnabled();
  expect(screen.queryByText("Reload damage")).toBeNull();
  fireEvent.change(amount, { target: { value: "3" } });
  expect(screen.getByText("Damage taken")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Apply persistent damage" }));
  await waitFor(() =>
    expect(onApply).toHaveBeenCalledWith(
      expect.objectContaining({
        expected_encounter_revision: 2n,
        groups: [
          {
            episode_id: "bleed",
            damage: { kind: "table_result", amount: 3n },
          },
        ],
      }),
    ),
  );
});
