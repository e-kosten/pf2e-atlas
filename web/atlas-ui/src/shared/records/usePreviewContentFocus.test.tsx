import { act, render, screen } from "@testing-library/react";
import { createPortal } from "react-dom";
import { usePreviewContentFocus } from "./usePreviewContentFocus";

function Preview({
  open = true,
  trigger = true,
  modal = false,
}: {
  open?: boolean;
  trigger?: boolean;
  modal?: boolean;
}) {
  const focus = usePreviewContentFocus(open);
  return (
    <section {...focus} aria-label="Preview content">
      {trigger ? <button>Follow</button> : <h2>New content</h2>}
      {modal &&
        createPortal(
          <div role="dialog" aria-modal="true">
            <button>Modal action</button>
          </div>,
          document.body,
        )}
    </section>
  );
}

it("preserves portal-modal focus and never remembers its removed control as preview content", () => {
  const { rerender } = render(<Preview modal />);
  act(() => screen.getByRole("button", { name: "Follow" }).focus());
  const action = screen.getByRole("button", { name: "Modal action" });
  act(() => action.focus());
  rerender(<Preview trigger={false} modal />);
  expect(action).toHaveFocus();
  rerender(<Preview trigger={false} />);
  expect(document.body).toHaveFocus();
});

it("does not hand off lost content focus while the preview is closing", () => {
  const { rerender } = render(<Preview />);
  act(() => screen.getByRole("button", { name: "Follow" }).focus());
  rerender(<Preview open={false} trigger={false} />);
  expect(document.body).toHaveFocus();
});

it("does not introduce initial autofocus or focus on an unfocused content update", () => {
  const { rerender } = render(<Preview />);
  expect(document.body).toHaveFocus();
  rerender(<Preview trigger={false} />);
  expect(document.body).toHaveFocus();
});
