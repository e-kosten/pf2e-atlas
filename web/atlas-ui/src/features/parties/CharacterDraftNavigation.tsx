import { Button, Space } from "antd";
import { useBlocker } from "@tanstack/react-router";
import { useRef } from "react";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";

export function useDraftNavigation(
  dirty: () => boolean,
  pending: () => Promise<unknown> | null,
) {
  const waiting = useRef(0);
  const blocker = useBlocker({
    withResolver: true,
    shouldBlockFn: async ({ current, next }) => {
      if (current.pathname === next.pathname) return false;
      const write = pending();
      if (write) {
        waiting.current += 1;
        try {
          await write;
        } finally {
          waiting.current -= 1;
        }
      }
      return dirty();
    },
    enableBeforeUnload: () => Boolean(pending()) || dirty(),
  });
  return { blocker, waiting };
}

export function DraftDiscardDialog({
  open,
  title,
  description,
  onKeep,
  onDiscard,
}: {
  open: boolean;
  title: string;
  description: string;
  onKeep: () => void;
  onDiscard: () => void;
}) {
  const keepEditing = useRef<HTMLButtonElement>(null);
  return (
    <AtlasModal
      open={open}
      layout="confirmation"
      title={title}
      width={416}
      closable={false}
      onCancel={onKeep}
      afterOpenChange={(visible) => {
        if (visible) keepEditing.current?.focus({ preventScroll: true });
      }}
      footer={
        <Space>
          <Button ref={keepEditing} onClick={onKeep}>
            Keep editing
          </Button>
          <Button type="primary" danger onClick={onDiscard}>
            Discard
          </Button>
        </Space>
      }
    >
      {description}
    </AtlasModal>
  );
}
