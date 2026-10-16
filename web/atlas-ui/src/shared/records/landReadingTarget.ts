import scrollIntoView from "scroll-into-view-if-needed";

/** Land inside the actual reader, without moving the page or overlay ancestors. */
export function landReadingTarget(target: HTMLElement, boundary: HTMLElement) {
  if (!boundary.contains(target)) return;
  scrollIntoView(target, {
    boundary,
    block: "start",
    inline: "nearest",
    // Synchronous application keeps first landing and history restoration in
    // one reading transition, independent of CSS/browser smooth-scroll policy.
    behavior: (actions) => {
      for (const { el, top, left } of actions) {
        el.scrollTop = top;
        el.scrollLeft = left;
      }
    },
  });
}
