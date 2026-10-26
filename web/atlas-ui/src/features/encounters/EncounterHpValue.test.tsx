import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterHpValue } from "./EncounterHpValue";

function mount() {
  let resolve!: (value: boolean) => void;
  const onCommit = vi.fn().mockImplementation(
    () =>
      new Promise<boolean>((finish) => {
        resolve = finish;
      }),
  );
  render(
    <>
      <EncounterHpValue
        label="Current HP"
        value={42n}
        disabled={false}
        onCommit={onCommit}
      />
      <button>Next action</button>
    </>,
  );
  const open = () => {
    fireEvent.click(screen.getByRole("button", { name: "Edit Current HP" }));
    return screen.getByRole("textbox", { name: "Current HP" });
  };
  return { open, onCommit, finish: () => act(async () => resolve(true)) };
}

it("returns focus to the value after Escape without committing", () => {
  const { open, onCommit } = mount();
  const input = open();
  expect(input).toHaveFocus();
  fireEvent.change(input, { target: { value: "43" } });
  fireEvent.keyDown(input, { key: "Escape" });
  expect(onCommit).not.toHaveBeenCalled();
  expect(screen.getByRole("button", { name: "Edit Current HP" })).toHaveFocus();
});

it("keeps focus in a pending Enter edit, then returns it to the value", async () => {
  const { open, onCommit, finish } = mount();
  const input = open();
  fireEvent.change(input, { target: { value: "43" } });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(onCommit).toHaveBeenCalledExactlyOnceWith("43");
  expect(input).toHaveFocus();
  await finish();
  expect(screen.getByRole("button", { name: "Edit Current HP" })).toHaveFocus();
});

it("keeps a rejected formula focused and available for correction", async () => {
  const { open, onCommit } = mount();
  onCommit.mockRejectedValueOnce(new Error("Invalid HP formula"));
  const input = open();
  fireEvent.change(input, { target: { value: "300-" } });
  fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() => expect(input).not.toHaveAttribute("readonly"));
  expect(onCommit).toHaveBeenCalledExactlyOnceWith("300-");
  expect(input).toHaveValue("300-");
  expect(input).toHaveFocus();
});

it.each([false, true])(
  "preserves a move to another control while an edit finishes (submit first=%s)",
  async (submitFirst) => {
    const { open, onCommit, finish } = mount();
    const input = open();
    fireEvent.change(input, { target: { value: "43" } });
    if (submitFirst) fireEvent.keyDown(input, { key: "Enter" });
    const next = screen.getByRole("button", { name: "Next action" });
    act(() => next.focus());
    expect(onCommit).toHaveBeenCalledExactlyOnceWith("43");
    await finish();
    await waitFor(() => expect(screen.queryByRole("textbox")).toBeNull());
    expect(next).toHaveFocus();
  },
);
