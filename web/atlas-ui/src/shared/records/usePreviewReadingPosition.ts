import { useLayoutEffect, useRef } from "react";
import type { RecordDetailView } from "../../generated/atlas";
import type { RecordPreviewContentProps } from "./recordPreviewTypes";

// Both native preview hosts restore the actual scroll container, after the
// addressed document arrives. Background refreshes do not restart reading.
export function usePreviewReadingPosition(
  detail: RecordDetailView | undefined,
  history: RecordPreviewContentProps["readingHistory"],
  host: "popover" | "drawer",
  enabled = true,
) {
  const content = useRef<HTMLElement>(null);
  const restoredAddress = useRef<string | null>(null);
  useLayoutEffect(() => {
    if (!enabled || !detail || !history || restoredAddress.current === history.address)
      return;
    const node = content.current;
    const scroll = host === "drawer" ? node?.parentElement : node;
    if (!node || !scroll) return;
    scroll.scrollTop = history.restore.scroll;
    if (history.restore.focusHref)
      Array.from(node.querySelectorAll<HTMLAnchorElement>("a[href]"))
        .find((link) => link.getAttribute("href") === history.restore.focusHref)
        ?.focus({ preventScroll: true });
    restoredAddress.current = history.address;
  }, [enabled, detail, history, host]);
  return content;
}
