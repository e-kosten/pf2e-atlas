import { Button, Flex, Modal, Space, type ModalProps } from "antd";
import { useId, useLayoutEffect, useRef, useState } from "react";
import {
  FeedbackModalFrame,
  FeedbackOverlayActive,
} from "../../feedback/GlobalFeedback";
import { useModalLayout, type ModalLayout, type ModalWorkArea } from "./useModalLayout";

export type AtlasModalProps = Omit<ModalProps, "centered"> & {
  layout?: ModalLayout;
  workArea?: ModalWorkArea;
  confirmDiscard?: { when: () => boolean; onDiscard: () => void };
};

export function AtlasModal({
  open,
  modalRender,
  children,
  footer,
  onCancel,
  confirmDiscard,
  afterClose,
  styles,
  style,
  layout = "form",
  workArea,
  ...props
}: AtlasModalProps) {
  const [confirmingDiscard, setConfirmingDiscard] = useState(false);
  const draft = useRef<HTMLDivElement>(null);
  const keepEditing = useRef<HTMLButtonElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const discardQuestion = useId();
  useLayoutEffect(() => {
    if (confirmingDiscard) {
      keepEditing.current?.focus({ preventScroll: true });
    } else if (previousFocus.current) {
      const target = previousFocus.current.isConnected
        ? previousFocus.current
        : draft.current;
      previousFocus.current = null;
      // An Ant Select can open its popup synchronously when its input is focused.
      const frameId = requestAnimationFrame(() =>
        target?.focus({ preventScroll: true }),
      );
      return () => cancelAnimationFrame(frameId);
    }
  }, [confirmingDiscard]);
  const requestCancel: ModalProps["onCancel"] = (event) => {
    if (props.confirmLoading) return;
    if (confirmingDiscard) {
      setConfirmingDiscard(false);
    } else if (confirmDiscard?.when()) {
      const active = draft.current?.ownerDocument.activeElement;
      previousFocus.current =
        active instanceof HTMLElement && frame.current?.contains(active)
          ? active
          : draft.current;
      setConfirmingDiscard(true);
    } else {
      onCancel?.(event);
    }
  };
  const frame = useRef<HTMLDivElement>(null);
  const placement = useModalLayout({ open: !!open, layout, workArea, frame });
  return (
    <FeedbackOverlayActive value={!!open}>
      <Modal
        transitionName=""
        maskTransitionName=""
        {...props}
        open={open}
        onCancel={confirmDiscard ? requestCancel : onCancel}
        footer={
          confirmingDiscard ? (
            <Flex align="end" gap="small" vertical>
              <span id={discardQuestion}>Discard this draft?</span>
              <Space wrap>
                <Button
                  ref={keepEditing}
                  aria-describedby={discardQuestion}
                  onClick={() => setConfirmingDiscard(false)}
                >
                  Keep editing
                </Button>
                <Button danger type="primary" onClick={confirmDiscard?.onDiscard}>
                  Discard
                </Button>
              </Space>
            </Flex>
          ) : (
            footer
          )
        }
        afterClose={() => {
          previousFocus.current = null;
          setConfirmingDiscard(false);
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
      >
        {confirmDiscard ? (
          <div ref={draft} inert={confirmingDiscard} tabIndex={-1}>
            {children}
          </div>
        ) : (
          children
        )}
      </Modal>
    </FeedbackOverlayActive>
  );
}
