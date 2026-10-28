import { Drawer, type DrawerProps } from "antd";
import {
  FeedbackDrawerBody,
  FeedbackOverlayActive,
} from "../../feedback/GlobalFeedback";
/** Ant Drawer with shared recovery inside its focus boundary. */
export function AtlasDrawer({ open, children, ...props }: DrawerProps) {
  return (
    <FeedbackOverlayActive value={!!open}>
      <Drawer {...props} open={open}>
        <FeedbackDrawerBody active={!!open}>{children}</FeedbackDrawerBody>
      </Drawer>
    </FeedbackOverlayActive>
  );
}
