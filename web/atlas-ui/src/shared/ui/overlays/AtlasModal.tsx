import { useDesktopTab, useTabCloseGuard } from "../../../app/DesktopTabContext";
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
  className,
  styles,
  style,
  layout = "form",
  workArea,
  ...props
}: AtlasModalProps) {
  const tab = useDesktopTab();
  const tabHidden = tab?.active === false;
  const visible = !!open && !tabHidden;
  useTabCloseGuard(() =>
    open
      ? props.confirmLoading
        ? "pending"
        : confirmDiscard?.when()
          ? "dirty"
          : undefined
      : undefined,
  );
  const [confirmingDiscard, setConfirmingDiscard] = useState(false);
  const draft = useRef<HTMLDivElement>(null);
  const keepEditing = useRef<HTMLButtonElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const discardQuestion = useId();
  useLayoutEffect(() => {
    if (tabHidden) return;
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
  }, [confirmingDiscard, tabHidden]);
  const requestCancel: ModalProps["onCancel"] = (event) => {
    if (props.confirmLoading) return;
    if (confirmingDiscard) {
      setConfirmingDiscard(false);
    } else if (confirmDiscard?.when()) {
      const active = draft.current?.ownerDocument.activeElement;
      previousFocus.current =
        active instanceof HTMLElement && frame?.contains(active)
          ? active
          : draft.current;
      setConfirmingDiscard(true);
    } else {
      onCancel?.(event);
    }
  };
  const [frame, setFrame] = useState<HTMLDivElement | null>(null);
  const placement = useModalLayout({ open: !!open, layout, workArea, frame });
  return (
    <FeedbackOverlayActive value={visible}>
      <Modal
        transitionName=""
        maskTransitionName=""
        {...props}
        className={[
          className,
          placement.height <= 568 && "atlas-modal--short",
          placement.height <= 448 && "atlas-modal--tight",
        ]
          .filter(Boolean)
          .join(" ")}
        getContainer={props.getContainer ?? tab?.panel}
        keyboard={tabHidden ? false : props.keyboard}
        focusable={
          tabHidden ? { trap: false, focusTriggerAfterClose: false } : props.focusable
        }
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
              overflow: placement.scrollFrame ? "auto" : "hidden",
            },
            header: { ...custom?.header, flexShrink: 0 },
            body: {
              overflowY: "auto",
              ...custom?.body,
              minHeight: placement.scrollFrame && placement.fill ? 144 : 0,
              flex: placement.scrollFrame
                ? "0 0 auto"
                : placement.fill
                  ? "1 1 0"
                  : "1 1 auto",
              ...(placement.fill ? { height: placement.scrollFrame ? 144 : 0 } : {}),
              overscrollBehavior: "contain",
            },
            footer: { ...custom?.footer, flexShrink: 0 },
          };
        }}
        modalRender={(node) => (
          <FeedbackModalFrame active={!!open} frameRef={setFrame} fill={placement.fill}>
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
