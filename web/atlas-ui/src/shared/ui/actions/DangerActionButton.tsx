import { userErrorMessage } from "../../feedback/errorMessages";
import { ErrorNotice } from "../../feedback/ErrorNotice";
import { Button, Popconfirm } from "antd";
import { useRef, useState } from "react";
import type { ButtonProps, GetRef } from "antd";
import { useConfirmDangerAction } from "./useConfirmDangerAction";
import { restoreOverlayFocus } from "../overlays/restoreOverlayFocus";

type DangerActionButtonProps = Omit<ButtonProps, "danger" | "onClick"> & {
  confirmation?: "modal" | "popover";
  confirmContent?: string;
  confirmOkText: string;
  confirmTitle: string;
  onBeforeConfirm?: React.MouseEventHandler<HTMLElement>;
  onConfirm: () => void | Promise<unknown>;
};

export function DangerActionButton(props: DangerActionButtonProps) {
  return props.confirmation === "popover" ? (
    <PopoverDangerActionButton {...props} />
  ) : (
    <ModalDangerActionButton {...props} />
  );
}

function ModalDangerActionButton({
  confirmation: _confirmation,
  confirmContent,
  confirmOkText,
  confirmTitle,
  onBeforeConfirm,
  onConfirm,
  ...buttonProps
}: DangerActionButtonProps) {
  const { confirm: confirmDangerAction, dialog } = useConfirmDangerAction();
  return (
    <>
      <Button
        {...buttonProps}
        danger
        onClick={(event) => {
          onBeforeConfirm?.(event);
          event.currentTarget.focus({ preventScroll: true });
          confirmDangerAction({
            title: confirmTitle,
            content: confirmContent,
            okText: confirmOkText,
            onConfirm,
          });
        }}
      />
      {dialog}
    </>
  );
}

function PopoverDangerActionButton({
  confirmation: _confirmation,
  confirmContent,
  confirmOkText,
  confirmTitle,
  onBeforeConfirm,
  onConfirm,
  ...buttonProps
}: DangerActionButtonProps) {
  const [open, setOpen] = useState(false);
  const pendingRef = useRef(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const popup = useRef<GetRef<typeof Popconfirm>>(null);
  return (
    <Popconfirm
      ref={popup}
      title={confirmTitle}
      description={error ? <ErrorNotice error={error} title={error} /> : confirmContent}
      icon={null}
      open={open}
      afterOpenChange={(visible) => {
        if (!visible) restoreOverlayFocus(trigger.current, popup.current?.popupElement);
      }}
      onOpenChange={(next) => {
        if (!pendingRef.current) {
          setOpen(next);
          if (next) setError(null);
        }
      }}
      okText={confirmOkText}
      cancelText="Cancel"
      okButtonProps={{ danger: true, loading: pending }}
      cancelButtonProps={{ disabled: pending }}
      onConfirm={() => {
        if (pendingRef.current) return;
        pendingRef.current = true;
        setPending(true);
        setError(null);
        // Own closing explicitly: Ant's nullish-return close is ignored while pending.
        void (async () => {
          try {
            await Promise.resolve().then(onConfirm);
            setOpen(false);
          } catch (failure) {
            setError(userErrorMessage(failure, "Unable to remove. Try again."));
          } finally {
            pendingRef.current = false;
            setPending(false);
          }
        })();
      }}
    >
      <Button
        {...buttonProps}
        ref={trigger}
        danger
        onClick={(event) => {
          event.currentTarget.focus({ preventScroll: true });
          onBeforeConfirm?.(event);
        }}
      />
    </Popconfirm>
  );
}
