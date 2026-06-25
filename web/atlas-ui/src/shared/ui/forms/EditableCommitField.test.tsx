import { fireEvent, render, screen } from "@testing-library/react";
import { EditableCommitField } from "./EditableCommitField";

describe("EditableCommitField", () => {
  it("commits edited values on blur", () => {
    const onCommit = vi.fn();
    render(
      <EditableCommitField ariaLabel="Initiative" onCommit={onCommit} value="12" />,
    );

    fireEvent.change(screen.getByLabelText("Initiative"), {
      target: { value: "15" },
    });
    fireEvent.blur(screen.getByLabelText("Initiative"));

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

    fireEvent.change(screen.getByLabelText("HP"), {
      target: { value: "35" },
    });
    fireEvent.keyDown(screen.getByLabelText("HP"), { key: "Enter" });

    expect(onCommit).toHaveBeenCalledWith("35");
    expect(onKeyDown).not.toHaveBeenCalled();
  });

  it("reverts on Escape without committing", () => {
    const onCommit = vi.fn();
    render(<EditableCommitField ariaLabel="Name" onCommit={onCommit} value="Goblin" />);

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Goblin 2" },
    });
    fireEvent.keyDown(screen.getByLabelText("Name"), { key: "Escape" });

    expect(screen.getByLabelText("Name")).toHaveValue("Goblin");
    expect(onCommit).not.toHaveBeenCalled();
  });

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
