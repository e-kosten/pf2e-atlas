import { fireEvent, render, screen } from "@testing-library/react";
import { ResizablePaneGroup } from "./PaneLayout";

describe("ResizablePaneGroup", () => {
  it("resizes a pane using the configured delta direction and minimum width", () => {
    const { container } = render(<ResizableFixture />);
    const group = paneGroup(container);
    const handle = resizeHandle();
    installPointerCapture(handle);

    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 420px)",
    );

    fireEvent.pointerDown(handle, { clientX: 0, pointerId: 1 });
    fireEvent.pointerMove(handle, { clientX: 40, pointerId: 1 });
    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 380px)",
    );

    fireEvent.pointerMove(handle, { clientX: 200, pointerId: 1 });
    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 320px)",
    );

    fireEvent.pointerUp(handle, { pointerId: 1 });
  });

  it("stops resizing after pointer cancellation or lost capture", () => {
    const { container, rerender } = render(<ResizableFixture />);
    const group = paneGroup(container);
    const handle = resizeHandle();
    installPointerCapture(handle);

    fireEvent.pointerDown(handle, { clientX: 0, pointerId: 1 });
    fireEvent.pointerCancel(handle, { clientX: 40, pointerId: 1 });
    fireEvent.pointerMove(handle, { clientX: 100, pointerId: 1 });
    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 420px)",
    );

    rerender(<ResizableFixture />);
    const nextGroup = paneGroup(container);
    const nextHandle = resizeHandle();
    installPointerCapture(nextHandle);

    fireEvent.pointerDown(nextHandle, { clientX: 0, pointerId: 2 });
    fireEvent.lostPointerCapture(nextHandle, { clientX: 40, pointerId: 2 });
    fireEvent.pointerMove(nextHandle, { clientX: 100, pointerId: 2 });
    expect(nextGroup.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 420px)",
    );
  });

  it("does not resize from a disabled handle", () => {
    const { container } = render(<ResizableFixture disabled />);
    const group = paneGroup(container);
    const handle = resizeHandle();
    installPointerCapture(handle);

    fireEvent.pointerDown(handle, { clientX: 0, pointerId: 1 });
    fireEvent.pointerMove(handle, { clientX: 200, pointerId: 1 });

    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 420px)",
    );
  });

  it("resizes a pane from the keyboard", () => {
    const { container } = render(<ResizableFixture />);
    const group = paneGroup(container);
    const handle = resizeHandle();

    expect(handle).toHaveAttribute("tabindex", "0");
    expect(handle).toHaveAttribute("aria-valuemin", "320");
    expect(handle).toHaveAttribute("aria-valuenow", "420");
    expect(handle).toHaveAttribute("aria-valuetext", "420 pixels");

    fireEvent.keyDown(handle, { key: "ArrowRight" });
    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 396px)",
    );
    expect(handle).toHaveAttribute("aria-valuenow", "396");

    fireEvent.keyDown(handle, { key: "ArrowLeft", shiftKey: true });
    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 492px)",
    );
    expect(handle).toHaveAttribute("aria-valuenow", "492");

    fireEvent.keyDown(handle, { key: "Home" });
    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 320px)",
    );
    expect(handle).toHaveAttribute("aria-valuenow", "320");

    fireEvent.keyDown(handle, { key: "Enter" });
    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 420px)",
    );
    expect(handle).toHaveAttribute("aria-valuenow", "420");
  });

  it("ignores keyboard resizing from a disabled handle", () => {
    const { container } = render(<ResizableFixture disabled />);
    const group = paneGroup(container);
    const handle = resizeHandle();

    expect(handle).toHaveAttribute("tabindex", "-1");

    fireEvent.keyDown(handle, { key: "ArrowRight" });
    fireEvent.keyDown(handle, { key: "Home" });

    expect(group.style.gridTemplateColumns).toBe(
      "minmax(0, 1fr) var(--panel-gap) minmax(0, 420px)",
    );
    expect(handle).toHaveAttribute("aria-valuenow", "420");
  });
});

function ResizableFixture({ disabled = false }: { disabled?: boolean }) {
  return (
    <ResizablePaneGroup
      className="test-pane-group"
      widthSpecs={{ preview: { defaultWidth: 420, minWidth: 320 } }}
      items={(widths) => [
        {
          kind: "pane",
          key: "primary",
          column: "minmax(0, 1fr)",
          content: <section>Primary</section>,
        },
        {
          kind: "handle",
          key: "resize",
          disabled,
          label: "Resize preview",
          resizePane: "preview",
          deltaMultiplier: -1,
        },
        {
          kind: "pane",
          key: "preview",
          column: `minmax(0, ${widths.preview}px)`,
          content: <section>Preview</section>,
        },
      ]}
    />
  );
}

function paneGroup(container: HTMLElement): HTMLElement {
  const group = container.querySelector(".test-pane-group");
  if (!(group instanceof HTMLElement)) {
    throw new Error("pane group was not rendered");
  }
  return group;
}

function resizeHandle(): HTMLElement {
  return screen.getByRole("separator", { name: "Resize preview" });
}

function installPointerCapture(element: HTMLElement) {
  element.setPointerCapture = vi.fn();
  element.releasePointerCapture = vi.fn();
  element.hasPointerCapture = vi.fn(() => true);
}
