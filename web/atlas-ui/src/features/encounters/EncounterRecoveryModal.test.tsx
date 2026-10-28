import { fireEvent, screen, waitFor } from "@testing-library/react";
import { EncounterRecoveryModal } from "./EncounterRecoveryModal";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";
import type { EncounterSummaryView } from "../../generated/atlas";
const api = vi.hoisted(() => ({ previewEncounterRecovery: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
const encounter = {
  encounter: { encounter_key: "encounter" } as EncounterSummaryView,
  revision: 3n,
  participants: [
    encounterParticipant({ display_name: "Kyra", max_hp: 20n, current_hp: 10n }),
    encounterParticipant({
      participant_key: "enemy",
      display_name: "Goblin",
      side: "enemy",
    }),
  ],
};
it("previews editable overnight healing and submits selected participants once", async () => {
  api.previewEncounterRecovery.mockImplementation(async (_ref, request) => ({
    encounter_key: "encounter",
    encounter_revision: 3n,
    can_apply: request.action.participants.every(
      (p: { healing: bigint | null }) => p.healing !== null,
    ),
    changes: ["End the current turn"],
    participants: request.action.participants.map(
      (p: { participant_key: string; healing: bigint | null }) => ({
        participant_key: p.participant_key,
        display_name: "Kyra",
        current_hp: 10n,
        resulting_hp: p.healing == null ? 10n : 15n,
        healing: p.healing,
        changes: ["Remove Fatigued"],
        unavailable_reason: p.healing == null ? "Enter HP recovery" : null,
      }),
    ),
  }));
  const run = vi.fn().mockResolvedValue({ active: true, refreshed: true });
  const onClose = vi.fn();
  renderEncounter(
    <EncounterRecoveryModal
      encounter={encounter}
      mode="rest_overnight"
      mutations={{ run, pending: false }}
      onClose={onClose}
    />,
  );
  const apply = screen.getByRole("button", { name: "Rest overnight" });
  await screen.findByText("Enter HP recovery");
  expect(apply).toBeDisabled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "HP recovery for Kyra" }), {
    target: { value: "5" },
  });
  await waitFor(() => expect(apply).toBeEnabled());
  expect(screen.getByRole("checkbox", { name: "Goblin" })).not.toBeChecked();
  fireEvent.click(apply);
  await waitFor(() =>
    expect(run).toHaveBeenCalledExactlyOnceWith({
      kind: "recovery",
      request: {
        expected_encounter_revision: 3n,
        action: {
          kind: "rest_overnight",
          participants: [{ participant_key: "actor", healing: 5n }],
        },
      },
    }),
  );
  expect(onClose).toHaveBeenCalledOnce();
});
it("shows a missing baseline and prevents a partial encounter reset", async () => {
  api.previewEncounterRecovery.mockResolvedValue({
    encounter_key: "encounter",
    encounter_revision: 3n,
    can_apply: false,
    changes: [],
    participants: [
      {
        participant_key: "actor",
        display_name: "Kyra",
        current_hp: 10n,
        resulting_hp: 10n,
        healing: null,
        changes: [],
        unavailable_reason: "No creation baseline was saved",
      },
    ],
  });
  const run = vi.fn();
  renderEncounter(
    <EncounterRecoveryModal
      encounter={encounter}
      mode="reset_encounter"
      mutations={{ run, pending: false }}
      onClose={vi.fn()}
    />,
  );
  await screen.findByText("No creation baseline was saved");
  expect(screen.getByRole("button", { name: "Reset encounter" })).toBeDisabled();
  expect(run).not.toHaveBeenCalled();
});
