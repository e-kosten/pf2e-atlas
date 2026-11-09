import { fireEvent, screen, waitFor } from "@testing-library/react";
import { EncounterManualTrackers } from "./EncounterManualTrackers";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";

it("retains an unsuccessful tracker draft and its original revisions across refresh", async () => {
  const onChange = vi.fn().mockRejectedValue(new Error("offline"));
  const participant = encounterParticipant({
    participant_key: "dragon",
    display_name: "Dragon",
    revision: 3,
  });
  const props = {
    participant,
    participants: [participant],
    encounterRevision: 7,
    disabled: false,
    pending: false,
    onChange,
  };
  const { rerender } = renderEncounter(<EncounterManualTrackers {...props} />);
  fireEvent.click(screen.getByRole("button", { name: "Add tracker" }));
  fireEvent.change(screen.getByLabelText("Name"), {
    target: { value: "Breath ready" },
  });
  fireEvent.change(screen.getByRole("spinbutton", { name: "Duration amount" }), {
    target: { value: "3" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Save tracker" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(1));
  expect(onChange).toHaveBeenCalledWith("dragon", {
    expected_encounter_revision: 7,
    expected_participant_revision: 3,
    action: {
      kind: "add",
      name: "Breath ready",
      duration: {
        amount: 3,
        unit: "rounds",
        timing: { participant_key: "dragon", event: "start" },
      },
    },
  });
  rerender(
    <EncounterManualTrackers
      {...props}
      encounterRevision={9}
      participant={{ ...participant, revision: 5 }}
      error="Check current values before trying again."
    />,
  );
  expect(screen.getByLabelText("Name")).toHaveValue("Breath ready");
  expect(screen.getByRole("spinbutton", { name: "Duration amount" })).toHaveValue("3");
  expect(screen.getByRole("button", { name: /Save tracker/ })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Use refreshed encounter" }));
  onChange.mockResolvedValue({ active: true, refreshed: true });
  fireEvent.click(screen.getByRole("button", { name: "Save tracker" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(2));
  expect(onChange.mock.calls[1][1].expected_encounter_revision).toBe(9);
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
          amount: 1,
          unit: "minutes",
          timing: { participant_key: "dragon", event: "end" },
        },
        remaining_rounds: 0,
        timing_label: "End of Dragon’s turn",
        timing_available: true,
      },
    ],
  });
  renderEncounter(
    <EncounterManualTrackers
      participant={participant}
      participants={[participant]}
      encounterRevision={7}
      disabled={false}
      pending={false}
      onChange={onChange}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent("Elapsed");
  expect(screen.getByRole("status").parentElement).toHaveTextContent(
    "1 minute · Elapsed",
  );
  expect(screen.queryByText("End of Dragon’s turn")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Restart Breath ready" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith(
      "dragon",
      expect.objectContaining({ action: { kind: "restart", tracker_key: "breath" } }),
    ),
  );
  fireEvent.click(screen.getByRole("button", { name: "Remove tracker Breath ready" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith(
      "dragon",
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
    encounterRevision: 1,
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
  rerender(<EncounterManualTrackers {...props} encounterRevision={2} />);
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  expect(onChange).toHaveBeenCalledTimes(1);
});
