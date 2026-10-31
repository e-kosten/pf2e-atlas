import { ErrorNotice } from "../../feedback/ErrorNotice";
import { AtlasModal } from "../overlays/AtlasModal";
import { Button, Space } from "antd";
import { useRef, useState } from "react";

type Confirmation = {
  content?: string;
  okText: string;
  onConfirm: () => void | Promise<unknown>;
  title: string;
};
export function useConfirmDangerAction() {
  const [request, setRequest] = useState<Confirmation | null>(null);
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const inFlight = useRef(false);
  const opener = useRef<HTMLElement | null>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  const close = () => {
    if (!inFlight.current) setOpen(false);
  };
  const submit = () => {
    if (!request || inFlight.current) return;
    inFlight.current = true;
    setPending(true);
    setError(null);
    void (async () => {
      try {
        await request.onConfirm();
        setOpen(false);
      } catch (failure) {
        setError(failure);
      } finally {
        inFlight.current = false;
        setPending(false);
      }
    })();
  };
  return {
    confirm: (next: Confirmation) => {
      if (inFlight.current) return;
      opener.current =
        document.activeElement instanceof HTMLElement ? document.activeElement : null;
      setError(null);
      setRequest(next);
      setOpen(true);
    },
    dialog: request && (
      <AtlasModal
        layout="confirmation"
        open={open}
        title={request.title}
        width={416}
        closable={false}
        keyboard={!pending}
        mask={{ closable: !pending }}
        onCancel={close}
        onOk={submit}
        afterOpenChange={(visible) => {
          if (visible)
            requestAnimationFrame(() => cancel.current?.focus({ preventScroll: true }));
        }}
        afterClose={() => {
          if (opener.current?.isConnected)
            opener.current.focus({ preventScroll: true });
          setRequest(null);
        }}
        footer={
          <Space>
            <Button ref={cancel} disabled={pending} onClick={close}>
              Cancel
            </Button>
            <Button type="primary" danger loading={pending} onClick={submit}>
              {request.okText}
            </Button>
          </Space>
        }
      >
        {request.content && <div>{request.content}</div>}
        {error != null && (
          <ErrorNotice
            error={error instanceof Error ? error : String(error)}
            title={`${request.okText} could not be confirmed`}
          />
        )}
      </AtlasModal>
    ),
  };
}
