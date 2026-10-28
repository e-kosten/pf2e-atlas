import { fireEvent, screen, waitFor } from "@testing-library/react";
import { EncounterRecoveryModal } from "./EncounterRecoveryModal";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";
import type {
  EncounterRecoveryAction,
  EncounterSummaryView,
  PreviewEncounterRecoveryRequest,
} from "../../generated/atlas";
const api = vi.hoisted(() => ({ previewEncounterRecovery: vi.fn() }));
vi.mock("../../api/atlasApi", () => api);
function recoveryMutations(run = vi.fn()) {
  return {
    run,
    pending: false,
    previewRecovery: (request: PreviewEncounterRecoveryRequest, signal?: AbortSignal) =>
      api.previewEncounterRecovery("encounter", request, signal),
  };
}
const encounter = {
  encounter: { encounter_key: "encounter" } as EncounterSummaryView,
  revision: 3n,
  participants: [
    encounterParticipant({ display_name: "Kyra", max_hp: 20n, current_hp: 10n }),
    encounterParticipant({
      participant_key: "enemy",
      display_name: "Goblin",
      side: "enemy",
      max_hp: 10n,
      current_hp: 3n,
    }),
  ],
};
function restPreview(_ref: string, request: { action: EncounterRecoveryAction }) {
  if (request.action.kind !== "rest_overnight")
    throw new Error("Expected rest preview");
  const full = request.action.healing_mode === "full";
  return Promise.resolve({
    encounter_key: "encounter",
    encounter_revision: 3n,
    can_apply: full || request.action.participants.every((p) => p.healing !== null),
    changes: ["End the current turn"],
    participants: request.action.participants.map((p) => ({
      participant_key: p.participant_key,
      display_name: p.participant_key === "actor" ? "Kyra" : "Goblin",
      current_hp: 10n,
      resulting_hp: full ? 20n : p.healing == null ? 10n : 15n,
      resulting_max_hp: 20n,
      healing: p.healing,
      changes: ["Remove Fatigued"],
      unavailable_reason:
        !full && p.healing === null ? "Enter the missing HP recovery amounts." : null,
    })),
  });
}
it("selects every participant and submits entered recovery only for those left selected", async () => {
  api.previewEncounterRecovery.mockImplementation(restPreview);
  const run = vi.fn().mockResolvedValue({ active: true, refreshed: true });
  const onClose = vi.fn();
  renderEncounter(
    <EncounterRecoveryModal
      encounter={encounter}
      mode="rest_overnight"
      mutations={recoveryMutations(run)}
      onClose={onClose}
    />,
  );
  const apply = screen.getByRole("button", { name: "Rest overnight" });
  await screen.findByText("Enter the missing HP recovery amounts.");
  expect(screen.getAllByText("Enter the missing HP recovery amounts.")).toHaveLength(1);
  expect(screen.getByRole("checkbox", { name: "Rest Goblin" })).toBeChecked();
  expect(apply).toBeDisabled();
  fireEvent.click(screen.getByRole("checkbox", { name: "Rest Goblin" }));
  fireEvent.change(screen.getByRole("spinbutton", { name: "HP recovery for Kyra" }), {
    target: { value: "5" },
  });
  await waitFor(() => expect(apply).toBeEnabled());
  fireEvent.click(apply);
  await waitFor(() =>
    expect(run).toHaveBeenCalledExactlyOnceWith({
      kind: "recovery",
      request: {
        expected_encounter_revision: 3n,
        action: {
          kind: "rest_overnight",
          healing_mode: "natural",
          participants: [{ participant_key: "actor", healing: 5n }],
        },
      },
    }),
  );
  expect(onClose).toHaveBeenCalledOnce();
});
it("offers full HP without requiring individual amounts", async () => {
  api.previewEncounterRecovery.mockImplementation(restPreview);
  const run = vi.fn().mockResolvedValue({ active: true, refreshed: true });
  renderEncounter(
    <EncounterRecoveryModal
      encounter={encounter}
      mode="rest_overnight"
      mutations={recoveryMutations(run)}
      onClose={vi.fn()}
    />,
  );
  await screen.findByText("Enter the missing HP recovery amounts.");
  fireEvent.click(screen.getByRole("radio", { name: "Full HP" }));
  const apply = screen.getByRole("button", { name: "Rest overnight" });
  await waitFor(() => expect(apply).toBeEnabled());
  expect(screen.queryByRole("spinbutton")).not.toBeInTheDocument();
  fireEvent.click(apply);
  await waitFor(() =>
    expect(run).toHaveBeenCalledWith({
      kind: "recovery",
      request: {
        expected_encounter_revision: 3n,
        action: {
          kind: "rest_overnight",
          healing_mode: "full",
          participants: [
            { participant_key: "actor", healing: null },
            { participant_key: "enemy", healing: null },
          ],
        },
      },
    }),
  );
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
        resulting_max_hp: 20n,
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
      mutations={recoveryMutations(run)}
      onClose={vi.fn()}
    />,
  );
  await screen.findByText("No creation baseline was saved");
  expect(screen.getByRole("button", { name: "Reset encounter" })).toBeDisabled();
  expect(run).not.toHaveBeenCalled();
});

it("keeps a cleared calculated amount blank and associates its shared validation", async () => {
  api.previewEncounterRecovery.mockImplementation(async (_ref, request) => {
    const result = await restPreview(_ref, request);
    return {
      ...result,
      can_apply: true,
      participants: result.participants.map((p) => ({
        ...p,
        healing: 5n,
        unavailable_reason: null,
      })),
    };
  });
  renderEncounter(
    <EncounterRecoveryModal
      encounter={encounter}
      mode="rest_overnight"
      mutations={recoveryMutations()}
      onClose={vi.fn()}
    />,
  );
  const input = await screen.findByRole("spinbutton", { name: "HP recovery for Kyra" });
  await waitFor(() => expect(input).toHaveDisplayValue("5"));
  fireEvent.change(input, { target: { value: "" } });
  await waitFor(() => expect(input).toHaveDisplayValue(""));
  expect(input).toHaveAccessibleDescription("Enter the missing HP recovery amounts.");
  expect(screen.getByRole("button", { name: "Rest overnight" })).toBeDisabled();
});
