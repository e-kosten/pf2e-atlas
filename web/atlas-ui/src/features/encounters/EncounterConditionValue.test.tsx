import { fireEvent, render, screen, waitFor } from "@testing-library/react";
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
