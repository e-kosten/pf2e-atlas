import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import type {
  ConditionEventPreviewView,
  EncounterParticipantConditionView,
} from "../../generated/atlas";
import { EncounterConditionControls } from "./EncounterConditionControls";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";

const result: ConditionEventPreviewView = {
  encounter_key: "fight",
  encounter_revision: 1,
  consequences: [],
};
function setup(value?: number) {
  const condition: EncounterParticipantConditionView = {
    condition_id: 42,
    condition_key: "conditionitems:drained",
    name: "Drained",
    value,
    immunity_override: false,
    parameters: { kind: "none" },
    scope: { kind: "everyone" },
    extra_action: { kind: "unspecified" },
    created_at: "then",
    updated_at: "then",
  };
  const actor = encounterParticipant({ conditions: [condition] });
  const preview = vi
    .fn<React.ComponentProps<typeof EncounterConditionControls>["onPreviewCondition"]>()
    .mockResolvedValue(result);
  const save = vi.fn().mockResolvedValue(true);
  const view = renderEncounter(
    <EncounterConditionControls
      disabled={false}
      addDisabled={false}
      encounterRef="fight"
      encounterRevision={1}
      pending={false}
      current={actor}
      participants={[actor]}
      conditionDefinitions={[
        {
          condition_ref: "conditionitems:drained",
          name: "Drained",
          editor_kind: "ordinary",
          has_value: true,
          default_value: 1,
          supports_extra_action: false,
          supports_duration_instead_of_value: false,
          scope_label: null,
          eligibility: { kind: "available" },
          automation_level: "automated",
          applies_to: [],
          categories: [],
        },
      ]}
      onReference={vi.fn()}
      onAddCondition={vi.fn()}
      onUpdateCondition={save}
      onRemoveCondition={vi.fn()}
      onPreviewCondition={preview}
    />,
  );
  function openNote() {
    fireEvent.click(screen.getByRole("button", { name: "Edit Drained" }));
    fireEvent.click(screen.getByRole("button", { name: "Add details" }));
    fireEvent.change(screen.getByRole("textbox", { name: "Condition note" }), {
      target: { value: "Round two" },
    });
    return within(screen.getByRole("dialog"));
  }
  return { view, preview, save, openNote };
}

it("restores the saved inline magnitude after a failed preview and retries the increment", async () => {
  const { preview, save } = setup(1);
  preview.mockRejectedValueOnce(new Error("Preview unavailable"));
  const increase = () =>
    fireEvent.click(screen.getByRole("button", { name: "Increase Drained value" }));
  increase();
  expect(await screen.findByText("Could not preview condition")).toBeVisible();
  expect(screen.getByRole("spinbutton", { name: "Drained value" })).toHaveValue("1");
  expect(save).not.toHaveBeenCalled();
  increase();
  await waitFor(() =>
    expect(save).toHaveBeenCalledWith("actor", expect.objectContaining({ value: 2 })),
  );
  expect(screen.queryByText("Could not preview condition")).toBeNull();
});

it("saves a note without supplying an unknown condition magnitude", async () => {
  const { save, preview, openNote } = setup();
  const dialog = openNote();
  fireEvent.click(dialog.getByRole("button", { name: "Apply" }));
  await waitFor(() => expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0][1]).toMatchObject({
    event_intent: "correction",
    note: "Round two",
  });
  expect(save.mock.calls[0][1]).not.toHaveProperty("value");
  for (const call of preview.mock.calls) expect(call[1]).not.toHaveProperty("value");
});

it("requires a valid magnitude once the GM edits it", async () => {
  const { save, openNote } = setup();
  const dialog = openNote();
  const magnitude = dialog.getByRole("spinbutton", { name: "Condition value" });
  const apply = dialog.getByRole("button", { name: "Apply" });
  fireEvent.change(magnitude, { target: { value: "2" } });
  fireEvent.change(magnitude, { target: { value: "" } });
  fireEvent.click(apply);
  expect(
    await dialog.findByText("Enter a whole number, one or greater."),
  ).toBeVisible();
  expect(save).not.toHaveBeenCalled();
  fireEvent.change(magnitude, { target: { value: "2" } });
  fireEvent.click(apply);
  await waitFor(() =>
    expect(save).toHaveBeenCalledWith(
      "actor",
      expect.objectContaining({ value: 2, event_intent: "apply" }),
    ),
  );
});

it("can dismiss a background preview before submitting", async () => {
  const { preview, save, openNote } = setup(1);
  let resolve!: (data: ConditionEventPreviewView) => void;
  preview.mockReturnValue(
    new Promise((done) => {
      resolve = done;
    }),
  );
  const dialog = openNote();
  await waitFor(() => expect(preview).toHaveBeenCalled());
  fireEvent.click(dialog.getByRole("button", { name: "Close" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  await act(async () => resolve(result));
  expect(save).not.toHaveBeenCalled();
});

it("retains the note when Apply cannot preview it and accepts a retry", async () => {
  const { preview, save, openNote } = setup(1);
  preview.mockRejectedValue(new Error("Preview unavailable"));
  const dialog = openNote();
  const apply = dialog.getByRole("button", { name: "Apply" });
  fireEvent.click(apply);
  expect(await dialog.findByText("Could not preview condition")).toBeVisible();
  expect(dialog.getByRole("button", { name: "Close" })).toBeEnabled();
  expect(dialog.getByRole("textbox", { name: "Condition note" })).toHaveValue(
    "Round two",
  );
  expect(save).not.toHaveBeenCalled();
  preview.mockResolvedValue(result);
  fireEvent.click(apply);
  await waitFor(() => expect(save).toHaveBeenCalledOnce());
});

it.each(["complete", "unmount"] as const)(
  "guards the submitted preview through %s",
  async (outcome) => {
    const { preview, save, view, openNote } = setup(1);
    let resolve!: (data: ConditionEventPreviewView) => void;
    const delayed = new Promise<ConditionEventPreviewView>((done) => {
      resolve = done;
    });
    preview.mockReturnValue(delayed);
    const dialog = openNote();
    const apply = dialog.getByRole("button", { name: "Apply" });
    fireEvent.click(apply);
    fireEvent.click(apply);
    await waitFor(() =>
      expect(dialog.getByRole("textbox", { name: "Condition note" })).toBeDisabled(),
    );
    expect(dialog.getByRole("button", { name: "Remove" })).toBeDisabled();
    expect(
      dialog.getByRole("button", { name: "Remove Condition value" }),
    ).toBeDisabled();
    expect(
      dialog.getByRole("button", { name: "Increase Condition value" }),
    ).toBeDisabled();
    expect(dialog.queryByRole("button", { name: "Close" })).toBeNull();
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(screen.getByRole("dialog")).toBeVisible();
    if (outcome === "unmount") view.unmount();
    await act(async () => resolve(result));
    if (outcome === "unmount") expect(save).not.toHaveBeenCalled();
    else await waitFor(() => expect(save).toHaveBeenCalledOnce());
  },
);
