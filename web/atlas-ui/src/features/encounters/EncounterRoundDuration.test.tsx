import { fireEvent, screen, waitFor } from "@testing-library/react";
import { EditEncounterModal } from "./EncounterModals";
import { renderEncounter } from "./encounterTestFixtures";
import type { EncounterSummaryView } from "../../generated/atlas";

const encounter: EncounterSummaryView = {
  encounter_key: "clock",
  slug: "clock",
  name: "Clock encounter",
  status: "draft",
  round_number: 1,
  round_duration_seconds: 6,
  participant_count: 0,
  created_at: "now",
  updated_at: "now",
};

it("saves an explicitly changed round duration in seconds", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  renderEncounter(
    <EditEncounterModal
      encounter={encounter}
      encounterNote={undefined}
      open
      onCancel={vi.fn()}
      onSave={onSave}
      onRefreshConflict={vi.fn()}
    />,
  );
  expect(screen.getByRole("spinbutton", { name: "Round duration" })).toHaveValue("6");
  fireEvent.change(screen.getByRole("spinbutton", { name: "Round duration" }), {
    target: { value: "30" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onSave).toHaveBeenCalledWith({
      round_duration_seconds: { expected: 6, value: 30 },
    }),
  );
});

it("keeps the current duration visible and omits it from unrelated edits", async () => {
  const onSave = vi.fn().mockResolvedValue(undefined);
  renderEncounter(
    <EditEncounterModal
      encounter={{ ...encounter, round_duration_seconds: 30 }}
      encounterNote={undefined}
      open
      onCancel={vi.fn()}
      onSave={onSave}
      onRefreshConflict={vi.fn()}
    />,
  );
  expect(screen.getByRole("spinbutton", { name: "Round duration" })).toHaveValue("30");
  fireEvent.change(screen.getByRole("textbox", { name: "Name" }), {
    target: { value: "Renamed" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSave).toHaveBeenCalledTimes(1));
  expect(onSave.mock.calls[0][0]).not.toHaveProperty("round_duration_seconds");
});
