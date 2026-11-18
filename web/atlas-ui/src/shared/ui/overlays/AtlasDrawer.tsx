import { useDesktopTab } from "../../../app/DesktopTabContext";
import { Drawer, type DrawerProps } from "antd";
import { useState } from "react";
import {
  FeedbackDrawerBody,
  FeedbackOverlayActive,
} from "../../feedback/GlobalFeedback";
import { useTabOverlayLifecycle } from "./useTabOverlayLifecycle";

/** Ant Drawer with shared recovery inside its focus boundary. */
export function AtlasDrawer({
  open,
  children,
  afterOpenChange,
  ...props
}: DrawerProps) {
  const tab = useDesktopTab();
  const [panel, setPanel] = useState<HTMLDivElement | null>(null);
  const overlay = useTabOverlayLifecycle({
    tab,
    open: !!open,
    restoreFocus: props.focusable?.focusTriggerAfterClose !== false,
    panel,
    afterOpenChange,
  });
  return (
    <FeedbackOverlayActive value={overlay.visible}>
      <Drawer
        {...props}
        {...(overlay.tabHidden
          ? { motion: { motionName: "" }, maskMotion: { motionName: "" } }
          : {})}
        panelRef={props.panelRef ?? setPanel}
        getContainer={props.getContainer ?? tab?.panel}
        keyboard={overlay.tabHidden ? false : props.keyboard}
        focusable={
          tab
            ? {
                ...props.focusable,
                trap: !overlay.tabHidden && props.focusable?.trap !== false,
                focusTriggerAfterClose: false,
              }
            : props.focusable
        }
        open={overlay.visible}
        forceRender={overlay.tabHidden ? false : props.forceRender}
        destroyOnHidden={tab && open ? false : props.destroyOnHidden}
        afterOpenChange={(shown) => {
          if (!tab) afterOpenChange?.(shown);
          else if (shown) overlay.notifyShown();
          else overlay.notifyHidden();
        }}
      >
        <FeedbackDrawerBody active={overlay.visible}>{children}</FeedbackDrawerBody>
      </Drawer>
    </FeedbackOverlayActive>
  );
}
