import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type {
  PersistentDamageGroupView,
  PersistentRecoveryResultView,
} from "../../generated/atlas";
import { EncounterPersistentRecovery } from "./EncounterPersistentRecovery";
const group: PersistentDamageGroupView = {
  episode_id: "fire-episode",
  damage_type: "fire",
  selected_source_id: 7n,
  sources: [
    { condition_id: 7n, formula: "1d6", multiplier: "normal", recovery_dc: 15n },
  ],
};
const receipt: PersistentRecoveryResultView = {
  encounter_key: "encounter",
  request_id: "receipt",
  episode_id: "fire-episode",
  damage_type: "fire",
  dc: 15n,
  roll: null,
  success: true,
};
function setup(current = group, mode: "explicit" | "end_turn" = "explicit") {
  const onRecover = vi.fn().mockResolvedValue(receipt);
  const onResult = vi.fn();
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentRecovery
        group={current}
        mode={mode}
        available={true}
        participantKey="actor"
        encounterRevision={11n}
        disabled={false}
        pending={false}
        damageType="Fire"
        onRecover={onRecover}
        onResult={onResult}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire recovery" }));
  return { onRecover, onResult };
}
it("rolls recovery without requiring a table result and preserves the episode identity", async () => {
  const { onRecover, onResult } = setup();
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith({
      resolution: "roll",
      mode: "explicit",
      boundary_key: null,
      expected_encounter_revision: 11n,
      participant_key: "actor",
      episode_id: "fire-episode",
      dc_override: null,
    }),
  );
  expect(onResult).toHaveBeenCalledWith(receipt);
});
it("keeps recovery results inside the popover and offers no separate table-success flow", async () => {
  setup();
  expect(await screen.findByRole("spinbutton", { name: "Recovery DC" })).toHaveValue(
    "15",
  );
  expect(screen.queryByRole("button", { name: "Succeeded" })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await screen.findByText("Recovered · DC 15");
  expect(screen.getByRole("button", { name: "Roll" })).toBeDisabled();
  expect(screen.getByRole("spinbutton", { name: "Recovery DC" })).toHaveValue("15");
  expect(screen.getByRole("spinbutton", { name: "Recovery DC" })).toBeDisabled();
});
it("requests a missing DC for the precedence-selected source without another source picker", async () => {
  const { onRecover } = setup({
    ...group,
    sources: [{ ...group.sources[0], recovery_dc: null }],
  });
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  expect(screen.queryByRole("combobox", { name: "Recovery damage source" })).toBeNull();
  await screen.findByText("Enter a whole number.");
  expect(onRecover).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Recovery DC" }), {
    target: { value: "10" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith(
      expect.objectContaining({ dc_override: 10n }),
    ),
  );
});

it("uses the edited DC and keeps the previous result's DC when preparing another roll", async () => {
  const { onRecover } = setup();
  onRecover.mockResolvedValueOnce({ ...receipt, dc: 10n, success: false });
  fireEvent.change(await screen.findByRole("spinbutton", { name: "Recovery DC" }), {
    target: { value: "10" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Roll" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledWith(
      expect.objectContaining({ dc_override: 10n }),
    ),
  );
  await screen.findByText("Still active · DC 10");
  fireEvent.change(screen.getByRole("spinbutton", { name: "Recovery DC" }), {
    target: { value: "7" },
  });
  expect(screen.getByText("Still active · DC 10")).toBeVisible();
});

it("presents a recorded end-turn check as a result without asserting the effect is still active", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentRecovery
        group={group}
        mode="end_turn"
        available={false}
        recordedResult={{ ...receipt, success: false }}
        participantKey="actor"
        encounterRevision={12n}
        disabled={false}
        pending={false}
        damageType="Fire"
        onRecover={vi.fn()}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "View Fire recovery" }));
  await waitFor(() =>
    expect(screen.getByText("Recovery failed · DC 15")).toBeVisible(),
  );
  expect(screen.queryByRole("spinbutton", { name: "Recovery DC" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Roll" })).toBeNull();
});

it.each([true, false])(
  "hands focus from the submitted form to its retained end-turn result (success=%s)",
  async (success) => {
    const { onRecover } = setup(group, "end_turn");
    onRecover.mockResolvedValueOnce({ ...receipt, success, dc: success ? 0n : 100n });
    const roll = await screen.findByRole("button", { name: /^Roll$/ });
    roll.focus();
    fireEvent.click(roll);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "View Fire recovery" })).toHaveFocus(),
    );
    expect(roll).not.toBeInTheDocument();
  },
);

it("preserves a focus destination chosen while end-turn recovery is pending", async () => {
  const { onRecover } = setup(group, "end_turn");
  let resolve!: (result: PersistentRecoveryResultView) => void;
  onRecover.mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  const roll = await screen.findByRole("button", { name: /^Roll$/ });
  roll.focus();
  fireEvent.click(roll);
  await waitFor(() => expect(onRecover).toHaveBeenCalledTimes(1));
  render(<button>Another task</button>);
  const outside = screen.getByRole("button", { name: "Another task" });
  outside.focus();
  resolve({ ...receipt, dc: 0n });
  await screen.findByRole("button", { name: "View Fire recovery" });
  expect(outside).toHaveFocus();
});

it("waits for the canonical update to enable the retained result before restoring focus", async () => {
  let resolve!: (result: PersistentRecoveryResultView) => void;
  const onRecover = vi.fn(
    () => new Promise<PersistentRecoveryResultView>((done) => (resolve = done)),
  );
  const content = (pending: boolean) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentRecovery
        group={group}
        mode="end_turn"
        available={true}
        participantKey="actor"
        encounterRevision={11n}
        disabled={pending}
        pending={pending}
        damageType="Fire"
        onRecover={onRecover}
      />
    </ConfigProvider>
  );
  const view = render(content(false));
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire recovery" }));
  const roll = await screen.findByRole("button", { name: /^Roll$/ });
  roll.focus();
  fireEvent.click(roll);
  await waitFor(() => expect(onRecover).toHaveBeenCalledTimes(1));
  view.rerender(content(true));
  resolve(receipt);
  const result = await screen.findByRole("button", { name: "View Fire recovery" });
  expect(result).toBeDisabled();
  expect(result).not.toHaveFocus();
  view.rerender(content(false));
  await waitFor(() => expect(result).toHaveFocus());
});

it("clears the episode from a table outcome without requiring a recovery DC", async () => {
  const { onRecover, onResult } = setup({
    ...group,
    sources: [{ ...group.sources[0], recovery_dc: null }],
  });
  const cleared = { ...receipt, dc: null };
  onRecover.mockResolvedValue(cleared);
  fireEvent.click(await screen.findByRole("button", { name: "Clear effect" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenCalledExactlyOnceWith({
      resolution: "clear",
      mode: "explicit",
      boundary_key: null,
      expected_encounter_revision: 11n,
      participant_key: "actor",
      episode_id: "fire-episode",
      dc_override: null,
    }),
  );
  expect(onResult).toHaveBeenCalledWith(cleared);
  await waitFor(() => expect(screen.getByText("Recovered")).toBeVisible());
});

it("allows clearing after a failed check at the updated encounter revision", async () => {
  const onRecover = vi.fn().mockResolvedValueOnce({ ...receipt, success: false });
  const content = (revision: bigint) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterPersistentRecovery
        group={group}
        mode="explicit"
        available={true}
        participantKey="actor"
        encounterRevision={revision}
        disabled={false}
        pending={false}
        damageType="Fire"
        onRecover={onRecover}
      />
    </ConfigProvider>
  );
  const view = render(content(11n));
  fireEvent.click(screen.getByRole("button", { name: "Roll Fire recovery" }));
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await screen.findByText("Still active · DC 15");
  view.rerender(content(12n));
  onRecover.mockResolvedValueOnce({ ...receipt, dc: null });
  fireEvent.click(screen.getByRole("button", { name: "Clear effect" }));
  await waitFor(() =>
    expect(onRecover).toHaveBeenLastCalledWith(
      expect.objectContaining({
        resolution: "clear",
        expected_encounter_revision: 12n,
        dc_override: null,
      }),
    ),
  );
  await waitFor(() => expect(screen.getByText("Recovered")).toBeVisible());
});

it("replaces a failed roll request error after a successful table clear", async () => {
  const { onRecover } = setup();
  onRecover.mockRejectedValueOnce(new Error("Could not confirm recovery."));
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await screen.findByText("Couldn’t confirm the roll.");
  onRecover.mockResolvedValueOnce({ ...receipt, dc: null });
  fireEvent.click(screen.getByRole("button", { name: "Clear effect" }));
  await waitFor(() => expect(screen.getByText("Recovered")).toBeVisible());
  expect(screen.queryByText("Couldn’t confirm the roll.")).toBeNull();
});

it("retains an actual failed check when recording a subsequent table recovery", async () => {
  const { onRecover } = setup();
  onRecover.mockResolvedValueOnce({
    ...receipt,
    success: false,
    roll: {
      formula: "1d20",
      total: 5,
      expression: {
        kind: "dice",
        dice: [{ sides: 20, value: 5, kept: true }],
        total: 5,
      },
    },
  });
  fireEvent.click(await screen.findByRole("button", { name: "Roll" }));
  await screen.findByText("Still active · DC 15");
  onRecover.mockResolvedValueOnce({ ...receipt, dc: null });
  fireEvent.click(screen.getByRole("button", { name: "Clear effect" }));
  await waitFor(() => expect(screen.getByText("Recovered · DC 15")).toBeVisible());
  expect(screen.getByRole("status", { name: "1d20: 5" })).toBeVisible();
});
