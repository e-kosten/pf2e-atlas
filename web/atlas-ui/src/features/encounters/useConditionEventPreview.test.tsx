import { useState } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import {
  useConditionEventPreview,
  type ConditionEventInput,
} from "./useConditionEventPreview";
const request: ConditionEventInput = {
  expected_encounter_revision: 3n,
  condition_id: 1n,
  name: "Example",
  parameters: { kind: "none" },
  scope: { kind: "everyone" },
  extra_action: { kind: "unspecified" },
  immunity_override: false,
  event_intent: "apply",
  decisions: { keep_upright: false },
};
function Harness({
  load,
  commit,
}: {
  load: Parameters<typeof useConditionEventPreview>[1];
  commit: (request: ConditionEventInput) => void;
}) {
  const preview = useConditionEventPreview([], load);
  const [note, setNote] = useState(false);
  return (
    <>
      <button onClick={() => preview.setDraft(request)}>Edit draft</button>
      <button
        onClick={() => {
          setNote(true);
          preview.setDraft({ ...request, note: "changed" });
        }}
      >
        Edit note
      </button>
      {preview.review}
      <button
        disabled={!preview.ready}
        onClick={() =>
          commit(preview.apply(note ? { ...request, note: "changed" } : request))
        }
      >
        Apply
      </button>
      {preview.errorNotice}
    </>
  );
}
it("shows sole HP consequences before the first Apply and preserves its explicit opt-out", async () => {
  const load = vi.fn().mockResolvedValue({
    consequences: [{ kind: "direct_hp_loss", participant_key: "p", amount: 8n }],
  });
  const commit = vi.fn();
  render(<Harness load={load} commit={commit} />);
  fireEvent.click(screen.getByText("Edit draft"));
  const effect = await screen.findByRole("checkbox", { name: "Lose 8 HP" });
  expect(effect).toBeChecked();
  expect(commit).not.toHaveBeenCalled();
  fireEvent.click(effect);
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(commit).toHaveBeenCalledExactlyOnceWith({
    ...request,
    event_intent: "correction",
  });
  expect(screen.queryByRole("dialog")).toBeNull();
});
it("mixed consequences offer only the supported Prone opt-out", async () => {
  const load = vi.fn().mockResolvedValue({
    consequences: [
      { kind: "direct_hp_loss", participant_key: "p", amount: 8n },
      { kind: "become_prone", participant_key: "p" },
      { kind: "drop_held_items", participant_key: "p" },
    ],
  });
  const commit = vi.fn();
  render(<Harness load={load} commit={commit} />);
  fireEvent.click(screen.getByText("Edit draft"));
  fireEvent.click(await screen.findByRole("checkbox", { name: "Applies Prone" }));
  expect(screen.queryByRole("checkbox", { name: "Lose 8 HP" })).toBeNull();
  expect(screen.getByText("p: Drop held items")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(commit).toHaveBeenCalledExactlyOnceWith({
    ...request,
    decisions: { keep_upright: true },
  });
});
it("keeps Apply disabled while the preview fails", async () => {
  render(
    <Harness
      load={vi.fn().mockRejectedValue(new Error("Preview unavailable"))}
      commit={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByText("Edit draft"));
  await screen.findByText("Preview unavailable");
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
});

it("retains a consequence opt-out when only a note changes", async () => {
  const load = vi.fn().mockResolvedValue({
    consequences: [{ kind: "become_prone", participant_key: "p" }],
  });
  const commit = vi.fn();
  render(<Harness load={load} commit={commit} />);
  fireEvent.click(screen.getByText("Edit draft"));
  fireEvent.click(await screen.findByRole("checkbox", { name: "Applies Prone" }));
  fireEvent.click(screen.getByText("Edit note"));
  await waitFor(() => expect(load).toHaveBeenCalledTimes(2));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  expect(screen.getByRole("checkbox", { name: "Applies Prone" })).not.toBeChecked();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(commit).toHaveBeenCalledExactlyOnceWith({
    ...request,
    note: "changed",
    decisions: { keep_upright: true },
  });
});
