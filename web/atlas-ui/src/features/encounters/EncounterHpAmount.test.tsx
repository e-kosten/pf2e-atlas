import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EncounterHpAmount } from "./EncounterHpAmount";

it("submits signed Enter once as raw Heal intent and keeps negative button inputs raw", async () => {
  let finish!: (saved: boolean) => void;
  const onAction = vi
    .fn()
    .mockImplementationOnce(
      () =>
        new Promise<boolean>((resolve) => {
          finish = resolve;
        }),
    )
    .mockResolvedValue(true);
  const { rerender } = render(
    <EncounterHpAmount disabled={false} onAction={onAction} />,
  );
  const input = screen.getByLabelText("HP change");
  input.focus();
  fireEvent.change(input, { target: { value: "-10 + 2" } });
  fireEvent.keyDown(input, { key: "Enter" });
  fireEvent.keyDown(input, { key: "Enter", repeat: true });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(onAction).toHaveBeenCalledExactlyOnceWith({ kind: "heal", amount: "-10 + 2" });
  rerender(<EncounterHpAmount disabled pending onAction={onAction} />);
  expect(input).toHaveFocus();
  expect(input).not.toBeDisabled();
  expect(input).toHaveAttribute("readonly");
  await act(async () => finish(true));
  rerender(<EncounterHpAmount disabled={false} onAction={onAction} />);
  expect(input).toHaveValue("");
  expect(input).toHaveFocus();
  fireEvent.change(input, { target: { value: "-4" } });
  fireEvent.click(screen.getByRole("button", { name: "Damage" }));
  await waitFor(() =>
    expect(onAction).toHaveBeenLastCalledWith({ kind: "damage", amount: "-4" }),
  );
});

it("keeps Enter inert in the Damage-only review control", () => {
  const onAction = vi.fn();
  render(
    <EncounterHpAmount disabled={false} actions={["damage"]} onAction={onAction} />,
  );
  const input = screen.getByLabelText("HP change");
  fireEvent.change(input, { target: { value: "10" } });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(onAction).not.toHaveBeenCalled();
});

it.each(["unconfirmed", "rejected"])(
  "retains a %s HP change and permits an explicit retry",
  async (failure) => {
    const onAction = vi.fn();
    if (failure === "unconfirmed") onAction.mockResolvedValueOnce(false);
    else onAction.mockRejectedValueOnce(new Error("Write failed"));
    onAction.mockResolvedValueOnce(true);
    render(<EncounterHpAmount disabled={false} onAction={onAction} />);
    const input = screen.getByLabelText("HP change");
    fireEvent.change(input, { target: { value: " 10 + 2 " } });
    fireEvent.click(screen.getByRole("button", { name: "Damage" }));
    await act(async () => {});
    expect(input).toHaveValue(" 10 + 2 ");
    expect(onAction).toHaveBeenCalledExactlyOnceWith({
      kind: "damage",
      amount: " 10 + 2 ",
    });
    fireEvent.click(screen.getByRole("button", { name: "Damage" }));
    await waitFor(() => expect(input).toHaveValue(""));
    expect(onAction).toHaveBeenCalledTimes(2);
    expect(onAction).toHaveBeenLastCalledWith({ kind: "damage", amount: " 10 + 2 " });
  },
);
