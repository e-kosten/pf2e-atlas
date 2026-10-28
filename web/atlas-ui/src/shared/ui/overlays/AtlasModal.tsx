import { Modal, type ModalProps } from "antd";
import {
  FeedbackModalFrame,
  FeedbackOverlayActive,
} from "../../feedback/GlobalFeedback";

export function feedbackModalStyles(
  styles?: ModalProps["styles"],
): ModalProps["styles"] {
  return (info) => {
    const custom = typeof styles === "function" ? styles(info) : styles;
    return {
      ...custom,
      container: {
        display: "flex",
        flexDirection: "column",
        minHeight: 0,
        ...custom?.container,
      },
      header: { flexShrink: 0, ...custom?.header },
      body: { minHeight: 0, overflowY: "auto", ...custom?.body },
      footer: { flexShrink: 0, ...custom?.footer },
    };
  };
}

/** Ant Modal with one reachable host for shell recovery, outside scrolling forms. */
export function AtlasModal({ open, modalRender, styles, ...props }: ModalProps) {
  return (
    <FeedbackOverlayActive value={!!open}>
      <Modal
        {...props}
        open={open}
        styles={feedbackModalStyles(styles)}
        modalRender={(node) => (
          <FeedbackModalFrame active={!!open}>
            {modalRender ? modalRender(node) : node}
          </FeedbackModalFrame>
        )}
      />
    </FeedbackOverlayActive>
  );
}
