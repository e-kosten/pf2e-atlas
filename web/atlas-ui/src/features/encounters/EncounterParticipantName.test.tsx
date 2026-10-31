import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterParticipantName } from "./EncounterParticipantName";
import { encounterParticipant } from "./encounterTestFixtures";

const participant = encounterParticipant({
  display_name: "Goblin",
  participant_kind: "creature",
});

function openEditor() {
  fireEvent.click(screen.getByRole("button", { name: "Edit Participant name" }));
  return screen.getByRole("textbox", { name: "Participant name" });
}

it("cancels a draft with Escape and restores keyboard focus", () => {
  const onUpdate = vi.fn();
  render(
    <EncounterParticipantName
      participant={participant}
      disabled={false}
      onUpdate={onUpdate}
    />,
  );
  const input = openEditor();
  expect(input).toHaveFocus();
  fireEvent.change(input, { target: { value: "Captain" } });
  fireEvent.keyDown(input, { key: "Escape" });
  expect(onUpdate).not.toHaveBeenCalled();
  expect(screen.getByRole("button", { name: "Edit Participant name" })).toHaveFocus();
  expect(openEditor()).toHaveValue("Goblin");
});

it("rejects whitespace, retains the draft, and clears validation on cancel", async () => {
  const onUpdate = vi.fn();
  render(
    <EncounterParticipantName
      participant={participant}
      disabled={false}
      onUpdate={onUpdate}
    />,
  );
  const input = openEditor();
  fireEvent.change(input, { target: { value: "   " } });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Enter a participant name.",
  );
  await waitFor(() => expect(input).not.toHaveAttribute("readonly"));
  expect(input).toHaveValue("   ");
  expect(onUpdate).not.toHaveBeenCalled();
  fireEvent.keyDown(input, { key: "Escape" });
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});

it("retains a failed save for retry, submits once on blur, and preserves other settings", async () => {
  const onUpdate = vi
    .fn()
    .mockRejectedValueOnce(new Error("Save failed"))
    .mockResolvedValue(true);
  render(
    <EncounterParticipantName
      participant={participant}
      disabled={false}
      onUpdate={onUpdate}
    />,
  );
  const input = openEditor();
  fireEvent.change(input, { target: { value: " Goblin captain " } });
  fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() => expect(onUpdate).toHaveBeenCalledOnce());
  await waitFor(() => expect(input).not.toHaveAttribute("readonly"));
  expect(input).toHaveValue(" Goblin captain ");
  fireEvent.blur(input);
  await waitFor(() => expect(onUpdate).toHaveBeenCalledTimes(2));
  expect(onUpdate).toHaveBeenLastCalledWith(
    expect.objectContaining({
      participant_key: participant.participant_key,
      expected_revision: participant.revision,
      display_name: "Goblin captain",
      side: participant.side,
      hp_edit: false,
      max_hp_edit: false,
      variant_edit: false,
    }),
  );
  await waitFor(() => expect(screen.queryByRole("textbox")).not.toBeInTheDocument());
});

it("keeps a pending save read-only and restores focus after canonical refresh", async () => {
  let resolveSave!: (refreshed: boolean) => void;
  const onUpdate = vi.fn(
    () =>
      new Promise<boolean>((resolve) => {
        resolveSave = resolve;
      }),
  );
  const { rerender } = render(
    <EncounterParticipantName
      participant={participant}
      disabled={false}
      onUpdate={onUpdate}
    />,
  );
  const input = openEditor();
  fireEvent.change(input, { target: { value: "Captain" } });
  fireEvent.keyDown(input, { key: "Enter" });
  rerender(
    <EncounterParticipantName participant={participant} disabled onUpdate={onUpdate} />,
  );
  expect(input).toHaveAttribute("readonly");
  expect(input).toHaveFocus();
  fireEvent.keyDown(input, { key: "Enter" });
  expect(onUpdate).toHaveBeenCalledOnce();
  rerender(
    <EncounterParticipantName
      participant={{ ...participant, display_name: "Captain", revision: 2n }}
      disabled={false}
      onUpdate={onUpdate}
    />,
  );
  resolveSave(true);
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Edit Participant name" })).toHaveFocus(),
  );
  expect(screen.getByRole("heading", { name: "Captain" })).toHaveTextContent("Captain");
});

it("does not expose an editor while mutations are disabled", () => {
  render(
    <EncounterParticipantName participant={participant} disabled onUpdate={vi.fn()} />,
  );
  expect(screen.getByRole("button", { name: "Edit Participant name" })).toBeDisabled();
});
