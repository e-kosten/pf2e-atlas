import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { PreviewPopover, usePreviewPopoverClose } from "./PreviewPopover";

describe("PreviewPopover", () => {
  it("opens once for pointerdown-focus-click and closes outside", async () => {
    const onOpenChange = vi.fn();
    renderPreview(onOpenChange);
    const trigger = screen.getByRole("button", { name: "Preview trigger" });

    fireEvent.pointerDown(trigger);
    fireEvent.focus(trigger);
    fireEvent.click(trigger);

    expect(screen.getByRole("dialog", { name: "Preview details" })).toBeInTheDocument();
    expect(document.querySelector(".preview-popover")).toHaveClass(
      "ant-popover-placement-bottom",
    );
    expect(onOpenChange).toHaveBeenCalledTimes(1);
    expect(onOpenChange).toHaveBeenLastCalledWith(true);

    fireEvent.mouseDown(document.body);
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Preview details" }),
      ).not.toBeInTheDocument(),
    );
  });

  it("supports keyboard activation and restores trigger focus after Escape", async () => {
    renderPreview();
    const trigger = screen.getByRole("button", { name: "Preview trigger" });
    trigger.focus();

    fireEvent.keyDown(trigger, { key: "Enter" });
    fireEvent.click(trigger);
    expect(screen.getByRole("dialog", { name: "Preview details" })).toBeInTheDocument();

    const close = screen.getByRole("button", { name: "Close preview" });
    close.focus();
    fireEvent.keyDown(close, { key: "Escape" });

    await waitFor(() => expect(document.activeElement).toBe(trigger));
    expect(
      screen.queryByRole("dialog", { name: "Preview details" }),
    ).not.toBeInTheDocument();
  });
});

function renderPreview(onOpenChange = vi.fn()) {
  return render(
    <PreviewPopover
      actions={<ClosePreviewButton />}
      ariaLabel="Preview details"
      content={<p>Popover content</p>}
      onOpenChange={onOpenChange}
      title="Preview"
    >
      {(open) => (
        <button aria-expanded={open} type="button">
          Preview trigger
        </button>
      )}
    </PreviewPopover>,
  );
}

function ClosePreviewButton() {
  const close = usePreviewPopoverClose();
  return (
    <button onClick={close} type="button">
      Close preview
    </button>
  );
}
