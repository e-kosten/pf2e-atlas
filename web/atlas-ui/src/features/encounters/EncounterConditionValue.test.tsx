import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterConditionValue } from "./EncounterConditionValue";

it("rejects cleared and fractional values visibly, then commits an integer once", async () => {
  const onCommit = vi.fn().mockResolvedValue(true);
  render(
    <EncounterConditionValue
      name="Frightened"
      value={1n}
      disabled={false}
      onCommit={onCommit}
    />,
  );
  const input = screen.getByRole("spinbutton", { name: "Frightened value" });
  for (const value of ["", "1.5"]) {
    fireEvent.change(input, { target: { value } });
    fireEvent.blur(input);
    expect(
      await screen.findByText("Enter a whole number, zero or greater."),
    ).toBeInTheDocument();
    expect(onCommit).not.toHaveBeenCalled();
  }
  fireEvent.change(input, { target: { value: "3" } });
  fireEvent.keyDown(input, { key: "Enter" });
  await waitFor(() => expect(onCommit).toHaveBeenCalledExactlyOnceWith(3n));
});

it("keeps a rejected direct edit available for correction", async () => {
  const onCommit = vi.fn().mockRejectedValue(new Error("Unavailable"));
  render(
    <EncounterConditionValue
      name="Frightened"
      value={1n}
      disabled={false}
      onCommit={onCommit}
    />,
  );
  const input = screen.getByRole("spinbutton", { name: "Frightened value" });
  fireEvent.change(input, { target: { value: "3" } });
  fireEvent.blur(input);
  await waitFor(() => expect(onCommit).toHaveBeenCalledExactlyOnceWith(3n));
  expect(input).toHaveValue("3");
});

it("commits native plus and minus steps once and disables them during pending recovery", async () => {
  const onCommit = vi.fn().mockResolvedValue(true);
  const { rerender } = render(
    <EncounterConditionValue
      name="Frightened"
      value={1n}
      disabled={false}
      onCommit={onCommit}
    />,
  );
  const plus = screen.getByRole("button", { name: "Increase Value" });
  fireEvent.mouseDown(plus);
  fireEvent.mouseUp(plus);
  await waitFor(() => expect(onCommit).toHaveBeenCalledExactlyOnceWith(2n));
  rerender(
    <EncounterConditionValue
      name="Frightened"
      value={2n}
      disabled={false}
      onCommit={onCommit}
    />,
  );
  const minus = screen.getByRole("button", { name: "Decrease Value" });
  fireEvent.mouseDown(minus);
  fireEvent.mouseUp(minus);
  await waitFor(() => expect(onCommit).toHaveBeenNthCalledWith(2, 1n));
  rerender(
    <EncounterConditionValue
      name="Frightened"
      value={1n}
      disabled
      onCommit={onCommit}
    />,
  );
  expect(
    screen.queryByRole("button", { name: "Increase Value" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "Decrease Value" }),
  ).not.toBeInTheDocument();
  expect(screen.getByRole("spinbutton", { name: "Frightened value" })).toBeDisabled();
});
