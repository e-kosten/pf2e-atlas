import { act, fireEvent, render, screen } from "@testing-library/react";
import { EditableCommitField } from "./EditableCommitField";

describe("EditableCommitField", () => {
  it("commits edited values on blur", () => {
    const onCommit = vi.fn();
    render(
      <EditableCommitField ariaLabel="Initiative" onCommit={onCommit} value="12" />,
    );

    const input = screen.getByLabelText("Initiative");
    act(() => input.focus());
    fireEvent.change(input, {
      target: { value: "15" },
    });
    act(() => input.blur());

    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onCommit).toHaveBeenCalledWith("15");
  });

  it("commits on Enter and stops row keyboard handlers when requested", () => {
    const onCommit = vi.fn();
    const onKeyDown = vi.fn();
    render(
      <div aria-label="Test row" onKeyDown={onKeyDown} role="button" tabIndex={0}>
        <EditableCommitField
          ariaLabel="HP"
          onCommit={onCommit}
          stopPropagation
          value="40"
        />
      </div>,
    );

    const input = screen.getByLabelText("HP");
    act(() => input.focus());
    fireEvent.change(input, {
      target: { value: "35" },
    });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onCommit).toHaveBeenCalledWith("35");
    expect(onKeyDown).not.toHaveBeenCalled();
    expect(input).not.toHaveFocus();
  });

  it("reverts on Escape without committing", () => {
    const onCommit = vi.fn();
    render(<EditableCommitField ariaLabel="Name" onCommit={onCommit} value="Goblin" />);

    const input = screen.getByLabelText("Name");
    act(() => input.focus());
    fireEvent.change(input, {
      target: { value: "Goblin 2" },
    });
    fireEvent.keyDown(input, { key: "Escape" });

    expect(screen.getByLabelText("Name")).toHaveValue("Goblin");
    expect(onCommit).not.toHaveBeenCalled();
    expect(input).not.toHaveFocus();
  });

  it.each(["Enter", "Escape", "blur"])(
    "does not commit an unchanged value on %s",
    (completion) => {
      const onCommit = vi.fn();
      render(<EditableCommitField ariaLabel="HP" onCommit={onCommit} value="40" />);
      const input = screen.getByLabelText("HP");
      act(() => input.focus());
      if (completion === "blur") {
        act(() => input.blur());
      } else {
        fireEvent.keyDown(input, { key: completion });
      }
      expect(onCommit).not.toHaveBeenCalled();
    },
  );

  it.each(["Enter", "Escape"])(
    "can edit again after finishing with %s",
    (completion) => {
      const onCommit = vi.fn();
      render(<EditableCommitField ariaLabel="HP" onCommit={onCommit} value="40" />);
      const input = screen.getByLabelText("HP");
      act(() => input.focus());
      fireEvent.change(input, { target: { value: "35" } });
      fireEvent.keyDown(input, { key: completion });
      onCommit.mockClear();

      act(() => input.focus());
      fireEvent.change(input, { target: { value: "30" } });
      act(() => input.blur());

      expect(onCommit).toHaveBeenCalledExactlyOnceWith("30");
    },
  );

  it("updates the draft value when the committed value changes", () => {
    const onCommit = vi.fn();
    const { rerender } = render(
      <EditableCommitField ariaLabel="Name" onCommit={onCommit} value="Goblin" />,
    );

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Goblin 2" },
    });
    rerender(
      <EditableCommitField ariaLabel="Name" onCommit={onCommit} value="Goblin 3" />,
    );

    expect(screen.getByLabelText("Name")).toHaveValue("Goblin 3");
  });
});
