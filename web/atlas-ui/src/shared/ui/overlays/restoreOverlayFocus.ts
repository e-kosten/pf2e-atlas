/** Restore focus when local content hides or disappears; respect a deliberate move. */
export function restoreOverlayFocus(
  trigger: HTMLElement | null | (() => HTMLElement | null),
  closingContent: HTMLElement | null | undefined,
) {
  const getTrigger = typeof trigger === "function" ? trigger : () => trigger;
  const document = closingContent?.ownerDocument ?? getTrigger()?.ownerDocument;
  const previousFocus = document?.activeElement;
  const focusWasInside = !!previousFocus && !!closingContent?.contains(previousFocus);
  requestAnimationFrame(() => {
    const trigger = getTrigger();
    const active = trigger?.ownerDocument.activeElement;
    if (
      trigger?.isConnected &&
      (active === trigger.ownerDocument.body ||
        (focusWasInside && active === previousFocus))
    )
      trigger.focus({ preventScroll: true });
  });
}
