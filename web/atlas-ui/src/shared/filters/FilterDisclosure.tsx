import { FilterOutlined } from "@ant-design/icons";
import { Button, Popover, theme } from "antd";
import {
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import { restoreOverlayFocus } from "../ui/overlays/restoreOverlayFocus";
import { PrimaryToolbar } from "../ui/PrimaryToolbar";

export function FilterDisclosure({
  narrow,
  searchEntry,
  summary,
  children,
  count,
  childOverlayOpen,
  clear,
  canClear,
}: {
  narrow: boolean;
  searchEntry: ReactNode;
  summary: ReactNode;
  children: ReactNode;
  count: number;
  childOverlayOpen: boolean;
  clear: () => void;
  canClear: boolean;
}) {
  const { token } = theme.useToken();
  const [open, setOpen] = useState(false);
  const [fit, setFit] = useState<{ width: number; height: number }>();
  const anchor = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const desktopPanel = useRef<HTMLElement>(null);
  const inlinePanel = useRef<HTMLElement>(null);
  const panel = narrow ? inlinePanel : desktopPanel;
  const id = useId();
  const panelId = id + (narrow ? "-inline" : "-popover");

  useLayoutEffect(() => {
    if (narrow || !anchor.current) return;
    const node = anchor.current;
    function measure() {
      const { width, bottom } = node.getBoundingClientRect();
      const height = Math.max(112, Math.floor(window.innerHeight - bottom) - 16);
      if (width)
        setFit((current) =>
          current?.width === width && current.height === height
            ? current
            : { width, height },
        );
    }
    measure();
    if (!open) return;
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(measure);
    observer?.observe(node);
    window.addEventListener("resize", measure);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", measure);
    };
  }, [narrow, open]);

  useLayoutEffect(() => {
    if (
      narrow &&
      open &&
      !childOverlayOpen &&
      (document.activeElement === trigger.current ||
        document.activeElement === document.body ||
        desktopPanel.current?.contains(document.activeElement))
    )
      inlinePanel.current?.focus({ preventScroll: true });
  }, [narrow, open, childOverlayOpen]);

  function close() {
    const focusInside = panel.current?.contains(document.activeElement);
    restoreOverlayFocus(trigger.current, panel.current);
    setOpen(false);
    if (narrow && focusInside)
      requestAnimationFrame(() => {
        if (document.activeElement === trigger.current)
          trigger.current?.scrollIntoView?.({ block: "nearest" });
      });
  }

  function escape(event: KeyboardEvent<HTMLElement>) {
    if (
      event.key !== "Escape" ||
      event.defaultPrevented ||
      !open ||
      childOverlayOpen ||
      !event.currentTarget.contains(event.target as Node)
    )
      return;
    event.preventDefault();
    close();
  }

  const content = (
    // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- Escape dismisses this keyboard-focusable panel.
    <section
      className="pane-filter-panel"
      ref={panel}
      id={panelId}
      role="dialog"
      tabIndex={-1}
      aria-labelledby={panelId + "-title"}
      style={
        !narrow && fit
          ? ({ "--pane-filter-available-height": fit.height + "px" } as CSSProperties)
          : undefined
      }
      onKeyDown={escape}
    >
      <h3 id={panelId + "-title"}>Filter saved records</h3>
      <div className="pane-filter-panel__body">{children}</div>
      <div className="pane-filter-panel__actions">
        <Button type="text" disabled={!canClear} onClick={clear}>
          Clear filters
        </Button>
        <Button onClick={close}>Done</Button>
      </div>
    </section>
  );

  return (
    <>
      <Popover
        trigger={narrow ? [] : "click"}
        placement="bottomRight"
        // Links and Advanced are sibling overlays; their dialogs belong above this panel.
        zIndex={token.zIndexPopupBase - 1}
        arrow={false}
        open={open && !narrow}
        onOpenChange={(next) => {
          if (!next && !childOverlayOpen) close();
        }}
        afterOpenChange={(next) => {
          if (
            next &&
            (document.activeElement === trigger.current ||
              document.activeElement === document.body)
          )
            panel.current?.focus({ preventScroll: true });
        }}
        styles={{
          root: { width: fit?.width, display: narrow ? "none" : undefined },
          container: { padding: 0 },
        }}
        content={narrow ? null : content}
      >
        <div className="pane-filter-anchor" ref={anchor}>
          <PrimaryToolbar className="filter-controls__search-row">
            {searchEntry}
            <Button
              ref={trigger}
              className="pane-filter-trigger"
              icon={<FilterOutlined aria-hidden="true" />}
              aria-expanded={open}
              aria-haspopup="dialog"
              aria-controls={open ? panelId : undefined}
              data-applied={count > 0}
              onClick={(event) => {
                event.stopPropagation();
                if (open) close();
                else {
                  event.currentTarget.focus({ preventScroll: true });
                  setOpen(true);
                }
              }}
              onKeyDown={escape}
            >
              {count ? `Filters (${count})` : "Filters"}
            </Button>
          </PrimaryToolbar>
          {summary}
        </div>
      </Popover>
      {narrow && (
        <div className="pane-filter-inline" hidden={!open}>
          {content}
        </div>
      )}
    </>
  );
}
