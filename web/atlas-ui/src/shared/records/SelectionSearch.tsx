import { Button, Popover } from "antd";
import { Search } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { searchDestination } from "../../app/destinations";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";
import { useSearchProfile } from "../search/SearchProfileContext";

const SCOPE = "[data-selection-search]";
const DISABLED = '[data-selection-search-disabled="true"]';
const EDITABLE =
  'input, textarea, select, button, [role="textbox"], [contenteditable]:not([contenteditable="false"])';

type SelectedText = {
  text: string;
  scope: Element;
  container: HTMLElement;
  left: number;
  top: number;
  origin: HTMLElement | null;
};

function elementAt(node: Node | null) {
  return node instanceof Element ? node : node?.parentElement;
}

function selectedText(): SelectedText | null {
  if (document.activeElement?.closest(EDITABLE)) return null;
  const selection = window.getSelection();
  if (!selection || selection.isCollapsed || selection.rangeCount !== 1) return null;
  const range = selection.getRangeAt(0);
  const start = elementAt(range.startContainer);
  const end = elementAt(range.endContainer);
  const scope = start?.closest(SCOPE);
  if (
    !scope ||
    scope.closest(DISABLED) ||
    end?.closest(SCOPE) !== scope ||
    start?.closest(EDITABLE) ||
    end?.closest(EDITABLE) ||
    range.cloneContents().querySelector(EDITABLE)
  )
    return null;
  const text = selection.toString().replace(/\s+/g, " ").trim();
  if (!text) return null;
  const rect = Array.from(range.getClientRects())
    .filter(
      (rect) =>
        rect.width > 0 &&
        rect.height > 0 &&
        rect.bottom > 0 &&
        rect.top < window.innerHeight &&
        rect.right > 0 &&
        rect.left < window.innerWidth,
    )
    .slice(-1)[0];
  if (!rect) return null;
  return {
    text,
    scope,
    // Keep the action inside its reader's dialog and keyboard focus boundary.
    container: scope.closest<HTMLElement>('[role="dialog"]') ?? document.body,
    left: Math.max(8, Math.min(window.innerWidth - 8, rect.left + rect.width / 2)),
    top: Math.min(window.innerHeight - 8, rect.bottom),
    origin:
      document.activeElement instanceof HTMLElement ? document.activeElement : null,
  };
}

/** One selection action across full records, preview drawers and encounters. */
export function SelectionSearch() {
  const [selected, setSelected] = useState<SelectedText | null>(null);
  const current = useRef(selected);
  const popup = useRef<HTMLDivElement>(null);
  const { id: profile } = useSearchProfile();
  useEffect(() => {
    current.current = selected;
  }, [selected]);
  useEffect(() => {
    let selecting = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const dismiss = () => {
      clearTimeout(timer);
      current.current = null;
      setSelected(null);
    };
    const insidePopup = (target: EventTarget | null) =>
      target instanceof Node && popup.current?.contains(target);
    const read = () => {
      clearTimeout(timer);
      if (selecting || insidePopup(document.activeElement)) return;
      timer = setTimeout(() => {
        const next = selectedText();
        current.current = next;
        setSelected(next);
      }, 180);
    };
    const pointerDown = (event: PointerEvent) => {
      if (insidePopup(event.target)) return;
      selecting = true;
      dismiss();
    };
    const pointerUp = () => {
      selecting = false;
      read();
    };
    const focusIn = (event: FocusEvent) => {
      if (current.current && !insidePopup(event.target)) dismiss();
    };
    const keyDown = (event: KeyboardEvent) => {
      const value = current.current;
      if (!value) return;
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        if (insidePopup(document.activeElement) && value.origin?.isConnected)
          value.origin.focus({ preventScroll: true });
        dismiss();
      } else if (
        event.key === "Tab" &&
        !event.shiftKey &&
        !insidePopup(document.activeElement)
      ) {
        event.preventDefault();
        popup.current
          ?.querySelector<HTMLAnchorElement>("a")
          ?.focus({ preventScroll: true });
      }
    };
    const scroll = (event: Event) => {
      if (!insidePopup(event.target)) dismiss();
    };
    document.addEventListener("selectionchange", read);
    document.addEventListener("pointerdown", pointerDown, true);
    document.addEventListener("pointerup", pointerUp);
    document.addEventListener("pointercancel", pointerUp);
    document.addEventListener("focusin", focusIn);
    document.addEventListener("keydown", keyDown, true);
    document.addEventListener("scroll", scroll, true);
    window.addEventListener("resize", dismiss);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("selectionchange", read);
      document.removeEventListener("pointerdown", pointerDown, true);
      document.removeEventListener("pointerup", pointerUp);
      document.removeEventListener("pointercancel", pointerUp);
      document.removeEventListener("focusin", focusIn);
      document.removeEventListener("keydown", keyDown, true);
      document.removeEventListener("scroll", scroll, true);
      window.removeEventListener("resize", dismiss);
    };
  }, []);
  useEffect(() => {
    if (!selected) return;
    const observer = new MutationObserver(() => {
      if (selected.scope.closest(DISABLED)) setSelected(null);
    });
    observer.observe(selected.scope, {
      attributes: true,
      attributeFilter: ["data-selection-search-disabled"],
    });
    return () => observer.disconnect();
  }, [selected]);
  if (!selected) return null;
  const params = new URLSearchParams({ q: selected.text });
  if (profile) params.set("search_profile", profile);
  const destination = searchDestination({ query: `?${params}` });
  return createPortal(
    <Popover
      open
      trigger={[]}
      placement="bottom"
      arrow={false}
      getPopupContainer={() => selected.container}
      styles={{ container: { padding: 4 } }}
      content={
        <div ref={popup} role="group" aria-label="Search selected text">
          <Button
            type="text"
            icon={<Search size={16} aria-hidden="true" />}
            href={destinationHref(destination)}
            onMouseDown={(event) => {
              // Preserve the selection while a mouse activates the native link.
              if (event.button === 0) event.preventDefault();
            }}
            onClick={(event) => {
              if (selected.scope.closest(DISABLED)) {
                event.preventDefault();
                setSelected(null);
                return;
              }
              if (!isPrimaryActivation(event)) return;
              event.preventDefault();
              setSelected(null);
              void navigateDestination(destination, { dismissReadingOverlays: true });
            }}
          >
            Search Atlas
          </Button>
        </div>
      }
    >
      <span
        aria-hidden="true"
        style={{
          position: "fixed",
          left: selected.left,
          top: selected.top,
          width: 1,
          height: 1,
          pointerEvents: "none",
        }}
      />
    </Popover>,
    selected.container,
  );
}
