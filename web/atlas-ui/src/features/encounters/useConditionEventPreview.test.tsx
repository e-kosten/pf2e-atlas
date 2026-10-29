import { renderEncounter as render } from "./encounterTestFixtures";
import { useState } from "react";
import { fireEvent, screen, waitFor } from "@testing-library/react";
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
  const [value, setValue] = useState<bigint>();
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
      <button
        onClick={() => {
          setValue(2n);
          preview.setDraft({ ...request, value: 2n });
        }}
      >
        Edit value
      </button>
      <button
        onClick={async () => {
          const prepared = await preview.prepare({ ...request, value });
          if (prepared) commit(prepared);
        }}
      >
        Save immediately
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
  await screen.findByText("Could not preview condition");
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
  expect(load).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled();
  expect(screen.getByRole("checkbox", { name: "Applies Prone" })).not.toBeChecked();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(commit).toHaveBeenCalledExactlyOnceWith({
    ...request,
    note: "changed",
    decisions: { keep_upright: true },
  });
});

it("resets an old opt-out after consequences change and then return", async () => {
  const load = vi
    .fn()
    .mockResolvedValueOnce({
      consequences: [{ kind: "become_prone", participant_key: "p" }],
    })
    .mockResolvedValueOnce({ consequences: [] })
    .mockResolvedValueOnce({
      consequences: [{ kind: "become_prone", participant_key: "p" }],
    });
  render(<Harness load={load} commit={vi.fn()} />);
  fireEvent.click(screen.getByText("Edit draft"));
  fireEvent.click(await screen.findByRole("checkbox", { name: "Applies Prone" }));
  fireEvent.click(screen.getByText("Edit value"));
  await waitFor(() => expect(load).toHaveBeenCalledTimes(2));
  await waitFor(() =>
    expect(screen.queryByRole("checkbox", { name: "Applies Prone" })).toBeNull(),
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByText("Edit draft"));
  expect(await screen.findByRole("checkbox", { name: "Applies Prone" })).toBeChecked();
});

it("saves the latest edit during debounce when there are no application consequences", async () => {
  const load = vi.fn().mockResolvedValue({ consequences: [] });
  const commit = vi.fn();
  render(<Harness load={load} commit={commit} />);
  fireEvent.click(screen.getByText("Edit draft"));
  fireEvent.click(screen.getByText("Edit value"));
  fireEvent.click(screen.getByText("Save immediately"));
  await waitFor(() =>
    expect(commit).toHaveBeenCalledExactlyOnceWith({ ...request, value: 2n }),
  );
});
it("shows newly changed consequences before allowing an immediate save", async () => {
  const load = vi.fn().mockImplementation(async (draft: ConditionEventInput) => ({
    consequences: [
      { kind: "direct_hp_loss", participant_key: "p", amount: draft.value ?? 1n },
    ],
  }));
  const commit = vi.fn();
  render(<Harness load={load} commit={commit} />);
  fireEvent.click(screen.getByText("Edit draft"));
  await screen.findByRole("checkbox", { name: "Lose 1 HP" });
  fireEvent.click(screen.getByText("Edit value"));
  fireEvent.click(screen.getByText("Save immediately"));
  await screen.findByRole("checkbox", { name: "Lose 2 HP" });
  expect(commit).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText("Save immediately"));
  await waitFor(() =>
    expect(commit).toHaveBeenCalledExactlyOnceWith({ ...request, value: 2n }),
  );
});

it("retains the same HP preview and checkbox while the next value loads, without enabling stale Apply", async () => {
  let resolveNext!: (
    preview: import("../../generated/atlas").ConditionEventPreviewView,
  ) => void;
  const first = {
    encounter_key: "encounter",
    encounter_revision: 3n,
    consequences: [
      { kind: "direct_hp_loss" as const, participant_key: "p", amount: 1n },
    ],
    hp_change: {
      current_before: 20n,
      current_after: 19n,
      maximum_before: 20n,
      maximum_after: 19n,
    },
  };
  const load = vi
    .fn()
    .mockResolvedValueOnce(first)
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveNext = resolve;
        }),
    );
  render(<Harness load={load} commit={vi.fn()} />);
  fireEvent.click(screen.getByText("Edit draft"));
  const region = await screen.findByRole("region", { name: "HP change" });
  const checkbox = await screen.findByRole("checkbox", { name: "Lose 1 HP" });
  fireEvent.click(checkbox);
  fireEvent.click(screen.getByText("Edit value"));
  expect(screen.getByRole("region", { name: "HP change" })).toBe(region);
  expect(checkbox).not.toBeChecked();
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  await waitFor(() => expect(load).toHaveBeenCalledTimes(2));
  resolveNext({
    ...first,
    consequences: [{ kind: "direct_hp_loss", participant_key: "p", amount: 2n }],
    hp_change: { ...first.hp_change, current_after: 18n, maximum_after: 18n },
  });
  await screen.findByRole("checkbox", { name: "Lose 2 HP" });
  expect(screen.getByRole("region", { name: "HP change" })).toBe(region);
  expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled();
});
