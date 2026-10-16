import { useLayoutEffect, useRef, type FocusEvent, type RefObject } from "react";

/** Keep keyboard reading inside a preview when following replaces its focused content. */
export function usePreviewContentFocus(
  open: boolean,
  target?: RefObject<HTMLElement | null>,
) {
  const localRef = useRef<HTMLElement>(null);
  const ref = target ?? localRef;
  const focused = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => {
    if (!open) {
      focused.current = null;
      return;
    }
    const container = ref.current;
    if (
      container &&
      focused.current &&
      !focused.current.isConnected &&
      container.ownerDocument.activeElement === container.ownerDocument.body
    ) {
      focused.current = null;
      container.focus({ preventScroll: true });
    }
  });
  return {
    ref,
    tabIndex: -1,
    onFocusCapture(event: FocusEvent<HTMLElement>) {
      // React focus events from portal modals can bubble through this owner.
      if (event.currentTarget.contains(event.target)) focused.current = event.target;
    },
    onBlurCapture(event: FocusEvent<HTMLElement>) {
      // An intentional blur/move emits blur while the element is connected;
      // removing a focused DOM node does not. Respect moves to body as well.
      if (event.target.isConnected && event.currentTarget.contains(event.target))
        focused.current = null;
    },
  };
}
