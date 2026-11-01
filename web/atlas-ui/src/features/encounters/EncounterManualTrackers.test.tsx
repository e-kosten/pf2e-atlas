import { fireEvent, screen, waitFor } from "@testing-library/react";
import { EncounterManualTrackers } from "./EncounterManualTrackers";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";

it("retains an unsuccessful tracker draft and its original revisions across refresh", async () => {
  const onChange = vi.fn().mockRejectedValue(new Error("offline"));
  const participant = encounterParticipant({
    participant_key: "dragon",
    display_name: "Dragon",
    revision: 3n,
  });
  const props = {
    participant,
    participants: [participant],
    encounterRevision: 7n,
    disabled: false,
    pending: false,
    onChange,
  };
  const { rerender } = renderEncounter(<EncounterManualTrackers {...props} />);
  fireEvent.click(screen.getByRole("button", { name: "Add tracker" }));
  fireEvent.change(screen.getByLabelText("Name"), {
    target: { value: "Breath ready" },
  });
  fireEvent.change(screen.getByRole("spinbutton", { name: "Tracker duration" }), {
    target: { value: "3" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save tracker" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(1));
  expect(onChange.mock.calls[0][0]).toMatchObject({
    expected_encounter_revision: 7n,
    expected_participant_revision: 3n,
    participant_key: "dragon",
    action: {
      kind: "add",
      name: "Breath ready",
      duration: {
        amount: 3n,
        unit: "rounds",
        timing: { participant_key: "dragon", event: "end" },
      },
    },
  });
  rerender(
    <EncounterManualTrackers
      {...props}
      encounterRevision={9n}
      participant={{ ...participant, revision: 5n }}
      error="Check current values before trying again."
    />,
  );
  expect(screen.getByLabelText("Name")).toHaveValue("Breath ready");
  expect(screen.getByRole("spinbutton", { name: "Tracker duration" })).toHaveValue("3");
  expect(screen.getByRole("button", { name: /Save tracker/ })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Use refreshed encounter" }));
  onChange.mockResolvedValue({ active: true, refreshed: true });
  fireEvent.click(screen.getByRole("button", { name: "Save tracker" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(2));
  expect(onChange.mock.calls[1][0].expected_encounter_revision).toBe(9n);
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
});

it("keeps a ready tracker visible and sends separate restart and removal actions", async () => {
  const onChange = vi.fn().mockResolvedValue({ active: true, refreshed: true });
  const participant = encounterParticipant({
    participant_key: "dragon",
    manual_trackers: [
      {
        tracker_key: "breath",
        name: "Breath ready",
        duration: {
          amount: 1n,
          unit: "minutes",
          timing: { participant_key: "dragon", event: "end" },
        },
        remaining_rounds: 0n,
        timing_label: "End of Dragon’s turn",
        timing_available: true,
      },
    ],
  });
  renderEncounter(
    <EncounterManualTrackers
      participant={participant}
      participants={[participant]}
      encounterRevision={7n}
      disabled={false}
      pending={false}
      onChange={onChange}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent("Ready");
  fireEvent.click(screen.getByRole("button", { name: "Restart Breath ready" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ action: { kind: "restart", tracker_key: "breath" } }),
    ),
  );
  fireEvent.click(screen.getByRole("button", { name: "Remove tracker Breath ready" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ action: { kind: "remove", tracker_key: "breath" } }),
    ),
  );
});

it("prevents resubmitting a saved tracker while its failed refresh recovers", async () => {
  let finish!: (result: { active: boolean; refreshed: boolean }) => void;
  const onChange = vi.fn(
    () =>
      new Promise<{ active: boolean; refreshed: boolean }>((resolve) => {
        finish = resolve;
      }),
  );
  const participant = encounterParticipant();
  const props = {
    participant,
    participants: [participant],
    encounterRevision: 1n,
    disabled: false,
    pending: false,
    onChange,
  };
  const { rerender } = renderEncounter(<EncounterManualTrackers {...props} />);
  fireEvent.click(screen.getByRole("button", { name: "Add tracker" }));
  fireEvent.change(screen.getByLabelText("Name"), {
    target: { value: "Saved countdown" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save tracker" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(1));
  rerender(
    <EncounterManualTrackers
      {...props}
      disabled
      pending
      recovery={<p>Saved, but the encounter could not refresh</p>}
    />,
  );
  finish({ active: true, refreshed: false });
  await waitFor(() =>
    expect(
      screen.getByText("Saved, but the encounter could not refresh"),
    ).toBeVisible(),
  );
  rerender(
    <EncounterManualTrackers
      {...props}
      disabled
      recovery={<p>Saved, but the encounter could not refresh</p>}
    />,
  );
  expect(screen.getByRole("button", { name: /Save tracker/ })).toBeDisabled();
  rerender(<EncounterManualTrackers {...props} encounterRevision={2n} />);
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  expect(onChange).toHaveBeenCalledTimes(1);
});
