import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterConditionValue } from "./EncounterConditionValue";
it("rejects blank, fractional and zero values without applying", async () => {
  const onCommit = vi.fn();
  render(
    <EncounterConditionValue
      name="Frightened"
      value={2n}
      disabled={false}
      onCommit={onCommit}
    />,
  );
  const input = screen.getByRole("spinbutton", { name: "Frightened value" });
  for (const value of ["", "1.5", "0"]) {
    fireEvent.change(input, { target: { value } });
    fireEvent.keyDown(input, { key: "Enter" });
    await waitFor(() =>
      expect(screen.getByText("Enter a whole number, one or greater.")).toBeVisible(),
    );
  }
  expect(onCommit).not.toHaveBeenCalled();
});
it("uses Remove at one and never commits zero", async () => {
  const onCommit = vi.fn();
  const onClear = vi.fn().mockResolvedValue(true);
  render(
    <EncounterConditionValue
      name="Frightened"
      value={1n}
      disabled={false}
      onCommit={onCommit}
      onClear={onClear}
    />,
  );
  expect(
    screen.queryByRole("button", { name: "Decrease Frightened value" }),
  ).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Remove Frightened" }));
  await waitFor(() => expect(onClear).toHaveBeenCalledOnce());
  expect(onCommit).not.toHaveBeenCalled();
});
it("steps positively once and keeps a rejected draft", async () => {
  const onCommit = vi.fn().mockRejectedValue(new Error("Unavailable"));
  render(
    <EncounterConditionValue
      name="Frightened"
      value={1n}
      disabled={false}
      onCommit={onCommit}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Increase Frightened value" }));
  await waitFor(() => expect(onCommit).toHaveBeenCalledExactlyOnceWith(2n));
  expect(screen.getByRole("spinbutton")).toHaveValue("2");
});

it("keeps its focused draft read-only while a guarded write is pending", async () => {
  let finish!: (value: boolean) => void;
  const onCommit = vi.fn(
    () =>
      new Promise<boolean>((resolve) => {
        finish = resolve;
      }),
  );
  const props = { name: "Frightened", value: 1n, disabled: false, onCommit };
  const { rerender } = render(<EncounterConditionValue {...props} />);
  const input = screen.getByRole("spinbutton", { name: "Frightened value" });
  act(() => input.focus());
  fireEvent.change(input, { target: { value: "2" } });
  fireEvent.keyDown(input, { key: "Enter" });
  rerender(<EncounterConditionValue {...props} disabled />);
  expect(input).toHaveFocus();
  expect(input).not.toBeDisabled();
  expect(input).toHaveAttribute("readonly");
  fireEvent.keyDown(input, { key: "Enter" });
  expect(onCommit).toHaveBeenCalledExactlyOnceWith(2n);
  await act(async () => finish(true));
  rerender(<EncounterConditionValue {...props} value={2n} />);
  expect(input).toHaveFocus();
  expect(input).toHaveValue("2");
  expect(input).not.toHaveAttribute("readonly");
});
