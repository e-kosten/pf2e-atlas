import { useLayoutEffect, type RefObject } from "react";

/** Align sibling headers, including wrapped controls, without fixing their height. */
export function useBrowseHeaderAlignment(
  root: RefObject<HTMLDivElement | null>,
  enabled: boolean,
  hasResultsHeader: boolean,
) {
  useLayoutEffect(() => {
    const host = root.current;
    if (!host || !enabled || !hasResultsHeader) return;
    const rows = Array.from(
      host.querySelectorAll<HTMLElement>(".browse-pane-header__content"),
    );
    const align = () => {
      // Measure the natural inner rows, never their stretched outer headers.
      // This lets both headers shrink again after controls stop wrapping.
      const height = Math.max(
        0,
        ...rows.map((row) => row.getBoundingClientRect().height),
      );
      if (height > 0)
        host.style.setProperty("--browse-header-height", `${Math.ceil(height) + 1}px`);
    };
    const observer = new ResizeObserver(align);
    rows.forEach((row) => observer.observe(row));
    align();
    return () => {
      observer.disconnect();
      host.style.removeProperty("--browse-header-height");
    };
  }, [root, enabled, hasResultsHeader]);
}
