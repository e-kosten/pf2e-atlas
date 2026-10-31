import { Modal, type ModalProps } from "antd";
import { useRef } from "react";
import {
  FeedbackModalFrame,
  FeedbackOverlayActive,
} from "../../feedback/GlobalFeedback";
import { useModalLayout, type ModalLayout, type ModalWorkArea } from "./useModalLayout";

export type AtlasModalProps = Omit<ModalProps, "centered"> & {
  layout?: ModalLayout;
  workArea?: ModalWorkArea;
};

export function AtlasModal({
  open,
  modalRender,
  afterClose,
  styles,
  style,
  layout = "form",
  workArea,
  ...props
}: AtlasModalProps) {
  const frame = useRef<HTMLDivElement>(null);
  const placement = useModalLayout({ open: !!open, layout, workArea, frame });
  return (
    <FeedbackOverlayActive value={!!open}>
      <Modal
        transitionName=""
        maskTransitionName=""
        {...props}
        open={open}
        afterClose={() => {
          placement.reset();
          afterClose?.();
        }}
        style={{ ...style, ...placement.style }}
        styles={(info) => {
          const custom = typeof styles === "function" ? styles(info) : styles;
          return {
            ...custom,
            ...placement.styles,
            container: {
              ...custom?.container,
              display: "flex",
              flexDirection: "column",
              minHeight: 0,
              maxHeight: "100%",
              flex: "1 1 auto",
              overflow: "hidden",
            },
            header: { ...custom?.header, flexShrink: 0 },
            body: {
              overflowY: "auto",
              ...custom?.body,
              minHeight: 0,
              flex: layout === "browse" ? "1 1 0" : "1 1 auto",
              ...(layout === "browse" ? { height: 0 } : {}),
              overscrollBehavior: "contain",
            },
            footer: { ...custom?.footer, flexShrink: 0 },
          };
        }}
        modalRender={(node) => (
          <FeedbackModalFrame
            active={!!open}
            frameRef={frame}
            fill={layout === "browse"}
          >
            {modalRender ? modalRender(node) : node}
          </FeedbackModalFrame>
        )}
      />
    </FeedbackOverlayActive>
  );
}
