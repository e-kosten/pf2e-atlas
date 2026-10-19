import { useCallback, useLayoutEffect, useRef } from "react";

/** Enter the result surface without stealing focus again when its async result arrives. */
export function useDicePopoverFocus(open: boolean) {
  const trigger = useRef<HTMLButtonElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const wasOpen = useRef(open);
  const moveFocus = useCallback((visible: boolean) => {
    if (visible !== wasOpen.current) return;
    const document = trigger.current?.ownerDocument;
    if (!document) return;
    const active = document.activeElement;
    // The overlay transition can finish after a user has already chosen a field,
    // nested portal control, or outside destination. Never replace that choice.
    if (visible) {
      if (active === document.body || active === trigger.current)
        content.current?.focus({ preventScroll: true });
    } else if (
      active === document.body ||
      (active && content.current?.contains(active))
    ) {
      trigger.current?.focus({ preventScroll: true });
    }
  }, []);
  const contentRef = useCallback(
    (node: HTMLDivElement | null) => {
      content.current = node;
      if (open) moveFocus(true);
    },
    [open, moveFocus],
  );
  useLayoutEffect(() => {
    const previous = wasOpen.current;
    wasOpen.current = open;
    if (open || previous) moveFocus(open);
  }, [open, moveFocus]);
  return { trigger, content: contentRef, afterOpenChange: moveFocus };
}
