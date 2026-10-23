/** Restore focus lost when an anchored popup hides; respect a deliberate move. */
export function restoreOverlayFocus(
  trigger: HTMLElement | null,
  closingContent: HTMLElement | null | undefined,
) {
  const previousFocus = trigger?.ownerDocument.activeElement;
  const focusWasInside = !!previousFocus && !!closingContent?.contains(previousFocus);
  requestAnimationFrame(() => {
    const active = trigger?.ownerDocument.activeElement;
    if (
      trigger?.isConnected &&
      (active === trigger.ownerDocument.body ||
        (focusWasInside && active === previousFocus))
    )
      trigger.focus({ preventScroll: true });
  });
}
