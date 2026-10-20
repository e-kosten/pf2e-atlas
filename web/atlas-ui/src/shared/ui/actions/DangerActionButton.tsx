import { Alert, Button, Popconfirm } from "antd";
import { useRef, useState } from "react";
import type { ButtonProps } from "antd";
import { useConfirmDangerAction } from "./useConfirmDangerAction";

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
  const confirmDangerAction = useConfirmDangerAction();
  return (
    <Button
      {...buttonProps}
      danger
      onClick={(event) => {
        onBeforeConfirm?.(event);
        confirmDangerAction({
          title: confirmTitle,
          content: confirmContent,
          okText: confirmOkText,
          onConfirm,
        });
      }}
    />
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
  return (
    <Popconfirm
      title={confirmTitle}
      description={error ? <Alert type="error" title={error} /> : confirmContent}
      icon={null}
      open={open}
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
            setError(
              failure instanceof Error
                ? failure.message
                : "Unable to remove. Try again.",
            );
          } finally {
            pendingRef.current = false;
            setPending(false);
          }
        })();
      }}
    >
      <Button {...buttonProps} danger onClick={onBeforeConfirm} />
    </Popconfirm>
  );
}
