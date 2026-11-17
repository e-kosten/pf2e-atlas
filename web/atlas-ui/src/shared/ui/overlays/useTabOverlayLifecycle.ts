import { useCallback, useLayoutEffect, useRef } from "react";
import {
  canFocusTabElement,
  type DesktopTabRuntime,
} from "../../../app/DesktopTabContext";
import { restoreOverlayFocus } from "./restoreOverlayFocus";

export function useTabOverlayLifecycle({
  tab,
  open,
  restoreFocus,
  panel,
  afterOpenChange,
  afterClose,
}: {
  tab: DesktopTabRuntime | null;
  open: boolean;
  restoreFocus: boolean;
  panel: HTMLElement | null;
  afterOpenChange?: (open: boolean) => void;
  afterClose?: () => void;
}) {
  const tabHidden = tab?.active === false;
  const visible = open && !tabHidden;
  const state = useRef({
    logicalOpen: false,
    physicalOpen: false,
    openingNotified: false,
    opener: null as HTMLElement | null,
  });

  const finishClose = useCallback(() => {
    const session = state.current;
    if (!session.logicalOpen) return;
    if (tab?.active && restoreFocus) {
      const target = canFocusTabElement(session.opener)
        ? session.opener
        : tab.panel().querySelector<HTMLElement>("#atlas-main");
      restoreOverlayFocus(target, panel);
    }
    session.logicalOpen = false;
    session.openingNotified = false;
    session.opener = null;
    afterClose?.();
    afterOpenChange?.(false);
  }, [afterClose, afterOpenChange, panel, restoreFocus, tab?.active]);

  useLayoutEffect(() => {
    if (!tab) return;
    const session = state.current;
    if (open && !session.logicalOpen) {
      session.logicalOpen = true;
      const host = tab.panel();
      const active = host.ownerDocument.activeElement;
      if (tab.active && active instanceof HTMLElement && host.contains(active))
        session.opener = active;
    }
    if (visible) session.physicalOpen = true;
    // A write may finish after Ant has already closed the suspended frame.
    if (!open && !session.physicalOpen) finishClose();
  }, [tab, open, visible, finishClose]);

  return {
    tabHidden,
    visible,
    notifyShown() {
      if (visible && !state.current.openingNotified) {
        state.current.openingNotified = true;
        afterOpenChange?.(true);
      }
    },
    notifyHidden() {
      if (visible) return;
      state.current.physicalOpen = false;
      if (!open) finishClose();
    },
  };
}
