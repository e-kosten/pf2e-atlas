import { act, fireEvent, render, screen } from "@testing-library/react";
import { EditableValue } from "./EditableValue";

it.each([false, true])(
  "restores focus when the saved value becomes enabled unless focus moved elsewhere (%s)",
  async (moveFocus) => {
    let finishSave!: (refreshed: boolean) => void;
    const onCommit = vi.fn(
      () => new Promise<boolean>((resolve) => (finishSave = resolve)),
    );
    const view = (disabled: boolean) => (
      <>
        <EditableValue
          label="Name"
          value="Goblin"
          disabled={disabled}
          onCommit={onCommit}
        />
        <input aria-label="Other field" />
      </>
    );
    const { rerender } = render(view(false));
    fireEvent.click(screen.getByRole("button", { name: "Edit Name" }));
    const input = screen.getByRole("textbox", { name: "Name" });
    fireEvent.change(input, { target: { value: "Captain" } });
    fireEvent.keyDown(input, { key: "Enter" });
    rerender(view(true));
    await act(async () => finishSave(true));
    const trigger = screen.getByRole("button", { name: "Edit Name" });
    expect(trigger).toBeDisabled();
    expect(trigger).not.toHaveFocus();
    const other = screen.getByRole("textbox", { name: "Other field" });
    if (moveFocus) other.focus();
    rerender(view(false));
    expect(moveFocus ? other : trigger).toHaveFocus();
    expect(onCommit).toHaveBeenCalledExactlyOnceWith("Captain");
  },
);
