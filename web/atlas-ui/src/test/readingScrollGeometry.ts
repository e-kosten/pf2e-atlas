/** Fixed jsdom layout for a real reading host and its landing target. */
export function readingScrollGeometry(
  host: HTMLElement,
  targetSelector = "[data-atlas-activity-anchor]",
) {
  const root = document.documentElement;
  const priorOuterScroll = root.scrollTop;
  root.scrollTop = 0;
  host.style.overflowY = "auto";
  host.style.overflowX = "auto";
  host.style.border = "0px solid transparent";
  for (const [property, value] of Object.entries({
    clientHeight: 300,
    offsetHeight: 300,
    scrollHeight: 3000,
    clientWidth: 500,
    offsetWidth: 500,
    scrollWidth: 500,
  }))
    Object.defineProperty(host, property, { configurable: true, value });
  const originalRect = HTMLElement.prototype.getBoundingClientRect;
  const rect = vi
    .spyOn(HTMLElement.prototype, "getBoundingClientRect")
    .mockImplementation(function (this: HTMLElement) {
      if (this === host) return new DOMRect(0, 50, 500, 300);
      if (host.contains(this) && this.matches(targetSelector))
        return new DOMRect(0, 150 - host.scrollTop, 400, 30);
      return originalRect.call(this);
    });
  // Native scrollIntoView affects every scrollable ancestor. This sentinel
  // distinguishes that old behavior from a boundary-scoped library action.
  const originalScroll = HTMLElement.prototype.scrollIntoView;
  HTMLElement.prototype.scrollIntoView = function () {
    if (host.contains(this)) {
      host.scrollTop = 100;
      root.scrollTop = 719;
    }
  };
  return {
    restore() {
      rect.mockRestore();
      HTMLElement.prototype.scrollIntoView = originalScroll;
      root.scrollTop = priorOuterScroll;
    },
  };
}
