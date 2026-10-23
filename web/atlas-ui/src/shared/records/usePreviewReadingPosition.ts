import { useCallback, useLayoutEffect, useRef } from "react";
import type { RecordDetailView, RecordNavigationView } from "../../generated/atlas";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";
import { landReadingTarget } from "./landReadingTarget";

// The reference Drawer restores its actual scroll container after the
// addressed document arrives. Background refreshes do not restart reading.
export function usePreviewReadingPosition(
  detail: RecordDetailView | undefined,
  history: RecordPreviewContentProps["readingHistory"],
  enabled = true,
  landing?: RecordNavigationView,
) {
  const content = useRef<HTMLElement>(null);
  const enabledRef = useRef(enabled);
  const readyLanding = useRef<string | undefined>(undefined);
  useLayoutEffect(() => {
    enabledRef.current = enabled;
  }, [enabled]);
  const restoredAddress = useRef<string | null>(null);
  const address = history?.address;
  const restore = history?.restore;
  const restoringHistory = history?.restoringHistory;
  const restorePosition = useCallback(() => {
    if (!enabledRef.current || !address || !restore) return;
    const node = content.current;
    const scroll = node?.parentElement;
    if (!node || !scroll) return;
    scroll.scrollTop = restore.scroll;
    if (restore.focusHref)
      Array.from(node.querySelectorAll<HTMLAnchorElement>("a[href]"))
        .find((link) => link.getAttribute("href") === restore.focusHref)
        ?.focus({ preventScroll: true });
    restoredAddress.current = address;
  }, [address, restore]);
  const onLandingReady = useCallback(
    (target?: HTMLElement) => {
      if (!enabledRef.current) return;
      readyLanding.current = address;
      if (restoringHistory) restorePosition();
      else {
        const node = content.current;
        const scroll = node?.parentElement;
        if (target && scroll) landReadingTarget(target, scroll);
        if (address) restoredAddress.current = address;
      }
    },
    [address, restoringHistory, restorePosition],
  );
  useLayoutEffect(() => {
    // A parent landing may render an embedded ability asynchronously. Its renderer
    // reports its target when ready, so only the active host scrolls or restores.
    if (
      detail &&
      (!landing || readyLanding.current === address) &&
      restoredAddress.current !== address
    )
      restorePosition();
  }, [enabled, detail, landing, address, restorePosition]);
  return { ref: content, onLandingReady };
}
