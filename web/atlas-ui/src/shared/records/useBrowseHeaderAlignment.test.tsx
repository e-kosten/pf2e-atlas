import { act, render, screen } from "@testing-library/react";
import { useRef } from "react";
import { BrowsePaneHeader } from "./BrowsePaneHeader";
import { useBrowseHeaderAlignment } from "./useBrowseHeaderAlignment";

function Headers({ wide = true }: { wide?: boolean }) {
  const host = useRef<HTMLDivElement>(null);
  useBrowseHeaderAlignment(host, wide, true);
  return (
    <div ref={host} data-testid="browse">
      <BrowsePaneHeader className="reference-results__header">
        <input data-header="results" aria-label="Current page" defaultValue="1" />
      </BrowsePaneHeader>
      <BrowsePaneHeader className="reference-reader__header">
        <span data-header="reader">Preview</span>
      </BrowsePaneHeader>
    </div>
  );
}

it("aligns natural header rows through wrapping and shrinking without remounting controls, and releases alignment on phone", () => {
  const originalObserver = globalThis.ResizeObserver;
  let notify!: ResizeObserverCallback;
  const disconnect = vi.fn();
  const heights = { results: 50, reader: 54 };
  vi.stubGlobal(
    "ResizeObserver",
    class {
      constructor(callback: ResizeObserverCallback) {
        notify = callback;
      }
      observe() {}
      disconnect = disconnect;
    },
  );
  const measure = vi
    .spyOn(HTMLElement.prototype, "getBoundingClientRect")
    .mockImplementation(function (this: HTMLElement) {
      const which = this.classList.contains("browse-pane-header__content")
        ? this.querySelector<HTMLElement>("[data-header]")?.dataset.header
        : undefined;
      return new DOMRect(0, 0, 300, which ? heights[which as keyof typeof heights] : 0);
    });
  const { rerender, unmount } = render(<Headers />);
  try {
    const host = screen.getByTestId("browse");
    const input = screen.getByRole("textbox", { name: "Current page" });
    input.focus();
    expect(host.style.getPropertyValue("--browse-header-height")).toBe("55px");

    heights.results = 86;
    act(() => notify([], {} as ResizeObserver));
    expect(host.style.getPropertyValue("--browse-header-height")).toBe("87px");

    heights.results = 50;
    heights.reader = 50;
    act(() => notify([], {} as ResizeObserver));
    expect(host.style.getPropertyValue("--browse-header-height")).toBe("51px");
    expect(screen.getByRole("textbox", { name: "Current page" })).toBe(input);
    expect(input).toHaveFocus();

    rerender(<Headers wide={false} />);
    expect(host.style.getPropertyValue("--browse-header-height")).toBe("");
    expect(disconnect).toHaveBeenCalledOnce();
  } finally {
    unmount();
    measure.mockRestore();
    vi.stubGlobal("ResizeObserver", originalObserver);
  }
});
