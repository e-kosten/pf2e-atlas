import { useEffect, type RefObject } from "react";

/** Focus an explicitly opened menu; cancel pending work when it closes. */
export function useMenuFocus(
  open: boolean,
  trigger: RefObject<HTMLButtonElement | null>,
  popup: RefObject<HTMLDivElement | null>,
) {
  useEffect(() => {
    if (!open) return;
    const focusMenu = () => {
      const document = trigger.current?.ownerDocument;
      if (!document) return;
      const active = document.activeElement;
      if (active !== trigger.current && active !== document.body) return;
      const item = popup.current?.querySelector<HTMLElement>(
        '[role="menuitem"]:not([aria-disabled="true"])',
      );
      item?.focus({ preventScroll: true });
      if (!item || document.activeElement !== item)
        frame = requestAnimationFrame(focusMenu);
    };
    let frame = requestAnimationFrame(focusMenu);
    return () => cancelAnimationFrame(frame);
  }, [open, trigger, popup]);
}
