import { act, render, screen } from "@testing-library/react";
import { useEffect } from "react";
import { useDicePopoverFocus } from "./useDicePopoverFocus";

function Harness({
  open,
  transition,
}: {
  open: boolean;
  transition: (callback: (visible: boolean) => void) => void;
}) {
  const { trigger, content, afterOpenChange } = useDicePopoverFocus(open);
  useEffect(() => transition(afterOpenChange), [transition, afterOpenChange]);
  return (
    <>
      <button ref={trigger}>Opener</button>
      <div ref={content} tabIndex={-1} role="dialog">
        <input aria-label="Manual total" />
      </div>
      <button>Outside destination</button>
    </>
  );
}

it("does not steal a field or nested-portal choice when the opening transition finishes", () => {
  const transition = vi.fn();
  render(<Harness open transition={transition} />);
  const finish = transition.mock.calls[0][0] as (visible: boolean) => void;
  const field = screen.getByLabelText("Manual total");
  act(() => field.focus());
  act(() => finish(true));
  expect(field).toHaveFocus();
  // A nested portal is outside the popup DOM just like this external control.
  const outside = screen.getByRole("button", { name: "Outside destination" });
  act(() => outside.focus());
  act(() => finish(true));
  expect(outside).toHaveFocus();
});

it("preserves an outside-click focus destination during close and its delayed transition", () => {
  const transition = vi.fn();
  const { rerender } = render(<Harness open transition={transition} />);
  const finish = transition.mock.calls[0][0] as (visible: boolean) => void;
  const outside = screen.getByRole("button", { name: "Outside destination" });
  act(() => outside.focus());
  rerender(<Harness open={false} transition={transition} />);
  act(() => finish(false));
  expect(outside).toHaveFocus();
});

it("restores its opener when closing while focus remains in the result", () => {
  const transition = vi.fn();
  const { rerender } = render(<Harness open transition={transition} />);
  act(() => screen.getByLabelText("Manual total").focus());
  rerender(<Harness open={false} transition={transition} />);
  expect(screen.getByRole("button", { name: "Opener" })).toHaveFocus();
});

it("ignores an obsolete opening callback after the popup has closed", () => {
  const transition = vi.fn();
  const { rerender } = render(<Harness open transition={transition} />);
  const finish = transition.mock.calls[0][0] as (visible: boolean) => void;
  rerender(<Harness open={false} transition={transition} />);
  act(() => finish(true));
  expect(screen.getByRole("button", { name: "Opener" })).toHaveFocus();
});
