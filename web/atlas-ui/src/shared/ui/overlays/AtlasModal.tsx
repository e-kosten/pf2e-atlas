import { useDesktopTab, useTabCloseGuard } from "../../../app/DesktopTabContext";
import { Button, Flex, Modal, Space, type ModalProps } from "antd";
import { useCallback, useId, useLayoutEffect, useRef, useState } from "react";
import {
  FeedbackModalFrame,
  FeedbackOverlayActive,
} from "../../feedback/GlobalFeedback";
import { ModalEditorExpansion, type ExpandedField } from "./EditorExpansion";
import { useTabOverlayLifecycle } from "./useTabOverlayLifecycle";
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
  afterOpenChange,
  className,
  styles,
  style,
  layout = "form",
  workArea,
  ...props
}: AtlasModalProps) {
  const tab = useDesktopTab();
  const tabHidden = tab?.active === false;
  useTabCloseGuard(() =>
    open
      ? props.confirmLoading
        ? "pending"
        : confirmDiscard?.when()
          ? "dirty"
          : undefined
      : undefined,
  );
  const [field, expand] = useState<ExpandedField | null>(null);
  const collapseButton = useRef<HTMLSpanElement>(null);
  const returnFocus = useRef<HTMLButtonElement | null>(null);
  const focusedField = useRef<ExpandedField | null>(null);
  const compactScroll = useRef(0);
  const release = useCallback((id: string) => {
    expand((current) => (current?.id === id ? null : current));
  }, []);
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
    if (event.defaultPrevented) return;
    if (field) {
      returnFocus.current = field.trigger;
      expand(null);
      return;
    }
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
  const placement = useModalLayout({
    open: !!open && !tabHidden,
    layout: field ? "document" : layout,
    workArea: field ? undefined : workArea,
    frame,
  });
  useLayoutEffect(() => {
    if (tabHidden || !open) return;
    if (field && focusedField.current !== field) {
      collapseButton.current?.closest("button")?.focus({ preventScroll: true });
    } else if (!field && returnFocus.current) {
      const body = frame?.querySelector(".ant-modal-body");
      if (body) body.scrollTop = compactScroll.current;
      const trigger = returnFocus.current;
      returnFocus.current = null;
      if (trigger.isConnected && !trigger.disabled)
        trigger.focus({ preventScroll: true });
      else
        frame
          ?.querySelector<HTMLButtonElement>(".ant-modal-close")
          ?.focus({ preventScroll: true });
    }
    focusedField.current = field;
  }, [field, frame, open, tabHidden]);
  const resetPlacement = placement.reset;
  const focusable = props.focusable;
  const resetAndNotifyClose = useCallback(() => {
    previousFocus.current = null;
    returnFocus.current = null;
    expand(null);
    setConfirmingDiscard(false);
    resetPlacement();
    afterClose?.();
  }, [afterClose, resetPlacement]);
  const overlay = useTabOverlayLifecycle({
    tab,
    open: !!open,
    restoreFocus: focusable?.focusTriggerAfterClose !== false && props.mask !== false,
    panel: frame,
    afterClose: resetAndNotifyClose,
    afterOpenChange,
  });
  return (
    <FeedbackOverlayActive value={overlay.visible}>
      <Modal
        transitionName=""
        maskTransitionName=""
        {...props}
        title={field?.title ?? props.title}
        width={field ? "100%" : props.width}
        closable={
          field
            ? {
                closeIcon: <span ref={collapseButton}>Collapse editor</span>,
                "aria-label": "Collapse editor",
              }
            : props.closable
        }
        mask={field ? { closable: false } : props.mask}
        className={[
          className,
          placement.height <= 568 && "atlas-modal--short",
          placement.height <= 448 && "atlas-modal--tight",
        ]
          .filter(Boolean)
          .join(" ")}
        getContainer={props.getContainer ?? tab?.panel}
        keyboard={tabHidden ? false : field ? true : props.keyboard}
        focusable={
          tab
            ? {
                trap: !tabHidden && focusable?.trap !== false,
                focusTriggerAfterClose: false,
              }
            : focusable
        }
        open={overlay.visible}
        forceRender={tabHidden ? false : props.forceRender}
        destroyOnHidden={tab && open ? false : props.destroyOnHidden}
        afterOpenChange={(shown) => {
          if (!tab) afterOpenChange?.(shown);
          else if (shown) overlay.notifyShown();
        }}
        onCancel={field || confirmDiscard ? requestCancel : onCancel}
        footer={
          field ? null : confirmingDiscard ? (
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
          if (!tab) resetAndNotifyClose();
          else overlay.notifyHidden();
        }}
        style={{ ...style, ...placement.style }}
        styles={(info) => {
          const custom = typeof styles === "function" ? styles(info) : styles;
          return {
            ...custom,
            ...placement.styles,
            ...(field && {
              close: {
                ...custom?.close,
                width: "auto",
                paddingInline: "var(--space-control)",
              },
            }),
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
        <ModalEditorExpansion.Provider
          value={
            layout === "form"
              ? {
                  field,
                  release,
                  expand: (next) => {
                    compactScroll.current =
                      frame?.querySelector(".ant-modal-body")?.scrollTop ?? 0;
                    expand(next);
                  },
                }
              : null
          }
        >
          <div
            className="atlas-modal-fields"
            data-editor-expanded={field ? true : undefined}
            ref={draft}
            inert={confirmingDiscard}
            tabIndex={confirmDiscard ? -1 : undefined}
            onSubmitCapture={(event) => {
              // Nested picker dialogs have their own form and portal.
              if (field && event.currentTarget.contains(event.target as Node)) {
                event.preventDefault();
                event.stopPropagation();
              }
            }}
            style={
              field || confirmDiscard
                ? placement.fill
                  ? { height: "100%" }
                  : undefined
                : { display: "contents" }
            }
          >
            {children}
          </div>
        </ModalEditorExpansion.Provider>
      </Modal>
    </FeedbackOverlayActive>
  );
}
