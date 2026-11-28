import { useLayoutEffect, useRef, type RefObject } from "react";

const motion = { duration: 160, easing: "ease-out" };
const reducedMotion = "(prefers-reduced-motion: reduce)";

export function useSearchToolbarMotion(
  enabled: boolean,
  fieldIds: readonly string[],
  toolbar: RefObject<HTMLElement | null>,
  chrome: RefObject<HTMLDivElement | null>,
  quickControls: RefObject<HTMLDivElement | null>,
) {
  const previousFields = useRef<readonly string[]>([]);

  useLayoutEffect(() => {
    const frame = toolbar.current;
    const content = chrome.current;
    if (!enabled || !frame?.animate || !content) return;

    const preference = window.matchMedia(reducedMotion);
    let settled = frame.getBoundingClientRect();
    let resize: Animation | undefined;
    const observer = new ResizeObserver(() => {
      const start =
        resize?.playState === "running" ? frame.getBoundingClientRect() : settled;
      resize?.cancel();
      const end = frame.getBoundingClientRect();
      settled = end;
      if (
        preference.matches ||
        !start.height ||
        !end.height ||
        start.width !== end.width
      )
        return;
      if (Math.abs(start.height - end.height) < 1) return;
      // Clip the newly wrapped controls while the results pane makes room for them.
      resize = frame.animate(
        [start.height, end.height].map((height) => ({
          height: `${height}px`,
          clipPath: "inset(0)",
        })),
        motion,
      );
    });
    observer.observe(content);
    const stop = () => resize?.cancel();
    preference.addEventListener("change", stop);
    return () => {
      observer.disconnect();
      preference.removeEventListener("change", stop);
      stop();
    };
  }, [chrome, enabled, toolbar]);

  useLayoutEffect(() => {
    const previous = previousFields.current;
    previousFields.current = enabled ? fieldIds : [];
    if (!enabled || !previous.length || !fieldIds.some((id) => !previous.includes(id)))
      return;
    if (window.matchMedia(reducedMotion).matches) return;
    for (const control of quickControls.current?.querySelectorAll<HTMLElement>(
      "[data-quick-field]",
    ) ?? []) {
      const id = control.dataset.quickField;
      if (id && !previous.includes(id))
        control.animate?.([{ opacity: 0.55 }, { opacity: 1 }], motion);
    }
  }, [enabled, fieldIds, quickControls]);
}
