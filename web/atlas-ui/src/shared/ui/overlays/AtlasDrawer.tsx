import { useDesktopTab } from "../../../app/DesktopTabContext";
import { Drawer, type DrawerProps } from "antd";
import {
  FeedbackDrawerBody,
  FeedbackOverlayActive,
} from "../../feedback/GlobalFeedback";
/** Ant Drawer with shared recovery inside its focus boundary. */
export function AtlasDrawer({ open, children, ...props }: DrawerProps) {
  const tab = useDesktopTab();
  const visible = !!open && (tab?.active ?? true);
  return (
    <FeedbackOverlayActive value={visible}>
      <Drawer
        {...props}
        getContainer={props.getContainer ?? tab?.panel}
        keyboard={tab?.active === false ? false : props.keyboard}
        open={open}
      >
        <FeedbackDrawerBody active={visible}>{children}</FeedbackDrawerBody>
      </Drawer>
    </FeedbackOverlayActive>
  );
}
