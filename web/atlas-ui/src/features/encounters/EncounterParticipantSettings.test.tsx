import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ConfigProvider } from "antd";
import type { ReactNode } from "react";
import { EncounterParticipantSettings } from "./EncounterParticipantSettings";
import type { EncounterParticipantView } from "../../generated/atlas";

const wrapper = ({ children }: { children: ReactNode }) => (
  <ConfigProvider theme={{ token: { motion: false } }}>{children}</ConfigProvider>
);

const participant: EncounterParticipantView = {
  revision: 7n,
  participant_key: "pc",
  position: 1n,
  initiative_order: 1n,
  display_name: "Mira",
  hp_origin: "explicit",
  variant_origin: "explicit",
  participant_kind: "pc",
  side: "pc",
  participant_variant: "normal",
  status: "active",
  max_hp: 20n,
  current_hp: 12n,
  temporary_hp: 3n,
  initiative: 14n,
  defeated: false,
  hazard_disabled: false,
  cast_unavailable_reason: null,
  hidden: false,
  note: "Lantern",
  note_hint: "Lantern",
  conditions: [],
};

it("saves participant metadata as one complete revision-checked write and cancels without writing", async () => {
  const onUpdate = vi.fn().mockResolvedValue(true);
  render(
    <EncounterParticipantSettings
      participant={participant}
      disabled={false}
      pending={false}
      onUpdate={onUpdate}
    />,
    { wrapper },
  );
  fireEvent.click(screen.getByRole("button", { name: "Participant settings" }));
  const dialog = await screen.findByRole("dialog", { name: "Participant settings" });
  fireEvent.change(within(dialog).getByLabelText("Participant name"), {
    target: { value: "Mira Dawn" },
  });
  fireEvent.change(within(dialog).getByLabelText("Participant notes"), {
    target: { value: "North door" },
  });
  expect(onUpdate).not.toHaveBeenCalled();
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onUpdate).toHaveBeenCalledOnce());
  expect(onUpdate).toHaveBeenCalledWith(
    expect.objectContaining({
      participant_key: "pc",
      expected_revision: 7n,
      display_name: "Mira Dawn",
      initiative: 14n,
      note: "North door",
      current_hp: 12n,
      max_hp: 20n,
      temporary_hp: 3n,
      hp_edit: false,
      max_hp_edit: false,
      variant_edit: false,
    }),
  );
  await waitFor(() => expect(dialog).not.toBeVisible());
  fireEvent.click(screen.getByRole("button", { name: "Participant settings" }));
  const reopened = await screen.findByRole("dialog", { name: "Participant settings" });
  fireEvent.change(within(reopened).getByLabelText("Participant name"), {
    target: { value: "Discard" },
  });
  fireEvent.click(within(reopened).getByRole("button", { name: "Cancel" }));
  expect(onUpdate).toHaveBeenCalledOnce();
});

it("retains failed metadata drafts while refreshing untouched fields and expected revision", async () => {
  const onUpdate = vi.fn().mockRejectedValue(new Error("Conflict"));
  const { rerender } = render(
    <EncounterParticipantSettings
      participant={participant}
      disabled={false}
      pending={false}
      onUpdate={onUpdate}
    />,
    { wrapper },
  );
  fireEvent.click(screen.getByRole("button", { name: "Participant settings" }));
  const dialog = await screen.findByRole("dialog", { name: "Participant settings" });
  fireEvent.change(within(dialog).getByLabelText("Participant name"), {
    target: { value: "Retained name" },
  });
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onUpdate).toHaveBeenCalledOnce());
  rerender(
    <EncounterParticipantSettings
      participant={{ ...participant, revision: 8n, initiative: 19n, current_hp: 5n }}
      disabled={false}
      pending={false}
      error="Participant changed"
      onUpdate={onUpdate}
    />,
  );
  await waitFor(() =>
    expect(within(dialog).getByText("Participant changed")).toBeVisible(),
  );
  expect(within(dialog).getByLabelText("Participant name")).toHaveValue(
    "Retained name",
  );
  expect(within(dialog).getByLabelText("Participant initiative")).toHaveValue("19");
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(onUpdate).toHaveBeenLastCalledWith(
      expect.objectContaining({
        expected_revision: 8n,
        current_hp: 5n,
        initiative: 19n,
        display_name: "Retained name",
      }),
    ),
  );
});

it("retains the editor after a committed write until canonical read recovery completes", async () => {
  const onUpdate = vi.fn().mockResolvedValue(false);
  const { rerender } = render(
    <EncounterParticipantSettings
      participant={participant}
      disabled={false}
      pending={false}
      onUpdate={onUpdate}
    />,
    { wrapper },
  );
  fireEvent.click(screen.getByRole("button", { name: "Participant settings" }));
  const dialog = await screen.findByRole("dialog", { name: "Participant settings" });
  fireEvent.change(within(dialog).getByLabelText("Participant name"), {
    target: { value: "Saved name" },
  });
  // The feature mutation owner disables editing before returning a saved/read-failed outcome.
  let complete: (value: boolean) => void = () => {};
  onUpdate.mockReturnValue(
    new Promise<boolean>((resolve) => {
      complete = resolve;
    }),
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onUpdate).toHaveBeenCalledOnce());
  rerender(
    <EncounterParticipantSettings
      participant={participant}
      disabled
      pending={false}
      onUpdate={onUpdate}
      recovery={<div>Saved; retry the read</div>}
    />,
  );
  await act(async () => complete(false));
  await waitFor(() => expect(dialog).toBeVisible());
  expect(within(dialog).getByLabelText("Participant name")).toHaveValue("Saved name");
  expect(within(dialog).getByRole("button", { name: "Save" })).toBeDisabled();
  rerender(
    <EncounterParticipantSettings
      participant={{ ...participant, display_name: "Saved name", revision: 8n }}
      disabled={false}
      pending={false}
      onUpdate={onUpdate}
    />,
  );
  await waitFor(() => expect(dialog).not.toBeVisible());
  expect(onUpdate).toHaveBeenCalledOnce();
});
